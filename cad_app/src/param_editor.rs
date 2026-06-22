//! Parametric MODE glue — lets the app's normal tools draw into a drawing that
//! the `cad_param` constraint solver then governs. We deliberately reuse ALL the
//! existing tools/modifiers (Line, Circle, trim, snaps, …); this module only
//! adds a constraint LAYER + a Solve step. `cad_param` stays the isolated,
//! swappable solver backend (kept separate for safety / dev / library-testing);
//! the core kernel `Document`/`Geom` are not modified.
//!
//! Slice 1 constrains LINE geometry (and the points where line endpoints meet —
//! coincident endpoints are auto-merged into one sketch point, so connected
//! lines move together). Circles/arcs are drawn normally but not yet constrained
//! (needs scalar unknowns like radius in cad_param — a later slice).

use cad_kernel::{dobject::Handle, Document, Geom, Vec2};
use cad_param::{solve, Constraint, Sketch};

/// A user constraint, referencing drawing geometry by stable HANDLE (so it
/// survives edits/index shuffles). Resolved to sketch ids at solve time.
#[derive(Clone, Copy, Debug)]
pub enum CRef {
    Horizontal(Handle),
    Vertical(Handle),
    Parallel(Handle, Handle),
    Perpendicular(Handle, Handle),
    Equal(Handle, Handle),
    Length(Handle, f64),
}

impl CRef {
    pub fn label(&self) -> String {
        match self {
            CRef::Horizontal(_) => "horizontal".into(),
            CRef::Vertical(_) => "vertical".into(),
            CRef::Parallel(..) => "parallel".into(),
            CRef::Perpendicular(..) => "perpendicular".into(),
            CRef::Equal(..) => "equal length".into(),
            CRef::Length(_, d) => format!("length = {d}"),
        }
    }
}

/// Per-session parametric state held by the app (always present; `active` gates
/// the panel + behaviour). Constraints live here, NOT in the core Document.
pub struct ParamSession {
    pub active: bool,
    pub constraints: Vec<CRef>,
    pub status: String,
    pub length_input: String,
}

impl ParamSession {
    pub fn new() -> Self {
        Self { active: false, constraints: Vec::new(), status: String::new(),
               length_input: "100".into() }
    }
}

fn intern(pts: &mut Vec<Vec2>, p: Vec2) -> usize {
    const EPS: f64 = 1e-6;
    for (i, q) in pts.iter().enumerate() {
        if (*q - p).len() < EPS {
            return i;
        }
    }
    pts.push(p);
    pts.len() - 1
}

/// Build a `cad_param::Sketch` from the drawing's LINE dobjects (coincident
/// endpoints merged into shared points). Returns the sketch, the per-line
/// (point_a, point_b) ids, and a handle→line-id map for resolving constraints.
fn sketch_from_doc(
    doc: &Document,
) -> (Sketch, Vec<(usize, usize)>, std::collections::HashMap<Handle, usize>, Vec<usize>) {
    use std::collections::HashMap;
    let mut lines: Vec<(Handle, usize, Vec2, Vec2)> = Vec::new(); // (handle, dobj idx, a, b)
    for (idx, d) in doc.dobjects.iter().enumerate() {
        if let Geom::Line(l) = &d.geom {
            lines.push((d.handle, idx, l.a, l.b));
        }
    }
    let mut pts: Vec<Vec2> = Vec::new();
    let mut line_pts: Vec<(usize, usize)> = Vec::with_capacity(lines.len());
    for (_, _, a, b) in &lines {
        let pa = intern(&mut pts, *a);
        let pb = intern(&mut pts, *b);
        line_pts.push((pa, pb));
    }
    let mut sk = Sketch::new();
    for p in &pts {
        sk.add_point(p.x, p.y);
    }
    let mut line_id: HashMap<Handle, usize> = HashMap::new();
    for (k, (h, _, _, _)) in lines.iter().enumerate() {
        let (pa, pb) = line_pts[k];
        line_id.insert(*h, sk.add_line(pa, pb));
    }
    let dobj_idx: Vec<usize> = lines.iter().map(|(_, idx, _, _)| *idx).collect();
    (sk, line_pts, line_id, dobj_idx)
}

/// Build the sketch, apply the session's constraints, solve, and write the
/// solved point positions back into the drawing's lines (shared points keep
/// connected lines together). Returns a status string.
pub fn solve_doc(doc: &mut Document, session: &ParamSession) -> String {
    let (mut sk, line_pts, line_id, dobj_idx) = sketch_from_doc(doc);
    if sk.lines.is_empty() {
        return "parametric: no lines to solve".into();
    }
    // Ground the sketch: anchor the first point at its current location so the
    // solver has a fixed reference (otherwise the whole sketch can float).
    if !sk.points.is_empty() {
        let p0 = sk.points[0];
        sk.add(Constraint::Fixed { p: 0, x: p0.x, y: p0.y });
    }
    // Translate handle-based user constraints to sketch ids.
    for c in &session.constraints {
        match *c {
            CRef::Horizontal(h) => if let Some(&l) = line_id.get(&h) {
                sk.add(Constraint::Horizontal { line: l });
            },
            CRef::Vertical(h) => if let Some(&l) = line_id.get(&h) {
                sk.add(Constraint::Vertical { line: l });
            },
            CRef::Parallel(a, b) => if let (Some(&la), Some(&lb)) = (line_id.get(&a), line_id.get(&b)) {
                sk.add(Constraint::Parallel { a: la, b: lb });
            },
            CRef::Perpendicular(a, b) => if let (Some(&la), Some(&lb)) = (line_id.get(&a), line_id.get(&b)) {
                sk.add(Constraint::Perpendicular { a: la, b: lb });
            },
            CRef::Equal(a, b) => if let (Some(&la), Some(&lb)) = (line_id.get(&a), line_id.get(&b)) {
                sk.add(Constraint::EqualLength { a: la, b: lb });
            },
            CRef::Length(h, d) => if let Some(&l) = line_id.get(&h) {
                let ln = sk.lines[l];
                sk.add(Constraint::Distance { p: ln.a, q: ln.b, d });
            },
        }
    }
    let rep = solve(&mut sk);
    // Write solved coords back into the drawing's lines.
    for (k, &idx) in dobj_idx.iter().enumerate() {
        let (pa, pb) = line_pts[k];
        if let Some(d) = doc.dobjects.get_mut(idx) {
            if let Geom::Line(l) = &mut d.geom {
                l.a = sk.points[pa];
                l.b = sk.points[pb];
            }
        }
    }
    format!(
        "solved: {} pts, {} lines, {} constraints · dof={} · rms={:.2e}",
        sk.points.len(), sk.lines.len(), session.constraints.len(), rep.dof, rep.residual
    )
}
