use eframe::egui::{
    self, Align, Button, CornerRadius, Frame, Key, Layout, Margin, Response, RichText, Stroke, TextEdit, Ui,
    vec2,
};

use crate::theme::*;
use crate::units::{Unit, UnitGroup, UnitPrefs, from_display, num_text, to_display, to_precision};

// Small uppercase heading at the top of a panel section.
pub fn section_title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text.to_uppercase()).size(11.0).strong().extra_letter_spacing(0.9).color(TEXT_DIM));
}

fn tag(text: &str) -> RichText {
    RichText::new(text).font(mono(11.0)).color(TEXT_DIM)
}

pub fn primary_button(ui: &mut Ui, text: &str, width: f32) -> Response {
    let b = Button::new(RichText::new(text).strong().color(BG)).fill(ACCENT).corner_radius(RADIUS_SM);
    ui.add_sized(vec2(width, 30.0), b)
}

pub fn secondary_button(ui: &mut Ui, text: &str, width: f32) -> Response {
    let b = Button::new(RichText::new(text).strong().color(TEXT))
        .fill(INPUT)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(RADIUS_SM);
    ui.add_sized(vec2(width, 30.0), b)
}

#[derive(Clone, Copy)]
pub enum FieldUnit {
    Group(UnitGroup), // a picker; the value is always in the stored unit
    Fixed(&'static str),
}

#[derive(Default)]
pub struct FieldOut {
    pub value: Option<f64>, // set when a new valid value was committed
    pub submitted: bool,    // committed with Enter
}

// Number input laid out as [label | value | unit] in one box.
// While focused, it holds free text that isn't parsed, so half-typed numbers
// like "0.00" or "1e" are left alone. It's only parsed and applied on Enter
// or when focus leaves; anything invalid snaps back to the last value.
pub fn field(
    ui: &mut Ui,
    id_salt: &str,
    label: &str,
    unit: FieldUnit,
    value: f64,
    min: Option<f64>,
    prefs: &mut UnitPrefs,
) -> FieldOut {
    let te_id = ui.make_persistent_id(("field", id_salt));
    let focused = ui.memory(|m| m.has_focus(te_id));
    let u = match unit {
        FieldUnit::Group(g) => prefs.unit_of(g),
        FieldUnit::Fixed(_) => Unit { label: "", factor: 1.0 },
    };
    let shown = num_text(to_display(value, u));
    let mut out = FieldOut::default();

    Frame::new()
        .fill(INPUT)
        .stroke(Stroke::new(1.0, if focused { ACCENT } else { BORDER }))
        .corner_radius(RADIUS_SM)
        .inner_margin(Margin { left: 8, right: 6, top: 0, bottom: 0 })
        .show(ui, |ui| {
            ui.set_height(28.0);
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(tag(label));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    match unit {
                        FieldUnit::Group(g) => unit_picker(ui, id_salt, g, prefs),
                        FieldUnit::Fixed(t) => {
                            ui.label(tag(t));
                        }
                    }

                    let mut text = if focused {
                        ui.data(|d| d.get_temp::<String>(te_id)).unwrap_or_else(|| shown.clone())
                    } else {
                        shown.clone()
                    };
                    let resp = ui.add(
                        TextEdit::singleline(&mut text)
                            .id(te_id)
                            .frame(Frame::NONE)
                            .font(mono(12.0))
                            .align(egui::Align2::RIGHT_CENTER)
                            .margin(Margin::ZERO)
                            .desired_width(ui.available_width()),
                    );
                    if resp.has_focus() {
                        ui.data_mut(|d| d.insert_temp(te_id, text.clone()));
                    }
                    if resp.lost_focus() {
                        ui.data_mut(|d| d.remove::<String>(te_id));
                        let raw = text.trim();
                        if let Ok(n) = raw.parse::<f64>()
                            && n.is_finite()
                            && min.is_none_or(|m| n >= m)
                        {
                            out.value = Some(from_display(n, u));
                        }
                        out.submitted = ui.input(|i| i.key_pressed(Key::Enter));
                    }
                });
            });
        });
    out
}

// Unit picker: looks like the plain unit tag, opens a list of the group's units.
fn unit_picker(ui: &mut Ui, id_salt: &str, g: UnitGroup, prefs: &mut UnitPrefs) {
    let mut sel = prefs.label(g);
    let button = Button::new(tag(&format!("{sel} ▾"))).frame_when_inactive(false).corner_radius(4);
    let resp = ui
        .push_id(("unit", id_salt), |ui| {
            ui.spacing_mut().button_padding = vec2(2.0, 1.0);
            ui.add(button)
        })
        .inner;
    egui::Popup::menu(&resp).show(|ui| {
        for u in g.units() {
            ui.selectable_value(&mut sel, u.label, RichText::new(u.label).font(mono(11.0)));
        }
    });
    prefs.set(g, sel);
}

// Field with reference bodies for a sense of scale (Moon … Sun): a chip sets
// the value to exactly that body's, and the line below says which reference
// the current value is closest to. Used for mass and radius.
pub fn ref_field(
    ui: &mut Ui,
    id_salt: &str,
    label: &str,
    group: UnitGroup,
    refs: &[Unit], // also present in the group's unit list, under the same labels
    value: f64,
    prefs: &mut UnitPrefs,
) -> FieldOut {
    let mut out = FieldOut::default();
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        out = field(ui, id_salt, label, FieldUnit::Group(group), value, None, prefs);

        ui.columns(refs.len(), |cols| {
            for (col, r) in cols.iter_mut().zip(refs) {
                col.spacing_mut().item_spacing.x = 4.0;
                let active = value == r.factor;
                let b = Button::new(RichText::new(r.label).size(11.0).color(if active { BG } else { TEXT_DIM }))
                    .fill(if active { ACCENT } else { INPUT })
                    .stroke(Stroke::new(1.0, if active { ACCENT } else { BORDER }))
                    .corner_radius(4);
                if col.add_sized(vec2(col.available_width(), 22.0), b).clicked() {
                    out.value = Some(r.factor);
                    prefs.set(group, r.label);
                }
            }
        });

        // Closest reference on a log scale, if the value is within 1000× of it.
        let nearest = (value > 0.0)
            .then(|| {
                refs.iter()
                    .map(|r| (r, (value / r.factor).log10().abs()))
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .filter(|(_, dist)| *dist < 3.0)
            })
            .flatten();
        if let Some((r, _)) = nearest
            && prefs.label(group) != r.label
        {
            ui.label(tag(&format!("≈ {} × {}", to_precision(value / r.factor, 3), r.label)));
        }
    });
    out
}

// Row of mutually exclusive options; returns the clicked one, if any.
pub fn segmented<T: Copy + PartialEq>(ui: &mut Ui, options: &[(T, &str)], current: T) -> Option<T> {
    let mut picked = None;
    Frame::new()
        .fill(INPUT)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(RADIUS_SM)
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            // explicit direction: the caller may sit in a right-to-left layout
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for &(v, label) in options {
                    let active = v == current;
                    let b = Button::new(RichText::new(label).size(11.0).color(if active { BG } else { TEXT_DIM }))
                        .fill(if active { ACCENT } else { egui::Color32::TRANSPARENT })
                        .corner_radius(CornerRadius::same(4));
                    if ui.add(b).clicked() {
                        picked = Some(v);
                    }
                }
            });
        });
    picked
}

// Checkbox in the panel style: a bordered square, filled with the accent when on.
pub fn check(ui: &mut Ui, value: &mut bool, text: &str) -> Response {
    let mut resp = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
            let p = ui.painter();
            if *value {
                p.rect_filled(rect, 3, ACCENT);
                p.text(rect.center(), egui::Align2::CENTER_CENTER, "✔", mono(10.0), BG);
            } else {
                p.rect_filled(rect, 3, INPUT);
                p.rect_stroke(rect, 3, Stroke::new(1.0, TEXT_DIM.gamma_multiply(0.6)), egui::StrokeKind::Inside);
            }
            ui.label(RichText::new(text).size(12.0).color(TEXT_DIM));
        })
        .response
        .interact(egui::Sense::click());
    if resp.clicked() {
        *value = !*value;
        resp.mark_changed();
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}
