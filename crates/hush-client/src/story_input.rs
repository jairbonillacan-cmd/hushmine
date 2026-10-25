//! Del teclado y el ratón a las órdenes de la expedición, y de sus eventos
//! a los avisos del HUD.

use super::*;

impl StoryApp {
    pub(super) fn tick_orders(&mut self, input: &Input) -> Vec<StoryOrder> {
        let mut orders = Vec::new();
        if self.paused || self.craft.is_open() {
            orders.push(StoryOrder::Move(MoveInput::default()));
            orders.push(StoryOrder::Mine(None));
            return orders;
        }
        let movement = move_input_from_camera(
            input,
            &self.controls,
            self.camera.flat_forward(),
            self.camera.flat_right(),
        );
        orders.push(StoryOrder::Move(movement));
        orders.push(StoryOrder::Sneak(movement.crouch));
        if let Some(craft) = self.craft.take_pending() {
            orders.push(craft);
        }
        // Picar: mantener el clic izquierdo sobre el bloque apuntado.
        let mine = if self.held_to_dig {
            self.target.map(|hit| hit.block)
        } else {
            None
        };
        orders.push(StoryOrder::Mine(mine));
        // Colocar e interactuar.
        if input.was_button_pressed(MouseButton::Right)
            && let Some(hit) = self.target
        {
            let cell = std::array::from_fn(|i| hit.block[i] + hit.normal[i]);
            orders.push(StoryOrder::Place(cell));
        }
        if input.was_pressed(KeyCode::KeyE)
            && let Some(hit) = self.target
        {
            orders.push(StoryOrder::Interact(hit.block));
        }
        if input.was_pressed(KeyCode::KeyR) {
            orders.push(StoryOrder::Eat);
        }
        for (idx, key) in [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
        ]
        .into_iter()
        .enumerate()
        {
            if input.was_pressed(key) {
                orders.push(StoryOrder::Select(idx as u8));
            }
        }
        // Ataque cuerpo a cuerpo con la herramienta empuñada.
        if input.was_button_pressed(MouseButton::Left)
            && !self.held_to_dig
            && let Some(hit) = self.target
        {
            orders.push(StoryOrder::Attack(hit.block));
        }
        orders
    }

    pub(super) fn step_simulation(&mut self, dt: f32, orders: Vec<StoryOrder>) {
        self.accumulator += dt;
        let mut ticks = 0;
        while self.accumulator >= TICK_SECONDS && ticks < MAX_TICKS_PER_FRAME {
            self.expedition.step(&orders);
            self.prev_eye = self.eye;
            self.eye = self.eye_from_body();
            self.accumulator -= TICK_SECONDS;
            ticks += 1;
        }
        if ticks == MAX_TICKS_PER_FRAME {
            self.accumulator = 0.0;
        }
    }

    pub(super) fn consume_events(&mut self) {
        use hush_sim::story::StoryEventKind;
        for event in self.expedition.take_events() {
            match event.kind {
                StoryEventKind::MilestoneReached(m) => {
                    let milestone = hush_sim::story::Milestone::ALL
                        .get(usize::from(m.saturating_sub(1)))
                        .copied();
                    if let Some(milestone) = milestone {
                        self.banner = Some((milestone.label(), 4.0));
                    }
                }
                StoryEventKind::Victory => self.banner = Some(("Has vuelto a la superficie", 10.0)),
                StoryEventKind::SiegeStarted => {
                    self.banner = Some(("Empieza el asedio: aguanta cinco minutos", 5.0))
                }
                StoryEventKind::ToolBroke(_) => {
                    self.banner = Some(("Se te ha roto la herramienta", 2.0))
                }
                _ => {}
            }
        }
    }
}
