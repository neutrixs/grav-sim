mod camera;
mod canvas;
mod draft;
mod sim;
mod theme;
mod time;
mod units;
mod widgets;

use std::time::Instant;

use eframe::egui::{
    self, Align, Align2, Area, Button, CentralPanel, Frame, Id, Key, Label, Layout, Margin, RichText,
    ScrollArea, Sense, Stroke, Ui, pos2, vec2,
};

use camera::Camera;
use canvas::{Interaction, View, grid_step};
use draft::{DraftMode, DraftState};
use sim::{Sim, StepMode};
use theme::*;
use time::{format_clock, format_duration};
use units::{BODY_MASSES, BODY_RADII, UnitGroup, UnitPrefs, format_distance, format_length, group_digits};
use widgets::{FieldUnit, check, field, primary_button, ref_field, secondary_button, section_title, segmented};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("grav-sim").with_inner_size([1280.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native("grav-sim", options, Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))))
}

struct App {
    sim: Sim,
    drafts: DraftState,
    view: View,
    prefs: UnitPrefs,
    interaction: Interaction,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        theme::apply(ctx);
        App {
            sim: Sim::new(),
            drafts: DraftState::default(),
            view: View { cam: Camera { x: 0.0, y: 0.0, scale: 50.0 }, following: None, grid_follows: true },
            prefs: UnitPrefs::default(),
            interaction: Interaction::default(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.sim.running {
            self.sim.tick(Instant::now());
            ctx.request_repaint(); // keep frames coming while time runs
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.drafts.cancel();
        }

        let speed_unit = self.prefs.unit_of(UnitGroup::Velocity);
        CentralPanel::no_frame().show(ui, |ui| {
            canvas::show(ui, &mut self.interaction, &mut self.view, &mut self.drafts, &self.sim, speed_unit);
        });

        self.side_panel(&ctx);
        self.time_panel(&ctx);
        self.scale_label(&ctx);
    }
}

// Floating panel background.
fn surface() -> Frame {
    Frame::new().fill(PANEL).stroke(Stroke::new(1.0, BORDER)).corner_radius(RADIUS)
}

// A padded section of a panel, full panel width.
fn section<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(PANEL_W - 28.0);
            ui.spacing_mut().item_spacing.y = 8.0;
            add(ui)
        })
        .inner
}

// 1px line between sections.
fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(PANEL_W, 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, BORDER);
}

fn legend(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(TEXT_DIM));
}

impl App {
    fn side_panel(&mut self, ctx: &egui::Context) {
        let max_bottom = ctx.content_rect().bottom() - 56.0;
        Area::new(Id::unique("side")).anchor(Align2::LEFT_TOP, vec2(EDGE, EDGE)).show(ctx, |ui| {
            surface().show(ui, |ui| {
                ui.set_width(PANEL_W);
                ui.spacing_mut().item_spacing.y = 0.0;
                section(ui, |ui| {
                    if self.drafts.draft.is_some() {
                        self.draft_form(ui);
                    } else if primary_button(ui, "Place body", ui.available_width()).clicked() {
                        self.drafts.start(self.view.cam.x, self.view.cam.y);
                    }
                });
                divider(ui);
                // the list scrolls instead of growing past the window
                section(ui, |ui| {
                    let list_h = (max_bottom - ui.cursor().top() - 14.0).max(40.0);
                    ScrollArea::vertical().max_height(list_h).min_scrolled_height(list_h).show(ui, |ui| self.body_list(ui));
                });
            });
        });
    }

    fn draft_form(&mut self, ui: &mut Ui) {
        let hint = match self.drafts.mode {
            DraftMode::Placing => "Click on the canvas to place it",
            DraftMode::Aiming => "Click again to set its velocity",
            DraftMode::Editing => "Drag the body or the arrow tip to adjust",
        };
        section_title(ui, "New body");
        ui.label(RichText::new(hint).size(12.0).color(ACCENT));

        let prefs = &mut self.prefs;
        let Some(d) = &mut self.drafts.draft else { return };
        let mut submitted = false;

        legend(ui, "Position");
        ui.columns(2, |c| {
            let o = field(&mut c[0], "x", "x", FieldUnit::Group(UnitGroup::Position), d.x, None, prefs);
            if let Some(v) = o.value {
                d.x = v;
            }
            submitted |= o.submitted;
            let o = field(&mut c[1], "y", "y", FieldUnit::Group(UnitGroup::Position), d.y, None, prefs);
            if let Some(v) = o.value {
                d.y = v;
            }
            submitted |= o.submitted;
        });

        legend(ui, "Velocity");
        let vel = FieldUnit::Group(UnitGroup::Velocity);
        ui.columns(2, |c| {
            let o = field(&mut c[0], "vx", "vx", vel, d.vx, None, prefs);
            if let Some(v) = o.value {
                d.set_velocity_xy(v, d.vy);
            }
            submitted |= o.submitted;
            let o = field(&mut c[1], "vy", "vy", FieldUnit::Group(UnitGroup::Velocity), d.vy, None, prefs);
            if let Some(v) = o.value {
                d.set_velocity_xy(d.vx, v);
            }
            submitted |= o.submitted;
        });
        ui.columns(2, |c| {
            let o = field(&mut c[0], "speed", "|v|", FieldUnit::Group(UnitGroup::Velocity), d.speed, Some(0.0), prefs);
            if let Some(v) = o.value {
                d.set_speed(v);
            }
            submitted |= o.submitted;
            let o = field(&mut c[1], "angle", "θ", FieldUnit::Group(UnitGroup::Angle), d.angle, None, prefs);
            if let Some(v) = o.value {
                d.set_angle(v);
            }
            submitted |= o.submitted;
        });

        legend(ui, "Body");
        let o = ref_field(ui, "mass", "mass", UnitGroup::Mass, BODY_MASSES, d.mass, prefs);
        if let Some(v) = o.value {
            d.mass = v;
        }
        submitted |= o.submitted;
        let o = ref_field(ui, "radius", "r", UnitGroup::Radius, BODY_RADII, d.radius, prefs);
        if let Some(v) = o.value {
            d.radius = v;
        }
        submitted |= o.submitted;

        ui.add_space(4.0);
        let (mut cancel, mut add) = (false, false);
        ui.columns(2, |c| {
            let w = c[0].available_width();
            cancel = secondary_button(&mut c[0], "Cancel", w).clicked();
            add = primary_button(&mut c[1], "Add", w).clicked();
        });

        if cancel {
            self.drafts.cancel();
        } else if add || submitted {
            self.drafts.confirm(&mut self.sim);
        }
        if !self.drafts.error.is_empty() {
            ui.label(RichText::new(&self.drafts.error).size(12.0).color(ERROR));
        }
    }

    fn body_list(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            section_title(ui, "Bodies");
            ui.label(RichText::new(self.sim.bodies.len().to_string()).size(11.0).strong().color(TEXT));
        });

        if let Some(id) = self.view.following {
            check(ui, &mut self.view.grid_follows, &format!("Grid moves with #{id}"));
        }

        if self.sim.bodies.is_empty() {
            ui.label(RichText::new("None yet").size(12.0).color(TEXT_DIM));
            return;
        }

        // fixed column widths, so changing numbers don't shift the columns:
        // every cell is its own child ui at an exact rect
        let id_w = 44.0;
        let row = |ui: &mut Ui| -> [Ui; 3] {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
            let col_w = (r.width() - id_w - 16.0) / 2.0;
            let rect = |x: f32, w: f32| egui::Rect::from_min_size(pos2(r.left() + x, r.top()), vec2(w, r.height()));
            let child = |ui: &mut Ui, rect, layout| ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout));
            [
                child(ui, rect(0.0, id_w), Layout::left_to_right(Align::Center)),
                child(ui, rect(id_w + 8.0, col_w), Layout::right_to_left(Align::Center)),
                child(ui, rect(id_w + 16.0 + col_w, col_w), Layout::right_to_left(Align::Center)),
            ]
        };
        let text = |t: String, color| Label::new(RichText::new(t).font(mono(12.0)).color(color)).truncate();

        ui.spacing_mut().item_spacing.y = 2.0;
        let [mut id, mut x, mut y] = row(ui);
        id.add(text("id".into(), TEXT_DIM));
        x.add(text("x".into(), TEXT_DIM));
        y.add(text("y".into(), TEXT_DIM));

        for b in &self.sim.bodies {
            let followed = self.view.following == Some(b.id);
            let color = if followed { ACCENT } else { TEXT };
            let [mut id, mut x, mut y] = row(ui);
            // the id doubles as the follow toggle
            id.spacing_mut().button_padding = vec2(4.0, 0.0);
            let button = Button::new(RichText::new(format!("#{}", b.id)).font(mono(12.0)).color(if followed {
                BG
            } else {
                TEXT_DIM
            }))
            .fill(if followed { ACCENT } else { egui::Color32::TRANSPARENT })
            .corner_radius(4)
            .min_size(vec2(0.0, 18.0));
            let hint = if followed { "Stop following" } else { "Keep this body in the centre" };
            if id.add(button).on_hover_text(hint).clicked() {
                self.view.toggle_follow(b.id);
            }
            x.add(text(format_distance(b.x), color));
            y.add(text(format_distance(b.y), color));
        }
    }

    fn time_panel(&mut self, ctx: &egui::Context) {
        Area::new(Id::unique("time")).anchor(Align2::RIGHT_TOP, vec2(-EDGE, EDGE)).show(ctx, |ui| {
            surface().show(ui, |ui| {
                ui.set_width(PANEL_W);
                ui.spacing_mut().item_spacing.y = 0.0;

                Frame::new().inner_margin(Margin::same(8)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        let icon = if self.sim.running { "⏸" } else { "▶" };
                        let play = Button::new(RichText::new(icon).size(13.0).color(BG))
                            .fill(ACCENT)
                            .corner_radius(RADIUS_SM)
                            .min_size(vec2(32.0, 32.0));
                        if ui.add(play).on_hover_text(if self.sim.running { "Pause" } else { "Play" }).clicked() {
                            self.sim.toggle();
                        }
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.label(RichText::new(format!("t = {}", format_clock(self.sim.time))).font(mono(13.0)));
                            let s = self.sim.slowdown;
                            if self.sim.running && s > 1.05 {
                                let text = if s.is_finite() { format!("{s:.1}× slower") } else { "stalled".into() };
                                ui.label(RichText::new(text).font(mono(11.0)).color(WARN));
                            }
                        });
                    });
                });
                divider(ui);

                section(ui, |ui| {
                    section_title(ui, "Time");
                    let o = field(ui, "speed", "speed", FieldUnit::Group(UnitGroup::Speed), self.sim.cfg.speed, None, &mut self.prefs);
                    if let Some(v) = o.value {
                        self.sim.set_speed(v);
                    }

                    ui.horizontal(|ui| {
                        legend(ui, "Step");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let modes = [(StepMode::Fixed, "Fixed"), (StepMode::Dynamic, "Dynamic"), (StepMode::Auto, "Auto")];
                            if let Some(m) = segmented(ui, &modes, self.sim.cfg.step_mode) {
                                self.sim.set_step_mode(m);
                            }
                        });
                    });

                    match self.sim.cfg.step_mode {
                        StepMode::Fixed => {
                            let unit = FieldUnit::Group(UnitGroup::Dt);
                            let o = field(ui, "dt", "dt", unit, self.sim.cfg.fixed_dt, None, &mut self.prefs);
                            if let Some(v) = o.value {
                                self.sim.set_fixed_dt(v);
                            }
                        }
                        StepMode::Dynamic => {
                            let unit = FieldUnit::Fixed("× speed");
                            let o = field(ui, "dt_factor", "dt =", unit, self.sim.cfg.dt_factor, None, &mut self.prefs);
                            if let Some(v) = o.value {
                                self.sim.set_dt_factor(v);
                            }
                        }
                        StepMode::Auto => {}
                    }

                    // physics steps per real second at the current settings
                    let steps = format!("{:.0}", (self.sim.cfg.speed / self.sim.dt()).round());
                    let readout = format!("dt = {} · {} steps/s", format_duration(self.sim.dt()), group_digits(&steps, ','));
                    ui.label(RichText::new(readout).font(mono(11.0)).color(TEXT_DIM));

                    check(ui, &mut self.sim.multithread, "Multithread");
                });
            });
        });
    }

    // Spacing of the thick grid lines, bottom left: "▢ 10 m".
    fn scale_label(&self, ctx: &egui::Context) {
        let r = ctx.content_rect();
        let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, Id::unique("scale")));
        let y = r.bottom() - EDGE - 6.0;
        let tick = egui::Rect::from_min_size(pos2(r.left() + EDGE, y - 5.0), vec2(10.0, 10.0));
        p.rect_stroke(tick, 0, Stroke::new(1.0, TEXT_DIM), egui::StrokeKind::Inside);
        let label = format_length(grid_step(self.view.cam.scale) * 10.0);
        p.text(pos2(tick.right() + 8.0, y), Align2::LEFT_CENTER, label, mono(12.0), TEXT_DIM);
    }
}
