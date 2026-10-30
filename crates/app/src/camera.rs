use eframe::egui::{Pos2, Rect};

// World space is in metres with +y up; screen space is points with +y down.
pub struct Camera {
    pub x: f64, // world point at the centre of the canvas
    pub y: f64,
    pub scale: f64, // points per metre
}

pub const MIN_SCALE: f64 = 1e-12;
pub const MAX_SCALE: f64 = 1e6;

impl Camera {
    pub fn world_to_screen(&self, r: Rect, wx: f64, wy: f64) -> (f64, f64) {
        let c = r.center();
        ((wx - self.x) * self.scale + c.x as f64, c.y as f64 - (wy - self.y) * self.scale)
    }

    pub fn screen_to_world(&self, r: Rect, p: Pos2) -> (f64, f64) {
        let c = r.center();
        ((p.x - c.x) as f64 / self.scale + self.x, (c.y - p.y) as f64 / self.scale + self.y)
    }

    // Zoom by `factor`, keeping the world point under `p` fixed on screen.
    pub fn zoom_at(&mut self, r: Rect, p: Pos2, factor: f64) {
        let (wx, wy) = self.screen_to_world(r, p);
        self.scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        let c = r.center();
        self.x = wx - (p.x - c.x) as f64 / self.scale;
        self.y = wy - (c.y - p.y) as f64 / self.scale;
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.x -= dx as f64 / self.scale;
        self.y += dy as f64 / self.scale;
    }
}
