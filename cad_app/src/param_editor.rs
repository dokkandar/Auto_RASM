//! Parametric sketch editor — an ISOLATED panel over the `cad_param` constraint
//! solver. Self-contained (its own egui window + canvas + view transform); the
//! only app hooks are a field, one render call, and a File-menu item. The core
//! kernel / Document / geometry are never touched (see the independent-module
//! rule). Drag a point and the solver keeps every constraint satisfied.

use cad_kernel::Vec2;
use cad_param::{read_rsmp, solve, write_rsmp, Constraint, Sketch};

pub struct ParamEditor {
    pub sketch: Sketch,
    status: String,
    path: String,
    // view: sketch-world → screen
    scale: f32,
    offset: egui::Vec2,
    fitted: bool,
    drag_point: Option<usize>,
}

impl ParamEditor {
    /// Demo: a perturbed quad constrained to stay an axis-aligned rectangle with
    /// ONE corner anchored — but width/height are FREE (2 DOF), so dragging the
    /// opposite corner resizes it while the solver keeps it rectangular.
    pub fn demo() -> Self {
        let mut s = Sketch::new();
        let p0 = s.add_point(0.0, 0.0);
        let p1 = s.add_point(120.0, 8.0);
        let p2 = s.add_point(115.0, 70.0);
        let p3 = s.add_point(5.0, 64.0);
        let l0 = s.add_line(p0, p1);
        let l1 = s.add_line(p1, p2);
        let l2 = s.add_line(p2, p3);
        let l3 = s.add_line(p3, p0);
        s.add(Constraint::Fixed { p: p0, x: 0.0, y: 0.0 });
        s.add(Constraint::Horizontal { line: l0 });
        s.add(Constraint::Vertical { line: l1 });
        s.add(Constraint::Horizontal { line: l2 });
        s.add(Constraint::Vertical { line: l3 });
        let mut e = ParamEditor {
            sketch: s, status: String::new(), path: "sketch.rsmp".into(),
            scale: 1.0, offset: egui::vec2(0.0, 0.0), fitted: false, drag_point: None,
        };
        e.do_solve();
        e
    }

    fn do_solve(&mut self) {
        let r = solve(&mut self.sketch);
        self.status = format!(
            "dof={}   {}   iters={}   rms={:.2e}",
            r.dof, if r.converged { "converged" } else { "not converged" },
            r.iterations, r.residual);
    }

    fn is_fixed(&self, i: usize) -> bool {
        self.sketch.constraints.iter()
            .any(|c| matches!(c, Constraint::Fixed { p, .. } if *p == i))
    }

    fn fit(&mut self, rect: egui::Rect) {
        if self.sketch.points.is_empty() { return; }
        let (mut mn, mut mx) = (self.sketch.points[0], self.sketch.points[0]);
        for p in &self.sketch.points {
            mn.x = mn.x.min(p.x); mn.y = mn.y.min(p.y);
            mx.x = mx.x.max(p.x); mx.y = mx.y.max(p.y);
        }
        let w = (mx.x - mn.x).max(1.0);
        let h = (mx.y - mn.y).max(1.0);
        let s = ((rect.width() as f64 - 60.0) / w).min((rect.height() as f64 - 60.0) / h);
        self.scale = (s as f32).clamp(0.05, 1000.0);
        let c = (mn + mx) * 0.5;
        self.offset = egui::vec2(-c.x as f32, -c.y as f32);
        self.fitted = true;
    }

    fn w2s(&self, p: Vec2, rect: egui::Rect) -> egui::Pos2 {
        let c = rect.center();
        egui::pos2(c.x + (p.x as f32 + self.offset.x) * self.scale,
                   c.y - (p.y as f32 + self.offset.y) * self.scale)
    }
    fn s2w(&self, s: egui::Pos2, rect: egui::Rect) -> Vec2 {
        let c = rect.center();
        Vec2::new(((s.x - c.x) / self.scale - self.offset.x) as f64,
                  (-(s.y - c.y) / self.scale - self.offset.y) as f64)
    }

    /// Render the window. Returns false once the user closes it.
    pub fn render(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        egui::Window::new("Parametric Sketch  ·  cad_param")
            .id(egui::Id::new("param_editor"))
            .open(&mut open)
            .resizable(true)
            .default_size(egui::vec2(640.0, 520.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Solve").clicked() { self.do_solve(); }
                    if ui.button("Reset demo").clicked() { *self = ParamEditor::demo(); }
                    ui.separator();
                    ui.add(egui::TextEdit::singleline(&mut self.path).desired_width(150.0));
                    if ui.button("Save .rsmp").clicked() {
                        match std::fs::write(&self.path, write_rsmp(&self.sketch)) {
                            Ok(_) => self.status = format!("saved {}", self.path),
                            Err(e) => self.status = format!("save error: {e}"),
                        }
                    }
                    if ui.button("Load .rsmp").clicked() {
                        match std::fs::read_to_string(&self.path)
                            .map_err(|e| e.to_string())
                            .and_then(|t| read_rsmp(&t))
                        {
                            Ok(s) => { self.sketch = s; self.fitted = false; self.do_solve(); }
                            Err(e) => self.status = format!("load error: {e}"),
                        }
                    }
                });
                ui.label(egui::RichText::new(&self.status).weak());
                ui.small("drag a yellow point — the solver keeps the constraints \
                          (red = fixed anchor)");

                let avail = ui.available_size();
                let (resp, painter) =
                    ui.allocate_painter(avail, egui::Sense::click_and_drag());
                let rect = resp.rect;
                painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(16, 20, 26));
                if !self.fitted { self.fit(rect); }

                // lines
                for l in &self.sketch.lines {
                    let a = self.w2s(self.sketch.points[l.a], rect);
                    let b = self.w2s(self.sketch.points[l.b], rect);
                    painter.line_segment([a, b],
                        egui::Stroke::new(1.6, egui::Color32::from_rgb(150, 200, 235)));
                }
                // points
                for i in 0..self.sketch.points.len() {
                    let sp = self.w2s(self.sketch.points[i], rect);
                    let col = if self.is_fixed(i) {
                        egui::Color32::from_rgb(240, 120, 120)
                    } else {
                        egui::Color32::from_rgb(245, 220, 110)
                    };
                    painter.circle_filled(sp, 4.5, col);
                }

                // ---- drag a (non-fixed) point; solve with it pinned ----------
                if resp.drag_started() {
                    if let Some(cur) = resp.interact_pointer_pos() {
                        let mut best = None;
                        let mut best_d = 14.0;
                        for i in 0..self.sketch.points.len() {
                            if self.is_fixed(i) { continue; }
                            let d = self.w2s(self.sketch.points[i], rect).distance(cur);
                            if d < best_d { best_d = d; best = Some(i); }
                        }
                        self.drag_point = best;
                    }
                }
                if let Some(i) = self.drag_point {
                    if resp.dragged() {
                        if let Some(cur) = resp.interact_pointer_pos() {
                            let w = self.s2w(cur, rect);
                            // Pin the dragged point to the cursor and solve so the
                            // rest of the sketch follows the constraints.
                            let mut tmp = self.sketch.clone();
                            tmp.points[i] = w;
                            tmp.add(Constraint::Fixed { p: i, x: w.x, y: w.y });
                            let r = solve(&mut tmp);
                            self.sketch.points = tmp.points;
                            self.status = format!(
                                "drag p{i} → ({:.1},{:.1})   rms={:.2e}", w.x, w.y, r.residual);
                            ctx.request_repaint();
                        }
                    }
                }
                if resp.drag_stopped() {
                    self.drag_point = None;
                    self.do_solve();
                }
            });
        open
    }
}
