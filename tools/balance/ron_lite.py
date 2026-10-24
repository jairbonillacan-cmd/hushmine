"""Parser ligero del subconjunto de RON (Rusty Object Notation) para HUSHMINE.

Soporta structs con o sin nombre, listas, enteros con guiones bajos, booleanos,
cadenas y variantes de enumeración simples. Prohíbe explícitamente números en
coma flotante o con punto decimal para garantizar determinismo entero.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

# Expresión regular para detectar números con punto decimal fuera de cadenas y comentarios.
_DECIMAL_RE = re.compile(r"(?<![A-Za-z_])[+-]?\d[\d_]*\.[\d_]+")

# Especificación léxica del subconjunto de RON usado en data/*.ron.
_TOKEN_SPEC = re.compile(
    r"""
    (?P<WS>\s+)
  | (?P<COMMENT>//[^\n]*)
  | (?P<STRING>"(?:\\.|[^"\\])*")
  | (?P<FLOAT>[+-]?\d[\d_]*\.[\d_]+)
  | (?P<INT>[+-]?\d(?:[\d_]*\d)?)
  | (?P<LPAREN>\()
  | (?P<RPAREN>\))
  | (?P<LBRACKET>\[)
  | (?P<RBRACKET>\])
  | (?P<LBRACE>\{)
  | (?P<RBRACE>\})
  | (?P<COLON>:)
  | (?P<COMMA>,)
  | (?P<IDENT>[A-Za-z_][A-Za-z0-9_]*)
  | (?P<MISMATCH>.)
    """,
    re.VERBOSE,
)


def _strip_comments_and_strings(text: str) -> str:
    """Elimina comentarios de línea y literales de cadena conservando huecos."""
    out: list[str] = []
    i = 0
    n = len(text)
    while i < n:
        ch = text[i]
        if ch == "/" and i + 1 < n and text[i + 1] == "/":
            while i < n and text[i] != "\n":
                i += 1
        elif ch == '"':
            out.append('""')
            i += 1
            while i < n and text[i] != '"':
                if text[i] == "\\":
                    i += 2
                else:
                    i += 1
            i += 1
        else:
            out.append(ch)
            i += 1
    return "".join(out)


def has_decimal_numbers(text: str) -> bool:
    """Devuelve True si el texto RON contiene literales numéricos con punto decimal."""
    cleaned = _strip_comments_and_strings(text)
    return bool(_DECIMAL_RE.search(cleaned))


class _Token:
    """Token léxico interno con tipo, valor y posición."""

    __slots__ = ("kind", "value", "pos")

    def __init__(self, kind: str, value: str, pos: int) -> None:
        self.kind = kind
        self.value = value
        self.pos = pos


def _tokenize(text: str) -> list[_Token]:
    """Convierte el texto RON en una secuencia de tokens validados."""
    if has_decimal_numbers(text):
        raise ValueError("Los números con punto decimal están prohibidos en archivos RON")

    tokens: list[_Token] = []
    for match in _TOKEN_SPEC.finditer(text):
        kind = match.lastgroup
        value = match.group()
        pos = match.start()
        if kind in ("WS", "COMMENT") or kind is None:
            continue
        if kind == "FLOAT":
            raise ValueError(
                f"Los números con punto decimal están prohibidos en archivos RON: {value!r}"
            )
        if kind == "MISMATCH":
            raise ValueError(f"Carácter inesperado {value!r} en la posición {pos}")
        tokens.append(_Token(kind, value, pos))
    return tokens


class _Parser:
    """Analizador sintáctico descendente recursivo para el subconjunto de RON."""

    def __init__(self, tokens: list[_Token]) -> None:
        self.tokens = tokens
        self.index = 0

    def peek(self, offset: int = 0) -> _Token | None:
        pos = self.index + offset
        if 0 <= pos < len(self.tokens):
            return self.tokens[pos]
        return None

    def consume(self, expected_kind: str | None = None) -> _Token:
        tok = self.peek()
        if tok is None:
            raise ValueError("Fin de entrada inesperado al analizar RON")
        if expected_kind is not None and tok.kind != expected_kind:
            raise ValueError(
                f"Se esperaba {expected_kind} pero se encontró {tok.kind} ({tok.value!r}) "
                f"en la posición {tok.pos}"
            )
        self.index += 1
        return tok

    def parse_root(self) -> Any:
        if not self.tokens:
            raise ValueError("El contenido RON está vacío")
        value = self.parse_value()
        if self.peek() is not None:
            extra = self.peek()
            assert extra is not None
            raise ValueError(
                f"Token sobrante tras la raíz RON: {extra.value!r} en la posición {extra.pos}"
            )
        return value

    def parse_value(self) -> Any:
        tok = self.peek()
        if tok is None:
            raise ValueError("Se esperaba un valor RON pero se alcanzó el final del archivo")

        if tok.kind == "INT":
            self.consume("INT")
            return int(tok.value.replace("_", ""))

        if tok.kind == "STRING":
            self.consume("STRING")
            inner = tok.value[1:-1]
            return (
                inner.replace(r"\"", '"')
                .replace(r"\\", "\\")
                .replace(r"\n", "\n")
                .replace(r"\t", "\t")
                .replace(r"\r", "\r")
            )

        if tok.kind == "LBRACKET":
            return self.parse_list()

        if tok.kind == "LBRACE":
            return self.parse_map()

        if tok.kind == "LPAREN":
            return self.parse_paren_body()

        if tok.kind == "IDENT":
            ident = self.consume("IDENT").value
            if ident == "true":
                return True
            if ident == "false":
                return False
            nxt = self.peek()
            if nxt is not None and nxt.kind == "LPAREN":
                return self.parse_paren_body()
            return ident

        raise ValueError(
            f"Token inesperado {tok.kind} ({tok.value!r}) en la posición {tok.pos}"
        )

    def parse_list(self) -> list[Any]:
        self.consume("LBRACKET")
        items: list[Any] = []
        while True:
            nxt = self.peek()
            if nxt is None:
                raise ValueError("Lista RON sin cerrar ']'")
            if nxt.kind == "RBRACKET":
                self.consume("RBRACKET")
                return items
            items.append(self.parse_value())
            sep = self.peek()
            if sep is not None and sep.kind == "COMMA":
                self.consume("COMMA")
            elif sep is not None and sep.kind == "RBRACKET":
                continue
            else:
                raise ValueError("Se esperaba ',' o ']' en la lista RON")

    def parse_map(self) -> dict[Any, Any]:
        self.consume("LBRACE")
        mapping: dict[Any, Any] = {}
        while True:
            nxt = self.peek()
            if nxt is None:
                raise ValueError("Mapa RON sin cerrar '}'")
            if nxt.kind == "RBRACE":
                self.consume("RBRACE")
                return mapping
            key = self.parse_value()
            self.consume("COLON")
            val = self.parse_value()
            mapping[key] = val
            sep = self.peek()
            if sep is not None and sep.kind == "COMMA":
                self.consume("COMMA")
            elif sep is not None and sep.kind == "RBRACE":
                continue
            else:
                raise ValueError("Se esperaba ',' o '}' en el mapa RON")

    def parse_paren_body(self) -> Any:
        """Analiza `(campo: valor, ...)` como dict o `(a, b, ...)` como tupla."""
        self.consume("LPAREN")
        nxt = self.peek()
        if nxt is not None and nxt.kind == "RPAREN":
            self.consume("RPAREN")
            return {}

        second = self.peek(1)
        is_named_struct = (
            nxt is not None
            and nxt.kind == "IDENT"
            and second is not None
            and second.kind == "COLON"
        )

        if is_named_struct:
            fields: dict[str, Any] = {}
            while True:
                cur = self.peek()
                if cur is None:
                    raise ValueError("Struct RON sin cerrar ')'")
                if cur.kind == "RPAREN":
                    self.consume("RPAREN")
                    return fields
                field_name = self.consume("IDENT").value
                self.consume("COLON")
                field_val = self.parse_value()
                fields[field_name] = field_val
                sep = self.peek()
                if sep is not None and sep.kind == "COMMA":
                    self.consume("COMMA")
                elif sep is not None and sep.kind == "RPAREN":
                    continue
                else:
                    raise ValueError(
                        f"Se esperaba ',' o ')' tras el campo {field_name!r}"
                    )

        elements: list[Any] = []
        while True:
            cur = self.peek()
            if cur is None:
                raise ValueError("Tupla RON sin cerrar ')'")
            if cur.kind == "RPAREN":
                self.consume("RPAREN")
                return tuple(elements)
            elements.append(self.parse_value())
            sep = self.peek()
            if sep is not None and sep.kind == "COMMA":
                self.consume("COMMA")
            elif sep is not None and sep.kind == "RPAREN":
                continue
            else:
                raise ValueError("Se esperaba ',' o ')' en la tupla RON")


def loads(text: str) -> Any:
    """Analiza una cadena en formato RON y devuelve estructuras nativas de Python."""
    tokens = _tokenize(text)
    return _Parser(tokens).parse_root()


def load(path: str | Path) -> Any:
    """Lee un archivo `.ron` desde disco y devuelve su estructura en Python."""
    file_path = Path(path)
    content = file_path.read_text(encoding="utf-8")
    return loads(content)
