//! Esqueleto por piezas rígidas: modelos, validación y el minero base.
//!
//! Cada modelo es un árbol de piezas rígidas (cajas texturizadas con pivote)
//! según `docs/graficos/02-animacion.md` §2. Las medidas están en píxeles de
//! textura y [`PIXELS_PER_BLOCK`] fija la escala canónica: 32 píxeles = 1
//! bloque. Los índices de las piezas van en orden topológico: todo padre
//! aparece antes que sus hijos.

/// Escala canónica del juego: 32 píxeles de textura = 1 bloque.
pub const PIXELS_PER_BLOCK: f32 = 32.0;

/// Pieza rígida del modelo: caja texturizada que gira alrededor de su pivote.
#[derive(Clone, Debug)]
pub struct PartSpec {
    pub id: String,
    /// Pieza madre; `None` solo en la raíz.
    pub parent: Option<usize>,
    /// Pivote (articulación) en píxeles del espacio del modelo, en reposo.
    pub pivot_px: [f32; 3],
    /// Esquina mínima del cuboide, en el espacio del modelo (no del pivote).
    pub cuboid_min_px: [f32; 3],
    /// Esquina máxima del cuboide, en el espacio del modelo (no del pivote).
    pub cuboid_max_px: [f32; 3],
    /// Esquina superior izquierda de la región de textura de la pieza.
    pub uv: [u32; 2],
}

/// Enganche de un accesorio (lámpara, herramienta, carga) a una pieza.
#[derive(Clone, Debug)]
pub struct AttachmentSpec {
    pub id: String,
    pub part_index: usize,
    /// Desplazamiento respecto al pivote de la pieza, en píxeles.
    pub offset_px: [f32; 3],
}

/// Errores de validación al construir un [`RigModel`].
#[derive(Clone, Debug, PartialEq)]
pub enum RigError {
    /// La cadena de padres vuelve a una pieza ya visitada.
    ParentCycle { part_index: usize },
    /// El padre no existe o no tiene índice menor que el hijo.
    ParentNotBeforeChild {
        part_index: usize,
        parent_index: usize,
    },
    /// Un enganche apunta a una pieza que no existe.
    UnknownAttachmentPart {
        attachment_id: String,
        part_index: usize,
    },
}

/// Modelo validado: árbol de piezas en orden topológico más sus enganches.
#[derive(Clone, Debug)]
pub struct RigModel {
    parts: Vec<PartSpec>,
    attachments: Vec<AttachmentSpec>,
}

impl RigModel {
    /// Valida y construye el modelo: sin ciclos, con padres de índice menor
    /// que el hijo y con todo enganche apuntando a una pieza existente.
    pub fn new(parts: Vec<PartSpec>, attachments: Vec<AttachmentSpec>) -> Result<Self, RigError> {
        if let Some(part_index) = find_parent_cycle(&parts) {
            return Err(RigError::ParentCycle { part_index });
        }
        for (part_index, part) in parts.iter().enumerate() {
            if let Some(parent_index) = part.parent {
                // Un padre fuera de rango también tiene índice mayor que el
                // hijo, así que esta comprobación cubre ambos casos.
                if parent_index >= part_index {
                    return Err(RigError::ParentNotBeforeChild {
                        part_index,
                        parent_index,
                    });
                }
            }
        }
        for attachment in &attachments {
            if attachment.part_index >= parts.len() {
                return Err(RigError::UnknownAttachmentPart {
                    attachment_id: attachment.id.clone(),
                    part_index: attachment.part_index,
                });
            }
        }
        Ok(Self { parts, attachments })
    }

    /// Piezas en orden topológico.
    pub fn parts(&self) -> &[PartSpec] {
        &self.parts
    }

    /// Enganches del modelo.
    pub fn attachments(&self) -> &[AttachmentSpec] {
        &self.attachments
    }

    /// Índice de la pieza con ese identificador (la primera coincidencia).
    pub fn part_index(&self, id: &str) -> Option<usize> {
        self.parts.iter().position(|part| part.id == id)
    }

    /// Índice del enganche con ese identificador.
    pub fn attachment_index(&self, id: &str) -> Option<usize> {
        self.attachments
            .iter()
            .position(|attachment| attachment.id == id)
    }

    /// El minero base: 14 piezas y 3 enganches (`lamp`, `tool` y `back`).
    ///
    /// Convención del espacio: **Y hacia arriba**, pies en `y = 0`, **−Z es
    /// el frente** (la cara) y **+Z la espalda**. Las medidas están en
    /// píxeles de textura; con [`PIXELS_PER_BLOCK`] = 32, la altura total de
    /// 54 px ≈ 1,7 bloques casa con la altura de colisión del jugador en
    /// `hush-sim` (`HEIGHT = 17/10` bloques), de modo que la silueta calza
    /// con la hitbox. Referencias de proporción: piernas de 24 px
    /// (0,75 bloques) con pivotes de cadera a `y = 24`; hombros a `y = 42`;
    /// ojos ≈ `y = 49`, frente a `EYE_HEIGHT = 1,55` bloques = 49,6 px del
    /// simulador.
    pub fn miner_default() -> Self {
        // El modelo integrado es válido por construcción; la prueba
        // `miner_default_is_topological_and_rebuildable` lo pasa siempre por
        // las mismas reglas que un modelo cargado de disco.
        Self {
            parts: miner_parts(),
            attachments: miner_attachments(),
        }
    }
}

/// Busca un ciclo entre los padres.
///
/// Recorre la cadena de padres de cada pieza con `Vec::get` (nunca indexa
/// directo) y con un tope de `parts.len()` pasos: una cadena sin ciclo visita
/// cada pieza a lo sumo una vez, así que con una visita más todo regreso cae
/// dentro del recorrido. Devuelve la pieza donde se detectó el regreso.
fn find_parent_cycle(parts: &[PartSpec]) -> Option<usize> {
    let mut visited = vec![false; parts.len()];
    for start in 0..parts.len() {
        visited.fill(false);
        let mut current = start;
        for _ in 0..=parts.len() {
            if visited[current] {
                return Some(current);
            }
            visited[current] = true;
            let Some(parent) = parts.get(current).and_then(|part| part.parent) else {
                break; // Raíz de la cadena.
            };
            if parts.get(parent).is_none() {
                break; // Fuera de rango: lo reporta la validación de orden.
            }
            current = parent;
        }
    }
    None
}

/// Construye una pieza del minero con sus medidas en píxeles del modelo.
fn miner_part(
    id: &str,
    parent: Option<usize>,
    pivot_px: [f32; 3],
    cuboid_min_px: [f32; 3],
    cuboid_max_px: [f32; 3],
    uv: [u32; 2],
) -> PartSpec {
    PartSpec {
        id: id.into(),
        parent,
        pivot_px,
        cuboid_min_px,
        cuboid_max_px,
        uv,
    }
}

/// Las 14 piezas del minero, ya en orden topológico (todo padre antes que
/// sus hijos).
fn miner_parts() -> Vec<PartSpec> {
    vec![
        miner_part(
            "pelvis",
            None,
            [0.0, 27.0, 0.0],
            [-5.0, 24.0, -3.0],
            [5.0, 30.0, 3.0],
            [0, 32],
        ),
        miner_part(
            "torso",
            Some(0),
            [0.0, 30.0, 0.0],
            [-5.0, 30.0, -3.0],
            [5.0, 44.0, 3.0],
            [0, 0],
        ),
        miner_part(
            "head",
            Some(1),
            [0.0, 44.0, 0.0],
            [-5.0, 44.0, -5.0],
            [5.0, 54.0, 5.0],
            [40, 0],
        ),
        miner_part(
            "upper_arm_l",
            Some(1),
            [-7.0, 42.0, 0.0],
            [-9.0, 30.0, -2.0],
            [-5.0, 42.0, 2.0],
            [0, 48],
        ),
        miner_part(
            "upper_arm_r",
            Some(1),
            [7.0, 42.0, 0.0],
            [5.0, 30.0, -2.0],
            [9.0, 42.0, 2.0],
            [20, 48],
        ),
        miner_part(
            "lower_arm_l",
            Some(3),
            [-7.0, 30.0, 0.0],
            [-9.0, 18.0, -2.0],
            [-5.0, 30.0, 2.0],
            [0, 64],
        ),
        miner_part(
            "lower_arm_r",
            Some(4),
            [7.0, 30.0, 0.0],
            [5.0, 18.0, -2.0],
            [9.0, 30.0, 2.0],
            [20, 64],
        ),
        miner_part(
            "hand_l",
            Some(5),
            [-7.0, 18.0, 0.0],
            [-9.0, 12.0, -2.0],
            [-5.0, 18.0, 2.0],
            [0, 80],
        ),
        miner_part(
            "hand_r",
            Some(6),
            [7.0, 18.0, 0.0],
            [5.0, 12.0, -2.0],
            [9.0, 18.0, 2.0],
            [20, 80],
        ),
        miner_part(
            "upper_leg_l",
            Some(0),
            [-2.5, 24.0, 0.0],
            [-5.0, 12.0, -2.5],
            [0.0, 24.0, 2.5],
            [40, 48],
        ),
        miner_part(
            "upper_leg_r",
            Some(0),
            [2.5, 24.0, 0.0],
            [0.0, 12.0, -2.5],
            [5.0, 24.0, 2.5],
            [64, 48],
        ),
        miner_part(
            "lower_leg_l",
            Some(9),
            [-2.5, 12.0, 0.0],
            [-5.0, 0.0, -2.5],
            [0.0, 12.0, 2.5],
            [40, 68],
        ),
        miner_part(
            "lower_leg_r",
            Some(10),
            [2.5, 12.0, 0.0],
            [0.0, 0.0, -2.5],
            [5.0, 12.0, 2.5],
            [64, 68],
        ),
        miner_part(
            "pack",
            Some(1),
            [0.0, 44.0, 3.0],
            [-4.0, 32.0, 3.0],
            [4.0, 44.0, 7.0],
            [40, 20],
        ),
    ]
}

/// Los 3 enganches del minero: lámpara del casco, herramienta en la mano
/// derecha y carga a la espalda.
fn miner_attachments() -> Vec<AttachmentSpec> {
    vec![
        // Lámpara del casco, al frente (−Z).
        AttachmentSpec {
            id: "lamp".into(),
            part_index: 2, // head
            offset_px: [0.0, 6.0, -6.0],
        },
        // Herramienta en la mano derecha (el ejemplo RON del doc).
        AttachmentSpec {
            id: "tool".into(),
            part_index: 8, // hand_r
            offset_px: [0.0, -2.0, -1.0],
        },
        // Enganche de carga a la espalda (+Z).
        AttachmentSpec {
            id: "back".into(),
            part_index: 1, // torso
            offset_px: [0.0, 8.0, 4.0],
        },
    ]
}

#[cfg(test)]
#[path = "skeleton_tests.rs"]
mod tests;
