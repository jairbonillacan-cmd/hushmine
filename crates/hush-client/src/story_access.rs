//! Accesores del modo Historia que usa el HUD (story_hud).

use super::*;

// Accesores que usa el HUD (`story_hud`).
impl StoryApp {
    pub fn expedition(&self) -> &Expedition {
        &self.expedition
    }

    pub fn craft_menu(&self) -> &CraftMenu {
        &self.craft
    }

    pub fn craft_rows(&self) -> Vec<(String, bool)> {
        self.craft.rows(&self.expedition)
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn f3_open(&self) -> bool {
        self.f3_open
    }

    pub fn banner(&self) -> Option<(&'static str, f32)> {
        self.banner
    }

    /// (fps, 1% bajo, frametime ms).
    pub fn fps_stats(&self) -> (f32, f32, f32) {
        self.fps.stats()
    }

    /// Rumbo cardinal dominante desde la guiñada de la cámara.
    pub fn facing(&self) -> &'static str {
        // yaw 0 mira a -Z (norte); crece hacia -X (oeste).
        let deg = self.camera.yaw.to_degrees().rem_euclid(360.0);
        match deg {
            d if d < 45.0 || d >= 315.0 => "NORTE (-Z)",
            d if d < 135.0 => "OESTE (-X)",
            d if d < 225.0 => "SUR (+Z)",
            _ => "ESTE (+X)",
        }
    }

    pub fn pitch_deg(&self) -> f32 {
        self.camera.pitch.to_degrees()
    }

    pub fn yaw_deg(&self) -> f32 {
        self.camera.yaw.to_degrees().rem_euclid(360.0)
    }

    pub fn mesh_stats(&self) -> MeshStats {
        self.last_mesh
    }

    pub fn chunk_stats(&self) -> (usize, usize) {
        let total = self.expedition.world().chunk_count();
        self.terrain
            .as_ref()
            .map_or((0, total), |t| (t.meshed_chunks(), total))
    }

    pub fn target(&self) -> Option<RayHit> {
        self.target
    }
}
