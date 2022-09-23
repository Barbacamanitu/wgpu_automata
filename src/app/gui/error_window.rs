use egui::{Align2, Context};

pub struct ErrorWindow {
    heading: String,
    error: String,
    should_close: bool,
}

impl ErrorWindow {
    pub fn new(heading: &str, err: &str) -> ErrorWindow {
        ErrorWindow {
            heading: heading.to_owned(),
            error: err.to_owned(),
            should_close: false,
        }
    }

    pub fn should_close(&self) -> bool {
        self.should_close
    }
    pub fn ui(&mut self, ctx: &Context) {
        egui::Window::new("Error")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(self.heading.as_str());
                    ui.add_space(20.0);
                    ui.label(self.error.as_str());
                    ui.add_space(20.0);
                    if ui.button("OK").clicked() {
                        self.should_close = true;
                    }
                });
            });
    }
}
