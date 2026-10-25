//! Geometría del panel: una sola función coloca todos los rectángulos para
//! que el dibujo y el ratón nunca discrepen.

use crate::state::{LauncherState, PAGES, Page};
use crate::theme::{LATO_ANCHO, MARGEN};
use grisu_render::UiRect;

/// Rectángulo lateral pulsable.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub rect: UiRect,
    pub slot: HitSlot,
}

/// Objetivo de un rectángulo pulsable.
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
    pub sidebar_bg: UiRect,
    pub sidebar: Vec<Hit>,
    pub content: UiRect,
    pub header: UiRect,
    pub items: Vec<ItemSlot>,
    pub summary: UiRect,
    pub status: UiRect,
}

impl Layout {
    /// Objetivo lateral bajo el punto, si lo hay.
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

/// Altura de la cabecera de página dentro del área de contenido.
const HEADER_H: f32 = 56.0;
/// Altura de la franja de estado inferior.
const STATUS_H: f32 = 42.0;

/// Coloca todos los rectángulos para una ventana de `width`×`height`.
pub fn layout(state: &LauncherState, width: f32, height: f32) -> Layout {
    let w = width.max(320.0);
    let h = height.max(240.0);
    let sidebar_bg = UiRect::new(MARGEN, MARGEN, LATO_ANCHO, h - 2.0 * MARGEN, [0.0; 4]);
    let sidebar = sidebar_hits(&sidebar_bg, h);

    let content_x = MARGEN + LATO_ANCHO + MARGEN;
    let content = UiRect::new(
        content_x,
        MARGEN,
        (w - content_x - MARGEN).max(0.0),
        h - 2.0 * MARGEN,
        [0.0; 4],
    );
    let header = UiRect::new(
        content.rect[0],
        content.rect[1],
        content.rect[2],
        HEADER_H,
        [0.0; 4],
    );
    let status = UiRect::new(
        content.rect[0],
        content.rect[1] + content.rect[3] - STATUS_H,
        content.rect[2],
        STATUS_H,
        [0.0; 4],
    );

    let items_top = header.rect[1] + HEADER_H + 14.0;
    let items_h = (status.rect[1] - items_top - 10.0).max(0.0);
    let cw = content.rect[2];
    let items = match state.page() {
        Page::Play => {
            let big_x = content.rect[0] + cw * 0.15;
            let big_w = cw * 0.70;
            // Botones anclados al pie, como manda el panel: JUGAR bien grande
            // y debajo de nada más que la franja de estado.
            // Pila anclada al pie, de abajo a arriba: JUGAR (el grande),
            // COMPILAR y SERVIDOR.
            let play_y = status.rect[1] - 12.0 - 88.0;
            let compile_y = play_y - 10.0 - 46.0;
            let server_y = compile_y - 10.0 - 46.0;
            vec![
                ItemSlot {
                    rect: UiRect::new(content.rect[0], items_top, cw, 76.0, [0.0; 4]),
                    item: 0,
                },
                ItemSlot {
                    rect: UiRect::new(big_x, play_y, big_w, 88.0, [0.0; 4]),
                    item: 1,
                },
                ItemSlot {
                    rect: UiRect::new(big_x, compile_y, big_w, 46.0, [0.0; 4]),
                    item: 2,
                },
                ItemSlot {
                    rect: UiRect::new(big_x, server_y, big_w, 46.0, [0.0; 4]),
                    item: 3,
                },
            ]
        }
        Page::Instalaciones => {
            let count = state.item_count(Page::Instalaciones).max(1) as f32;
            let card_h = (items_h / count - 10.0).clamp(34.0, 58.0);
            let visible = ((items_h + 10.0) / (card_h + 10.0)) as usize;
            let total = state.item_count(Page::Instalaciones);
            let selected = state.selected();
            let first = selected
                .saturating_sub(visible.saturating_sub(1))
                .min(total.saturating_sub(1));
            (first..total)
                .take(visible)
                .map(|item| ItemSlot {
                    rect: UiRect::new(
                        content.rect[0],
                        items_top + (item - first) as f32 * (card_h + 10.0),
                        cw,
                        card_h,
                        [0.0; 4],
                    ),
                    item,
                })
                .collect()
        }
        Page::Opciones => {
            let row_h = 48.0;
            (0..state.item_count(Page::Opciones))
                .map(|item| ItemSlot {
                    rect: UiRect::new(
                        content.rect[0] + 6.0,
                        items_top + item as f32 * (row_h + 12.0),
                        cw - 12.0,
                        row_h,
                        [0.0; 4],
                    ),
                    item,
                })
                .collect()
        }
        Page::Acerca => Vec::new(),
    };

    let summary_h = if state.page() == Page::Play {
        56.0
    } else {
        0.0
    };
    let summary = UiRect::new(content.rect[0], items_top + 90.0, cw, summary_h, [0.0; 4]);

    Layout {
        sidebar_bg,
        sidebar,
        content,
        header,
        items,
        summary,
        status,
    }
}

/// Botones de la barra lateral: una entrada por página más Salir abajo.
fn sidebar_hits(sidebar_bg: &UiRect, height: f32) -> Vec<Hit> {
    let btn_h = 46.0;
    let gap = 8.0;
    let mut hits = Vec::with_capacity(PAGES.len() + 1);
    for (idx, page) in PAGES.iter().enumerate() {
        let y = sidebar_bg.rect[1] + 10.0 + idx as f32 * (btn_h + gap);
        hits.push(Hit {
            rect: UiRect::new(
                sidebar_bg.rect[0] + 10.0,
                y,
                sidebar_bg.rect[2] - 20.0,
                btn_h,
                [0.0; 4],
            ),
            slot: HitSlot::SidebarPage(*page),
        });
    }
    let exit_y = height - MARGEN - 10.0 - btn_h;
    hits.push(Hit {
        rect: UiRect::new(
            sidebar_bg.rect[0] + 10.0,
            exit_y,
            sidebar_bg.rect[2] - 20.0,
            btn_h,
            [0.0; 4],
        ),
        slot: HitSlot::Exit,
    });
    hits
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

    #[test]
    fn sidebar_has_page_buttons_and_exit() {
        let lay = layout(&state(Page::Play), 1024.0, 640.0);
        assert_eq!(lay.sidebar.len(), PAGES.len() + 1);
        assert_eq!(lay.sidebar[0].slot, HitSlot::SidebarPage(Page::Play));
        assert_eq!(lay.sidebar[3].slot, HitSlot::SidebarPage(Page::Acerca));
        assert_eq!(lay.sidebar[4].slot, HitSlot::Exit);
        assert!(lay.sidebar[4].rect.rect[1] > lay.sidebar[3].rect.rect[1]);
    }

    #[test]
    fn play_page_stacks_four_items_above_the_status_bar() {
        let lay = layout(&state(Page::Play), 1024.0, 640.0);
        assert_eq!(lay.items.len(), 4);
        let card = lay.item_rect(0).expect("tarjeta");
        let play = lay.item_rect(1).expect("boton jugar");
        assert!(play.rect[1] > card.rect[1] + card.rect[3]);
        // Orden vertical: servidor, compilar, jugar (de abajo a arriba).
        let server = lay.item_rect(3).expect("boton servidor");
        let compile = lay.item_rect(2).expect("boton compilar");
        assert!(server.rect[1] < compile.rect[1]);
        assert!(compile.rect[1] + compile.rect[3] < play.rect[1] + play.rect[3]);
        assert!(play.rect[1] + play.rect[3] <= lay.status.rect[1]);
        let center = (
            card.rect[0] + card.rect[2] * 0.5,
            card.rect[1] + card.rect[3] * 0.5,
        );
        assert_eq!(lay.item_at(center.0, center.1), Some(0));
        assert_eq!(lay.item_at(2.0, 2.0), None);
    }

    #[test]
    fn instalaciones_cards_scroll_to_keep_selection_visible() {
        let mut s = state(Page::Instalaciones);
        assert_eq!(lay_items(&s), 3);
        for _ in 0..10 {
            s.versions.push(s.versions[0].clone());
        }
        for _ in 0..12 {
            s.move_selection(1);
        }
        let scrolled = layout(&s, 1024.0, 640.0);
        assert!(!scrolled.items.is_empty());
        assert!(scrolled.items.iter().any(|slot| slot.item == 12));
        let last = scrolled.items.last().expect("tarjetas");
        assert!(last.rect.rect[1] + last.rect.rect[3] <= scrolled.status.rect[1]);
    }

    fn lay_items(s: &LauncherState) -> usize {
        layout(s, 1024.0, 640.0).items.len()
    }

    #[test]
    fn opciones_rows_fit_inside_content() {
        let lay = layout(&state(Page::Opciones), 1024.0, 640.0);
        assert_eq!(lay.items.len(), 5);
        let last = lay.items[4].rect;
        assert!(last.rect[1] + last.rect[3] < lay.status.rect[1]);
    }

    #[test]
    fn acerca_has_no_items() {
        let lay = layout(&state(Page::Acerca), 1024.0, 640.0);
        assert!(lay.items.is_empty());
    }
}
