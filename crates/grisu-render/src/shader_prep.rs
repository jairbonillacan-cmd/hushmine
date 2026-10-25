//! Preprocesador propio de WGSL: `#include` y variantes con `#ifdef`
//! (docs/graficos/04-shaders.md §1, ADR 0003 punto 6). Las directivas
//! ocupan líneas propias que desaparecen de la salida; los errores señalan
//! su fichero y línea (`"principal"` es la fuente de `preprocess_wgsl`).

use std::fmt;

/// Preset de calidad; cada variante activa su define en el shader.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QualityPreset {
    Low,
    Medium,
    High,
    Ultra,
}

impl QualityPreset {
    /// Define que activa este preset (mismo orden que las variantes).
    pub fn define(self) -> &'static str {
        const DEFINES: [&str; 4] = ["PRESET_LOW", "PRESET_MEDIUM", "PRESET_HIGH", "PRESET_ULTRA"];
        DEFINES[self as usize]
    }
}

/// Configuración de una variante de shader.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShaderConfig {
    pub preset: QualityPreset,
    /// true compila la variante para navegador (WebGL2).
    pub webgl2: bool,
}

#[rustfmt::skip]
impl Default for ShaderConfig {
    /// La variante nativa de escritorio actual: preset alto sin WebGL2.
    fn default() -> Self {
        Self { preset: QualityPreset::High, webgl2: false }
    }
}

impl ShaderConfig {
    /// Defines que activan esta variante, p. ej. `["PRESET_HIGH"]`.
    pub fn defines(&self) -> Vec<&'static str> {
        match self.webgl2 {
            false => vec![self.preset.define()],
            true => vec![self.preset.define(), "WEBGL2"],
        }
    }
}

/// Fallo de preprocesado o de validación; `line` (1-based) refiere a `file`.
#[rustfmt::skip]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ShaderPrepError {
    UnterminatedConditional { file: String, line: usize },
    UnknownInclude { file: String, line: usize, name: String },
    IncludeCycle { file: String, line: usize, name: String },
    UnknownDirective { file: String, line: usize, directive: String },
    StrayElse { file: String, line: usize },
    StrayEndif { file: String, line: usize },
    InvalidDirective { file: String, line: usize, detail: String },
    InvalidStructure { file: String, detail: String },
}

#[rustfmt::skip]
impl fmt::Display for ShaderPrepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ShaderPrepError::*;
        match self {
            UnterminatedConditional { file, line } =>
                write!(f, "condicional sin #endif (abierto en la línea {line} de {file})"),
            UnknownInclude { file, line, name } =>
                write!(f, "#include desconocido «{name}» (línea {line} de {file})"),
            IncludeCycle { file, line, name } =>
                write!(f, "ciclo de includes: «{name}» ya en curso (línea {line} de {file})"),
            UnknownDirective { file, line, directive } =>
                write!(f, "directiva desconocida «#{directive}» (línea {line} de {file})"),
            StrayElse { file, line } => write!(f, "#else sin #ifdef (línea {line} de {file})"),
            StrayEndif { file, line } => write!(f, "#endif sin #ifdef (línea {line} de {file})"),
            InvalidDirective { file, line, detail } =>
                write!(f, "directiva inválida (línea {line} de {file}): {detail}"),
            InvalidStructure { file, detail } =>
                write!(f, "estructura inválida ({file}): {detail}"),
        }
    }
}

impl std::error::Error for ShaderPrepError {}

/// Constructores cortos de errores con fichero y línea del hallazgo.
#[rustfmt::skip]
impl ShaderPrepError {
    fn directive_error(file: &str, line: usize, detail: String) -> Self {
        Self::InvalidDirective { file: file.into(), line, detail }
    }
    fn unknown_include(file: &str, line: usize, name: &str) -> Self {
        Self::UnknownInclude { file: file.into(), line, name: name.into() }
    }
    fn include_cycle(file: &str, line: usize, name: &str) -> Self {
        Self::IncludeCycle { file: file.into(), line, name: name.into() }
    }
    fn unknown_directive(file: &str, line: usize, word: &str) -> Self {
        Self::UnknownDirective { file: file.into(), line, directive: word.into() }
    }
    fn stray_else(file: &str, line: usize) -> Self { Self::StrayElse { file: file.into(), line } }
    fn stray_endif(file: &str, line: usize) -> Self { Self::StrayEndif { file: file.into(), line } }
    fn unterminated(file: &str, line: usize) -> Self {
        Self::UnterminatedConditional { file: file.into(), line }
    }
}

/// Preprocesa `source` con los `includes` y `defines` activos: resuelve
/// `#include` (recursivo) y `#ifdef`/`#ifndef`/`#else`/`#endif` anidados.
#[rustfmt::skip]
pub fn preprocess_wgsl(source: &str, includes: &[(&str, &str)], defines: &[&str]) -> Result<String, ShaderPrepError> {
    let mut lines = Vec::new();
    let mut cycle = Vec::new();
    preprocess_file("principal", source, includes, defines, &mut cycle, &mut lines)?;
    Ok(lines.join("\n") + "\n")
}

/// Condicional abierto: dónde se abrió, si el contexto exterior emitía y si la rama actual emite.
#[rustfmt::skip]
struct Branch { open_line: usize, parent_active: bool, taken: bool, active: bool, had_else: bool }

/// Preprocesa un fichero empujando las líneas supervivientes en `out`;
/// `cycle` acumula los includes en curso y la rama más interna resume las exteriores.
#[rustfmt::skip]
fn preprocess_file(file: &str, source: &str, includes: &[(&str, &str)], defines: &[&str],
    cycle: &mut Vec<String>, out: &mut Vec<String>) -> Result<(), ShaderPrepError> {
    let mut stack: Vec<Branch> = Vec::new();
    for (idx, line) in source.lines().enumerate() {
        let line_no = idx + 1;
        let active = stack.last().is_none_or(|b| b.active);
        let Some(rest) = line.trim_start().strip_prefix('#') else {
            if active { out.push(line.to_string()); }
            continue;
        };
        let (word, tail) = split_first_word(rest);
        match word {
            "ifdef" | "ifndef" => {
                let (symbol, _) = split_first_word(tail);
                if symbol.is_empty() {
                    return Err(ShaderPrepError::directive_error(
                        file, line_no, "#{word} sin símbolo".into()));
                }
                let condition = defines.contains(&symbol) == (word == "ifdef");
                stack.push(Branch { open_line: line_no, parent_active: active,
                    taken: active && condition, active: active && condition, had_else: false });
            }
            "else" => {
                let Some(branch) = stack.last_mut() else {
                    return Err(ShaderPrepError::stray_else(file, line_no));
                };
                if branch.had_else {
                    return Err(ShaderPrepError::directive_error(file, line_no, "#else repetido".into()));
                }
                branch.had_else = true;
                branch.active = branch.parent_active && !branch.taken;
                branch.taken = true;
            }
            "endif" => {
                if stack.pop().is_none() {
                    return Err(ShaderPrepError::stray_endif(file, line_no));
                }
            }
            "include" if active => {
                let name = quoted(tail).ok_or_else(
                    || ShaderPrepError::directive_error(file, line_no, "#include sin nombre entre comillas".into()))?;
                let Some((_, content)) = includes.iter().find(|(k, _)| *k == name) else {
                    return Err(ShaderPrepError::unknown_include(file, line_no, name));
                };
                if cycle.iter().any(|k| k == name) {
                    return Err(ShaderPrepError::include_cycle(file, line_no, name));
                }
                cycle.push(name.to_string());
                let nested = preprocess_file(name, content, includes, defines, cycle, out);
                cycle.pop();
                nested?;
            }
            // En una rama inactiva las demás directivas se ignoran (como en C).
            _ if !active => {}
            _ => return Err(ShaderPrepError::unknown_directive(file, line_no, word)),
        }
    }
    if let Some(open) = stack.first() {
        return Err(ShaderPrepError::unterminated(file, open.open_line));
    }
    Ok(())
}

/// Divide `text` (ya sin `#`) en su primera palabra y el resto.
fn split_first_word(text: &str) -> (&str, &str) {
    let text = text.trim_start();
    text.split_once(char::is_whitespace).unwrap_or((text, ""))
}

/// Texto entre la primera pareja de comillas dobles de `text`.
fn quoted(text: &str) -> Option<&str> {
    let start = text.find('"')? + 1;
    let end = text[start..].find('"')? + start;
    (start != end).then_some(&text[start..end])
}

/// Preprocesa y valida los dos shaders del crate en las 8 variantes (4 presets × navegador o no).
pub fn validate_all_shader_variants() -> Result<usize, ShaderPrepError> {
    use QualityPreset::{High, Low, Medium, Ultra};
    const SHADERS: &[(&str, &str)] = &[
        ("voxel.wgsl", include_str!("voxel.wgsl")),
        ("cube.wgsl", include_str!("cube.wgsl")),
    ];
    for webgl2 in [false, true] {
        for preset in [Low, Medium, High, Ultra] {
            let defines = ShaderConfig { preset, webgl2 }.defines();
            for (file, source) in SHADERS {
                validate_structure(file, &preprocess_wgsl(source, &[], &defines)?)?;
            }
        }
    }
    Ok(8)
}

/// Validación estructural sin GPU de una fuente ya preprocesada: balance
/// de llaves y paréntesis, puntos de entrada, bindings únicos y sin residuos.
#[rustfmt::skip]
fn validate_structure(file: &str, wgsl: &str) -> Result<(), ShaderPrepError> {
    let invalid = |d: String| ShaderPrepError::InvalidStructure { file: file.to_string(), detail: d };
    let stripped = strip_comments(wgsl);
    let (mut braces, mut parens) = (0i32, 0i32);
    for ch in stripped.chars() {
        match ch {
            '{' => braces += 1,
            '}' => braces -= 1,
            '(' => parens += 1,
            ')' => parens -= 1,
            _ => continue,
        }
        if braces < 0 || parens < 0 { return Err(invalid("cierre sin su apertura".into())); }
    }
    if braces != 0 || parens != 0 {
        return Err(invalid(format!("quedan {braces} llaves y {parens} paréntesis")));
    }
    for stage in ["@vertex", "@fragment"] {
        if !stripped.contains(stage) {
            return Err(invalid(format!("falta el punto de entrada {stage}")));
        }
    }
    for (n, l) in wgsl.lines().enumerate() {
        if l.trim_start().starts_with('#') {
            return Err(invalid(format!("directiva residual, línea {}", n + 1)));
        }
    }
    let mut seen = std::collections::HashSet::new();
    for (g, b) in group_binding_pairs(&stripped).map_err(invalid)? {
        if !seen.insert((g, b)) {
            return Err(invalid(format!("duplicado @group({g}) @binding({b})")));
        }
    }
    Ok(())
}

/// Pares `@group(G) @binding(B)` en orden de aparición; falla si un `@binding` va sin `@group`.
fn group_binding_pairs(text: &str) -> Result<Vec<(u32, u32)>, String> {
    // (posición, es @binding, valor); luego se ordena por aparición.
    let mut marks: Vec<(usize, bool, u32)> = Vec::new();
    for (key, is_binding) in [("@group", false), ("@binding", true)] {
        let mut from = 0;
        while let Some(found) = text[from..].find(key) {
            let at = from + found;
            if let Some(value) = number_after(text, at + key.len()) {
                marks.push((at, is_binding, value));
            }
            from = at + key.len();
        }
    }
    marks.sort_by_key(|(at, _, _)| *at);
    let (mut pairs, mut group) = (Vec::new(), None);
    for (_, is_binding, value) in marks {
        if is_binding {
            pairs.push((group.ok_or("binding sin @group")?, value));
            group = None;
        } else {
            group = Some(value);
        }
    }
    Ok(pairs)
}

/// Entero `(n)` a partir de `at`, tolerando espacios alrededor.
#[rustfmt::skip]
fn number_after(text: &str, at: usize) -> Option<u32> {
    let rest = text[at..].trim_start().strip_prefix('(')?.trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    (end > 0 && rest[end..].trim_start().starts_with(')')).then(|| rest[..end].parse().unwrap_or(0))
}

/// Copia `wgsl` con los comentarios (`//...` y `/*...*/`) cambiados por
/// espacios: conserva saltos de línea y cadenas con comillas.
#[rustfmt::skip]
fn strip_comments(wgsl: &str) -> String {
    let chars: Vec<char> = wgsl.chars().collect();
    let mut out = String::with_capacity(wgsl.len());
    // Estados: comentario de línea, de bloque o cadena; `skip` consume tras '\'.
    let (mut in_line, mut in_block, mut in_str, mut skip) = (false, false, false, false);
    for (i, &c) in chars.iter().enumerate() {
        if skip { skip = false; out.push(c); continue; }
        let next = chars.get(i + 1).copied();
        if in_line {
            out.push(if c == '\n' { '\n' } else { ' ' });
            in_line = c != '\n';
        } else if in_block {
            if c == '*' && next == Some('/') { out.push_str("  "); in_block = false; }
            else { out.push(if c == '\n' { '\n' } else { ' ' }); }
        } else if in_str {
            if c == '\\' { skip = true; } else if c == '"' { in_str = false; }
            out.push(c);
        } else {
            match (c, next) {
                ('/', Some('/')) => { in_line = true; out.push_str("  "); }
                ('/', Some('*')) => { in_block = true; out.push_str("  "); }
                ('"', _) => { in_str = true; out.push(c); }
                _ => out.push(c),
            }
        }
    }
    out
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::ShaderPrepError as E;
    use super::*;

    fn prep(source: &str, includes: &[(&str, &str)], defines: &[&str]) -> String {
        preprocess_wgsl(source, includes, defines).unwrap()
    }

    #[test]
    fn includes_anidados_sustituyen_y_los_errores_senalan_fichero() {
        let nested = [("a.wgsl", "// A\n#include \"b.wgsl\"\n// fin A"), ("b.wgsl", "// B")];
        assert_eq!(prep("#include \"a.wgsl\"", &nested, &[]), "// A\n// B\n// fin A\n");
        // Include desconocido, ciclo y error que refiere el fichero del include.
        let err = preprocess_wgsl("#include \"no.wgsl\"", &[], &[]).unwrap_err();
        assert!(matches!(err, E::UnknownInclude { name, line: 1, .. } if name == "no.wgsl"));
        let err = preprocess_wgsl("#include \"a.wgsl\"", &[("a.wgsl", "#include \"a.wgsl\"")], &[]).unwrap_err();
        assert!(matches!(err, E::IncludeCycle { name, .. } if name == "a.wgsl"));
        let err = preprocess_wgsl("#include \"a.wgsl\"", &[("a.wgsl", "#endif")], &[]).unwrap_err();
        assert!(matches!(err, E::StrayEndif { file, line: 1, .. } if file == "a.wgsl"));
    }

    #[test]
    fn ramas_eligen_las_lineas_correctas() {
        let source = "antes\n#ifdef FOO\nmedio\n#else\notro\n#endif\ndespués";
        assert_eq!(prep(source, &[], &[]), "antes\notro\ndespués\n");
        assert_eq!(prep(source, &[], &["FOO"]), "antes\nmedio\ndespués\n");
        assert_eq!(prep("#ifndef FOO\nsin\n#else\ncon\n#endif", &[], &[]), "sin\n");
        // En rama inactiva las directivas se ignoran y el anidamiento queda equilibrado.
        let inactive = "#ifdef FOO\n#include \"no.wgsl\"\n#ifdef BAR\nx\n#endif\n#endif\nvisible";
        assert_eq!(prep(inactive, &[], &[]), "visible\n");
    }

    #[test]
    fn condicionales_y_directivas_rotos_dan_error() {
        assert!(matches!(preprocess_wgsl("#else", &[], &[]), Err(E::StrayElse { .. })));
        assert!(matches!(preprocess_wgsl("a\n#endif", &[], &[]), Err(E::StrayEndif { line: 2, .. })));
        // Un segundo `#else` para el mismo condicional es un error (como en C).
        let err = preprocess_wgsl("#ifdef F\na\n#else\nb\n#else\n#endif", &[], &[]).unwrap_err();
        assert!(matches!(err, E::InvalidDirective { line: 5, .. }));
        let err = preprocess_wgsl("// cero\n#ifdef FOO\nuno\n#ifdef BAR\ndos", &[], &[]).unwrap_err();
        assert!(matches!(err, E::UnterminatedConditional { line: 2, .. }));
    }

    #[test]
    fn defines_de_las_ocho_variantes() {
        use QualityPreset::*;
        for (preset, define) in [(Low, "PRESET_LOW"), (Medium, "PRESET_MEDIUM"),
            (High, "PRESET_HIGH"), (Ultra, "PRESET_ULTRA")] {
            assert_eq!(ShaderConfig { preset, webgl2: false }.defines(), [define]);
            assert_eq!(ShaderConfig { preset, webgl2: true }.defines(), [define, "WEBGL2"]);
        }
        assert_eq!(ShaderConfig::default(), ShaderConfig { preset: QualityPreset::High, webgl2: false });
    }

    #[test]
    fn validacion_detecta_defectos_y_aprueba_los_reales() {
        let entries = "@vertex\nfn v() {}\n@fragment\nfn f() {}";
        assert!(validate_structure("t.wgsl", entries).is_ok());
        for (wgsl, clue) in [("@vertex\nfn v() { // {", "llave"),
            ("#version\n@vertex\nfn v() {}\n@fragment\nfn f() {}", "residual")] {
            let err = validate_structure("t.wgsl", wgsl).unwrap_err();
            assert!(matches!(&err, E::InvalidStructure { detail, .. } if detail.contains(clue)));
        }
        // El mismo par @group/@binding dos veces es un binding duplicado.
        let dup = format!("@group(0) @binding(0) var a: sampler;\n@group(0) @binding(0) var b: sampler;\n{entries}");
        let err = validate_structure("t.wgsl", &dup).unwrap_err();
        assert!(matches!(err, E::InvalidStructure { detail, .. } if detail.contains("duplicado")));
        assert_eq!(validate_all_shader_variants(), Ok(8));
    }
}
