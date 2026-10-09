pub(crate) const APP_NAME: &str = "Vyas";

pub(crate) fn draw(ctx: &egui::Context) {
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("titlebar"))
        .fixed_pos(screen.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(screen.width(), 28.0), egui::Sense::hover());

            ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);

            let title_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.min.y), rect.max);

            ui.painter().with_clip_rect(title_rect).text(
                title_rect.center(),
                egui::Align2::CENTER_CENTER,
                APP_NAME,
                egui::FontId::monospace(13.0),
                egui::Color32::WHITE,
            );

            let response = ui.interact(
                title_rect,
                ui.id().with("titlebar_drag"),
                egui::Sense::click_and_drag(),
            );

            if response.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
        });
}
