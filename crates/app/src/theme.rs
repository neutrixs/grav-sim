use eframe::egui::{self, Color32, CornerRadius, FontId, Stroke};

// Same palette as the web app's tokens.css.
pub const BG: Color32 = Color32::from_rgb(0x0b, 0x0d, 0x12);
pub const GRID: Color32 = Color32::from_rgb(0x9f, 0xb3, 0xd1);
pub const TEXT: Color32 = Color32::from_rgb(0xd6, 0xdd, 0xe8);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x7d, 0x88, 0x99);
pub const PANEL: Color32 = Color32::from_rgba_premultiplied(14, 17, 24, 217); // rgba(17, 20, 28, 0.85)
pub const PANEL_SOLID: Color32 = Color32::from_rgb(0x16, 0x1a, 0x23);
pub const INPUT: Color32 = Color32::from_rgba_premultiplied(10, 10, 10, 10); // rgba(255, 255, 255, 0.04)
pub const BORDER: Color32 = Color32::from_rgba_premultiplied(20, 20, 20, 20); // rgba(255, 255, 255, 0.08)
pub const ACCENT: Color32 = Color32::from_rgb(0x7a, 0xa2, 0xff);
pub const ERROR: Color32 = Color32::from_rgb(0xff, 0x7a, 0x7a);
pub const WARN: Color32 = Color32::from_rgb(0xf5, 0xb8, 0x5a);
pub const BODY: Color32 = Color32::from_rgb(0xf2, 0xc2, 0x7b);

pub const RADIUS: u8 = 10;
pub const RADIUS_SM: u8 = 6;
pub const EDGE: f32 = 16.0; // distance of floating UI from the window edge
pub const PANEL_W: f32 = 264.0;

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

pub fn apply(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.interact_size.y = 22.0;

        let v = &mut style.visuals;
        v.panel_fill = BG;
        v.window_fill = PANEL_SOLID;
        v.extreme_bg_color = INPUT;
        v.override_text_color = Some(TEXT);
        v.selection.bg_fill = ACCENT.linear_multiply(0.35);
        v.selection.stroke = Stroke::new(1.0, ACCENT);
        v.text_cursor.stroke = Stroke::new(1.5, ACCENT);
        v.window_stroke = Stroke::new(1.0, BORDER);
        v.menu_corner_radius = CornerRadius::same(RADIUS_SM);

        for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
            w.corner_radius = CornerRadius::same(4);
            w.bg_stroke = Stroke::NONE;
        }
        v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        v.widgets.inactive.bg_fill = INPUT;
        v.widgets.hovered.weak_bg_fill = BORDER;
        v.widgets.hovered.bg_fill = BORDER;
        v.widgets.active.weak_bg_fill = BORDER;
        v.widgets.open.weak_bg_fill = BORDER;
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
        v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    });
}
