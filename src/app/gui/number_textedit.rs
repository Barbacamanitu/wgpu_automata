pub struct FloatTextEdit {
    min: f32,
    max: f32,
    val: f32,
    val_str: String,
}

impl FloatTextEdit {
    pub fn new(min: f32, max: f32, initial: f32) -> FloatTextEdit {
        let clamped = initial.clamp(min, max);
        FloatTextEdit {
            min: min,
            max: max,
            val: clamped,
            val_str: clamped.to_string(),
        }
    }

    pub fn value(&self) -> f32 {
        self.val
    }

    pub fn set_value(&mut self, v: f32) {
        self.val = v;
    }

    pub fn validate(&mut self) {
        let default = self.val.clamp(self.min, self.max);
        let num = match self.val_str.parse::<f32>() {
            Ok(parsed) => parsed.clamp(self.min, self.max),
            Err(_) => default,
        };
        self.val = num;
        self.val_str = num.to_string();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, size: Option<impl Into<egui::Vec2>>) -> egui::Response {
        let te = egui::TextEdit::singleline(&mut self.val_str);
        let r = match size {
            Some(s) => ui.add_sized(s, te),
            None => ui.add(te),
        };
        if r.lost_focus() {
            //self.validate();
        }
        r
    }
}
