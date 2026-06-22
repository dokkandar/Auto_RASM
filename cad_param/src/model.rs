//! The parametric sketch data model — points, lines, and constraints.
//!
//! A `Sketch` is parameterised by its POINTS (each `Vec2` is two unknowns, x/y).
//! Lines reference two point ids; the solver moves the points so every
//! constraint's residual goes to zero. This is `cad_param`'s OWN structure — it
//! is not the kernel `Document`.

use cad_kernel::Vec2;

pub type PointId = usize;
pub type LineId = usize;

/// A line segment defined by two point ids (a `cad_param` line, not a kernel one).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub a: PointId,
    pub b: PointId,
}

/// A geometric constraint. Each contributes one or two residual equations the
/// solver drives to zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Constraint {
    /// Pin a point to a fixed world location (anchor). 2 residuals.
    Fixed { p: PointId, x: f64, y: f64 },
    /// Two points coincide. 2 residuals.
    Coincident { p: PointId, q: PointId },
    /// Distance between two points equals `d`. 1 residual.
    Distance { p: PointId, q: PointId, d: f64 },
    /// A line is horizontal (endpoints share y). 1 residual.
    Horizontal { line: LineId },
    /// A line is vertical (endpoints share x). 1 residual.
    Vertical { line: LineId },
    /// Two lines are parallel (direction cross-product = 0). 1 residual.
    Parallel { a: LineId, b: LineId },
    /// Two lines are perpendicular (direction dot-product = 0). 1 residual.
    Perpendicular { a: LineId, b: LineId },
    /// Two lines have equal length. 1 residual.
    EqualLength { a: LineId, b: LineId },
    /// A point lies on a line (infinite line through the segment). 1 residual.
    PointOnLine { p: PointId, line: LineId },
}

impl Constraint {
    /// How many residual equations this constraint contributes.
    pub fn residual_count(&self) -> usize {
        match self {
            Constraint::Fixed { .. } | Constraint::Coincident { .. } => 2,
            _ => 1,
        }
    }
}

/// A parametric sketch: points (the unknowns), lines, and constraints.
#[derive(Clone, Debug, Default)]
pub struct Sketch {
    pub points: Vec<Vec2>,
    pub lines: Vec<Line>,
    pub constraints: Vec<Constraint>,
}

impl Sketch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_point(&mut self, x: f64, y: f64) -> PointId {
        self.points.push(Vec2::new(x, y));
        self.points.len() - 1
    }

    pub fn add_line(&mut self, a: PointId, b: PointId) -> LineId {
        self.lines.push(Line { a, b });
        self.lines.len() - 1
    }

    pub fn add(&mut self, c: Constraint) {
        self.constraints.push(c);
    }

    /// Total residual equations (the height of the system the solver builds).
    pub fn residual_dim(&self) -> usize {
        self.constraints.iter().map(|c| c.residual_count()).sum()
    }

    /// Degrees of freedom = 2·points − residual equations. Negative/zero ⇒
    /// fully (or over-) constrained; positive ⇒ under-constrained (LM still
    /// solves, staying near the current geometry).
    pub fn dof(&self) -> i64 {
        2 * self.points.len() as i64 - self.residual_dim() as i64
    }
}
