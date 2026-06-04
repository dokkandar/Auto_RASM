// Hatch pattern catalog — hardcoded, no external .pat files.
//
// Each pattern is a list of LINE FAMILIES. A family is an infinite set
// of parallel lines all at the same angle, spaced uniformly. The
// renderer (cad_app) clips them against the resolved hatch boundary
// using even-odd along each line. Solid hatches don't go through here.
//
// Pattern names match the industry-standard AutoCAD vocabulary
// (ANSI31, BRICK, NET, EARTH, …) so files exchange cleanly. The
// geometry of each pattern is derived independently — no copy of
// AutoCAD's `acad.pat` or LibreCAD's GPL'd .dxf pattern files. The
// names are not trademarks (ANSI is a real standards body; the rest
// are English words used in CAD vocabulary for decades).

#[derive(Clone, Debug)]
pub struct LineFamily {
    /// Direction of the lines, in radians measured CCW from +X.
    pub angle:    f64,
    /// Anchor — one specific line in the family passes through this
    /// point. The rest are stepped from this anchor by `spacing` in
    /// the family's normal direction.
    pub base_x:   f64,
    pub base_y:   f64,
    /// Perpendicular distance between consecutive parallel lines, in
    /// pattern's unit scale (multiplied by the hatch's `scale` field
    /// at render time).
    pub spacing:  f64,
}

/// Resolve a canonical pattern name (case-insensitive) to its list of
/// line families. Unknown names return an empty Vec — render produces
/// no lines but doesn't crash.
///
/// Each entry below is documented with a one-line ASCII sketch so the
/// reader can match name → visual at a glance.
pub fn lookup(name: &str) -> Vec<LineFamily> {
    let up = name.to_ascii_uppercase();
    let pi = std::f64::consts::PI;
    match up.as_str() {
        // ANSI31 — 45° diagonals  / / / / /
        "ANSI31" => vec![
            LineFamily { angle: pi / 4.0,        base_x: 0.0, base_y: 0.0, spacing: 3.175 },
        ],
        // ANSI32 — 45° diagonal pairs (close + far spacing alternating)
        //   ||  ||  ||
        // approximated as two interleaved families at the same angle
        "ANSI32" => vec![
            LineFamily { angle: pi / 4.0, base_x: 0.0,  base_y: 0.0, spacing: 6.350 },
            LineFamily { angle: pi / 4.0, base_x: 1.59, base_y: 1.59, spacing: 6.350 },
        ],
        // ANSI33 — 135° diagonals at 3 mm
        "ANSI33" => vec![
            LineFamily { angle: 3.0 * pi / 4.0,  base_x: 0.0, base_y: 0.0, spacing: 3.175 },
        ],
        // ANSI37 — perpendicular crosshatch at 45° and 135°  X X X
        "ANSI37" | "EARTH" => vec![
            LineFamily { angle: pi / 4.0,        base_x: 0.0, base_y: 0.0, spacing: 3.175 },
            LineFamily { angle: 3.0 * pi / 4.0,  base_x: 0.0, base_y: 0.0, spacing: 3.175 },
        ],
        // CROSS / NET — horizontal + vertical grid  + + +
        "CROSS" | "NET" => vec![
            LineFamily { angle: 0.0,             base_x: 0.0, base_y: 0.0, spacing: 5.0 },
            LineFamily { angle: pi / 2.0,        base_x: 0.0, base_y: 0.0, spacing: 5.0 },
        ],
        // ANGLE — horizontal + vertical, coarser than CROSS
        "ANGLE" => vec![
            LineFamily { angle: 0.0,             base_x: 0.0, base_y: 0.0, spacing: 6.35 },
            LineFamily { angle: pi / 2.0,        base_x: 0.0, base_y: 0.0, spacing: 6.35 },
        ],
        // BRICK — horizontal courses + vertical perpends. v1 has no
        // running-bond offset (every-other-row half-shift); that needs
        // the per-vertex offset in the .pat format which we don't
        // encode yet. Still reads as masonry.
        "BRICK" => vec![
            LineFamily { angle: 0.0,             base_x: 0.0, base_y: 0.0, spacing: 8.0 },
            LineFamily { angle: pi / 2.0,        base_x: 0.0, base_y: 0.0, spacing: 4.0 },
        ],
        // CONCRETE — diagonal hatches both ways, looser spacing
        "CONCRETE" => vec![
            LineFamily { angle: pi / 4.0,        base_x: 0.0, base_y: 0.0, spacing: 5.0 },
            LineFamily { angle: 3.0 * pi / 4.0,  base_x: 0.0, base_y: 0.0, spacing: 5.0 },
        ],
        // LINE — single horizontal-line family (matches AutoCAD's
        // basic "LINE" pattern). Useful as a clean baseline.
        "LINE" | "HORIZONTAL" => vec![
            LineFamily { angle: 0.0,             base_x: 0.0, base_y: 0.0, spacing: 3.175 },
        ],
        // DOTS / GRAVEL approximation — fine perpendicular crosshatch
        // produces a dotted texture at typical zoom.
        "DOTS" => vec![
            LineFamily { angle: 0.0,             base_x: 0.0, base_y: 0.0, spacing: 1.0 },
            LineFamily { angle: pi / 2.0,        base_x: 0.0, base_y: 0.0, spacing: 1.0 },
        ],
        // Unknown name → no families; renderer draws nothing for this
        // hatch. The hatch dobject itself remains in the doc and the
        // user can rename to a valid pattern later.
        _ => Vec::new(),
    }
}

/// Catalog of every recognised pattern name. Useful for UI listings
/// (dropdown / chooser) and for tests that enumerate patterns to
/// verify every one resolves to a non-empty family list.
pub const PATTERN_NAMES: &[&str] = &[
    "SOLID",       // sentinel — actually rendered via the Solid arm
    "ANSI31", "ANSI32", "ANSI33", "ANSI37",
    "CROSS", "NET", "ANGLE", "BRICK",
    "CONCRETE", "EARTH", "LINE", "DOTS",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_pattern_resolves() {
        for name in PATTERN_NAMES {
            if *name == "SOLID" { continue; }   // SOLID is the no-line case
            let fams = lookup(name);
            assert!(!fams.is_empty(),
                "pattern '{}' resolved to no line families", name);
            for f in &fams {
                assert!(f.spacing > 0.0,
                    "pattern '{}' has non-positive spacing", name);
            }
        }
    }

    #[test]
    fn unknown_pattern_is_empty() {
        assert!(lookup("NO_SUCH_PATTERN").is_empty());
        assert!(lookup("").is_empty());
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let a = lookup("ANSI31");
        let b = lookup("ansi31");
        let c = lookup("Ansi31");
        assert_eq!(a.len(), b.len());
        assert_eq!(b.len(), c.len());
    }
}
