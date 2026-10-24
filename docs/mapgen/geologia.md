# Especificación Geológica y Generación Procedural de Mapas (`hush-sim` / `grisu-voxel`)

Este documento define la arquitectura matemática, estratigráfica y competitiva del generador procedural de mapas de **HUSHMINE**. Toda la generación es estrictamente determinista, opera exclusivamente con aritmética entera de 64 bits y respeta los catálogos canónicos `data/blocks.ron`, `data/seismograph.ron` y `data/economy.ron`.

---

## 1. Arquitectura de Coordenadas, Chunks y Determinismo Entero

### 1.1 Dominio Espacial y Partición en Chunks

El mundo subterráneo ocupa un dominio ortogonal discreto de vóxeles:

$$\Omega = [0, W-1] \times [0, H-1] \times [0, D-1] \subset \mathbb{Z}^3$$

donde los ejes siguen la convención del motor **Grisú**:
- **$X \in [0, W-1]$:** eje horizontal Este-Oeste ($x = 0$ límite Oeste / lado `Copper`; $x = W-1$ límite Este / lado `Cobalt`).
- **$Y \in [0, H-1]$:** eje vertical de profundidad ($y = 0$ lecho abisal indestructible; $y = H-1 = 63$ techo de roca madre).
- **$Z \in [0, D-1]$:** eje horizontal Norte-Sur ($z = D/2$ plano axial de las bases y cámaras centrales).

Según el tamaño de escuadra (`team_size` en `data/economy.ron`), existen dos geometrías canónicas de mapa, ambas divididas en chunks cúbicos de $16 \times 16 \times 16 = 4\,096$ vóxeles (`1 byte` `u8` por celda):

| Escala competitiva | Dimensiones $(W \times D \times H)$ | Malla de chunks $(C_x \times C_z \times C_y)$ | Total chunks | Total vóxeles | Memoria base (`u8`) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`1v1` a `6v6`** (`team_size: 1..=6`) | $64 \times 64 \times 64$ | $4 \times 4 \times 4$ | $64$ | $262\,144$ | $256\text{ KB}$ |
| **`7v7` a `8v8`** (`team_size: 7..=8`) | $96 \times 96 \times 64$ | $6 \times 6 \times 4$ | $144$ | $589\,824$ | $576\text{ KB}$ |

### 1.2 Índice Lineal Canónico y Direccionamiento de Chunks

Para el volcado de instantáneas, el cálculo del digest SHA-256 de validación y la serialización determinista de mundo, el buffer plano de vóxeles se indexa en orden $Y \to Z \to X$:

$$\text{idx}(x, y, z) = (y \cdot D + z) \cdot W + x$$

En la representación particionada por chunks de $16^3$ en `grisu-voxel`, cada coordenada global $(x, y, z)$ se descompone mediante desplazamientos de bits en coordenada de chunk y coordenada local:

```text
cx = x >> 4,   cy = y >> 4,   cz = z >> 4
lx = x & 15,   ly = y & 15,   lz = z & 15
chunk_local_idx(lx, ly, lz) = (ly << 8) | (lz << 4) | lx
```

### 1.3 Catálogo Canónico de Bloques (`data/blocks.ron`)

Cada celda almacena un identificador `u8` correspondiente a la **Convención A** (`0..=17`). El equipo propietario (`Copper` o `Cobalt`) **nunca** codifica nuevos ids de bloque: se registra en la tabla dispersa de metadatos de celda cuando `team_owned == true`.

| ID (`u8`) | Identificador | Categoría geológica / funcional | Dureza (`miner` / `prospector`) | Propiedades físicas clave |
| :---: | :--- | :--- | :---: | :--- |
| `0` | `air` | Vacío transitable / caverna | `0` / `0` | No sólido, no transmite ruido |
| `1` | `dirt` | Corteza somera blanda | `8` / `12` | Sólido, excavación rápida |
| `2` | `rock` | Matriz litológica común | `15` / `24` | Sólido, roca base del manto |
| `3` | `hard_rock` | Dique estructural / frontera | `45` / `90` | Sólido, blindaje geológico |
| `4` | `ore` | Mena ferrosa ($4\text{ uds/bloque}$) | `18` / `28` | Sólido, **conductor eléctrico** |
| `5` | `gold` | Filón aurífero ($100\text{ \$/bloque}$) | `24` / `36` | Sólido, **conductor eléctrico** |
| `6` | `diamond` | Kimberlita ($400\text{ \$/bloque}$) | `36` / `54` | Sólido, aislante eléctrico |
| `7` | `lava` | Magma celular letal | `0` / `0` | Fluido no picable, emite ruido al fluir |
| `8` | `team_block` | Mampostería de base e ingeniería | `15` / `24` | `team_owned: true`, coste `1 ore` |
| `9` | `force_field` | Esclusa permeable al equipo propio | `30` / `45` | `team_owned: true`, coste `4 ore` |
| `10` | `ladder` | Escala de pozo vertical | `10` / `15` | `team_owned: true`, no sólido, coste `1 ore` |
| `11` | `jump_pad` | Impulsor vertical | `20` / `30` | `team_owned: true`, coste `6 ore` |
| `12` | `electric` | Emisor de red de alta tensión | `30` / `45` | `team_owned: true`, conductor, coste `8 ore` |
| `13` | `rail` | Vía rápida y línea conductora | `12` / `18` | `team_owned: true`, conductor, coste `2 ore` |
| `14` | `beacon` | Baliza lumínica y táctica | `20` / `20` | `team_owned: true`, coste `5 ore` |
| `15` | `explosive` | Carga de demolición | `30` / `30` | `team_owned: true` (`sapper`: `6` ticks), coste `10 ore` |
| `16` | `bucket_station` | Terminal de línea de cangilones | `35` / `45` | `team_owned: true`, conductor, coste `12 ore` |
| `17` | `bucket_post` | Poste intermedio de cangilones | `20` / `30` | `team_owned: true`, conductor, coste `3 ore` |

### 1.4 Frontera Indestructible (`INV-09`)

Para impedir fugas de fluido, caídas fuera del dominio $\Omega$ o excavaciones que rodeen el perímetro del mapa, el paso de sellado final impone `3 hard_rock` en las seis caras exteriores del volumen:
- **Piso abisal:** $y = 0$ para todo $(x, z) \in [0, W-1] \times [0, D-1]$.
- **Techo de roca madre:** $y = H - 1 = 63$ para todo $(x, z) \in [0, W-1] \times [0, D-1]$.
- **Muros perimetrales:** $x \in \{0, W-1\}$ y $z \in \{0, D-1\}$ para todo $y \in [0, H-1]$.

En tiempo de partida, `hush-sim` rechaza cualquier orden de picado o demolición sobre coordenadas pertenecientes al borde del dominio $\Omega$, haciendo que el cascarón perimetral sea perpetuamente indestructible.

### 1.5 Algoritmo RNG (`Xorshift64` + `hash3d` Entero) y Contrato de Portabilidad a Rust

El generador prohíbe el uso de `f32`, `f64`, funciones trigonométricas o generadores de la biblioteca estándar. Toda fuente de entropía se deriva de dos primitivas de 64 bits sin signo (`u64`):

1. **Secuenciador `Xorshift64` con pre-mezcla `SplitMix64`:**
   La semilla de entrada `seed: u64` se blanquea mediante un paso de `SplitMix64` para garantizar que `seed == 0` nunca produzca el estado absorbente nulo y que semillas consecutivas diverjan en el primer ciclo.

```rust
// Especificación exacta para el port a Rust en hush-sim / grisu-voxel:
// DEBE usar aritmética modular wrapping_* y tipos u64 puros (cero f32/f64).
pub struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    pub const fn new(seed: u64) -> Self {
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        let state = z ^ (z >> 31);
        Self {
            state: if state == 0 { 0x9E37_79B9_7F4A_7C15 } else { state },
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    #[inline]
    pub fn below(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        // Reducción entera de Lemire usando los 32 bits altos (idéntica a tools/mapgen/rng.py)
        (((self.next_u64() >> 32).wrapping_mul(u64::from(bound))) >> 32) as u32
    }
}
```

2. **Función de ruido espacial determinista `hash3d(x, y, z, seed)`:**
   Para evaluar rugosidad de estratos, venas basálticas y micro-perturbaciones en $O(1)$ de forma idéntica a `tools/mapgen/rng.py`, se emplea un hash espacial entero de 64 bits:

```rust
#[inline]
pub const fn splitmix64_step(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[inline]
pub const fn hash3d(x: i64, y: i64, z: i64, seed: u64) -> u64 {
    let mut h = seed ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= (z as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
    splitmix64_step(h)
}
```

> [!IMPORTANT]
> **Contrato de equivalencia bit a bit Python $\leftrightarrow$ Rust:** El futuro port en Rust dentro de `hush-sim` y `grisu-voxel` **debe implementar exactamente las mismas constantes hexadecimales, desplazamientos `(<< 13, >> 7, << 17)`, reducción `(((next_u64() >> 32) * bound) >> 32)` y orden de etapas** con aritmética `wrapping_*` en `u64`. Para cualquier par `(seed, team_size)`, el buffer lineal de bloques generado en Rust debe arrojar **exactamente el mismo hash SHA-256 byte a byte** que el prototipo de referencia.

---

## 2. Estratigrafía por Profundidad (Eje Vertical $Y \in [0, 63]$)

La columna geológica de $H = 64$ bloques de altura se estructura en tres grandes horizontes litológicos diseñados para acompasar las fases de partida (*Fiebre*, *Guerra Sorda* y *Despertar*):

```text
Y = 63  [================ TECHO SELLADO: 3 hard_rock (INV-09) ================]
Y = 62  . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        CORTEZA SUPERIOR Y SUBSUELO SOMERO (y = 48..62)
        - Matriz: 1 dirt (y >= 52) con transición rugosa a 2 rock (y = 48..55)
        - Bases de equipo en y = 51..55: Copper (x ~ 8) y Cobalt (x ~ W-9)
        - Recursos: alta densidad de 4 ore y filones someros de 5 gold (Fiebre)
Y = 48  -----------------------------------------------------------------------
Y = 47  : : : : : : : : : : : : : : : : : : : : : : : : : : : : : : : : : : : :
        MANTO INTERMEDIO PIZARROSO (y = 20..47)
        - Matriz: 2 rock con diques y nervaduras de 3 hard_rock
        - Red de cavernas naturales (0 air) y pozos de descenso (y = 24..52)
        - Vetas ricas de 4 ore y 5 gold; aparición de 6 diamond bajo y <= 34
        - [VETA MADRE / MOTHER LODE] en (W/2, 30, D/2) -> 2_000 $ (Respira)
        - Bolsas colgadas de 7 lava encapsuladas en y = 20..28 (Inquietud)
Y = 20  -----------------------------------------------------------------------
Y = 19  # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # #
        ZONA ABISAL Y MAGMÁTICA (y = 1..19)
        - Matriz: alternancia densa de 2 rock y escudos de 3 hard_rock
        - Grandes reservas profundas de 5 gold y 6 diamond
        - Cámaras presurizadas de 7 lava encapsuladas
        - [EL CORAZÓN / THE HEART] en (W/2, 6, D/2) -> 1_500 $ (Despertar)
Y = 0   [================= PISO ABISAL: 3 hard_rock (INV-09) =================]
```

### 2.1 Corteza Superior y Subsuelo Somero ($y \in [46, 62]$)

- **Litología base:** las cotas $y \in [48, 62]$ están compuestas mayoritariamente por `1 dirt` (dureza `8` ticks para el `miner`), permitiendo una expansión inicial veloz y silenciosa.
- **Horizonte de transición ($y \in [46, 48]$):** la frontera entre `1 dirt` y `2 rock` se ondula columna a columna de forma simétrica ($x_{\text{sym}} = \min(x, W - 1 - x)$) mediante la cota umbral entera:
  $$y_{\text{dirt}}(x, z) = 46 + \bigl(\text{hash3d}(x_{\text{sym}}, 0, z, \text{seed}) \bmod 3\bigr)$$
  Todo vóxel de matriz con $y \ge y_{\text{dirt}}(x, z)$ recibe `1 dirt`; por debajo recibe `2 rock`.
- **Recursos de fase de *Fiebre*:** concentra bolsadas frecuentes de `4 ore` ($4\text{ uds/bloque}$) indispensables para levantar las primeras líneas de `17 bucket_post`, `10 ladder` y `9 force_field`, junto con filones accesibles de `5 gold` ($100\text{ \$/bloque}$) que permiten las primeras entregas rápidas al banco sin necesidad de explosivos.

### 2.2 Manto Intermedio Pizarroso ($y \in [20, 47]$)

- **Litología base:** matriz continua de `2 rock` interrumpida por diques estructurales y costillas oblicuas de `3 hard_rock` que canalizan el tráfico subterráneo y obligan a usar al `sapper` o rodear formaciones duras.
- **Sistemas cavernarios:** grandes cavidades naturales de `0 air` que actúan como cámaras acústicas y encrucijadas para el tendido de líneas de cangilones (`16 bucket_station` / `17 bucket_post`) y raíles (`13 rail`).
- **Estratificación mineral:**
  - Filones medios y gruesos de `4 ore` y `5 gold` en todo el rango $y \in [20, 47]$.
  - Aparición de kimberlitas de `6 diamond` ($400\text{ \$/bloque}$) exclusivamente a partir de $y \le 34$.
- **Hito sísmico central:** en el centro geométrico del manto, $(W/2, 30, D/2)$, se emplaza la cámara acorazada de la **Veta Madre** (*Mother Lode*).

### 2.3 Zona Abisal y Magmática ($y \in [1, 19]$)

- **Litología base:** roca metamórfica profunda donde la proporción de `3 hard_rock` crece conforme $y \to 1$, formando contrafuertes naturales alrededor de las cámaras magmáticas.
- **Mineralización de alto riesgo:** alberga los racimos de mayor ley de `5 gold` y la máxima densidad de `6 diamond`, compensando la lejanía a las bases y la proximidad al magma.
- **Vulcanismo confinado:** bolsas presurizadas de `7 lava` selladas por cáscaras de `2 rock` y `3 hard_rock`, además del santuario central de **el Corazón** (*The Heart*) en $(W/2, 6, D/2)$.

---

## 3. Vetas Minerales, Cuevas y Bolsas de Lava

### 3.1 Geometría Entera de Racimos y Elipsoides

Tanto las vetas minerales como las cavernas y bolsas de lava se tallan mediante elipsoides discretos de semiejes enteros $(r_x, r_y, r_z) \in \mathbb{Z}^+$. Para evitar divisiones o raíces en coma flotante, la pertenencia de un vóxel $(x, y, z)$ respecto al centro $(c_x, c_y, c_z)$ con desplazamientos $(dx, dy, dz) = (|x - c_x|, |y - c_y|, |z - c_z|)$ se evalúa mediante la desigualdad entera exacta:

$$(dx \cdot r_y \cdot r_z)^2 + (dy \cdot r_x \cdot r_z)^2 + (dz \cdot r_x \cdot r_y)^2 \le (r_x \cdot r_y \cdot r_z)^2$$

Para dotar de textura orgánica a las formaciones sin perder determinismo, los vóxeles situados en la corteza exterior del elipsoide (donde el miembro izquierdo supera el $75\,\%$ del miembro derecho) se aceptan o descartan consultando `hash3d(x, y, z, seed + salt) % 100 < fill_pct`.

| Formación geológica | Bloque (`u8`) | Rango vertical $Y$ | Semiejes típicos $(r_x, r_y, r_z)$ | Notas de generación y conductividad |
| :--- | :---: | :---: | :---: | :--- |
| **Veta de mena** | `4 ore` | $y \in [10, 60]$ | $(2..4,\; 1..2,\; 2..4)$ | Alta frecuencia en $y \ge 40$; **conductor eléctrico** natural |
| **Filón de oro** | `5 gold` | $y \in [6, 58]$ | $(2..3,\; 1..2,\; 2..3)$ | Ley creciente con la profundidad; **conductor eléctrico** |
| **Bolsa de diamante** | `6 diamond` | $y \in [3, 34]$ | $(1..2,\; 1..2,\; 1..2)$ | Nódulos compactos profundos; aislante eléctrico |
| **Caverna natural** | `0 air` | $y \in [16, 50]$ | $(3..6,\; 2..3,\; 3..6)$ | Cadenas de 2 a 3 lóbulos; nunca perforan el borde $\Omega$ |
| **Dique basáltico** | `3 hard_rock` | $y \in [4, 46]$ | $(3..5,\; 1..3,\; 1..3)$ | Barreras tácticas que nunca bloquean el 100 % de las rutas |
| **Bolsa de magma** | `7 lava` | $y \in [4, 24]$ | $(2,\; 1,\; 2)$ | Siempre envuelta en cáscara sólida de `2 rock` / `3 hard_rock` |

### 3.2 Bolsas de Lava, Contención y Respuesta al Sismógrafo (`data/seismograph.ron`)

La generación de `7 lava` obedece tres invariantes de seguridad e interacción sistémica:

1. **Techo estricto de cota ($y \le 24$):** ninguna celda de `7 lava` se genera por encima de $y = 24$ (con los núcleos magmáticos en $y \in [5, 23]$ y su zócalo en el Corazón en $y = 4$).
2. **Encapsulamiento inicial (`INV-LAVA-SEAL`):** al generarse una bolsa de `7 lava`, todo su contorno exterior se sella con una cáscara de `2 rock` y `3 hard_rock`, impidiendo que la lava fluya libremente en el tick `0` hacia cavernas abiertas.
3. **Distancia de seguridad a las bases ($\ge 16$ bloques):** la distancia Chebyshev ($L_\infty$), euclídea y Manhattan ($L_1$) desde cualquier celda de `7 lava` hasta el volumen de las bases `Copper` y `Cobalt` es siempre **$\ge 28$ bloques** (superando ampliamente el mínimo de seguridad de 6 bloques).

A lo largo de la partida, la geología responde a los cuatro umbrales del Sismógrafo (`0..=100_000` a $30\text{ Hz}$):
- **`Stir` / *Revuelo* (`25_000`):** vibración de galerías; desprendimientos cosméticos de polvo y primer aviso acústico de los Silbadores (*Whistlers*).
- **`Breathe` / *Respira* (`50_000`):** fractura sísmica de los sellos laterales de `3 hard_rock` que custodian la cámara de la **Veta Madre** en $(W/2, 30, D/2)$, abriendo el acceso a sus `2_000 $`.
- **`Restless` / *Inquietud* (`75_000`):** colapso de las cáscaras de contención inferiores de las bolsas de `7 lava` colgadas en $y \in [20, 28]$ y apertura de grietas de revancha entre estratos.
- **`Wake` / *Despertar* (`100_000`):** fase terminal de `2_700` ticks ($90\text{ s}$). Se fractura el blindaje de **el Corazón** en $(W/2, 6, D/2)$ (`1_500 $`), el nivel abisal de `7 lava` asciende $+1$ estrato cada `90` ticks ($3\text{ s}$) desde el fondo, y las líneas de cangilones se detienen cuando restan `900` ticks ($30\text{ s}$).

---

## 4. Bases de los Equipos (`Copper` al Oeste, `Cobalt` al Este) y Simetría Cuasi-Espejo

### 4.1 Emplazamiento y Dotación Arquitectónica de las Bases

Las dos bases se sitúan enfrentadas sobre el eje $X$ en el plano central $z_b = D / 2$ y dentro de la franja segura de corteza $y \in [51, 55]$:
- **Base `Copper` (Oeste):** centro operativo en $(x_{\text{copper}}, y_b, z_b) = (8, 52, D/2)$, orientada hacia $+X$ (Este).
- **Base `Cobalt` (Este):** centro operativo reflejado en $(x_{\text{cobalt}}, y_b, z_b) = (W - 9, 52, D/2)$, orientada hacia $-X$ (Oeste).

Cada base se estampa de forma determinista con los siguientes elementos (con metadato de propiedad `Copper` o `Cobalt` en todos los bloques `8..=17`):

1. **Cámara interior despejada (`0 air`):** volumen interior libre de $7 \times 4 \times 7$ vóxeles ($x \in [x_b - 3, x_b + 3]$, $y \in [52, 55]$, $z \in [z_b - 3, z_b + 3]$) para la aparición (*spawn*) y tránsito de hasta 8 mineros por equipo.
2. **Solera de equipo (`8 team_block`):** losa firme de $7 \times 1 \times 7$ en $y = 51$ ($x \in [x_b - 3, x_b + 3]$, $z \in [z_b - 3, z_b + 3]$) en `8 team_block` (`94` bloques en total sumando ambas bases tras el paso del pozo).
3. **Pórtico de exclusión (`9 force_field`):** cortina frontal de `9 force_field` de $1 \times 3 \times 3$ en la cara orientada hacia el centro del mapa ($x = x_b + 3$ para `Copper`, $x = x_b - 3$ para `Cobalt`, $y \in [52, 54]$, $z \in [z_b - 1, z_b + 1]$; `18` bloques en total), permeable al equipo propietario y bloqueante para intrusos rivales.
4. **Terminal logístico (`16 bucket_station`, `17 bucket_post`) y baliza (`14 beacon`):**
   - Una estación `16 bucket_station` en $(x_b \mp 1, 52, z_b - 2)$ acompañada de su primer poste `17 bucket_post` en $(x_b \pm 1, 52, z_b - 2)$.
   - Una baliza central `14 beacon` en el techo $(x_b, 55, z_b)$ que ilumina la sala y sirve de guía en el HUD.
5. **Pozo maestro de descenso (`10 ladder`):** en el costado sur de la sala ($x = x_b$, $z = z_b + 2$), se excava un conducto vertical de `0 air` en $(x_b, y, z_b + 1)$ acompañado de una columna continua de `10 ladder` en $(x_b, y, z_b + 2)$ desde $y = 22$ hasta $y = 52$ (`31` bloques de escalera por base, `62` en total), conectando con corredores excavables en $y \in \{22, 30\}$ que garantizan el acceso a todo el subsuelo.

### 4.2 Equidad Competitiva mediante Simetría Cuasi-Espejo ($95\,\% / 5\,\%$)

Para garantizar la justicia competitiva sin que ambos equipos jueguen en mitades clónicas predecibles de memoria, el generador aplica una **reflexión axial cuasi-espejo** respecto al plano central $x \leftrightarrow W - 1 - x$:

- **Núcleo estructural reflejado ($95\,\%$):**
  Todas las cavernas primarias (`0 air`), bolsas de `7 lava`, diques de `3 hard_rock` y el $95\,\%$ de los racimos de `4 ore`, `5 gold` y `6 diamond` se muestrean en el semiespacio occidental $x \in [2, W/2 - 2]$ y se proyectan al semiespacio oriental mediante:
  $$x' = (W - 1) - x, \qquad y' = y, \qquad z' = z$$
- **Perturbación orgánica controlada ($5\,\%$):**
  - En los bordes de cada elipsoide reflejado, el $5\,\%$ de los vóxeles periféricos se evalúa con `hash3d(x', y', z', seed ^ 0xA5A5_5A5A)` en lugar de copiar ciegamente el borde occidental, desplazando la rugosidad local $\pm 1$ bloque.
  - Adicionalmente, un $5\,\%$ del presupuesto secundario de micro-bolsadas de `4 ore` y `5 gold` se posiciona con un desfase local $(\Delta y \in \{-1, 0, +1\}, \Delta z \in \{-2..+2\})$ independiente en cada mitad, siempre fuera de las cámaras centrales y sin interrumpir caminos de acceso.
- **Invariante de equidad de oro alcanzable (`INV-FAIR-GOLD`):**
  La asimetría relativa del oro alcanzable por cada equipo desde su propia base sin picar `3 hard_rock` ni cruzar `7 lava`:
  $$\Delta_{\text{gold}} = \frac{|G_{\text{reach}}(\text{Copper}) - G_{\text{reach}}(\text{Cobalt})|}{\max\bigl(G_{\text{reach}}(\text{Copper}), G_{\text{reach}}(\text{Cobalt})\bigr)} \times 100$$
  está garantizada por construcción en **$\Delta_{\text{gold}} \le 10\,\%$** (situándose típicamente en **$< 3\,\%$**).

---

## 5. Veta Madre (*Mother Lode*), el Corazón (*The Heart*) y Presupuesto Económico

### 5.1 Cámaras Sísmicas Centrales en el Plano de Simetría ($x = W/2, z = D/2$)

Sobre el eje central equidistante de ambas bases ($\text{dist}_X = W/2 - 8$) se tallan dos estructuras singulares que concentran la disputa entre equipos en la mitad y el final de la partida:

1. **La Veta Madre (*Mother Lode*) en $(W/2, 30, D/2)$:**
   - **Geometría:** cámara central en $x \in [W/2 - 4, W/2 + 3]$, $y \in [27, 33]$, $z \in [D/2 - 3, D/2 + 3]$ envuelta en matriz de `2 rock`.
   - **Apertura sísmica:** al cruzar el umbral `Breathe` / *Respira* (`50_000`), el sismo fractura la roca circundante y revela la dirección de la cámara en los instrumentos del Vigía.
   - **Reserva garantizada (`mother_lode_total_cash = 2_000 $`):** el núcleo interior contiene exactamente **12 bloques de `5 gold`** ($12 \times 100\text{ \$} = 1\,200\text{ \$}$) y **2 bloques centrales de `6 diamond`** ($2 \times 400\text{ \$} = 800\text{ \$}$), sumando exactamente **`2_000 $`**.

2. **El Corazón (*The Heart*) en $(W/2, 6, D/2)$:**
   - **Geometría:** cámara abisal en $x \in [W/2 - 4, W/2 + 3]$, $y \in [4, 9]$, $z \in [D/2 - 3, D/2 + 3]$.
   - **Blindaje y foso magmático:** asentado sobre un lecho de `3 hard_rock` ($y = 3$), una cubeta de `7 lava` ($y = 4$) y un anillo de contención de `3 hard_rock` ($y = 5$). Al llegar el Sismógrafo a `Wake` / *Despertar* (`100_000`), la cámara queda expuesta mientras comienza el ascenso global de lava.
   - **Reserva garantizada (`heart_total_cash = 1_500 $`):** el depósito central alberga **2 bloques de `6 diamond`** ($2 \times 400\text{ \$} = 800\text{ \$}$) y **7 bloques de `5 gold`** ($7 \times 100\text{ \$} = 700\text{ \$}$), sumando exactamente **`1_500 $`**.

### 5.2 Presupuesto de Recursos por Escala de Partida (`data/economy.ron`)

El generador calibra el número de filones de `4 ore`, `5 gold` y `6 diamond` en función del tamaño del mapa y de `team_size` para cumplir con holgura la condición de suficiencia económica:

> **Invariante de suficiencia (`INV-GOLD-BUDGET`):** Cada equipo (`Copper` y `Cobalt`) dispone de **al menos $1{,}5 \times \text{target\_cash}$** en bloques de `5 gold` ($100\text{ \$/bloque}$) directamente alcanzables mediante búsqueda en anchura 6-conexa (BFS) desde el interior de su base a través de bloques excavables `{0 air, 1 dirt, 2 rock, 4 ore, 5 gold, 6 diamond, 8..=17}`, **sin atravesar jamás `3 hard_rock` ni `7 lava`**, y sin contabilizar los `6 diamond`.

La siguiente tabla resume las metas de `data/economy.ron` y el presupuesto mínimo garantizado por el generador en cada modalidad:

| Modo (`team_size`) | Mapa $(W \times D \times H)$ | `target_cash` | Mínimo legal `5 gold` alcanzable / equipo ($1{,}5\times$) | Objetivo real `5 gold` alcanzable / equipo ($\approx 2{,}3\times$) | Oro total en mapa (`5 gold`) | Valor en `6 diamond` en mapa ($400\text{ \$/bl}$) | Veta Madre + Corazón | Mena ferrosa (`4 ore`, $4\text{ uds/bl}$) |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`1v1`** (`1`) | $64 \times 64 \times 64$ | `2_000 $` | `3_000 $` (30 bl.) | $\ge 4\,600\text{ \$}$ | $11\,000 - 14\,000\text{ \$}$ | $7\,600 - 9\,600\text{ \$}$ | `2_000 $` + `1_500 $` | $200 - 240$ bl. ($800 - 960\text{ uds}$) |
| **`2v2`** (`2`) | $64 \times 64 \times 64$ | `3_500 $` | `5_250 $` (53 bl.) | $\ge 8\,000\text{ \$}$ | $18\,000 - 21\,000\text{ \$}$ | $7\,600 - 9\,600\text{ \$}$ | `2_000 $` + `1_500 $` | $200 - 240$ bl. ($800 - 960\text{ uds}$) |
| **`3v3`** (`3`) | $64 \times 64 \times 64$ | `5_000 $` | `7_500 $` (75 bl.) | $\ge 11\,500\text{ \$}$ | $25\,000 - 28\,000\text{ \$}$ | $7\,600 - 9\,600\text{ \$}$ | `2_000 $` + `1_500 $` | $200 - 240$ bl. ($800 - 960\text{ uds}$) |
| **`4v4`** (`4`) | $64 \times 64 \times 64$ | `6_500 $` | `9_750 $` (98 bl.) | $\ge 15\,500\text{ \$}$ ($\ge 2{,}3\times$) | $31\,000 - 34\,000\text{ \$}$ | $7\,600 - 9\,600\text{ \$}$ | `2_000 $` + `1_500 $` | $200 - 240$ bl. ($800 - 960\text{ uds}$) |
| **`5v5`** (`5`) | $64 \times 64 \times 64$ | `8_000 $` | `12_000 $` (120 bl.) | $\ge 18\,400\text{ \$}$ ($\ge 2{,}3\times$) | $38\,000 - 41\,000\text{ \$}$ | $7\,600 - 9\,600\text{ \$}$ | `2_000 $` + `1_500 $` | $200 - 240$ bl. ($800 - 960\text{ uds}$) |
| **`6v6`** (`6`) | $64 \times 64 \times 64$ | `9_500 $` | `14_250 $` (143 bl.) | $\ge 21\,800\text{ \$}$ ($\ge 2{,}3\times$) | $44\,000 - 48\,000\text{ \$}$ | $7\,600 - 9\,600\text{ \$}$ | `2_000 $` + `1_500 $` | $200 - 240$ bl. ($800 - 960\text{ uds}$) |
| **`7v7`** (`7`) | $96 \times 96 \times 64$ | `11_000 $` | `16_500 $` (165 bl.) | $\ge 25\,300\text{ \$}$ ($\ge 2{,}3\times$) | $51\,000 - 55\,000\text{ \$}$ | $12\,000 - 14\,400\text{ \$}$ | `2_000 $` + `1_500 $` | $340 - 400$ bl. ($1\,360 - 1\,600\text{ uds}$) |
| **`8v8`** (`8`) | $96 \times 96 \times 64$ | `12_500 $` | `18_750 $` (188 bl.) | $\ge 28\,700\text{ \$}$ ($\ge 2{,}3\times$) | $58\,000 - 62\,000\text{ \$}$ | $12\,000 - 14\,400\text{ \$}$ | `2_000 $` + `1_500 $` | $340 - 400$ bl. ($1\,360 - 1\,600\text{ uds}$) |

---

## 6. Orden Canónico del Pipeline de Generación

Para garantizar la reproducibilidad entre implementaciones, el generador ejecuta exactamente diez fases secuenciales sobre el volumen $\Omega$:

1. **Fase 1 (Litología base):** relleno estratigráfico por columna $(x, z)$ con `1 dirt` ($y \ge y_{\text{dirt}}$), `2 rock` en el manto intermedio y matriz mixta `2 rock` / `3 hard_rock` en la zona abisal ($y \le 19$).
2. **Fase 2 (Diques de roca dura):** tallado cuasi-espejo de nervaduras de `3 hard_rock` en $y \in [6, 44]$ respetando los corredores de las bases.
3. **Fase 3 (Red de cavernas):** excavación cuasi-espejo ($95\,\% / 5\,\%$) de cámaras elipsoidales de `0 air` en $y \in [16, 50]$.
4. **Fase 4 (Bolsas de lava encapsuladas):** emplazamiento cuasi-espejo de cámaras de `7 lava` en $y \in [4, 28]$ con cáscara protectora de `2 rock` / `3 hard_rock` y exclusión $\ge 16$ bloques respecto a ambas bases.
5. **Fase 5 (Mineralización de `4 ore`, `5 gold` y `6 diamond`):** inyección cuasi-espejo de vetas en corteza, manto y abismo según el presupuesto de la Sección 5.2, asegurando que ningún filón reemplace celdas de `7 lava` ni rompa sus cáscaras de contención.
6. **Fase 6 (Cámaras sísmicas centrales):** estampado de la **Veta Madre** en $(W/2, 30, D/2)$ (`2_000 $`) y de **el Corazón** en $(W/2, 6, D/2)$ (`1_500 $`).
7. **Fase 7 (Bases y pozos maestros):** estampado de las bases `Copper` $(8, 52, D/2)$ y `Cobalt` $(W-9, 52, D/2)$, sus pozos verticales con `10 ladder` ($y = 24..52$) y sus galerías de conexión lateral hacia el manto.
8. **Fase 8 (Sellado de frontera `INV-09`):** imposición de `3 hard_rock` en $y \in \{0, H-1\}$, $x \in \{0, W-1\}$ y $z \in \{0, D-1\}$.
9. **Fase 9 (Verificación de alcanzabilidad BFS):** comprobación 6-conexa desde ambas bases para certificar $G_{\text{reach}} \ge 1{,}5 \times \text{target\_cash}$ y $\Delta_{\text{gold}} \le 10\,\%$.
10. **Fase 10 (Digest criptográfico):** cálculo del hash SHA-256 sobre el slice lineal de `W * D * H` bytes (`idx(x, y, z)`).
