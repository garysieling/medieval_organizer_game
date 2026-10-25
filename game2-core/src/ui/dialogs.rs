//! Dialog system: character-to-character conversations.
//! RimWorld-style dialog between characters with relationship context.

/// A dialog exchange
#[derive(Debug)]
pub struct Dialog {
    pub speaker: String,
    pub listener: String,
    pub topic: String,
    pub tone: Tone,
    pub relationship_context: f32, // how well they know each other
}

#[derive(Debug)]
pub enum Tone {
    Friendly,
    Hostile,
    Neutral,
    Flirty,
    Formal,
}

/// Generate a dialog line from context (simple heuristic now, LLM later)
pub fn generate_line(d: &Dialog) -> String {
    match d.tone {
        Tone::Friendly => format!("{} chats warmly with {} about {}", d.speaker, d.listener, d.topic),
        Tone::Hostile => format!("{} argues with {} about {}", d.speaker, d.listener, d.topic),
        Tone::Neutral => format!("{} mentions {} to {}", d.speaker, d.topic, d.listener),
        Tone::Flirty => format!("{} flirts with {} over {}", d.speaker, d.listener, d.topic),
        Tone::Formal => format!("{} formally addresses {} about {}", d.speaker, d.listener, d.topic),
    }
}
