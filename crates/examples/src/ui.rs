use vyas::prelude::*;

pub(crate) const APP_NAME: &str = "Vyas";

pub(crate) fn draw(ctx: &egui::Context, world: &World) {
    let fps = world.resource::<Fps>().value();

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
                egui::pos2(title_rect.max.x - 12.0, title_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                format!("FPS {fps:.1}"),
                egui::FontId::monospace(11.0),
                egui::Color32::GRAY,
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
