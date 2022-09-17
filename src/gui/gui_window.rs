use egui::{Align2, Context};

pub struct GuiWindow {
    name: String,
    age: u32,
}

impl GuiWindow {
    pub fn new() -> GuiWindow {
        GuiWindow {
            name: "Bob".to_owned(),
            age: 10,
        }
    }

    pub fn ui(&mut self, ctx: &Context) {
        let a = Align2::RIGHT_TOP;
        egui::Window::new("Automata")
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Simulation Settings");
                ui.horizontal(|ui| {
                    ui.label("Your name: ");
                    ui.text_edit_singleline(&mut self.name);
                });
                ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                if ui.button("Click each year").clicked() {
                    self.age += 1;
                }
                ui.label(format!("Hello '{}', age {}", self.name, self.age));
            });
    }
}
