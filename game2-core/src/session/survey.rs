//! The traveller's question (#1085): after a beat worth asking about -
//! the cause founded, a first burying, a job seen through, the lord's
//! rope on one of yours - a pedlar resting at the well puts the ask
//! in-world, and the answer lands in `feedback/` with the state around
//! it. Dev-facing: Gary reviews the bundle and files what it tells him.
//!
//! The ask rides the conversation box (`convo_panel_at`, decides:true),
//! the pick lands in `decide_at`, and the shot goes out through the
//! UiModel's one-frame `feedback_capture` flag for the client to save.

use crate::session::*;

/// A moment worth asking after. Each beat fires once per seat - the
/// traveller doesn't ask the same thing twice
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurveyBeat {
    /// The lord's rope on one of yours - the strongest negative beat
    MemberHanged,
    /// The first villager laid in the ground
    FirstDeath,
    /// The cause's first vote taken
    OrgFounded,
    /// The first odd job seen through
    QuestDone,
}

impl SurveyBeat {
    pub const ALL: [SurveyBeat; 4] = [
        SurveyBeat::MemberHanged,
        SurveyBeat::FirstDeath,
        SurveyBeat::OrgFounded,
        SurveyBeat::QuestDone,
    ];

    /// The bit this beat carries in the seat's `survey_seen` mask
    pub fn bit(self) -> u32 { 1 << (self as u32) }

    /// The slug the feedback file's named for
    pub fn slug(self) -> &'static str {
        match self {
            SurveyBeat::MemberHanged => "hanged",
            SurveyBeat::FirstDeath => "first-death",
            SurveyBeat::OrgFounded => "cause-founded",
            SurveyBeat::QuestDone => "quest-done",
        }
    }

    /// How the pedlar frames the asking, in his own words - the beat's
    /// why he stops you here and not somewhere else
    pub fn line(self) -> &'static str {
        match self {
            SurveyBeat::MemberHanged =>
                "A pedlar at the well keeps his voice low: \"They hanged one of yours, I hear - and still you'd say this village's a place to send a soul?\"",
            SurveyBeat::FirstDeath =>
                "A pedlar at the well nods at the fresh grave: \"Village buries one of its own today. The road hears everything - is it a place you'd send a cousin of yours to?\"",
            SurveyBeat::OrgFounded =>
                "A pedlar at the well grins into his cup: \"Word is the village found its voice - a cause of its own. Would you send a cousin of yours to join it?\"",
            SurveyBeat::QuestDone =>
                "A pedlar at the well tips his hat: \"They say a stranger did right by this village - a job seen through. Would you send a cousin of yours to it?\"",
        }
    }
}

/// The traveller's ask waiting on the seat's pick (#1085)
#[derive(Debug, Clone)]
pub struct SurveyOffer {
    pub beat: SurveyBeat,
    /// The tick the ask was made, for the bundle
    pub asked: u64,
}

/// The answer rows as the pedlar takes them: the option the player
/// picked, verbatim, plus whether the take was warm, middling or cold
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurveyTake {
    Warm,
    Middling,
    Cold,
}

/// Days before the traveller asks the same seat again - back-to-back
/// beats shouldn't nag
const SURVEY_WAIT: u64 = 20;

/// The options the conversation box carries for the ask, in order:
/// warm, middling, cold, and the wave-off that asks nothing and writes
/// nothing
pub const SURVEY_OPTIONS: [&str; 4] = [
    "\"I would - I'd send a cousin to this village tomorrow.\"",
    "\"It's fair enough - no better, no worse than most.\"",
    "\"I'd warn them off - this place wears a soul down.\"",
    "\"The road's questions aren't mine to answer.\"",
];

impl Game {
    /// Dawn of each day: if a beat's landed for a seat and the
    /// traveller's welcome back, he puts his question (#1085)
    pub(super) fn survey_dawn(&mut self, tick: u64) {
        for seat in 0..self.seats.len() {
            if self.seats[seat].agent { continue; }
            if self.seats[seat].survey.is_some() { continue; }
            if tick < self.seats[seat].survey_wait { continue; }
            let seen = self.seats[seat].survey_seen;
            let founded = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org).is_some();
            // the harshest beat asks first - one question at a time
            let hit = [
                (SurveyBeat::MemberHanged, self.martyr_grave.is_some()),
                (SurveyBeat::FirstDeath, self.died_on.iter().any(|d| d.is_some())),
                (SurveyBeat::OrgFounded, founded),
                (SurveyBeat::QuestDone, self.seats[seat].quests_done > 0),
            ].into_iter().find(|(b, hit)| *hit && seen & b.bit() == 0);
            if let Some((b, _)) = hit {
                self.seats[seat].survey_seen |= b.bit();
                self.seats[seat].survey = Some(SurveyOffer { beat: b, asked: tick });
            }
        }
    }

    /// The traveller's line as the seat's conversation-box panel rows
    /// (#1085): his asking, and the note that your word travels with him
    pub(in crate::session) fn survey_rows(&self, seat: usize) -> Vec<TalkRowV> {
        let Some(o) = &self.seats[seat].survey else { return vec![] };
        vec![
            TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                text: o.beat.line().to_string() },
            TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                text: "\"I carry word up and down the road. What do I tell them?\"".into() },
        ]
    }

    /// The pick on the traveller's ask: 0-2 are an answer, and it
    /// writes the bundle (question, the answer verbatim, and the state
    /// around it) and flags the client to save the frame; 3 waves him
    /// off - nothing asked, nothing written (#1085)
    pub(in crate::session) fn survey_answered(&mut self, seat: usize, pick: usize) {
        let Some(o) = self.seats[seat].survey.take() else { return };
        self.seats[seat].survey_wait = self.tick + SURVEY_WAIT;
        if pick >= 3 {
            self.log.push_ev(EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
                format!("t{}: the pedlar takes the hint and goes back to his pack.", self.tick));
            return;
        }
        let take = match pick {
            0 => SurveyTake::Warm,
            1 => SurveyTake::Middling,
            _ => SurveyTake::Cold,
        };
        let answer = SURVEY_OPTIONS[pick].to_string();
        self.log.push_ev(EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
            format!("t{}: you tell the pedlar - {}", self.tick, answer));
        self.survey_write(seat, o.beat, take, &answer);
    }

    /// The bundle the answer carries: the question, the answer verbatim,
    /// the beat it came off, and enough state around it to make the
    /// report reproducible - seed, day, camera, and the log's tail
    fn survey_write(&mut self, seat: usize, beat: SurveyBeat, take: SurveyTake, answer: &str) {
        let Some(dir) = self.feedback_dir.clone() else { return };
        let name = format!("survey-s{}-t{}-{}", seat, self.tick, beat.slug());
        let tail: Vec<String> = self.log.iter().rev().take(15).cloned().collect();
        let mut body = String::new();
        body.push_str(&format!("# The traveller asks ({})\n\n", beat.slug()));
        body.push_str(&format!("- beat: {:?}\n", beat));
        body.push_str(&format!("- take: {:?}\n", take));
        body.push_str(&format!("- seed: {}\n", self.seed));
        body.push_str(&format!("- tick: {}\n", self.tick));
        body.push_str(&format!("- day: {}\n", self.tick));
        body.push_str(&format!("- seat: {}\n", seat));
        if let Some((x, y, _)) = self.seats[seat].focus {
            body.push_str(&format!("- camera: ({:.1}, {:.1})\n", x, y));
        }
        body.push_str("\n## The ask\n\n");
        body.push_str(beat.line());
        body.push_str("\n\n## The answer (verbatim)\n\n");
        body.push_str(answer);
        body.push_str("\n\n## The log's tail\n\n");
        for line in tail.iter().rev() { body.push_str(&format!("- {}\n", line)); }
        body.push_str(&format!("\n_Screenshot: {}.bmp (saved by the client when it sees the flag)_\n", name));
        if std::fs::create_dir_all(&dir).is_err() { return; }
        let _ = std::fs::write(format!("{}/{}.txt", dir, name), body);
        // the client saves the frame under the same name the moment the
        // flag surfaces on its model
        self.seats[seat].survey_capture = Some((self.frame_n, name.clone()));
        self.log.push_ev(EvTag::new(Thread::Player, Valence::Neutral, Sev::Minor),
            format!("t{}: (feedback: {} written to {}/)", self.tick, name, dir));
    }
}
