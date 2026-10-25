//! Pruebas de `main`.

use super::world_chunk::{parse_chunk, sha256_hex};
use super::world_img::{MapLayer, SliceAxis, parse_pair, render_map, render_slice};

#[test]
fn parse_pair_accepts_two_i32_and_rejects_more() {
    assert_eq!(parse_pair("0,0"), Some((0, 0)));
    assert_eq!(parse_pair("-468750,468749"), Some((-468750, 468749)));
    assert_eq!(parse_pair("1,2,3"), None);
}

#[test]
fn parse_chunk_accepts_three_i32_and_rejects_two() {
    assert_eq!(
        parse_chunk("-468750,-1,468749"),
        Some((-468750, -1, 468749))
    );
    assert_eq!(parse_chunk("1,2"), None);
}

#[test]
fn sha256_hex_matches_fips_vector() {
    let hex = sha256_hex(b"abc");
    assert_eq!(hex.len(), 64);
    assert_eq!(
        hex,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn render_map_returns_size_by_size() {
    assert_eq!(
        render_map(42, (0, 0), 8, MapLayer::Height).dimensions(),
        (8, 8)
    );
}

#[test]
fn render_slice_dimensions_match_range() {
    assert_eq!(
        render_slice(42, SliceAxis::X, 0, (0, 0), (15, 8)).dimensions(),
        (16, 9)
    );
}
