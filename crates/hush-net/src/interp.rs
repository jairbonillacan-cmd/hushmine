//! Interpolación de los demás jugadores entre instantáneas.

use std::collections::{BTreeMap, VecDeque};

use grisu_voxel::FIXED_ONE;

use crate::proto::RemotePlayer;

/// Instantáneas que se guardan por jugador (unos 2 s a 30 Hz).
const MAX_SAMPLES: usize = 60;

#[derive(Debug, Default)]
pub struct RemoteTracks {
    latest_tick: u32,
    tracks: BTreeMap<u16, VecDeque<(u32, [i64; 3])>>,
    /// Si cada jugador iba agachado en su última instantánea.
    crouching: BTreeMap<u16, bool>,
}

impl RemoteTracks {
    pub fn push(&mut self, tick: u32, players: &[RemotePlayer]) {
        self.latest_tick = self.latest_tick.max(tick);
        for p in players {
            self.crouching.insert(p.id, p.crouching);
            let track = self.tracks.entry(p.id).or_default();
            track.push_back((tick, p.feet));
            if track.len() > MAX_SAMPLES {
                track.pop_front();
            }
        }
        // Quien no aparece en instantáneas recientes se ha ido.
        let latest = self.latest_tick;
        self.tracks
            .retain(|_, t| t.back().is_some_and(|(last, _)| latest - last < 30));
        let tracks = &self.tracks;
        self.crouching.retain(|id, _| tracks.contains_key(id));
    }

    /// Si el jugador iba agachado en su última instantánea.
    pub fn is_crouching(&self, id: u16) -> bool {
        self.crouching.get(&id).copied().unwrap_or(false)
    }

    /// Posición (pies, en bloques) de cada jugador en el tick `tick`,
    /// interpolada entre las dos instantáneas que lo rodean. En `f64` para no
    /// perder precisión lejos del origen.
    pub fn sample(&self, tick: f64) -> Vec<(u16, [f64; 3])> {
        let to_blocks = |c: i64| c as f64 / FIXED_ONE as f64;
        self.tracks
            .iter()
            .filter_map(|(&id, track)| {
                let after = track.iter().position(|(t, _)| *t as f64 >= tick);
                let pos = match after {
                    None => track.back()?.1.map(to_blocks),
                    Some(0) => track.front()?.1.map(to_blocks),
                    Some(i) => {
                        let (t0, a) = track[i - 1];
                        let (t1, b) = track[i];
                        let f = ((tick - t0 as f64) / (t1 - t0).max(1) as f64).clamp(0.0, 1.0);
                        std::array::from_fn(|k| {
                            to_blocks(a[k]) + (to_blocks(b[k]) - to_blocks(a[k])) * f
                        })
                    }
                };
                Some((id, pos))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_between_snapshots_and_clamps_outside() {
        let mut tracks = RemoteTracks::default();
        let at = |x: i64| RemotePlayer {
            id: 4,
            feet: [x * FIXED_ONE, 0, 0],
            crouching: false,
        };
        tracks.push(10, &[at(0)]);
        tracks.push(12, &[at(2)]);
        assert_eq!(tracks.sample(11.0), vec![(4, [1.0, 0.0, 0.0])]);
        assert_eq!(tracks.sample(5.0)[0].1[0], 0.0);
        assert_eq!(tracks.sample(20.0)[0].1[0], 2.0);
    }

    #[test]
    fn interpolates_far_from_the_origin_without_jumps() {
        let mut tracks = RemoteTracks::default();
        let base = 7_499_800;
        let at = |x: i64| RemotePlayer {
            id: 9,
            feet: [(base + x) * FIXED_ONE, 0, 0],
            crouching: false,
        };
        tracks.push(10, &[at(0)]);
        tracks.push(12, &[at(2)]);
        let mid = tracks.sample(11.0)[0].1[0];
        assert!((mid - (base as f64) - 1.0).abs() < 1e-9, "mid {mid}");
    }

    #[test]
    fn players_missing_for_a_second_are_forgotten() {
        let mut tracks = RemoteTracks::default();
        tracks.push(
            1,
            &[RemotePlayer {
                id: 1,
                feet: [0; 3],
                crouching: false,
            }],
        );
        tracks.push(40, &[]);
        assert!(tracks.sample(40.0).is_empty());
    }
}
