""" Primitivas pseudoaleatorias deterministas en aritmética entera de 64 bits.

Este módulo evita por completo el uso de coma flotante para garantizar que la
futura traducción a Rust produzca resultados idénticos bit a bit.
"""

from __future__ import annotations

MASK64: int = 0xFFFFFFFFFFFFFFFF
MASK32: int = 0xFFFFFFFF


def splitmix64_step(x: int) -> int:
    """Mezclador entero de 64 bits idéntico a `crates/hush-sim/src/rng.rs`."""
    z = (x + 0x9E3779B97F4A7C15) & MASK64
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK64
    return (z ^ (z >> 31)) & MASK64


class Xorshift64:
    """Generador pseudoaleatorio Xorshift64 inicializado con SplitMix64."""

    def __init__(self, seed: int) -> None:
        """Inicializa el estado de 64 bits nunca nulo a partir de la semilla."""
        self.seed: int = seed & MASK64
        self.state: int = splitmix64_step(self.seed)
        if self.state == 0:
            self.state = 0x9E3779B97F4A7C15

    def next_u64(self) -> int:
        """Avanza el estado mediante Xorshift64 y devuelve un entero de 64 bits."""
        x = self.state
        x ^= (x << 13) & MASK64
        x ^= (x >> 7) & MASK64
        x ^= (x << 17) & MASK64
        self.state = x
        return x

    def below(self, bound: int) -> int:
        """Devuelve un entero uniforme en `0..bound` usando reducción de Lemire."""
        if bound <= 0:
            return 0
        return (((self.next_u64() >> 32) * (bound & MASK32)) >> 32) & MASK32

    def randint(self, low: int, high: int) -> int:
        """Devuelve un entero inclusivo en el rango `[low, high]`."""
        if high <= low:
            return low
        return low + self.below(high - low + 1)


def hash3d(x: int, y: int, z: int, seed: int) -> int:
    """Hash espacial determinista de 64 bits sobre `(x, y, z, seed)`."""
    h = (seed & MASK64) ^ ((x & MASK64) * 0x9E3779B97F4A7C15) & MASK64
    h = (h ^ (((y & MASK64) * 0xBF58476D1CE4E5B9) & MASK64)) & MASK64
    h = (h ^ (((z & MASK64) * 0x94D049BB133111EB) & MASK64)) & MASK64
    return splitmix64_step(h)
