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
        research(&mut tree, "Vellum", 30.0);
        assert!(!can_research(&tree, "Printing Press")); // still blocked
        research(&mut tree, "Woodblock", 50.0);
        research(&mut tree, "Movable Type", 100.0);
        research(&mut tree, "Paper Mill", 70.0);
        assert!(can_research(&tree, "Printing Press")); // now unlocked
    }
}
