//! Wall junction solver — smart-dobject category, member #1.
//!
//! A wall is the offset of its centerline by ±thickness/2 (`Geom::Wall`
//! stores the centerline as identity and derives the two face lines). When
//! two walls share an endpoint (a "node"), their derived faces are MITRED
//! at that node instead of overlapping. This is **Model A**: walls stay
//! independent dobjects; the join is recomputed every frame from endpoint
//! coincidence — no persistent node graph.
//!
//! **Scenario 1 (L-corner, sharp miter)** — extracted from a user session
//! dump: offset both centerlines ±t/2 → 4 faces, then fillet-radius-0 the
//! adjacent face pairs (= extend/trim to their intersection = the miter).
//! Here that's done analytically: at the shared node, intersect each wall's
//! face with the neighbour's facing face → corner vertex → trim.
//! See `Smart_Dobjects.md` (scenarios 1b rounded and 2 T-junction are owed).

use cad_kernel::{Vec2, Wall};

/// World-unit tolerance for treating two wall endpoints as the same node.
pub const JOIN_TOL: f64 = 1e-4;

/// Derived (possibly mitred) faces of one wall — each a single segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallFaces {
    pub left:  (Vec2, Vec2),
    pub right: (Vec2, Vec2),
}

/// Infinite-line intersection: line through `p1` dir `d1` vs `p2` dir `d2`.
/// `None` when parallel.
fn line_intersect(p1: Vec2, d1: Vec2, p2: Vec2, d2: Vec2) -> Option<Vec2> {
    let cross = d1.x * d2.y - d1.y * d2.x;
    if cross.abs() < 1e-12 { return None; }
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let t = (dx * d2.y - dy * d2.x) / cross;
    Some(p1 + d1 * t)
}

/// Two walls are "the same" (so a wall never joins to itself / an exact dup).
fn same_wall(a: &Wall, b: &Wall) -> bool {
    let close = |p: Vec2, q: Vec2| (p - q).len() < JOIN_TOL;
    (close(a.start, b.start) && close(a.end, b.end))
        || (close(a.start, b.end) && close(a.end, b.start))
}

/// Derive `this` wall's faces, mitring each end whose node coincides with a
/// different wall's end. `all` is every wall in scope (may include `this`;
/// identical walls are skipped). `None` only for a degenerate wall.
///
/// Miter rule (symmetric, order-independent): at a node, relative to each
/// wall's OUTGOING direction (away from the node),
///   miter_inner = this.leftOut  ∩ neighbour.rightOut
///   miter_outer = this.rightOut ∩ neighbour.leftOut
/// and the node-side endpoint of each face is moved to the matching miter.
pub fn solve_faces(this: &Wall, all: &[Wall]) -> Option<WallFaces> {
    let ll = this.left_line()?;
    let rl = this.right_line()?;
    let mut left  = (ll.a, ll.b);
    let mut right = (rl.a, rl.b);

    for (node, at_start) in [(this.start, true), (this.end, false)] {
        let neighbor = all.iter().find(|n| {
            !same_wall(this, n)
                && !n.is_curved()   // curved corner walls meet tangentially — no miter
                && ((n.start - node).len() < JOIN_TOL || (n.end - node).len() < JOIN_TOL)
        });
        let Some(n) = neighbor else { continue };
        let (Some(nl), Some(nr)) = (n.left_line(), n.right_line()) else { continue };
        let n_at_start = (n.start - node).len() < JOIN_TOL;

        // "left-out" / "right-out" = faces relative to the outgoing dir.
        //   node == start: stored left is left-out  (node endpoint = .a)
        //   node == end:   stored right is left-out (node endpoint = .b)
        let this_lo = if at_start { ll } else { rl };
        let this_ro = if at_start { rl } else { ll };
        let n_lo    = if n_at_start { nl } else { nr };
        let n_ro    = if n_at_start { nr } else { nl };

        let dt = this_lo.b - this_lo.a;       // both this-faces share this dir
        let m1 = line_intersect(this_lo.a, dt, n_ro.a, n_ro.b - n_ro.a);
        let m2 = line_intersect(this_ro.a, dt, n_lo.a, n_lo.b - n_lo.a);

        if at_start {
            if let Some(m) = m1 { left.0  = m; }
            if let Some(m) = m2 { right.0 = m; }
        } else {
            if let Some(m) = m1 { right.1 = m; }
            if let Some(m) = m2 { left.1  = m; }
        }
    }
    Some(WallFaces { left, right })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(p: Vec2, q: Vec2) -> bool { (p - q).len() < 1e-6 }

    #[test]
    fn lone_wall_keeps_full_faces() {
        let w = Wall { start: Vec2::new(0.0, 0.0), end: Vec2::new(10.0, 0.0), thickness: 2.0, style: 0, bulge: 0.0 };
        let f = solve_faces(&w, &[w]).unwrap();
        assert!(close(f.left.0, Vec2::new(0.0, 1.0)));
        assert!(close(f.left.1, Vec2::new(10.0, 1.0)));
        assert!(close(f.right.0, Vec2::new(0.0, -1.0)));
        assert!(close(f.right.1, Vec2::new(10.0, -1.0)));
    }

    #[test]
    fn l_corner_90deg_miters_both_faces() {
        // A: (0,0)->(10,0)  B: (0,0)->(0,10), thickness 2, shared node (0,0).
        let a = Wall { start: Vec2::new(0.0, 0.0), end: Vec2::new(10.0, 0.0), thickness: 2.0, style: 0, bulge: 0.0 };
        let b = Wall { start: Vec2::new(0.0, 0.0), end: Vec2::new(0.0, 10.0), thickness: 2.0, style: 0, bulge: 0.0 };
        let all = vec![a, b];
        let fa = solve_faces(&a, &all).unwrap();
        // A's start-side faces miter to the inner (1,1) and outer (-1,-1).
        assert!(close(fa.left.0,  Vec2::new(1.0, 1.0)),  "inner miter, got {:?}", fa.left.0);
        assert!(close(fa.right.0, Vec2::new(-1.0, -1.0)), "outer miter, got {:?}", fa.right.0);
        // Far end untouched.
        assert!(close(fa.left.1,  Vec2::new(10.0, 1.0)));
        assert!(close(fa.right.1, Vec2::new(10.0, -1.0)));
    }

    #[test]
    fn l_corner_any_angle_meets_at_a_point() {
        // 45° corner: A east, B north-east. Faces must still meet (no gap):
        // the two inner faces share the inner miter, the two outer share outer.
        let a = Wall { start: Vec2::new(0.0, 0.0), end: Vec2::new(10.0, 0.0), thickness: 2.0, style: 0, bulge: 0.0 };
        let b = Wall { start: Vec2::new(0.0, 0.0), end: Vec2::new(7.07, 7.07), thickness: 2.0, style: 0, bulge: 0.0 };
        let all = vec![a, b];
        let fa = solve_faces(&a, &all).unwrap();
        let fb = solve_faces(&b, &all).unwrap();
        // A.start-left (inner) should coincide with B's matching inner face end.
        // Both inner faces meet at the same point; both outer faces meet too.
        let a_inner = fa.left.0;
        let a_outer = fa.right.0;
        let b_ends = [fb.left.0, fb.right.0];
        assert!(b_ends.iter().any(|p| close(*p, a_inner)),
            "A inner {:?} not shared by B {:?}", a_inner, b_ends);
        assert!(b_ends.iter().any(|p| close(*p, a_outer)),
            "A outer {:?} not shared by B {:?}", a_outer, b_ends);
    }
}
