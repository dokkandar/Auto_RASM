// Linetype table — a registry of named dash/gap patterns.
//
// Pattern semantics: a slice of alternating dash-then-gap lengths in world
// units. Positive = pen down (dash); negative would be a pen-down with a
// shape (not modelled yet). Empty slice = continuous.
//
// LinetypeId is a stable index. Built-in id 0 = "Continuous" — always
// present so anything that lacks an explicit linetype renders solid.

#[derive(Clone, Debug)]
pub struct Linetype {
    pub name:        String,
    pub description: String,
    /// Dash/gap pattern in world units. Even index = dash length,
    /// odd index = gap length. Empty = continuous.
    pub pattern:     Vec<f32>,
}

impl Linetype {
    pub fn continuous() -> Self {
        Self {
            name:        "Continuous".into(),
            description: "Solid line".into(),
            pattern:     Vec::new(),
        }
    }

    /// Simple dashed pattern: dash_len, gap_len.
    pub fn dashed(name: &str, dash: f32, gap: f32) -> Self {
        Self {
            name:        name.into(),
            description: format!("__ __ __  ({} / {})", dash, gap),
            pattern:     vec![dash, gap],
        }
    }

    /// Dash-dot pattern.
    pub fn dash_dot(name: &str, dash: f32, gap: f32) -> Self {
        Self {
            name:        name.into(),
            description: format!("__ . __ . __  ({} / {})", dash, gap),
            pattern:     vec![dash, gap, 0.0, gap],
        }
    }

    pub fn is_continuous(&self) -> bool { self.pattern.is_empty() }
}

pub struct LinetypeTable {
    pub linetypes: Vec<Linetype>,
}

impl LinetypeTable {
    /// Constructed with three built-ins always present at known ids:
    ///   0 = Continuous
    ///   1 = Dashed
    ///   2 = DashDot
    pub fn with_defaults() -> Self {
        Self {
            linetypes: vec![
                Linetype::continuous(),
                Linetype::dashed("Dashed",  6.0, 3.0),
                Linetype::dash_dot("DashDot", 6.0, 3.0),
            ],
        }
    }

    /// The reserved id of the "Continuous" linetype.
    pub const CONTINUOUS: u32 = 0;

    pub fn get(&self, id: u32) -> Option<&Linetype> {
        self.linetypes.get(id as usize)
    }

    pub fn add(&mut self, lt: Linetype) -> u32 {
        let id = self.linetypes.len() as u32;
        self.linetypes.push(lt);
        id
    }

    pub fn find(&self, name: &str) -> Option<u32> {
        self.linetypes.iter().position(|l| l.name.eq_ignore_ascii_case(name))
            .map(|i| i as u32)
    }

    pub fn len(&self) -> usize { self.linetypes.len() }
    pub fn is_empty(&self) -> bool { self.linetypes.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_continuous_at_id_zero() {
        let t = LinetypeTable::with_defaults();
        assert!(t.get(LinetypeTable::CONTINUOUS).unwrap().is_continuous());
    }

    #[test]
    fn find_is_case_insensitive() {
        let t = LinetypeTable::with_defaults();
        assert_eq!(t.find("continuous"), Some(0));
        assert_eq!(t.find("DASHED"), Some(1));
        assert_eq!(t.find("nope"), None);
    }
}
