//! Pruebas de la caché LRU determinista de chunks en estado `Base`.

use crate::lru::BaseChunkCache;
use grisu_voxel::ChunkPos;

/// Posición auxiliar para escribir las pruebas con coordenadas cortas.
fn pos(x: i32, y: i32, z: i32) -> ChunkPos {
    ChunkPos::new(x, y, z)
}

#[test]
fn expulsa_al_menos_usado_recientemente() {
    let a = pos(0, 0, 0);
    let b = pos(1, 0, 0);
    let c = pos(2, 0, 0);
    let mut cache = BaseChunkCache::new(2);
    cache.insert(a, 1);
    cache.insert(b, 2);
    // Refrescar `a` deja a `b` como el menos usado recientemente.
    assert_eq!(cache.get(a), Some(&1));
    assert_eq!(cache.insert(c, 3), Some((b, 2)));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(a), Some(&1));
    assert_eq!(cache.get(c), Some(&3));
    assert_eq!(cache.get(b), None);
}

#[test]
fn get_refresca_y_insertar_no_expulsa_al_reciente() {
    let a = pos(0, 0, 0);
    let b = pos(1, 0, 0);
    let c = pos(2, 0, 0);
    let mut cache = BaseChunkCache::new(2);
    cache.insert(a, 1);
    cache.insert(b, 2);
    // Refrescar `b` deja a `a` como el más viejo.
    assert_eq!(cache.get(b), Some(&2));
    assert_eq!(cache.insert(c, 3), Some((a, 1)));
    assert_eq!(cache.get(b), Some(&2));
    assert_eq!(cache.get(c), Some(&3));
    assert_eq!(cache.get(a), None);
}

#[test]
fn reemplazar_no_expulsa() {
    let a = pos(0, 0, 0);
    let b = pos(1, 0, 0);
    let c = pos(2, 0, 0);
    let mut cache = BaseChunkCache::new(2);
    cache.insert(a, 1);
    cache.insert(b, 2);
    // Reemplazar refresca la marca de `a` y no expulsa a nadie.
    assert_eq!(cache.insert(a, 10), None);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(a), Some(&10));
    // La marca refrescada salva a `a`: la expulsada es `b`.
    assert_eq!(cache.insert(c, 3), Some((b, 2)));
    assert_eq!(cache.get(a), Some(&10));
    assert_eq!(cache.get(c), Some(&3));
    assert_eq!(cache.get(b), None);
}

#[test]
fn remove_saca_y_devuelve() {
    let a = pos(0, 0, 0);
    let b = pos(1, 0, 0);
    let c = pos(2, 0, 0);
    let d = pos(3, 0, 0);
    let mut cache = BaseChunkCache::new(2);
    cache.insert(a, 1);
    assert_eq!(cache.remove(a), Some(1));
    assert!(cache.is_empty());
    assert_eq!(cache.remove(a), None);
    // `a` ya no existe: al llenar la caché se expulsa a `b`, una entrada
    // real, nunca a la posición eliminada.
    cache.insert(b, 2);
    cache.insert(c, 3);
    assert_eq!(cache.insert(d, 4), Some((b, 2)));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(a), None);
    assert_eq!(cache.get(c), Some(&3));
    assert_eq!(cache.get(d), Some(&4));
}

#[test]
fn capacidad_cero_no_guarda_nada() {
    let a = pos(0, 0, 0);
    let mut cache = BaseChunkCache::new(0);
    assert_eq!(cache.capacity(), 0);
    assert_eq!(cache.insert(a, 7), Some((a, 7)));
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.get(a), None);
}

#[test]
fn len_y_capacity_se_reflejan() {
    let a = pos(0, 0, 0);
    let b = pos(1, 0, 0);
    let mut cache = BaseChunkCache::new(3);
    assert_eq!(cache.capacity(), 3);
    cache.insert(a, 1);
    cache.insert(b, 2);
    cache.insert(pos(2, 0, 0), 3);
    assert_eq!(cache.len(), 3);
    assert!(cache.len() <= cache.capacity());
    // Insertar más allá de la capacidad expulsa sin pasarse del máximo.
    cache.insert(pos(3, 0, 0), 4);
    assert_eq!(cache.len(), 3);
    assert!(cache.len() <= cache.capacity());
    assert_eq!(cache.get(a), None);
    cache.remove(b);
    assert_eq!(cache.len(), 2);
    assert!(cache.len() <= cache.capacity());
}

#[test]
fn expulsiones_encadenadas_son_deterministas() {
    let a = pos(0, 0, 0);
    let b = pos(1, 0, 0);
    let c = pos(2, 0, 0);
    let d = pos(3, 0, 0);
    let mut cache = BaseChunkCache::new(2);
    cache.insert(a, 1);
    cache.insert(b, 2);
    // Sin `get` intermedios, las expulsiones siguen el orden de inserción.
    assert_eq!(cache.insert(c, 3), Some((a, 1)));
    assert_eq!(cache.insert(d, 4), Some((b, 2)));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(c), Some(&3));
    assert_eq!(cache.get(d), Some(&4));
}
