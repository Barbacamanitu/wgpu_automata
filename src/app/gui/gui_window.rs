use super::{
    error_window::ErrorWindow,
    neural_window::NeuralWindow,
    number_textedit::{self, FloatTextEdit},
};
use crate::app::{
    gpu_interface::GPUInterface,
    math::UVec2,
    neural::{NeuralFilter, NeuralParams},
    sim_renderer::{RendererType, SimulationRenderer},
    simulator::SimulationState,
    totalistic::TotalisticParams,
    App, SimParams,
};
use egui::{Align2, Context};

#[derive(PartialEq)]
enum SimulationType {
    Totalistic,
    Neural,
}

pub struct GuiWindow {
    pub sim_state: SimulationState,
    selected_simulation_type: SimulationType,
    sim_size: UVec2,
    rule_str: String,
    error_window: Option<ErrorWindow>,
    neural_window: NeuralWindow,
    updates_per_frame: u32,
    update_delay: u32,
}

impl GuiWindow {
    pub fn new() -> GuiWindow {
        GuiWindow {
            sim_state: SimulationState::default(),
            selected_simulation_type: SimulationType::Totalistic,
            rule_str: "B3/S23".to_owned(),
            sim_size: UVec2::new(512, 512),
            error_window: None,
            neural_window: NeuralWindow::new(),
            updates_per_frame: 0,
            update_delay: 0,
        }
    }

    fn remake_sim(
        &mut self,
        app: &mut App,
        sim_renderer: &mut SimulationRenderer,
        gpu: &GPUInterface,
    ) {
        let sim_params = match self.selected_simulation_type {
            SimulationType::Totalistic => SimParams::Totalistic(TotalisticParams {
                size: self.sim_size,
                rule_str: self.rule_str.clone(),
            }),
            SimulationType::Neural => SimParams::Neural(NeuralParams {
                size: self.sim_size,
                filter: self.neural_window.get_filter(),
            }),
        };
        let r_type = match self.selected_simulation_type {
            SimulationType::Totalistic => RendererType::Totalistic,
            SimulationType::Neural => RendererType::Neural,
        };
        let app_remake = app.remake(sim_params, gpu);
        match app_remake {
            Ok(_) => println!("Remade simulation:"),
            Err(err) => {
                println!("failed to make sim");
                let err_str = match err {
                    crate::app::RemakeError::RuleError => "Invalid Rule String",
                    crate::app::RemakeError::NeuralError => "Neural Error.",
                };
                let e = format!("Cause: {}", err_str);
                self.error_window = Some(ErrorWindow::new("Simulation Creation Error", e.as_str()));
            }
        }
        sim_renderer.set_renderer_type(r_type);
    }

    pub fn ui(
        &mut self,
        ctx: &Context,
        gpu: &GPUInterface,
        app: &mut App,
        sim_renderer: &mut SimulationRenderer,
    ) {
        let is_paused = self.sim_state.paused;

        //Check for error window close
        match &self.error_window {
            Some(ew) => {
                if ew.should_close() {
                    self.error_window = None
                }
            }
            None => {}
        };

        egui::Window::new("Automata")
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .show(ctx, |ui| {
                match &mut self.error_window {
                    Some(ew) => ew.ui(ctx),
                    None => {}
                }
                ui.heading("Simulation Settings");
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.sim_state.paused,
                        if is_paused { "Paused" } else { "Running" },
                    );
                });

                self.neural_window.ui(ctx, app);
                let filter = self.neural_window.get_filter();
                ui.label(format!(
                    "Simulation is {}",
                    if self.sim_state.paused {
                        "paused"
                    } else {
                        "running"
                    }
                ));
                ui.label(format!(
                    "FPS: {}, Updates/Sec: {}",
                    self.sim_state.fps, self.sim_state.ups
                ));
                ui.label(format!("Generation: {}", self.sim_state.generations));
                ui.separator();
                ui.heading("Simulation Type");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.selected_simulation_type,
                        SimulationType::Totalistic,
                        "Totalistic",
                    );
                    ui.radio_value(
                        &mut self.selected_simulation_type,
                        SimulationType::Neural,
                        "Neural",
                    );
                });
                //Rule
                ui.label("Rule String:");
                ui.add_sized([80.0, 15.0], egui::TextEdit::singleline(&mut self.rule_str));

                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(egui::Slider::new(&mut self.sim_size.x, 8..=8192).integer());
                    ui.label("Height:");
                    ui.add(egui::Slider::new(&mut self.sim_size.y, 8..=8192).integer());
                });

                if ui.button("Recreate Simulation").clicked() {
                    self.remake_sim(app, sim_renderer, gpu);
                }

                ui.separator();
                ui.heading("Simulation Update Rate");
                ui.label("Updates Per Frame:");
                ui.add(egui::Slider::new(&mut self.updates_per_frame, 1..=100).integer());
                ui.label("Update Delay:");
                ui.add(egui::Slider::new(&mut self.update_delay, 0..=1000).integer());
                if ui.button("Apply").clicked() {
                    app.time
                        .update_delays(self.updates_per_frame, self.update_delay);
                }
            });

        //Return gui response
    }
}
