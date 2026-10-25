//! Lo que la ventana del modo Historia añade a la vista del HUD: pausa,
//! menús, avisos y telemetría.

use super::*;

impl StoryApp {
    /// Vista del HUD de este fotograma.
    pub(super) fn hud_view(&self) -> HudView {
        let mut view = HudView::from_expedition(&self.expedition, self.target);
        view.paused = self.paused;
        view.f3_open = self.f3_open;
        view.craft_open = self.craft.is_open();
        view.craft_rows = self.craft.rows(&self.expedition);
        view.craft_selected = self.craft.selected();
        view.banner = self.banner;
        view.mesh_faces = self.last_mesh.faces;
        view.mesh_millis = self.last_mesh.millis as f32;
        let total = self.expedition.world().chunk_count();
        view.mesh_chunks = self
            .terrain
            .as_ref()
            .map_or((0, total), |t| (t.meshed_chunks(), total));
        view.fps = self.fps.stats();
        view.facing = self.facing();
        view.pitch = self.camera.pitch.to_degrees();
        view.yaw = self.camera.yaw.to_degrees().rem_euclid(360.0);
        view
    }

    /// Rumbo cardinal dominante desde la guiñada de la cámara.
    fn facing(&self) -> &'static str {
        // La guiñada 0 mira a -Z (norte) y crece hacia -X (oeste).
        match self.camera.yaw.to_degrees().rem_euclid(360.0) {
            d if !(45.0..315.0).contains(&d) => "norte",
            d if d < 135.0 => "oeste",
            d if d < 225.0 => "sur",
            _ => "este",
        }
    }
}
