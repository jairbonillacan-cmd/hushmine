//! Registro de últimos lanzamientos por versión: un TSV sencillo en la
//! carpeta `versions/` y el texto de edad relativa para las tarjetas.

use std::fs;
use std::path::{Path, PathBuf};

/// Ruta del registro de últimos lanzamientos (una línea por versión).
pub fn last_played_path(root: &Path) -> PathBuf {
    root.join("versions").join("last_played.tsv")
}

/// Carga el registro de últimos lanzamientos, ordenado del más reciente.
pub fn load_last_played(root: &Path) -> Vec<(String, u64)> {
    fs::read_to_string(last_played_path(root))
        .map(|text| parse_last_played(&text))
        .unwrap_or_default()
}

/// Anota (o actualiza) el último lanzamiento de una versión y lo guarda.
pub fn record_last_played(root: &Path, label: &str, now_secs: u64) {
    let mut entries = load_last_played(root);
    merge_entry(&mut entries, label, now_secs);
    let _ = fs::create_dir_all(root.join("versions"));
    let _ = fs::write(last_played_path(root), serialize_last_played(&entries));
}

/// Inserta o actualiza una entrada y deja las más recientes primero.
pub fn merge_entry(entries: &mut Vec<(String, u64)>, label: &str, now_secs: u64) {
    if let Some(slot) = entries.iter_mut().find(|(l, _)| l == label) {
        slot.1 = now_secs;
    } else {
        entries.push((label.to_owned(), now_secs));
    }
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries.truncate(32);
}

fn parse_last_played(text: &str) -> Vec<(String, u64)> {
    let mut list: Vec<(String, u64)> = text
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .filter_map(|(label, secs)| secs.trim().parse::<u64>().ok().map(|n| (label.into(), n)))
        .collect();
    list.sort_by_key(|e| std::cmp::Reverse(e.1));
    list
}

fn serialize_last_played(entries: &[(String, u64)]) -> String {
    let mut out = String::new();
    for (label, secs) in entries {
        out.push_str(label);
        out.push('\t');
        out.push_str(&secs.to_string());
        out.push('\n');
    }
    out
}

/// Edad relativa en español, para las tarjetas de Instalaciones.
pub fn format_age(now_secs: u64, then_secs: u64) -> String {
    let delta = now_secs.saturating_sub(then_secs);
    if delta < 60 {
        "JUGADO AHORA MISMO".into()
    } else if delta < 3600 {
        format!("JUGADO HACE {} MIN", delta / 60)
    } else if delta < 86_400 {
        format!("JUGADO HACE {} H", delta / 3600)
    } else {
        format!("JUGADO HACE {} D", delta / 86_400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_played_round_trips_and_orders() {
        let mut entries = Vec::new();
        merge_entry(&mut entries, "B", 100);
        merge_entry(&mut entries, "A", 200);
        merge_entry(&mut entries, "B", 300);
        assert_eq!(entries, vec![("B".into(), 300), ("A".into(), 200)]);
        let text = serialize_last_played(&entries);
        assert_eq!(parse_last_played(&text), entries);
        // Líneas corruptas se ignoran sin romper el resto.
        let mixed = format!("basura\n{text}B\tcorreo\n");
        assert_eq!(parse_last_played(&mixed), entries);
    }

    #[test]
    fn age_text_uses_buckets() {
        assert_eq!(format_age(100, 41), "JUGADO AHORA MISMO");
        assert_eq!(format_age(3_600, 60), "JUGADO HACE 59 MIN");
        assert_eq!(format_age(86_400, 60), "JUGADO HACE 23 H");
        assert_eq!(format_age(172_800 * 3, 0), "JUGADO HACE 6 D");
        assert_eq!(format_age(10, 9_999), "JUGADO AHORA MISMO");
    }
}
