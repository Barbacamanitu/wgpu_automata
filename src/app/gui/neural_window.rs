use egui::{Context, Response, Slider};

use crate::app::{
    neural::{Neural, NeuralFilter},
    App,
};

pub struct NeuralWindow {
    filter: NeuralFilter,
}

impl NeuralWindow {
    pub fn new() -> NeuralWindow {
        NeuralWindow {
            filter: NeuralFilter::default(),
        }
    }

    pub fn get_filter(&self) -> NeuralFilter {
        self.filter
    }
    pub fn ui(&mut self, ctx: &Context, app: &mut App) -> Response {
        let edit_size = [40.0, 15.0];
        let w = egui::Window::new("Neural Settings").show(ctx, |ui| {
            let slider_size = [10.0, 15.0];

            ui.horizontal(|ui| {
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[0], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[1], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[2], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
            });
            ui.horizontal(|ui| {
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[3], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[4], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[5], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
            });
            ui.horizontal(|ui| {
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[6], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[7], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
                ui.add_sized(
                    slider_size,
                    Slider::new(&mut self.filter.weights[8], -1.0..=1.0)
                        .logarithmic(false)
                        .smart_aim(false),
                );
            });
            if ui.button("Apply").clicked() {
                let neural = app.sim.as_any_mut().downcast_mut::<Neural>();
                match neural {
                    Some(n) => {
                        n.set_filter(self.filter.clone());
                    }
                    None => {}
                }
            }
        });

        w.unwrap().response
    }
}
