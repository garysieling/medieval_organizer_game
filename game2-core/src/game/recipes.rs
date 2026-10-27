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

    /// Make the output once: use the first input option the stock can
    /// cover, take those inputs out of it, and add the output. Returns
    /// which option was used, or None (stock untouched) if none fits.
    pub fn craft(&self, stock: &mut std::collections::HashMap<String, u32>) -> Option<usize> {
        let i = self.can_make(stock)?;
        for (g, q) in &self.input_options[i] {
            // can_make treats a missing good as 0, so a zero-quantity input may be absent
            if let Some(have) = stock.get_mut(g) { *have -= q; }
        }
        let (out, qty) = &self.output;
        *stock.entry(out.clone()).or_insert(0) += qty;
        Some(i)
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

    #[test]
    fn test_craft_consumes_inputs_and_adds_output() {
        let r = &printmaking_recipes()[1];
        let mut stock = std::collections::HashMap::new();
        stock.insert("woodblock".into(), 1);
        stock.insert("ink".into(), 3);
        stock.insert("paper".into(), 7);
        assert_eq!(r.craft(&mut stock), Some(0));
        assert_eq!(stock["print"], 5);
        assert_eq!(stock["ink"], 1);
        assert_eq!(stock["paper"], 2);
        assert_eq!(stock["woodblock"], 0);
        // The block is used up, so a second print run can't happen
        assert_eq!(r.craft(&mut stock), None);
        assert_eq!(stock["print"], 5, "a failed craft leaves stock untouched");
    }

    #[test]
    fn test_craft_falls_back_to_alternative_inputs() {
        let r = &printmaking_recipes()[0];
        let mut stock: std::collections::HashMap<String, u32> =
            [("quill", 2), ("ink", 2), ("papyrus", 1), ("vellum", 1)]
                .into_iter().map(|(g, q)| (g.to_string(), q)).collect();
        assert_eq!(r.craft(&mut stock), Some(0)); // papyrus first
        assert_eq!(r.craft(&mut stock), Some(1)); // out of papyrus: vellum
        assert_eq!(stock["document"], 2);
        assert_eq!(r.craft(&mut stock), None); // out of quills
    }
}
