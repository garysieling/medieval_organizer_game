//! Recipes: flexible crafting - multiple paths to same output.
//! Inspired by Against the Storm: creativity from alternatives.

/// A recipe with alternative input sets
#[derive(Debug)]
pub struct Recipe {
    pub name: String,
    pub output: (String, u32),
    /// Each inner vec is one valid way to make the output
    pub input_options: Vec<Vec<(String, u32)>>,
}

impl Recipe {
    /// Can we make this with available stock?
    pub fn can_make(&self, stock: &std::collections::HashMap<String, u32>) -> Option<usize> {
        for (i, inputs) in self.input_options.iter().enumerate() {
            if inputs.iter().all(|(g, q)| stock.get(g).copied().unwrap_or(0) >= *q) {
                return Some(i);
            }
        }
        None
    }
}

/// Printmaking recipes (detailed sim per Gary's interest)
pub fn printmaking_recipes() -> Vec<Recipe> {
    vec![
        Recipe {
            name: "Quill on Papyrus".into(),
            output: ("document".into(), 1),
            input_options: vec![
                vec![("quill".into(), 1), ("ink".into(), 1), ("papyrus".into(), 1)],
                vec![("quill".into(), 1), ("ink".into(), 1), ("vellum".into(), 1)],
            ],
        },
        Recipe {
            name: "Woodblock Print".into(),
            output: ("print".into(), 5),
            input_options: vec![
                vec![("woodblock".into(), 1), ("ink".into(), 2), ("paper".into(), 5)],
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alternative_inputs() {
        let r = &printmaking_recipes()[0];
        let mut stock = std::collections::HashMap::new();
        stock.insert("quill".into(), 1);
        stock.insert("ink".into(), 1);
        stock.insert("vellum".into(), 1); // no papyrus, but vellum works
        assert_eq!(r.can_make(&stock), Some(1)); // used option 2
    }
}
