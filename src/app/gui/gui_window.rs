use egui::{Align2, Context};

use crate::app::simulator::SimulationState;

pub struct GuiWindow {
    pub sim_state: SimulationState,
}

impl GuiWindow {
    pub fn new() -> GuiWindow {
        GuiWindow {
            sim_state: SimulationState::default(),
        }
    }

    pub fn ui(&mut self, ctx: &Context) {
        let a = Align2::RIGHT_TOP;
        egui::Window::new("Automata")
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Simulation Settings");
                ui.horizontal(|ui| {
                    ui.label("Paused: ");
                    ui.checkbox(&mut self.sim_state.paused, "Paused");
                });
                let pause_status = if self.sim_state.paused {
                    "paused"
                } else {
                    "running"
                };
                ui.label(format!("Simulation is {}", pause_status));
                ui.label(format!(
                    "FPS: {}, Updates/Sec: {}",
                    self.sim_state.fps, self.sim_state.ups
                ));
            });
    }
}
