// Color model — matches AutoCAD's color semantics so DXF round-trip
// (group codes 62 / 420 / 430) is lossless when I/O lands.
//
// Two indirection sentinels (ByLayer / ByBlock) are first-class — a Dobject
// can declare "use my layer's color" without storing a concrete RGB. The
// renderer resolves the chain at draw time via `resolve_color`.

use crate::layer::LayerTable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// Inherit from the entity's layer.
    ByLayer,
    /// Inherit from the containing block reference (when blocks land).
    /// Outside a block, behaves like ByLayer.
    ByBlock,
    /// AutoCAD Color Index — palette of 256 named colors.
    /// 0 reserved (ByBlock in DXF), 256 reserved (ByLayer in DXF);
    /// useful range is 1..=255.
    Aci(u8),
    /// 24-bit true colour, packed as `0x00RRGGBB`.
    TrueColor(u32),
}

impl Default for Color {
    fn default() -> Self { Color::ByLayer }
}

impl Color {
    /// Convenience constructor from byte components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::TrueColor(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    /// Unpack a TrueColor into (r, g, b). For ByLayer / ByBlock / Aci,
    /// returns None — callers must resolve first.
    pub fn rgb_bytes(self) -> Option<(u8, u8, u8)> {
        match self {
            Color::TrueColor(v) => Some((
                ((v >> 16) & 0xFF) as u8,
                ((v >>  8) & 0xFF) as u8,
                ( v        & 0xFF) as u8,
            )),
            _ => None,
        }
    }
}

/// Resolve a Dobject's color through the ByLayer / ByBlock chain. Returns a
/// concrete `(r, g, b)`. ByBlock falls back to ByLayer until block support
/// lands. ACI indices are resolved through `aci_palette` (minimal 7-color
/// table for now — covers the first row of AutoCAD's color picker).
pub fn resolve_color(c: Color, layer_id: u32, layers: &LayerTable) -> (u8, u8, u8) {
    match c {
        Color::TrueColor(_) => c.rgb_bytes().unwrap_or((255, 255, 255)),
        Color::Aci(idx)     => aci_palette(idx),
        Color::ByLayer | Color::ByBlock => {
            let layer_color = layers.get(layer_id)
                .map(|l| l.color)
                .unwrap_or(Color::TrueColor(0xFFFFFF));
            match layer_color {
                Color::ByLayer | Color::ByBlock => (255, 255, 255), // safety: break loop
                Color::TrueColor(_) => layer_color.rgb_bytes().unwrap(),
                Color::Aci(i)       => aci_palette(i),
            }
        }
    }
}

/// AutoCAD Color Index — minimal palette covering the first row plus a few
/// common indices. Real AutoCAD has all 256 mapped; this is enough for
/// hand-authored test drawings and DXF imports that use the standard
/// indices. Extend as needed.
pub fn aci_palette(idx: u8) -> (u8, u8, u8) {
    match idx {
        0 | 7 => (255, 255, 255),    // ByBlock / white-on-dark
        1     => (255,   0,   0),    // red
        2     => (255, 255,   0),    // yellow
        3     => (  0, 255,   0),    // green
        4     => (  0, 255, 255),    // cyan
        5     => (  0,   0, 255),    // blue
        6     => (255,   0, 255),    // magenta
        8     => (128, 128, 128),    // dark grey
        9     => (192, 192, 192),    // light grey
        _     => (255, 255, 255),    // fallback — full palette TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::{Layer, LayerTable};

    #[test]
    fn truecolor_pack_unpack() {
        let c = Color::rgb(255, 128, 64);
        assert_eq!(c.rgb_bytes(), Some((255, 128, 64)));
    }

    #[test]
    fn aci_basic_palette() {
        assert_eq!(aci_palette(1), (255, 0, 0));
        assert_eq!(aci_palette(3), (0, 255, 0));
    }

    #[test]
    fn resolve_bylayer_through_table() {
        let mut t = LayerTable::with_defaults();
        let id = t.add(Layer {
            name:       "WALLS".into(),
            color:      Color::Aci(3),     // green
            linetype:   0,
            lineweight: crate::lineweight::Lineweight::Default,
            visible:    true,
            locked:     false,
            frozen:     false,
            plottable:  true,
        });
        assert_eq!(resolve_color(Color::ByLayer, id, &t), (0, 255, 0));
    }

    #[test]
    fn resolve_truecolor_ignores_layer() {
        let t = LayerTable::with_defaults();
        let c = Color::rgb(10, 20, 30);
        assert_eq!(resolve_color(c, 0, &t), (10, 20, 30));
    }
}
