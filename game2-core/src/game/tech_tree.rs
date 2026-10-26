//! Tech trees: constellation-style progression.
//! Printmaking track is detailed per Gary's interest: feathers->lithography.

/// A technology node
#[derive(Debug)]
pub struct Tech {
    pub name: String,
    pub era: String,
    pub prereqs: Vec<String>,      // tech names required first
    pub cost: f32,                 // research points
    pub unlocks: Vec<String>,      // what it enables
    pub discovered: bool,
}

/// Printmaking tech tree (feathers->lithography, no further)
pub fn printmaking_tree() -> Vec<Tech> {
    vec![
        Tech { name: "Quill & Ink".into(), era: "Ancient".into(), prereqs: vec![], cost: 10.0, unlocks: vec!["document".into()], discovered: false },
        Tech { name: "Papyrus".into(), era: "Ancient".into(), prereqs: vec![], cost: 15.0, unlocks: vec!["papyrus".into()], discovered: false },
        Tech { name: "Vellum".into(), era: "Early".into(), prereqs: vec!["Papyrus".into()], cost: 25.0, unlocks: vec!["vellum".into()], discovered: false },
        Tech { name: "Woodblock".into(), era: "Early".into(), prereqs: vec!["Vellum".into()], cost: 40.0, unlocks: vec!["print".into()], discovered: false },
        Tech { name: "Movable Type".into(), era: "Middle".into(), prereqs: vec!["Woodblock".into()], cost: 80.0, unlocks: vec!["book".into()], discovered: false },
        Tech { name: "Paper Mill".into(), era: "Middle".into(), prereqs: vec!["Vellum".into()], cost: 60.0, unlocks: vec!["paper".into()], discovered: false },
        Tech { name: "Printing Press".into(), era: "Late".into(), prereqs: vec!["Movable Type".into(), "Paper Mill".into()], cost: 120.0, unlocks: vec!["mass_book".into()], discovered: false },
        Tech { name: "Stone Lithography".into(), era: "Late".into(), prereqs: vec!["Printing Press".into()], cost: 100.0, unlocks: vec!["lithograph".into()], discovered: false },
    ]
}

/// Can this tech be researched? All prereqs must be discovered
pub fn can_research(tree: &[Tech], tech_name: &str) -> bool {
    let tech = match tree.iter().find(|t| t.name == tech_name) { Some(t) => t, None => return false };
    tech.prereqs.iter().all(|p| tree.iter().any(|t| &t.name == p && t.discovered))
}

/// Research a tech if possible
pub fn research(tree: &mut Vec<Tech>, tech_name: &str, points: f32) -> bool {
    if !can_research(tree, tech_name) { return false; }
    if let Some(t) = tree.iter_mut().find(|t| t.name == tech_name) {
        if points >= t.cost { t.discovered = true; return true; }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prereq_gating() {
        let mut tree = printmaking_tree();
        assert!(!can_research(&tree, "Printing Press")); // needs Movable Type + Paper Mill
        research(&mut tree, "Papyrus", 20.0);
        research(&mut tree, "Vellum", 30.0);
        assert!(!can_research(&tree, "Printing Press")); // still blocked
        research(&mut tree, "Woodblock", 50.0);
        research(&mut tree, "Movable Type", 100.0);
        research(&mut tree, "Paper Mill", 70.0);
        assert!(can_research(&tree, "Printing Press")); // now unlocked
    }
}

/// Extended printmaking tree based on APHA history.
/// Web structure: substrate->methods->scale. Paper mill is the gate.
pub fn extended_printmaking_tree() -> Vec<Tech> {
    vec![
        Tech { name: "Papyrus".into(), era: "ancient".into(), prereqs: vec![], cost: 5.0, unlocks: vec!["writing".into()], discovered: false },
        Tech { name: "Parchment".into(), era: "ancient".into(), prereqs: vec!["Papyrus".into()], cost: 10.0, unlocks: vec!["codex".into()], discovered: false },
        Tech { name: "Paper".into(), era: "medieval".into(), prereqs: vec!["Parchment".into()], cost: 15.0, unlocks: vec!["cheap_writing".into()], discovered: false },
        Tech { name: "Paper Mill".into(), era: "medieval".into(), prereqs: vec!["Paper".into()], cost: 30.0, unlocks: vec!["printmaking".into()], discovered: false },
        Tech { name: "Carving".into(), era: "ancient".into(), prereqs: vec![], cost: 8.0, unlocks: vec!["blocks".into()], discovered: false },
        Tech { name: "Woodblock".into(), era: "medieval".into(), prereqs: vec!["Carving".into(), "Paper Mill".into()], cost: 20.0, unlocks: vec!["relief_prints".into()], discovered: false },
        Tech { name: "Woodcut Prints".into(), era: "medieval".into(), prereqs: vec!["Woodblock".into()], cost: 25.0, unlocks: vec!["playing_cards".into(), "devotional_images".into()], discovered: false },
        Tech { name: "Metalworking".into(), era: "ancient".into(), prereqs: vec![], cost: 12.0, unlocks: vec!["plates".into()], discovered: false },
        Tech { name: "Engraving".into(), era: "medieval".into(), prereqs: vec!["Metalworking".into(), "Paper Mill".into()], cost: 30.0, unlocks: vec!["fine_prints".into()], discovered: false },
        Tech { name: "Etching".into(), era: "renaissance".into(), prereqs: vec!["Engraving".into()], cost: 35.0, unlocks: vec!["detail_prints".into()], discovered: false },
        Tech { name: "Clay Type".into(), era: "medieval".into(), prereqs: vec!["Paper Mill".into()], cost: 25.0, unlocks: vec!["reusable_type".into()], discovered: false },
        Tech { name: "Metal Type".into(), era: "medieval".into(), prereqs: vec!["Clay Type".into(), "Metalworking".into()], cost: 35.0, unlocks: vec!["durable_type".into()], discovered: false },
        Tech { name: "Type Foundry".into(), era: "renaissance".into(), prereqs: vec!["Metal Type".into()], cost: 45.0, unlocks: vec!["type_production".into()], discovered: false },
        Tech { name: "Printing Press".into(), era: "renaissance".into(), prereqs: vec!["Type Foundry".into()], cost: 60.0, unlocks: vec!["mass_books".into()], discovered: false },
        Tech { name: "Chemistry".into(), era: "modern".into(), prereqs: vec!["Engraving".into()], cost: 50.0, unlocks: vec!["lithography".into()], discovered: false },
        Tech { name: "Lithography".into(), era: "modern".into(), prereqs: vec!["Chemistry".into()], cost: 70.0, unlocks: vec!["mass_images".into()], discovered: false },
        Tech { name: "Playing Cards".into(), era: "medieval".into(), prereqs: vec!["Woodcut Prints".into()], cost: 15.0, unlocks: vec!["mass_market".into()], discovered: false },
        Tech { name: "Devotional Images".into(), era: "medieval".into(), prereqs: vec!["Woodcut Prints".into()], cost: 15.0, unlocks: vec!["mass_market".into()], discovered: false },
        Tech { name: "Printed Books".into(), era: "renaissance".into(), prereqs: vec!["Printing Press".into()], cost: 40.0, unlocks: vec!["knowledge_spread".into()], discovered: false },
        Tech { name: "Scientific Illustration".into(), era: "renaissance".into(), prereqs: vec!["Etching".into(), "Printing Press".into()], cost: 50.0, unlocks: vec!["science_accelerates".into()], discovered: false },
    ]
}
