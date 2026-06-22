//! `.rsmp` — the parametric sketch file format, owned ENTIRELY by `cad_param`.
//!
//! A simple, human-readable line format (the core RSM/DXF code is never touched).
//! Non-parametric drawings keep using the core `.rsm`/`.dxf`; selecting
//! "parametric" in File ▸ New uses this instead.
//!
//! ```text
//! RSMP1
//! P <x> <y>                 ; a point (order = point id)
//! L <a> <b>                 ; a line between point ids
//! C fixed <p> <x> <y>
//! C coincident <p> <q>
//! C distance <p> <q> <d>
//! C horizontal <line>
//! C vertical <line>
//! C parallel <l1> <l2>
//! C perpendicular <l1> <l2>
//! C equal <l1> <l2>
//! C ponline <p> <line>
//! ```

use crate::model::{Constraint, Sketch};

const MAGIC: &str = "RSMP1";

pub fn write_rsmp(s: &Sketch) -> String {
    let mut out = String::from(MAGIC);
    out.push('\n');
    for p in &s.points {
        out.push_str(&format!("P {} {}\n", p.x, p.y));
    }
    for l in &s.lines {
        out.push_str(&format!("L {} {}\n", l.a, l.b));
    }
    for c in &s.constraints {
        match *c {
            Constraint::Fixed { p, x, y } => out.push_str(&format!("C fixed {p} {x} {y}\n")),
            Constraint::Coincident { p, q } => out.push_str(&format!("C coincident {p} {q}\n")),
            Constraint::Distance { p, q, d } => out.push_str(&format!("C distance {p} {q} {d}\n")),
            Constraint::Horizontal { line } => out.push_str(&format!("C horizontal {line}\n")),
            Constraint::Vertical { line } => out.push_str(&format!("C vertical {line}\n")),
            Constraint::Parallel { a, b } => out.push_str(&format!("C parallel {a} {b}\n")),
            Constraint::Perpendicular { a, b } => out.push_str(&format!("C perpendicular {a} {b}\n")),
            Constraint::EqualLength { a, b } => out.push_str(&format!("C equal {a} {b}\n")),
            Constraint::PointOnLine { p, line } => out.push_str(&format!("C ponline {p} {line}\n")),
        }
    }
    out
}

pub fn read_rsmp(text: &str) -> Result<Sketch, String> {
    let mut lines = text.lines();
    let header = lines.next().unwrap_or("").trim();
    if header != MAGIC {
        return Err(format!("not a .rsmp file (header `{header}`, expected `{MAGIC}`)"));
    }
    let mut s = Sketch::new();
    for (i, raw) in lines.enumerate() {
        let line = raw.split(';').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        let lineno = i + 2;
        let f = |k: usize| -> Result<f64, String> {
            t.get(k).ok_or_else(|| format!("line {lineno}: missing field {k}"))?
                .parse::<f64>().map_err(|e| format!("line {lineno}: bad number: {e}"))
        };
        let u = |k: usize| -> Result<usize, String> {
            t.get(k).ok_or_else(|| format!("line {lineno}: missing field {k}"))?
                .parse::<usize>().map_err(|e| format!("line {lineno}: bad index: {e}"))
        };
        match t[0] {
            "P" => { s.add_point(f(1)?, f(2)?); }
            "L" => { s.add_line(u(1)?, u(2)?); }
            "C" => {
                let kind = *t.get(1).ok_or_else(|| format!("line {lineno}: C with no kind"))?;
                let c = match kind {
                    "fixed" => Constraint::Fixed { p: u(2)?, x: f(3)?, y: f(4)? },
                    "coincident" => Constraint::Coincident { p: u(2)?, q: u(3)? },
                    "distance" => Constraint::Distance { p: u(2)?, q: u(3)?, d: f(4)? },
                    "horizontal" => Constraint::Horizontal { line: u(2)? },
                    "vertical" => Constraint::Vertical { line: u(2)? },
                    "parallel" => Constraint::Parallel { a: u(2)?, b: u(3)? },
                    "perpendicular" => Constraint::Perpendicular { a: u(2)?, b: u(3)? },
                    "equal" => Constraint::EqualLength { a: u(2)?, b: u(3)? },
                    "ponline" => Constraint::PointOnLine { p: u(2)?, line: u(3)? },
                    other => return Err(format!("line {lineno}: unknown constraint `{other}`")),
                };
                s.add(c);
            }
            other => return Err(format!("line {lineno}: unknown record `{other}`")),
        }
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Constraint;

    #[test]
    fn rsmp_round_trips() {
        let mut s = Sketch::new();
        let p0 = s.add_point(0.0, 0.0);
        let p1 = s.add_point(10.0, 0.0);
        let l0 = s.add_line(p0, p1);
        s.add(Constraint::Fixed { p: p0, x: 0.0, y: 0.0 });
        s.add(Constraint::Distance { p: p0, q: p1, d: 10.0 });
        s.add(Constraint::Horizontal { line: l0 });

        let text = write_rsmp(&s);
        let back = read_rsmp(&text).expect("parse");
        assert_eq!(back.points.len(), 2);
        assert_eq!(back.lines.len(), 1);
        assert_eq!(back.constraints.len(), 3);
        assert_eq!(back.constraints[1], Constraint::Distance { p: 0, q: 1, d: 10.0 });
        assert_eq!(back.lines[0], crate::model::Line { a: 0, b: 1 });
    }

    #[test]
    fn rejects_foreign_header() {
        assert!(read_rsmp("DXF\n0\nSECTION\n").is_err());
    }
}
