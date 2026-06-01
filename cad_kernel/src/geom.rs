// Geometric primitives. Tight, Copy, no virtual dispatch.

use crate::math::{Vec2, EPS, norm_angle};

#[derive(Clone, Copy, Debug)]
pub struct Line { pub a: Vec2, pub b: Vec2 }

#[derive(Clone, Copy, Debug)]
pub struct Circle { pub center: Vec2, pub radius: f64 }

#[derive(Clone, Copy, Debug)]
pub struct Arc {
    pub center: Vec2,
    pub radius: f64,
    pub start_angle: f64,   // radians, in [0, 2π)
    pub sweep_angle: f64,   // radians, in (0, 2π], positive = CCW from start
}

impl Arc {
    /// True if the given absolute angle lies on the arc's swept range.
    pub fn contains_angle(&self, abs_angle: f64) -> bool {
        let d = norm_angle(abs_angle - self.start_angle);
        d <= self.sweep_angle + EPS
    }

    pub fn endpoints(&self) -> (Vec2, Vec2) {
        let s = self.start_angle;
        let e = self.start_angle + self.sweep_angle;
        let p1 = self.center + Vec2::new(self.radius * s.cos(), self.radius * s.sin());
        let p2 = self.center + Vec2::new(self.radius * e.cos(), self.radius * e.sin());
        (p1, p2)
    }
}

/// Full ellipse, possibly rotated. The major-axis VECTOR stores both the
/// rotation (direction) and the semi-major length (magnitude); `ratio` is
/// the semi-minor / semi-major ratio in (0, 1]. ratio = 1 means circle.
///
/// Parametric form:
///   P(t) = center + a · cos(t) · û  +  b · sin(t) · v̂
/// where û = major̂, v̂ = û rotated 90° CCW, a = |major|, b = a·ratio.
#[derive(Clone, Copy, Debug)]
pub struct Ellipse {
    pub center: Vec2,
    pub major:  Vec2,
    pub ratio:  f64,
}

/// Partial ellipse — the elliptical analogue of `Arc`. `start_param` and
/// `sweep_param` are values of the parameter `t` (radians), NOT geometric
/// angles measured at the centre. For a circle they coincide; for a
/// stretched ellipse they don't.
#[derive(Clone, Copy, Debug)]
pub struct EllipseArc {
    pub ellipse:     Ellipse,
    pub start_param: f64,    // in [0, 2π)
    pub sweep_param: f64,    // (0, 2π], positive = CCW (in parameter space)
}

/// Pure geometry — the shape side of a `DObject`. Style / layer / handle
/// live on the outer `DObject` struct (see [`crate::dobject`]).
///
/// Future variants land here: Polyline, Text, MText, Hatch, BlockRef, Dim*,
/// Image, Wipeout, Viewport, Solid2D, Ray, Xline, Leader, MLeader, Tolerance,
/// Table. Each addition is a new arm + a new entry in every match below.
#[derive(Clone, Copy, Debug)]
pub enum Geom {
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Ellipse(Ellipse),
    EllipseArc(EllipseArc),
}

impl Geom {
    /// Return a copy of this geometry translated by `off`.
    pub fn translated(&self, off: Vec2) -> Geom {
        match self {
            Geom::Line(l) => Geom::Line(Line {
                a: l.a + off, b: l.b + off,
            }),
            Geom::Circle(c) => Geom::Circle(Circle {
                center: c.center + off, radius: c.radius,
            }),
            Geom::Arc(a) => Geom::Arc(Arc {
                center: a.center + off,
                radius: a.radius,
                start_angle: a.start_angle,
                sweep_angle: a.sweep_angle,
            }),
            Geom::Ellipse(e) => Geom::Ellipse(Ellipse {
                center: e.center + off,
                major:  e.major,
                ratio:  e.ratio,
            }),
            Geom::EllipseArc(ea) => Geom::EllipseArc(EllipseArc {
                ellipse:     Ellipse {
                    center: ea.ellipse.center + off,
                    major:  ea.ellipse.major,
                    ratio:  ea.ellipse.ratio,
                },
                start_param: ea.start_param,
                sweep_param: ea.sweep_param,
            }),
        }
    }

    /// Minimum distance from the dobject (its visible curve) to a point.
    pub fn distance_to_point(&self, p: Vec2) -> f64 {
        match self {
            Geom::Line(l)        => l.distance_to_point(p),
            Geom::Circle(c)      => c.distance_to_point(p),
            Geom::Arc(a)         => a.distance_to_point(p),
            Geom::Ellipse(e)     => e.distance_to_point(p),
            Geom::EllipseArc(ea) => ea.distance_to_point(p),
        }
    }
}

impl Line {
    /// Distance from this segment to a point (perpendicular if the foot is on
    /// the segment, otherwise the nearer endpoint).
    pub fn distance_to_point(&self, p: Vec2) -> f64 {
        let d = self.b - self.a;
        let len_sq = d.len_sq();
        if len_sq < EPS { return p.dist(self.a); }
        let t = ((p - self.a).dot(d) / len_sq).clamp(0.0, 1.0);
        let foot = self.a + d * t;
        p.dist(foot)
    }
}

impl Circle {
    /// Distance from this circle's curve to a point (always positive).
    pub fn distance_to_point(&self, p: Vec2) -> f64 {
        (p.dist(self.center) - self.radius).abs()
    }
}

impl Arc {
    /// Distance from this arc's visible curve to a point. If the point's angle
    /// from the centre is within the swept range, this is the radial distance;
    /// otherwise it's the distance to the nearer endpoint.
    pub fn distance_to_point(&self, p: Vec2) -> f64 {
        let v = p - self.center;
        let ang = v.angle();
        if self.contains_angle(ang) {
            (v.len() - self.radius).abs()
        } else {
            let (e1, e2) = self.endpoints();
            p.dist(e1).min(p.dist(e2))
        }
    }
}

impl Geom {
    /// Axis-aligned bounding box (min, max). For arcs / elliptical arcs this
    /// is the conservative bbox of the full underlying curve, not the tight
    /// per-quadrant one — good enough for viewport culling and fast to compute.
    pub fn bbox(&self) -> (Vec2, Vec2) {
        match self {
            Geom::Line(l) => (
                Vec2::new(l.a.x.min(l.b.x), l.a.y.min(l.b.y)),
                Vec2::new(l.a.x.max(l.b.x), l.a.y.max(l.b.y)),
            ),
            Geom::Circle(c) => (
                Vec2::new(c.center.x - c.radius, c.center.y - c.radius),
                Vec2::new(c.center.x + c.radius, c.center.y + c.radius),
            ),
            Geom::Arc(a) => (
                Vec2::new(a.center.x - a.radius, a.center.y - a.radius),
                Vec2::new(a.center.x + a.radius, a.center.y + a.radius),
            ),
            Geom::Ellipse(e)     => e.bbox(),
            Geom::EllipseArc(ea) => ea.ellipse.bbox(),
        }
    }
}

// ---- Ellipse / EllipseArc geometry ----------------------------------------

impl Ellipse {
    /// Semi-major axis length, `a`.
    pub fn semi_major(&self) -> f64 { self.major.len() }
    /// Semi-minor axis length, `b = a · ratio`.
    pub fn semi_minor(&self) -> f64 { self.semi_major() * self.ratio }

    /// Unit vector along the major axis (the "u" direction).
    pub fn u_hat(&self) -> Vec2 { self.major.normalized() }
    /// Unit vector along the minor axis (u rotated 90° CCW).
    pub fn v_hat(&self) -> Vec2 { self.u_hat().perp() }

    /// Point on the ellipse curve at parameter t (radians).
    /// P(t) = center + a·cos(t)·û + b·sin(t)·v̂
    pub fn point_at(&self, t: f64) -> Vec2 {
        let a = self.semi_major();
        let b = self.semi_minor();
        self.center + self.u_hat() * (a * t.cos()) + self.v_hat() * (b * t.sin())
    }

    /// Tangent vector (un-normalized) at parameter t. dP/dt.
    pub fn tangent_at(&self, t: f64) -> Vec2 {
        let a = self.semi_major();
        let b = self.semi_minor();
        self.u_hat() * (-a * t.sin()) + self.v_hat() * (b * t.cos())
    }

    /// Axis-aligned bbox of the FULL ellipse (regardless of any swept range).
    /// Derived from the rotated parametric form:
    ///   x_half = sqrt(a²·cos²θ + b²·sin²θ)
    ///   y_half = sqrt(a²·sin²θ + b²·cos²θ)
    /// In our representation (major = a · û, ratio = b/a) this simplifies to:
    ///   x_half = sqrt(major.x² + (ratio · major.y)²)
    ///   y_half = sqrt(major.y² + (ratio · major.x)²)
    pub fn bbox(&self) -> (Vec2, Vec2) {
        let mx = self.major.x;
        let my = self.major.y;
        let r2 = self.ratio * self.ratio;
        let hx = (mx * mx + r2 * my * my).sqrt();
        let hy = (my * my + r2 * mx * mx).sqrt();
        (Vec2::new(self.center.x - hx, self.center.y - hy),
         Vec2::new(self.center.x + hx, self.center.y + hy))
    }

    /// Closest parameter t to a world point `p`, found by Newton iteration on
    /// `f(t) = (P(t) - p) · P'(t) = 0`. Closed-form solving requires a
    /// quartic; this is fast, robust, and accurate enough for snap / hit-test.
    /// The initial guess is the angle of `p - center` in the ellipse's local
    /// frame, which is usually 1–2 iterations from the true root.
    pub fn nearest_param(&self, p: Vec2) -> f64 {
        let a = self.semi_major();
        if a < EPS { return 0.0; }
        let b = self.semi_minor();
        // Initial guess: rotate `p - center` into local frame, then take atan2
        // using the scaled coordinates so the angle matches PARAMETER space.
        let d = p - self.center;
        let lx = d.dot(self.u_hat());
        let ly = d.dot(self.v_hat());
        let mut t = (ly * a).atan2(lx * b);
        // 5 Newton iterations is more than enough for 1e-9 convergence in
        // double precision for any reasonable ratio.
        for _ in 0..5 {
            let pt = self.point_at(t);
            let dp = self.tangent_at(t);
            let d2 = -self.u_hat() * (a * t.cos()) - self.v_hat() * (b * t.sin());
            let f  = (pt - p).dot(dp);
            let fd = (pt - p).dot(d2) + dp.dot(dp);
            if fd.abs() < EPS { break; }
            t -= f / fd;
        }
        t.rem_euclid(std::f64::consts::TAU)
    }

    pub fn distance_to_point(&self, p: Vec2) -> f64 {
        let t = self.nearest_param(p);
        self.point_at(t).dist(p)
    }
}

#[cfg(test)]
mod ellipse_tests {
    use super::*;
    use crate::math::approx_eq;

    fn close(p: Vec2, x: f64, y: f64) -> bool {
        approx_eq(p.x, x) && approx_eq(p.y, y)
    }

    #[test]
    fn axis_aligned_ellipse_point_at() {
        // a = 5, b = 2, no rotation
        let e = Ellipse { center: Vec2::ZERO, major: Vec2::new(5.0, 0.0), ratio: 0.4 };
        assert!(close(e.point_at(0.0), 5.0, 0.0));
        assert!(close(e.point_at(std::f64::consts::FRAC_PI_2), 0.0, 2.0));
        assert!(close(e.point_at(std::f64::consts::PI), -5.0, 0.0));
    }

    #[test]
    fn rotated_ellipse_bbox() {
        // Rotate 90°: major now points up. Bbox half-extents swap.
        let e = Ellipse { center: Vec2::ZERO, major: Vec2::new(0.0, 5.0), ratio: 0.4 };
        let (mn, mx) = e.bbox();
        assert!(approx_eq(mx.x, 2.0));   // semi-minor along x
        assert!(approx_eq(mx.y, 5.0));   // semi-major along y
        assert!(approx_eq(mn.x, -2.0));
        assert!(approx_eq(mn.y, -5.0));
    }

    #[test]
    fn nearest_param_on_circle_is_atan2() {
        // ratio=1.0 means circle — nearest point on a circle is the radial.
        let e = Ellipse { center: Vec2::ZERO, major: Vec2::new(5.0, 0.0), ratio: 1.0 };
        let p = Vec2::new(10.0, 10.0);
        let t = e.nearest_param(p);
        let pt = e.point_at(t);
        // pt should be on the circle of r=5 in the same direction as p.
        assert!(approx_eq(pt.len(), 5.0));
        assert!(approx_eq((pt.y / pt.x).atan(), (10.0_f64 / 10.0_f64).atan()));
    }

    #[test]
    fn ellipse_arc_endpoints_and_contains() {
        let e = Ellipse { center: Vec2::ZERO, major: Vec2::new(5.0, 0.0), ratio: 0.4 };
        let ea = EllipseArc {
            ellipse: e,
            start_param: 0.0,
            sweep_param: std::f64::consts::FRAC_PI_2,
        };
        let (p1, p2) = ea.endpoints();
        assert!(close(p1, 5.0, 0.0));
        assert!(close(p2, 0.0, 2.0));
        assert!(ea.contains_param(0.0));
        assert!(ea.contains_param(std::f64::consts::FRAC_PI_4));
        assert!(!ea.contains_param(std::f64::consts::PI));
    }
}

impl EllipseArc {
    /// True if parameter `t` lies in the swept range, mod TAU.
    pub fn contains_param(&self, t: f64) -> bool {
        let d = (t - self.start_param).rem_euclid(std::f64::consts::TAU);
        d <= self.sweep_param + EPS
    }

    pub fn endpoints(&self) -> (Vec2, Vec2) {
        (self.ellipse.point_at(self.start_param),
         self.ellipse.point_at(self.start_param + self.sweep_param))
    }

    /// Distance from the visible arc to a point. If the nearest-on-full-
    /// ellipse parameter lies in the swept range, that's the foot; otherwise
    /// the answer is whichever endpoint is closer.
    pub fn distance_to_point(&self, p: Vec2) -> f64 {
        let t = self.ellipse.nearest_param(p);
        if self.contains_param(t) {
            self.ellipse.point_at(t).dist(p)
        } else {
            let (e1, e2) = self.endpoints();
            p.dist(e1).min(p.dist(e2))
        }
    }
}
