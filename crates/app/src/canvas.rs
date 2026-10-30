use eframe::egui::{Align2, CursorIcon, Painter, Pos2, Rect, Response, Shape, Stroke, Ui, pos2};

use crate::camera::Camera;
use crate::draft::{Draft, DraftMode, DraftState};
use crate::sim::{BodyData, Sim};
use crate::theme::*;
use crate::time::format_duration;
use crate::units::{Unit, to_precision};

// What the canvas shows besides the bodies: camera and the follow feature.
pub struct View {
    pub cam: Camera,
    // Id of the body the camera keeps centred, if any. Panning stops it.
    pub following: Option<usize>,
    // While following: grid lines pass through the followed body (true) or
    // stay fixed to the world origin (false).
    pub grid_follows: bool,
}

impl View {
    pub fn toggle_follow(&mut self, id: usize) {
        self.following = if self.following == Some(id) { None } else { Some(id) };
    }
}

// ---------------------------------------------------------------- grid

// Thinnest lines are never closer than this on screen.
const MIN_GAP_PX: f64 = 8.0;
const MAX_ALPHA: f32 = 0.35;

// A line's weight depends only on its on-screen gap, so it changes continuously
// while zooming: 0 at MIN_GAP_PX (invisible), 1 at 100× that (full thickness).
fn weight(gap_px: f64) -> f32 {
    ((gap_px / MIN_GAP_PX).log10() / 2.0).clamp(0.0, 1.0) as f32
}

// Spacing of the thinnest visible grid level: the smallest power of 10 metres
// whose gap on screen is at least MIN_GAP_PX.
pub fn grid_step(scale: f64) -> f64 {
    10f64.powf((MIN_GAP_PX / scale).log10().ceil())
}

// The lines pass through (ox, oy): the world origin normally, or the followed
// body, so the grid moves with it instead of sliding past it.
fn draw_grid(p: &Painter, cam: &Camera, r: Rect, ppp: f32, (ox, oy): (f64, f64)) {
    let step = grid_step(cam.scale);
    let (left, top) = cam.screen_to_world(r, r.min);
    let (right, bottom) = cam.screen_to_world(r, r.max);
    let (x0, x1) = (((left - ox) / step).floor() as i64, ((right - ox) / step).ceil() as i64);
    let (y0, y1) = (((bottom - oy) / step).floor() as i64, ((top - oy) / step).ceil() as i64);
    // centre of a physical pixel, so 1px lines stay crisp
    let crisp = |v: f64| ((v as f32 * ppp).round() + 0.5) / ppp;

    // Three levels: step, 10·step, 100·step. Each line is drawn once, at the
    // coarsest level its index belongs to (checked on the integer index k,
    // never on the float position).
    for level in 0..3 {
        let wgt = weight(step * 10f64.powi(level) * cam.scale);
        if wgt == 0.0 {
            continue;
        }
        let stroke = Stroke::new(1.0 + wgt, GRID.gamma_multiply(wgt * MAX_ALPHA));
        let belongs = |k: i64| {
            let a = k.abs();
            match level {
                2 => a % 100 == 0,
                1 => a % 10 == 0 && a % 100 != 0,
                _ => a % 10 != 0,
            }
        };
        let c = r.center();
        for k in (x0..=x1).filter(|&k| belongs(k)) {
            let sx = crisp((ox - cam.x + k as f64 * step) * cam.scale + c.x as f64);
            p.line_segment([pos2(sx, r.top()), pos2(sx, r.bottom())], stroke);
        }
        for k in (y0..=y1).filter(|&k| belongs(k)) {
            let sy = crisp(c.y as f64 - (oy - cam.y + k as f64 * step) * cam.scale);
            p.line_segment([pos2(r.left(), sy), pos2(r.right(), sy)], stroke);
        }
    }
}

// ---------------------------------------------------------------- bodies

// Bodies shrinking below this on screen are still drawn at this size.
const MIN_RADIUS_PX: f64 = 3.0;
// Extra click margin around a body, so tiny ones are still easy to hit.
const HIT_SLACK_PX: f64 = 4.0;

// The body drawn under screen point p, or None. Later bodies are drawn on
// top, so they win when several overlap.
fn body_at(cam: &Camera, r: Rect, bodies: &[BodyData], p: Pos2) -> Option<usize> {
    bodies.iter().rev().find_map(|b| {
        let (bx, by) = cam.world_to_screen(r, b.x, b.y);
        let rad = (b.radius * cam.scale).max(MIN_RADIUS_PX) + HIT_SLACK_PX;
        ((p.x as f64 - bx).powi(2) + (p.y as f64 - by).powi(2) <= rad * rad).then_some(b.id)
    })
}

// A circle, skipped when it's entirely off screen (a body flung to 1e150 m
// would otherwise be drawn at coordinates f32 can't hold).
fn visible_circle(r: Rect, cx: f64, cy: f64, rad: f64) -> bool {
    let (dx, dy) = ((cx - r.center().x as f64).abs(), (cy - r.center().y as f64).abs());
    dx <= r.width() as f64 / 2.0 + rad && dy <= r.height() as f64 / 2.0 + rad
}

fn draw_bodies(p: &Painter, cam: &Camera, r: Rect, bodies: &[BodyData]) {
    for b in bodies {
        let (sx, sy) = cam.world_to_screen(r, b.x, b.y);
        let rad = (b.radius * cam.scale).max(MIN_RADIUS_PX);
        if !visible_circle(r, sx, sy, rad + 60.0) {
            continue;
        }
        // zoomed far into a body: it covers the whole canvas
        let corner = (r.width() as f64).hypot(r.height() as f64);
        if rad > 1e5 && (sx - r.center().x as f64).hypot(sy - r.center().y as f64) + corner < rad {
            p.rect_filled(r, 0, BODY);
            continue;
        }
        let c = pos2(sx as f32, sy as f32);
        p.circle_filled(c, rad as f32, BODY);
        p.text(pos2((sx + rad + 6.0) as f32, sy as f32), Align2::LEFT_CENTER, format!("#{}", b.id), mono(11.0), TEXT_DIM);
    }
}

// ---------------------------------------------------------------- draft

// How close (px) the pointer must be to grab the arrow tip or a small body.
const GRAB_PX: f64 = 10.0;
const HEAD_PX: f64 = 10.0;

#[derive(Clone, Copy, PartialEq)]
enum Grab {
    Position,
    Velocity,
}

struct DraftPoints {
    cx: f64,
    cy: f64,
    tx: f64,
    ty: f64,
    r: f64,
}

// The arrow tip sits where the body would be after `arrow_time` seconds:
// the playback speed, i.e. one real second of simulation. So the arrow
// always shows how far the body travels per second you watch, whatever the
// time scale (at 1 d/s, a 100 km arrow means 100 km per day).
fn draft_points(cam: &Camera, r: Rect, d: &Draft, arrow_time: f64) -> DraftPoints {
    let (cx, cy) = cam.world_to_screen(r, d.x, d.y);
    let (tx, ty) = cam.world_to_screen(r, d.x + d.vx * arrow_time, d.y + d.vy * arrow_time);
    DraftPoints { cx, cy, tx, ty, r: (d.radius * cam.scale).max(MIN_RADIUS_PX) }
}

// What's under the pointer. The arrow tip wins over the body, so a zero
// velocity (tip at the centre) can still be pulled out.
fn hit_draft(cam: &Camera, r: Rect, d: &Draft, arrow_time: f64, p: Pos2) -> Option<Grab> {
    let dp = draft_points(cam, r, d, arrow_time);
    let (px, py) = (p.x as f64, p.y as f64);
    if (px - dp.tx).hypot(py - dp.ty) <= GRAB_PX {
        Some(Grab::Velocity)
    } else if (px - dp.cx).hypot(py - dp.cy) <= dp.r.max(GRAB_PX) {
        Some(Grab::Position)
    } else {
        None
    }
}

fn draw_draft(p: &Painter, cam: &Camera, r: Rect, d: &Draft, show_arrow: bool, speed_unit: Unit, arrow_time: f64) {
    let DraftPoints { cx, cy, mut tx, mut ty, r: rad } = draft_points(cam, r, d, arrow_time);
    let stroke = Stroke::new(1.5, ACCENT);

    // ghost body
    if visible_circle(r, cx, cy, rad) && rad < 1e6 {
        let c = pos2(cx as f32, cy as f32);
        p.circle_filled(c, rad as f32, ACCENT.gamma_multiply(0.2));
        let n = 96;
        let ring: Vec<Pos2> = (0..=n)
            .map(|i| {
                let a = i as f64 / n as f64 * std::f64::consts::TAU;
                pos2((cx + rad * a.cos()) as f32, (cy + rad * a.sin()) as f32)
            })
            .collect();
        p.extend(Shape::dashed_line(&ring, stroke, 4.0, 4.0));
    }

    if !show_arrow {
        return;
    }
    let len = (tx - cx).hypot(ty - cy);
    // an arrow far longer than the screen: cut it off, it only has to point the right way
    let max_len = 1e5;
    if len > max_len {
        tx = cx + (tx - cx) / len * max_len;
        ty = cy + (ty - cy) / len * max_len;
    }
    let len = len.min(max_len);
    let tip = pos2(tx as f32, ty as f32);
    if len > 1.0 {
        // shaft, stopping at the base of the head
        let (ux, uy) = ((tx - cx) / len, (ty - cy) / len);
        let head = HEAD_PX.min(len);
        let at = |x: f64, y: f64| pos2(x as f32, y as f32);
        p.line_segment([at(cx, cy), at(tx - ux * head, ty - uy * head)], stroke);

        // head
        p.add(Shape::convex_polygon(
            vec![
                tip,
                at(tx - ux * head - uy * head * 0.5, ty - uy * head + ux * head * 0.5),
                at(tx - ux * head + uy * head * 0.5, ty - uy * head - ux * head * 0.5),
            ],
            ACCENT,
            Stroke::NONE,
        ));

        let (lx, ly) = (tx + ux * 10.0 + 4.0, ty + uy * 10.0);
        let speed = to_precision(d.vx.hypot(d.vy) / speed_unit.factor, 4);
        p.text(at(lx, ly), Align2::LEFT_CENTER, format!("{speed} {}", speed_unit.label), mono(11.0), ACCENT);
        // what the arrow's length stands for
        let label = format!("arrow = {}", format_duration(arrow_time));
        p.text(at(lx, ly + 14.0), Align2::LEFT_CENTER, label, mono(11.0), ACCENT.gamma_multiply(0.6));
    }

    // grab handle at the tip
    p.circle(tip, 4.0, BG, stroke);
}

// ---------------------------------------------------------------- interaction

// A press that moves less than this (px) is a click, not a pan.
const CLICK_SLOP_PX: f32 = 4.0;

// What the current pointer drag is doing.
#[derive(Clone, Copy, PartialEq)]
enum Drag {
    Pan,
    Draft(Grab),
}

#[derive(Default)]
pub struct Interaction {
    drag: Option<Drag>,
    grab: (f64, f64), // keeps the body from jumping to the cursor when grabbed off-centre
    pan_moved: f32,   // px dragged so far in the current pan
}

fn round_to(v: f64, q: f64) -> f64 {
    to_precision((v / q).round() * q, 12)
}

// Values set with the cursor are rounded to a tenth of the finest visible
// grid step, so the fields show e.g. 12.3 instead of 12.30481920394.
fn snap(cam: &Camera, v: f64) -> f64 {
    round_to(v, grid_step(cam.scale) / 10.0)
}

// Velocities set with the arrow: the arrow is `arrow_time` seconds of travel,
// so a tenth of a grid step of arrow is that much less velocity. Rounded to a
// power of ten so the fields stay readable (1.2 km/s, not 1.157407 km/s).
fn snap_vel(cam: &Camera, v: f64, arrow_time: f64) -> f64 {
    round_to(v, 10f64.powf((grid_step(cam.scale) / 10.0 / arrow_time).log10().floor()))
}

impl Interaction {
    fn set_pos(&self, cam: &Camera, d: &mut Draft, (wx, wy): (f64, f64)) {
        d.x = snap(cam, wx + self.grab.0);
        d.y = snap(cam, wy + self.grab.1);
    }

    // The derived |v| and θ are rounded too (θ to 0.1°), so the fields stay short.
    fn set_vel(cam: &Camera, d: &mut Draft, (wx, wy): (f64, f64), t: f64) {
        d.set_velocity_xy(snap_vel(cam, (wx - d.x) / t, t), snap_vel(cam, (wy - d.y) / t, t));
        d.speed = snap_vel(cam, d.speed, t);
        d.angle = (d.angle * 10.0).round() / 10.0;
    }

    // Wheel zoom, drag-to-pan, clicking a body to follow it, and placing /
    // dragging the draft body.
    fn handle(&mut self, ui: &Ui, resp: &Response, view: &mut View, drafts: &mut DraftState, sim: &Sim) {
        let r = resp.rect;
        let arrow_time = sim.cfg.speed;
        let (pressed, released, delta, pos, scroll) = ui.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.primary_released(),
                i.pointer.delta(),
                i.pointer.latest_pos(),
                i.smooth_scroll_delta.y,
            )
        });
        let Some(pos) = pos else { return };
        let hovered = resp.hovered();
        let world = view.cam.screen_to_world(r, pos);

        if hovered && scroll != 0.0 {
            view.cam.zoom_at(r, pos, (scroll as f64 * 0.0015).exp());
        }

        if hovered && pressed {
            self.grab = (0.0, 0.0);
            let mode = drafts.mode;
            match (&mut drafts.draft, mode) {
                (Some(d), DraftMode::Placing) => {
                    self.set_pos(&view.cam, d, world);
                    drafts.mode = DraftMode::Aiming;
                }
                (Some(d), DraftMode::Aiming) => {
                    Self::set_vel(&view.cam, d, world, arrow_time);
                    drafts.mode = DraftMode::Editing;
                }
                (d, _) => {
                    let target = d.as_ref().and_then(|d| hit_draft(&view.cam, r, d, arrow_time, pos));
                    if let (Some(d), Some(Grab::Position)) = (d, target) {
                        self.grab = (d.x - world.0, d.y - world.1);
                    }
                    self.drag = Some(target.map_or(Drag::Pan, Drag::Draft));
                    self.pan_moved = 0.0;
                }
            }
        } else if self.drag.is_some() || hovered {
            let mode = drafts.mode;
            match (self.drag, &mut drafts.draft) {
                (Some(Drag::Pan), _) => {
                    self.pan_moved += delta.x.abs() + delta.y.abs();
                    if self.pan_moved > CLICK_SLOP_PX {
                        view.following = None; // a real drag: let go of the followed body
                    }
                    view.cam.pan(delta.x, delta.y);
                }
                (Some(Drag::Draft(Grab::Position)), Some(d)) => self.set_pos(&view.cam, d, world),
                (None, Some(d)) if mode == DraftMode::Placing => self.set_pos(&view.cam, d, world),
                (Some(Drag::Draft(Grab::Velocity)), Some(d)) => Self::set_vel(&view.cam, d, world, arrow_time),
                (None, Some(d)) if mode == DraftMode::Aiming => Self::set_vel(&view.cam, d, world, arrow_time),
                _ => {}
            }
        }

        if released {
            // a press that didn't really move is a click: follow (or unfollow) the body under it
            if self.drag == Some(Drag::Pan)
                && self.pan_moved <= CLICK_SLOP_PX
                && let Some(id) = body_at(&view.cam, r, &sim.bodies, pos)
            {
                view.toggle_follow(id);
            }
            self.drag = None;
        }

        // cursor hint
        if !hovered && self.drag.is_none() {
            return;
        }
        let icon = match (&drafts.draft, self.drag) {
            (_, Some(Drag::Pan)) => CursorIcon::Grabbing,
            (_, Some(Drag::Draft(Grab::Position))) => CursorIcon::Move,
            (_, Some(Drag::Draft(Grab::Velocity))) => CursorIcon::Crosshair,
            (None, None) if body_at(&view.cam, r, &sim.bodies, pos).is_some() => CursorIcon::PointingHand,
            (None, None) => CursorIcon::Grab,
            (Some(_), None) if drafts.mode != DraftMode::Editing => CursorIcon::Crosshair,
            (Some(d), None) => match hit_draft(&view.cam, r, d, arrow_time, pos) {
                Some(Grab::Position) => CursorIcon::Move,
                Some(Grab::Velocity) => CursorIcon::Crosshair,
                None => CursorIcon::Grab,
            },
        };
        ui.ctx().set_cursor_icon(icon);
    }
}

// The whole canvas for one frame: input, follow, then drawing.
pub fn show(ui: &mut Ui, interaction: &mut Interaction, view: &mut View, drafts: &mut DraftState, sim: &Sim, speed_unit: Unit) {
    let r = ui.max_rect();
    let resp = ui.allocate_rect(r, eframe::egui::Sense::click_and_drag());
    interaction.handle(ui, &resp, view, drafts, sim);

    // Keep the followed body in the centre (zoom still works around it).
    let mut origin = (0.0, 0.0);
    if let Some(id) = view.following {
        match sim.bodies.iter().find(|b| b.id == id) {
            Some(b) => {
                view.cam.x = b.x;
                view.cam.y = b.y;
                if view.grid_follows {
                    origin = (b.x, b.y);
                }
            }
            None => view.following = None,
        }
    }

    let p = ui.painter_at(r);
    p.rect_filled(r, 0, BG);
    draw_grid(&p, &view.cam, r, ui.ctx().pixels_per_point(), origin);
    draw_bodies(&p, &view.cam, r, &sim.bodies);
    if let Some(d) = &drafts.draft {
        draw_draft(&p, &view.cam, r, d, drafts.mode != DraftMode::Placing, speed_unit, sim.cfg.speed);
    }
}
