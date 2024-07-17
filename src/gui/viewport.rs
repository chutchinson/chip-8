use egui::{Color32, Rounding, Stroke, Pos2, Vec2, Rect};

fn ui_viewport (ui: &mut egui::Ui, screen: [u8; 4096], scale: f32) -> egui::Response {
    let scale = scale * 4.0;
    let desired_size = [64.0 * scale, 32.0 * scale];
    let (rect, mut response) = ui.allocate_exact_size(desired_size.into(), egui::Sense::click());

    if ui.is_rect_visible(rect) {

        let rounding = Rounding::from(1.0);
        let dark = Color32::from_rgb(0, 0, 0);
        let light = Color32::from_rgb(255, 255, 255);
        let stroke = Stroke::new(0.0, Color32::TRANSPARENT);

        ui.painter().rect(rect, rounding, dark, stroke);

        for x in 0..64 {
            for y in 0..32 {
                let min = rect.min + Vec2::new(x as f32, y as f32) * scale;
                let max = min + Vec2::new(scale, scale);
                let cell = Rect::from_min_max(min, max);
                let idx = (y * 64) + x;
                if screen[idx] == 0 { continue; }
                let texel = if screen[idx] > 0 { light } else { dark };
                ui.painter().rect(cell, rounding, texel, stroke);
            }
        }

        ui.ctx().request_repaint();
    }

    response
}

pub fn viewport(screen: [u8; 4096], scale: f32) -> impl egui::Widget {
    move |ui: &mut egui::Ui| ui_viewport(ui, screen, scale)
}
