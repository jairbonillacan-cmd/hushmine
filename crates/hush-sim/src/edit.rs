//! Cavar y poner bloques: validación en enteros.
//!
//! El cliente apunta con un raycast y propone la celda; aquí se decide si la
//! acción es legal. Nunca se hace `panic!` con datos que vienen de fuera.

use grisu_voxel::{AIR, FIXED_ONE, VoxelWorld};
use serde::{Deserialize, Serialize};

use crate::{BlockKind, PlayerBody};

/// Alcance máximo desde los ojos hasta el centro de la celda: 5 bloques.
pub const REACH: i64 = 5 * FIXED_ONE as i64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditError {
    OutOfBounds,
    /// Los bordes y el fondo del mapa no se pueden cavar.
    Boundary,
    TooFar,
    NotDiggable,
    Occupied,
    /// Hay un jugador ocupando la celda.
    BlockedByPlayer,
    InvalidBlock,
    UnknownPlayer,
}

/// Comprueba que la celda está al alcance de los ojos del jugador.
fn check_reach(body: &PlayerBody, cell: [i32; 3]) -> Result<(), EditError> {
    let eye = body.eye();
    let center = cell.map(|c| i64::from(c) * i64::from(FIXED_ONE) + i64::from(FIXED_ONE) / 2);
    let d2: i64 = (0..3).map(|i| (center[i] - i64::from(eye[i])).pow(2)).sum();
    if d2 > REACH * REACH {
        Err(EditError::TooFar)
    } else {
        Ok(())
    }
}

fn is_boundary(world: &VoxelWorld, cell: [i32; 3]) -> bool {
    let [sx, _, sz] = world.size().map(|s| s as i32);
    cell[1] == 0 || cell[0] == 0 || cell[2] == 0 || cell[0] == sx - 1 || cell[2] == sz - 1
}

/// Cava la celda y devuelve el bloque que había.
pub fn dig(
    world: &mut VoxelWorld,
    body: &PlayerBody,
    cell: [i32; 3],
) -> Result<BlockKind, EditError> {
    if !world.in_bounds(cell[0], cell[1], cell[2]) {
        return Err(EditError::OutOfBounds);
    }
    if is_boundary(world, cell) {
        return Err(EditError::Boundary);
    }
    check_reach(body, cell)?;
    let block = BlockKind::from_id(world.get(cell[0], cell[1], cell[2]))
        .map_err(|_| EditError::InvalidBlock)?;
    if !block.is_diggable() {
        return Err(EditError::NotDiggable);
    }
    world.set(cell[0], cell[1], cell[2], AIR);
    Ok(block)
}

/// Pone un bloque en una celda vacía que no ocupe ningún jugador.
pub fn place(
    world: &mut VoxelWorld,
    body: &PlayerBody,
    others: &[PlayerBody],
    cell: [i32; 3],
    block: BlockKind,
) -> Result<(), EditError> {
    if block == BlockKind::Air || block == BlockKind::Lava {
        return Err(EditError::InvalidBlock);
    }
    if !world.in_bounds(cell[0], cell[1], cell[2]) {
        return Err(EditError::OutOfBounds);
    }
    check_reach(body, cell)?;
    if world.is_solid(cell[0], cell[1], cell[2]) {
        return Err(EditError::Occupied);
    }
    if others
        .iter()
        .chain([body])
        .any(|p| p.aabb().overlaps_cell(cell))
    {
        return Err(EditError::BlockedByPlayer);
    }
    world.set(cell[0], cell[1], cell[2], block.id());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::spawn_point;

    fn setup() -> (VoxelWorld, PlayerBody) {
        let mut w = VoxelWorld::new([32, 16, 32]).unwrap();
        w.fill([0, 0, 0], [31, 3, 31], BlockKind::Rock.id());
        w.set(9, 3, 8, BlockKind::Lava.id());
        let body = PlayerBody::new(spawn_point(&w, 8, 8));
        (w, body)
    }

    #[test]
    fn digs_nearby_rock() {
        let (mut w, body) = setup();
        assert_eq!(dig(&mut w, &body, [8, 3, 8]), Ok(BlockKind::Rock));
        assert!(!w.is_solid(8, 3, 8));
        assert_eq!(dig(&mut w, &body, [8, 3, 8]), Err(EditError::NotDiggable));
    }

    #[test]
    fn rejects_far_boundary_lava_and_outside() {
        let (mut w, body) = setup();
        assert_eq!(dig(&mut w, &body, [20, 3, 8]), Err(EditError::TooFar));
        assert_eq!(dig(&mut w, &body, [8, 0, 8]), Err(EditError::Boundary));
        assert_eq!(dig(&mut w, &body, [9, 3, 8]), Err(EditError::NotDiggable));
        assert_eq!(dig(&mut w, &body, [8, -1, 8]), Err(EditError::OutOfBounds));
        assert_eq!(
            dig(&mut w, &body, [i32::MAX, 3, 8]),
            Err(EditError::OutOfBounds)
        );
    }

    #[test]
    fn places_but_not_inside_players() {
        let (mut w, body) = setup();
        assert_eq!(
            place(&mut w, &body, &[], [10, 4, 8], BlockKind::Dirt),
            Ok(())
        );
        assert_eq!(
            place(&mut w, &body, &[], [10, 4, 8], BlockKind::Dirt),
            Err(EditError::Occupied)
        );
        assert_eq!(
            place(&mut w, &body, &[], [8, 4, 8], BlockKind::Dirt),
            Err(EditError::BlockedByPlayer)
        );
        let other = PlayerBody::new(spawn_point(&w, 6, 8));
        assert_eq!(
            place(&mut w, &body, &[other], [6, 5, 8], BlockKind::Dirt),
            Err(EditError::BlockedByPlayer)
        );
        assert_eq!(
            place(&mut w, &body, &[], [7, 4, 8], BlockKind::Lava),
            Err(EditError::InvalidBlock)
        );
    }
}
