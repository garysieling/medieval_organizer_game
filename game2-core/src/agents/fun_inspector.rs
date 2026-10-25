//! Fun Inspector: evaluates playability, engagement, pacing.
//! Uses the fun rubric (wiki/fun-rubric.md) to score playtests.

/// Fun assessment of a playtest session
#[derive(Debug)]
pub struct FunAssessment {
    pub session_id: String,
    pub scores: FunScores,
    pub fun_moments: Vec<String>,
    pub grind_moments: Vec<String>,
    pub overall: f32,
    pub verdict: String,
}

#[derive(Debug)]
pub struct FunScores {
    pub tension: u8,        // 1-5
    pub progress: u8,
    pub surprise: u8,
    pub care: u8,
    pub decisions: u8,
    pub retention: u8,
}

/// Fun Inspector agent
pub struct FunInspector {
    pub name: String,
    pub rubric_path: String,
}

impl FunInspector {
    pub fn new() -> Self {
        Self {
            name: "Fun Inspector".into(),
            rubric_path: "wiki/fun-rubric.md".into(),
        }
    }

    /// Score a playtest against the rubric
    pub fn assess(&self, session_id: &str, scores: FunScores, fun: Vec<String>, grind: Vec<String>) -> FunAssessment {
        let avg = (scores.tension + scores.progress + scores.surprise
            + scores.care + scores.decisions + scores.retention) as f32 / 6.0;
        let verdict = if avg >= 4.0 { "Fun - keep it up" }
            else if avg >= 3.0 { "Decent - needs work" }
            else if avg >= 2.0 { "Meh - grind detected" }
            else { "Not fun - major issues" };
        FunAssessment {
            session_id: session_id.into(),
            scores,
            fun_moments: fun,
            grind_moments: grind,
            overall: avg,
            verdict: verdict.into(),
        }
    }

    /// Auto-flag red flags from a sim log
    pub fn detect_red_flags(&self, sim_log: &[(u64, f32, bool)]) -> Vec<String> {
        let mut flags = vec![];
        // Check for >10 ticks with no change
        let mut streak = 0;
        for (_, _val, changed) in sim_log {
            if !changed { streak += 1; if streak > 10 { flags.push(format!("No change for {}+ ticks", streak)); break; } }
            else { streak = 0; }
        }
        flags
    }
}
