//! Pruebas de `controls`.

use super::*;

#[test]
fn rebind_swaps_duplicate_keys_and_round_trips() {
    let mut c = Controls::default();
    c.set(BindAction::Forward, KeyCode::KeyS);
    assert_eq!(c.forward, KeyCode::KeyS);
    assert_eq!(c.back, KeyCode::KeyW);
    c.sprint_mode = SprintMode::Toggle;
    c.adjust_sensitivity(30);
    c.adjust_fov(15);
    c.adjust_third_person_dist(1.0);
    let text = c.serialize();
    assert_eq!(Controls::deserialize(&text), c);
}

#[test]
fn bad_values_in_the_file_fall_back_or_clamp() {
    let c =
        Controls::deserialize("forward=NOPE\nsensitivity=9999\nfov=10\nthird_person=NaN\nbasura\n");
    assert_eq!(c.forward, KeyCode::KeyW);
    assert_eq!(c.sensitivity_pct, 300);
    assert_eq!(c.fov_deg, 60);
    assert_eq!(c.third_person_dist, Controls::default().third_person_dist);
    let far = Controls::deserialize("third_person=40\n");
    assert_eq!(far.third_person_dist, 6.0);
}

#[test]
fn every_bindable_key_has_a_readable_name() {
    for &(key, file_label) in KEY_TABLE {
        let name = key_name(key);
        assert!(!name.is_empty());
        // El nombre visible nunca va todo en mayúsculas salvo letras y F.
        if file_label.len() > 3 {
            assert_ne!(name, name.to_uppercase(), "{file_label}");
        }
    }
}
