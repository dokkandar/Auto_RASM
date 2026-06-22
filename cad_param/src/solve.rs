//! The constraint solver — damped least squares (Levenberg–Marquardt).
//!
//! Unknowns are the flat point-coordinate vector `x = [p0.x, p0.y, p1.x, …]`.
//! Each constraint contributes residual equations `r(x)`; we minimise `‖r‖²` by
//! Gauss–Newton with Marquardt damping:
//!
//! ```text
//!   (JᵀJ + λ·diag(JᵀJ)) Δx = −Jᵀr
//! ```
//!
//! The Jacobian `J` is numerical (central-ish forward difference) — robust and
//! enough for sketch sizes. The dense linear solve is Gaussian elimination with
//! partial pivoting, implemented here (no external deps).

use crate::model::{Constraint, Sketch};

/// Outcome of a solve.
#[derive(Clone, Copy, Debug)]
pub struct SolveReport {
    pub converged: bool,
    pub iterations: usize,
    /// RMS residual at the end (≈0 when satisfied).
    pub residual: f64,
    /// Degrees of freedom (see `Sketch::dof`).
    pub dof: i64,
}

#[inline]
fn pt(x: &[f64], i: usize) -> (f64, f64) {
    (x[2 * i], x[2 * i + 1])
}

/// Residual vector for the sketch at coordinate vector `x` (len = 2·points).
pub fn residuals(s: &Sketch, x: &[f64]) -> Vec<f64> {
    let mut r = Vec::with_capacity(s.residual_dim());
    let line_pts = |l: crate::model::Line| (pt(x, l.a), pt(x, l.b));
    for c in &s.constraints {
        match *c {
            Constraint::Fixed { p, x: fx, y: fy } => {
                let (px, py) = pt(x, p);
                r.push(px - fx);
                r.push(py - fy);
            }
            Constraint::Coincident { p, q } => {
                let (px, py) = pt(x, p);
                let (qx, qy) = pt(x, q);
                r.push(px - qx);
                r.push(py - qy);
            }
            Constraint::Distance { p, q, d } => {
                let (px, py) = pt(x, p);
                let (qx, qy) = pt(x, q);
                r.push(((px - qx).powi(2) + (py - qy).powi(2)).sqrt() - d);
            }
            Constraint::Horizontal { line } => {
                let ((_, ay), (_, by)) = line_pts(s.lines[line]);
                r.push(ay - by);
            }
            Constraint::Vertical { line } => {
                let ((ax, _), (bx, _)) = line_pts(s.lines[line]);
                r.push(ax - bx);
            }
            Constraint::Parallel { a, b } => {
                let ((ax, ay), (bx, by)) = line_pts(s.lines[a]);
                let ((cx, cy), (dx, dy)) = line_pts(s.lines[b]);
                // direction cross product = 0
                r.push((bx - ax) * (dy - cy) - (by - ay) * (dx - cx));
            }
            Constraint::Perpendicular { a, b } => {
                let ((ax, ay), (bx, by)) = line_pts(s.lines[a]);
                let ((cx, cy), (dx, dy)) = line_pts(s.lines[b]);
                // direction dot product = 0
                r.push((bx - ax) * (dx - cx) + (by - ay) * (dy - cy));
            }
            Constraint::EqualLength { a, b } => {
                let ((ax, ay), (bx, by)) = line_pts(s.lines[a]);
                let ((cx, cy), (dx, dy)) = line_pts(s.lines[b]);
                let la = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
                let lb = ((dx - cx).powi(2) + (dy - cy).powi(2)).sqrt();
                r.push(la - lb);
            }
            Constraint::PointOnLine { p, line } => {
                let (px, py) = pt(x, p);
                let ((ax, ay), (bx, by)) = line_pts(s.lines[line]);
                // cross(b-a, p-a) = 0
                r.push((bx - ax) * (py - ay) - (by - ay) * (px - ax));
            }
        }
    }
    r
}

fn sum_sq(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum()
}

/// Solve the sketch in place: move its points to satisfy the constraints as
/// closely as possible. Returns convergence info.
pub fn solve(s: &mut Sketch) -> SolveReport {
    let n = 2 * s.points.len();
    let dof = s.dof();
    let mut x: Vec<f64> = s.points.iter().flat_map(|p| [p.x, p.y]).collect();

    // HARD-fix: pin the coordinates named by `Fixed` and remove them from the
    // unknowns. Soft-penalising a Fixed lets the anchor tilt by a hair, which an
    // under-constrained DOF then exploits (sliding off to infinity). Hard
    // elimination keeps fixed points exact and the null space well-behaved.
    let mut locked = vec![false; n];
    for c in &s.constraints {
        if let Constraint::Fixed { p, x: fx, y: fy } = *c {
            if 2 * p + 1 < n {
                x[2 * p] = fx;
                x[2 * p + 1] = fy;
                locked[2 * p] = true;
                locked[2 * p + 1] = true;
            }
        }
    }
    let free: Vec<usize> = (0..n).filter(|i| !locked[*i]).collect();
    let nf = free.len();

    let mut r = residuals(s, &x);
    let m = r.len();
    if m == 0 || nf == 0 {
        for (i, p) in s.points.iter_mut().enumerate() { p.x = x[2 * i]; p.y = x[2 * i + 1]; }
        let rms = if m == 0 { 0.0 } else { (sum_sq(&r) / m as f64).sqrt() };
        return SolveReport { converged: rms < 1e-6, iterations: 0, residual: rms, dof };
    }

    let mut cost = sum_sq(&r);
    let mut lambda = 1e-3_f64;
    const MAX_ITER: usize = 200;
    const TOL: f64 = 1e-10; // RMS residual

    let mut iters = 0;
    while iters < MAX_ITER {
        iters += 1;
        if (cost / m as f64).sqrt() < TOL {
            break;
        }
        // Numerical Jacobian (m×n), row-major. CENTRAL differences (error
        // O(h²)) — forward differences (O(h)) left a ~1e-5 residual floor that
        // stalled bilinear constraints (perpendicular/parallel) short of tol.
        let h = 1e-6;
        let mut jac = vec![0.0; m * nf];
        for (k, &j) in free.iter().enumerate() {
            let old = x[j];
            x[j] = old + h;
            let rp = residuals(s, &x);
            x[j] = old - h;
            let rm = residuals(s, &x);
            x[j] = old;
            for i in 0..m {
                jac[i * nf + k] = (rp[i] - rm[i]) / (2.0 * h);
            }
        }
        // JᵀJ (nf×nf) and Jᵀr (nf) over the FREE coords.
        let mut jtj = vec![0.0; nf * nf];
        let mut jtr = vec![0.0; nf];
        for i in 0..m {
            for a in 0..nf {
                let jia = jac[i * nf + a];
                if jia == 0.0 {
                    continue;
                }
                jtr[a] += jia * r[i];
                for b in 0..nf {
                    jtj[a * nf + b] += jia * jac[i * nf + b];
                }
            }
        }
        // LM step with adaptive damping.
        let mut accepted = false;
        for _try in 0..10 {
            let mut a_mat = jtj.clone();
            for d in 0..nf {
                a_mat[d * nf + d] += lambda * jtj[d * nf + d] + 1e-12; // Marquardt + floor
            }
            let rhs: Vec<f64> = jtr.iter().map(|v| -v).collect();
            if let Some(dx) = solve_linear(a_mat, rhs, nf) {
                let mut x_new = x.clone();
                for (k, &j) in free.iter().enumerate() { x_new[j] = x[j] + dx[k]; }
                let r_new = residuals(s, &x_new);
                let cost_new = sum_sq(&r_new);
                if cost_new < cost {
                    x = x_new;
                    r = r_new;
                    cost = cost_new;
                    lambda = (lambda * 0.5).max(1e-12);
                    accepted = true;
                    break;
                } else {
                    lambda = (lambda * 4.0).min(1e12);
                }
            } else {
                lambda = (lambda * 4.0).min(1e12);
            }
        }
        if !accepted {
            break; // stuck (singular or no improvement)
        }
    }

    for (i, p) in s.points.iter_mut().enumerate() {
        p.x = x[2 * i];
        p.y = x[2 * i + 1];
    }
    let rms = (cost / m as f64).sqrt();
    SolveReport { converged: rms < 1e-6, iterations: iters, residual: rms, dof }
}

/// Solve a dense `n×n` system `A·x = b` by Gaussian elimination with partial
/// pivoting. `a` is row-major (consumed). Returns None if singular.
fn solve_linear(mut a: Vec<f64>, mut b: Vec<f64>, n: usize) -> Option<Vec<f64>> {
    for col in 0..n {
        // partial pivot
        let mut piv = col;
        let mut best = a[col * n + col].abs();
        for r in (col + 1)..n {
            let v = a[r * n + col].abs();
            if v > best {
                best = v;
                piv = r;
            }
        }
        if best < 1e-14 {
            return None;
        }
        if piv != col {
            for k in 0..n {
                a.swap(col * n + k, piv * n + k);
            }
            b.swap(col, piv);
        }
        let d = a[col * n + col];
        for r in (col + 1)..n {
            let f = a[r * n + col] / d;
            if f == 0.0 {
                continue;
            }
            for k in col..n {
                a[r * n + k] -= f * a[col * n + k];
            }
            b[r] -= f * b[col];
        }
    }
    // back-substitution
    let mut x = vec![0.0; n];
    for r in (0..n).rev() {
        let mut s = b[r];
        for k in (r + 1)..n {
            s -= a[r * n + k] * x[k];
        }
        x[r] = s / a[r * n + r];
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Constraint, Sketch};

    fn dist(s: &Sketch, p: usize, q: usize) -> f64 {
        (s.points[p] - s.points[q]).len()
    }

    #[test]
    fn distance_constraint_scales_segment() {
        let mut s = Sketch::new();
        let p0 = s.add_point(0.0, 0.0);
        let p1 = s.add_point(3.0, 4.0); // currently length 5
        s.add(Constraint::Fixed { p: p0, x: 0.0, y: 0.0 });
        s.add(Constraint::Distance { p: p0, q: p1, d: 10.0 });
        let rep = solve(&mut s);
        assert!(rep.converged, "rms={}", rep.residual);
        assert!((dist(&s, p0, p1) - 10.0).abs() < 1e-6);
        // p0 stayed put
        assert!(s.points[p0].len() < 1e-6);
    }

    #[test]
    fn perpendicular_makes_right_angle() {
        let mut s = Sketch::new();
        let a = s.add_point(0.0, 0.0);
        let b = s.add_point(10.0, 0.0);
        let c = s.add_point(2.0, 1.0); // skewed
        let l0 = s.add_line(a, b);
        let l1 = s.add_line(a, c);
        s.add(Constraint::Fixed { p: a, x: 0.0, y: 0.0 });
        s.add(Constraint::Fixed { p: b, x: 10.0, y: 0.0 });
        s.add(Constraint::Perpendicular { a: l0, b: l1 });
        let rep = solve(&mut s);
        assert!(rep.converged, "rms={}", rep.residual);
        let u = s.points[b] - s.points[a];
        let v = s.points[c] - s.points[a];
        assert!(u.dot(v).abs() < 1e-6, "dot={}", u.dot(v));
    }

    #[test]
    fn solves_a_rectangle() {
        // four perturbed corners → axis-aligned 10×5 rectangle anchored at origin
        let mut s = Sketch::new();
        let p0 = s.add_point(0.0, 0.0);
        let p1 = s.add_point(10.0, 0.3);
        let p2 = s.add_point(9.8, 5.0);
        let p3 = s.add_point(0.2, 4.9);
        let l0 = s.add_line(p0, p1); // bottom
        let l1 = s.add_line(p1, p2); // right
        let l2 = s.add_line(p2, p3); // top
        let l3 = s.add_line(p3, p0); // left
        s.add(Constraint::Fixed { p: p0, x: 0.0, y: 0.0 });
        s.add(Constraint::Horizontal { line: l0 });
        s.add(Constraint::Vertical { line: l1 });
        s.add(Constraint::Horizontal { line: l2 });
        s.add(Constraint::Vertical { line: l3 });
        s.add(Constraint::Distance { p: p0, q: p1, d: 10.0 });
        s.add(Constraint::Distance { p: p1, q: p2, d: 5.0 });
        let rep = solve(&mut s);
        assert!(rep.converged, "rms={}", rep.residual);
        let close = |a: cad_kernel::Vec2, x: f64, y: f64| (a.x - x).abs() < 1e-5 && (a.y - y).abs() < 1e-5;
        assert!(close(s.points[p0], 0.0, 0.0), "{:?}", s.points[p0]);
        assert!(close(s.points[p1], 10.0, 0.0), "{:?}", s.points[p1]);
        assert!(close(s.points[p2], 10.0, 5.0), "{:?}", s.points[p2]);
        assert!(close(s.points[p3], 0.0, 5.0), "{:?}", s.points[p3]);
        assert_eq!(rep.dof, 0); // exactly constrained
    }

    #[test]
    fn parallel_constraint() {
        let mut s = Sketch::new();
        let a = s.add_point(0.0, 0.0);
        let b = s.add_point(10.0, 0.0);
        let c = s.add_point(0.0, 5.0);
        let d = s.add_point(9.0, 6.0); // not parallel yet
        let l0 = s.add_line(a, b);
        let l1 = s.add_line(c, d);
        s.add(Constraint::Fixed { p: a, x: 0.0, y: 0.0 });
        s.add(Constraint::Fixed { p: b, x: 10.0, y: 0.0 });
        s.add(Constraint::Fixed { p: c, x: 0.0, y: 5.0 });
        s.add(Constraint::Parallel { a: l0, b: l1 });
        let rep = solve(&mut s);
        assert!(rep.converged, "rms={}", rep.residual);
        let u = s.points[b] - s.points[a];
        let v = s.points[d] - s.points[c];
        assert!((u.x * v.y - u.y * v.x).abs() < 1e-6); // cross ≈ 0
    }
}
