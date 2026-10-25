//! Geometría del lanzador: una sola función coloca todos los rectángulos para
//! que el dibujo y el ratón nunca discrepen.
//!
//! A la izquierda, la pared de la caseta (rótulo, menú, estado y salir). A
//! la derecha, el contenido de la página en una columna centrada.

use crate::state::{LauncherState, PAGES, Page};
use crate::theme::{GUTTER, NAV_ROW, TEXT_HEADING, TEXT_TITLE, WALL_FRACTION};
use grisu_render::{UiBatch, UiRect};

/// Rectángulo pulsable de la pared.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub rect: UiRect,
    pub slot: HitSlot,
}

/// Objetivo de un rectángulo pulsable de la pared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitSlot {
    SidebarPage(Page),
    Exit,
}

/// Elemento de página: rectángulo más índice lógico (fila o versión).
#[derive(Clone, Copy, Debug)]
pub struct ItemSlot {
    pub rect: UiRect,
    pub item: usize,
}

/// Todos los rectángulos de un fotograma, ya resueltos al tamaño de ventana.
pub struct Layout {
    /// Pared de la caseta.
    pub wall: UiRect,
    /// Posición del rótulo HUSHMINE (esquina superior izquierda).
    pub title: (f32, f32),
    /// Entradas del menú y Salir, de arriba abajo.
    pub sidebar: Vec<Hit>,
    /// Línea de estado, encima de Salir.
    pub status: (f32, f32),
    /// Columna del contenido de la página.
    pub content: UiRect,
    /// Monitor de fósforo de la página Jugar (vacío en las demás).
    pub monitor: UiRect,
    pub items: Vec<ItemSlot>,
}

impl Layout {
    /// Objetivo de la pared bajo el punto, si lo hay.
    pub fn sidebar_at(&self, x: f32, y: f32) -> Option<HitSlot> {
        self.sidebar
            .iter()
            .find(|h| h.rect.contains(x, y))
            .map(|h| h.slot)
    }

    /// Índice lógico del elemento bajo el punto, si lo hay.
    pub fn item_at(&self, x: f32, y: f32) -> Option<usize> {
        self.items
            .iter()
            .find(|s| s.rect.contains(x, y))
            .map(|s| s.item)
    }

    pub fn item_rect(&self, item: usize) -> Option<UiRect> {
        self.items.iter().find(|s| s.item == item).map(|s| s.rect)
    }
}

/// Ancho máximo de la columna de contenido.
const COLUMN_MAX: f32 = 520.0;
/// Alto del encabezado de las páginas con título (todas menos Jugar).
pub const HEADING_BLOCK: f32 = TEXT_HEADING * 7.0 + 52.0;
/// Alto de una fila de versión.
const VERSION_ROW: f32 = 72.0;
/// Alto de una fila de opciones.
const OPTION_ROW: f32 = 52.0;
/// Alto de la fila del selector de versión en Jugar.
const SELECTOR_H: f32 = 40.0;
/// Alto del botón principal.
const PLAY_H: f32 = 60.0;
/// Alto de los enlaces secundarios.
const LINK_H: f32 = 30.0;

/// Coloca todos los rectángulos para una ventana de `width`×`height`.
pub fn layout(state: &LauncherState, width: f32, height: f32) -> Layout {
    let w = width.max(640.0);
    let h = height.max(400.0);
    let wall_w = (w * WALL_FRACTION).clamp(330.0, 520.0);
    let wall = UiRect::new(0.0, 0.0, wall_w, h, [0.0; 4]);
    let title = (GUTTER, GUTTER);
    let nav_top = GUTTER + TEXT_TITLE * 7.0 + 96.0;
    let sidebar = wall_hits(wall_w, nav_top, h);
    let status = (GUTTER, h - GUTTER - NAV_ROW - 30.0);

    let cx = wall_w + GUTTER;
    let cy = GUTTER;
    let cw = (w - wall_w - 2.0 * GUTTER).max(0.0);
    let ch = h - 2.0 * GUTTER;
    let col_w = cw.min(COLUMN_MAX);
    let col_x = cx + (cw - col_w) * 0.5;

    let mut monitor = UiRect::new(col_x, cy, 0.0, 0.0, [0.0; 4]);
    let items = match state.page() {
        Page::Play => {
            // Monitor, selector, botón principal y enlaces, centrados en
            // vertical como un solo bloque.
            let below = 18.0 + SELECTOR_H + 14.0 + PLAY_H + 14.0 + LINK_H;
            let mon_h = (col_w * 0.56).min(ch - below - 24.0).max(120.0);
            let top = cy + ((ch - mon_h - below) * 0.5).max(0.0);
            monitor = UiRect::new(col_x, top, col_w, mon_h, [0.0; 4]);
            let selector_y = top + mon_h + 18.0;
            let play_y = selector_y + SELECTOR_H + 14.0;
            let links_y = play_y + PLAY_H + 14.0;
            let half = col_w * 0.5;
            vec![
                slot(col_x, selector_y, col_w, SELECTOR_H, 0),
                slot(col_x, play_y, col_w, PLAY_H, 1),
                slot(col_x + half, links_y, half, LINK_H, 2),
                slot(col_x, links_y, half, LINK_H, 3),
            ]
        }
        Page::Instalaciones => {
            let top = cy + HEADING_BLOCK;
            let total = state.item_count(Page::Instalaciones);
            let visible = (((cy + ch - top) / VERSION_ROW) as usize).max(1);
            let selected = state.selected();
            let first = selected
                .saturating_sub(visible.saturating_sub(1))
                .min(total.saturating_sub(1));
            (first..total)
                .take(visible)
                .map(|item| {
                    let y = top + (item - first) as f32 * VERSION_ROW;
                    slot(col_x, y, col_w, VERSION_ROW, item)
                })
                .collect()
        }
        Page::Opciones => {
            let top = cy + HEADING_BLOCK;
            (0..state.item_count(Page::Opciones))
                .map(|item| {
                    slot(
                        col_x,
                        top + item as f32 * OPTION_ROW,
                        col_w,
                        OPTION_ROW,
                        item,
                    )
                })
                .collect()
        }
        Page::Acerca => Vec::new(),
    };

    Layout {
        wall,
        title,
        sidebar,
        status,
        content: UiRect::new(col_x, cy, col_w, ch, [0.0; 4]),
        monitor,
        items,
    }
}

fn slot(x: f32, y: f32, w: f32, h: f32, item: usize) -> ItemSlot {
    ItemSlot {
        rect: UiRect::new(x, y, w, h, [0.0; 4]),
        item,
    }
}

/// Entradas del menú de la pared y Salir al pie.
fn wall_hits(wall_w: f32, nav_top: f32, height: f32) -> Vec<Hit> {
    let x = GUTTER - 16.0;
    let w = wall_w - 2.0 * GUTTER + 32.0;
    let mut hits: Vec<Hit> = PAGES
        .iter()
        .enumerate()
        .map(|(idx, page)| Hit {
            rect: UiRect::new(x, nav_top + idx as f32 * NAV_ROW, w, NAV_ROW, [0.0; 4]),
            slot: HitSlot::SidebarPage(*page),
        })
        .collect();
    hits.push(Hit {
        rect: UiRect::new(x, height - GUTTER - NAV_ROW, w, NAV_ROW, [0.0; 4]),
        slot: HitSlot::Exit,
    });
    hits
}

/// Parte superior de una línea de texto centrada en vertical en `rect`.
pub fn line_top(rect: &UiRect, scale: f32) -> f32 {
    rect.rect[1] + ((rect.rect[3] - UiBatch::text_height(scale)) * 0.5).round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::sample_versions_for_tests;

    fn state(page: Page) -> LauncherState {
        let mut s = LauncherState::new(sample_versions_for_tests(), Vec::new(), 0);
        s.select_page(page);
        s
    }

    fn inside(inner: &UiRect, outer: &UiRect) -> bool {
        let [x, y, w, h] = inner.rect;
        let [ox, oy, ow, oh] = outer.rect;
        x >= ox - 0.5 && y >= oy - 0.5 && x + w <= ox + ow + 0.5 && y + h <= oy + oh + 0.5
    }

    #[test]
    fn wall_has_page_entries_and_exit_at_the_bottom() {
        let lay = layout(&state(Page::Play), 1024.0, 640.0);
        assert_eq!(lay.sidebar.len(), PAGES.len() + 1);
        assert_eq!(lay.sidebar[0].slot, HitSlot::SidebarPage(Page::Play));
        assert_eq!(lay.sidebar[3].slot, HitSlot::SidebarPage(Page::Acerca));
        assert_eq!(lay.sidebar[4].slot, HitSlot::Exit);
        let exit = lay.sidebar[4].rect;
        let last_page = lay.sidebar[3].rect;
        assert!(exit.rect[1] > last_page.rect[1] + last_page.rect[3]);
        // Todo dentro de la pared.
        for hit in &lay.sidebar {
            assert!(hit.rect.rect[0] + hit.rect.rect[2] <= lay.wall.rect[2]);
        }
    }

    #[test]
    fn play_page_stacks_monitor_selector_button_and_links() {
        for (w, h) in [(1024.0, 640.0), (1920.0, 1080.0), (800.0, 500.0)] {
            let lay = layout(&state(Page::Play), w, h);
            assert_eq!(lay.items.len(), 4);
            let selector = lay.item_rect(0).expect("selector");
            let play = lay.item_rect(1).expect("jugar");
            let compile = lay.item_rect(2).expect("compilar");
            let server = lay.item_rect(3).expect("servidor");
            let mon = lay.monitor;
            assert!(selector.rect[1] >= mon.rect[1] + mon.rect[3], "{w}x{h}");
            assert!(play.rect[1] >= selector.rect[1] + selector.rect[3]);
            assert!(compile.rect[1] >= play.rect[1] + play.rect[3]);
            assert_eq!(compile.rect[1], server.rect[1]);
            for r in [mon, selector, play, compile, server] {
                assert!(inside(&r, &lay.content), "{w}x{h}: {r:?} fuera");
            }
            assert!(lay.content.rect[0] >= lay.wall.rect[2]);
            let c = (play.rect[0] + 4.0, play.rect[1] + 4.0);
            assert_eq!(lay.item_at(c.0, c.1), Some(1));
            assert_eq!(lay.item_at(2.0, 2.0), None);
        }
    }

    #[test]
    fn version_rows_scroll_to_keep_selection_visible() {
        let mut s = state(Page::Instalaciones);
        assert_eq!(layout(&s, 1024.0, 640.0).items.len(), 3);
        for _ in 0..10 {
            s.versions.push(s.versions[0].clone());
        }
        for _ in 0..12 {
            s.move_selection(1);
        }
        let lay = layout(&s, 1024.0, 640.0);
        assert!(lay.items.iter().any(|slot| slot.item == 12));
        for slot in &lay.items {
            assert!(inside(&slot.rect, &lay.content));
        }
    }

    #[test]
    fn option_rows_fit_inside_content() {
        let lay = layout(&state(Page::Opciones), 1024.0, 640.0);
        assert_eq!(lay.items.len(), 5);
        for slot in &lay.items {
            assert!(inside(&slot.rect, &lay.content));
        }
    }

    #[test]
    fn about_page_has_no_items() {
        assert!(layout(&state(Page::Acerca), 1024.0, 640.0).items.is_empty());
    }
}
