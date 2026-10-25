//! Hot reload: reload game components without restarting.
//! From backlog: "hot reloading seems important" for dev loops.
//! Components are registered by name and can be swapped/reloaded.

use std::collections::HashMap;

/// A reloadable component (system, module)
pub struct Component {
    pub name: String,
    pub version: u32,
    pub path: String,       // file path to watch
    pub last_loaded: u64,
}

/// Registry of components that can be hot-reloaded
pub struct HotReload {
    pub components: HashMap<String, Component>,
    pub tick: u64,
}

impl HotReload {
    pub fn new() -> Self {
        Self { components: HashMap::new(), tick: 0 }
    }

    /// Register a component for hot-reloading
    pub fn register(&mut self, name: &str, path: &str) {
        self.components.insert(name.to_string(), Component {
            name: name.into(), version: 1,
            path: path.into(), last_loaded: self.tick,
        });
    }

    /// Simulate a reload (bump version, reset timestamp)
    pub fn reload(&mut self, name: &str) -> bool {
        if let Some(c) = self.components.get_mut(name) {
            c.version += 1;
            c.last_loaded = self.tick;
            return true;
        }
        false
    }

    /// Check if a component is registered
    pub fn has(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }
}
