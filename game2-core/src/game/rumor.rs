//! Inside news travels (#1287): the cause's news - the meet's set, the
//! issue named, who promised what - carried member to member and ear to
//! ear through the street's chats, bent a little per hop and marked
//! where it lands on a wrong ear.

use crate::engine::ecs::{Entity, World};
use crate::game::organizing::MeetingPlan;
use crate::sim::community::Organization;

/// A piece of inside news the cause knows, as a villager can carry it
#[derive(Debug, Clone, PartialEq)]
pub enum News {
    /// The meet's set: this many days out, at this one's house
    MeetOn { days: u64, at: String },
    /// The issue the cause's named
    IssueIs { name: String },
    /// Who promised what on the plan
    Promised { who: String, what: &'static str },
}

impl News {
    /// What it's about: two tellings of the same piece are one news,
    /// however the telling's been bent
    pub fn topic(&self) -> &'static str {
        match self {
            News::MeetOn { .. } => "the meet",
            News::IssueIs { .. } => "the issue",
            News::Promised { what, .. } => what,
        }
    }
}

/// A fact as carried: the piece, and whether this copy's landed on a
/// wrong ear (#1287) - the mark the counter-intel seam reads. A
/// wrong-ear piece is told home to the castle once, then it's spent
/// (#1708): `carried` is the telling done
#[derive(Debug, Clone, PartialEq)]
pub struct RumorFact {
    pub news: News,
    pub wrong_ear: bool,
    pub carried: bool,
}

/// The inside news a non-member holds (#1287): what the street told
/// them, bent as it arrived. Members need no store - they read the
/// table like anyone in the room
#[derive(Debug, Default)]
pub struct Rumor {
    pub facts: Vec<RumorFact>,
}

impl Rumor {
    /// Whether a piece on this topic's already reached them
    pub fn heard(&self, topic: &str) -> bool {
        self.facts.iter().any(|f| f.news.topic() == topic)
    }

    /// Note a piece heard - the first telling of a topic's the one kept
    pub fn hear(&mut self, f: RumorFact) {
        if !self.heard(f.news.topic()) { self.facts.push(f); }
    }
}

/// The org's inside news today (#1287), built live from the plan and
/// the charter: the meet's set, who promised what, the issue named
pub fn news_of(world: &World, org: Entity, tick: u64, name: &dyn Fn(Entity) -> String) -> Vec<News> {
    let mut out = vec![];
    let Some(o) = world.get_component::<Organization>(org) else { return out };
    if let Some(p) = world.get_component::<MeetingPlan>(org)
        .or_else(|| world.get_component::<MeetingPlan>(o.leader)) {
        if let Some(day) = p.for_day.filter(|&d| d > tick) {
            let at = match p.venue {
                Some(crate::game::organizing::Venue::House(h, _)) => name(h),
                Some(crate::game::organizing::Venue::Church) => "the church steps".into(),
                Some(crate::game::organizing::Venue::Tavern) => "the tavern".into(),
                _ => "the square".into(),
            };
            out.push(News::MeetOn { days: day - tick, at });
        }
        for (who, what) in [
            (p.venue.and_then(|v| v.host()), "to host us"),
            (p.snacks, "to bring for the table"),
            (p.notes, "to keep the notes"),
            (p.opener, "to open us with a word"),
            (p.song, "to give us a song"),
            (p.closer, "to close us out"),
        ] {
            if let Some(w) = who { out.push(News::Promised { who: name(w), what }); }
        }
    }
    // the issue the cause's named: the campaign's tag where one's
    // founded, else the charter's own word
    let tag = world.query::<crate::game::campaign::Campaign>().iter().find_map(|&c|
        world.get_component::<crate::game::campaign::Campaign>(c)
            .filter(|cc| cc.org == org).map(|cc| cc.issue.clone()));
    match tag {
        Some(t) => out.push(News::IssueIs { name: crate::game::desires::issue_name(t) }),
        None if !o.charter.is_empty() => out.push(News::IssueIs { name: o.charter.clone() }),
        _ => {}
    }
    out
}

/// Whether the table holds a piece worth telling (#1709): the same
/// check `news_of` runs, with no names to fill - the gating for the
/// player's own "tell them what you've heard". The static charter's
/// public business - it opens no tell on its own; a live meet, a
/// promise given, or a campaign's named issue is the news
pub fn has_inside_news(world: &World, org: Entity, tick: u64) -> bool {
    news_of(world, org, tick, &|_| String::new()).iter().any(|n| match n {
        News::IssueIs { .. } => world.query::<crate::game::campaign::Campaign>().iter()
            .any(|&c| world.get_component::<crate::game::campaign::Campaign>(c)
                .map(|cc| cc.org == org).unwrap_or(false)),
        _ => true,
    })
}

/// A hop's drift (#1287): one small swap per telling, seeded - the day
/// off by one, or the name swapped for another member's. One slot at
/// most bends: subtle, not telephone-game compounding
pub fn bend(world: &World, org: Entity, news: &News, seed: u64, name: &dyn Fn(Entity) -> String) -> News {
    // names to swap in: the org's other members
    let swap = |cur: &str| -> Option<String> {
        let o = world.get_component::<Organization>(org)?;
        let mut names: Vec<String> = o.members.iter()
            .map(|&m| name(m)).filter(|n| n != cur).collect();
        names.sort();
        if names.is_empty() { None } else { Some(names[(seed % names.len() as u64) as usize].clone()) }
    };
    match news {
        News::MeetOn { days, at } => match seed % 3 {
            0 => News::MeetOn { days: days + 1, at: at.clone() },
            1 => News::MeetOn { days: (*days - 1).max(1), at: at.clone() },
            _ => News::MeetOn { days: *days, at: swap(at).unwrap_or_else(|| at.clone()) },
        },
        News::Promised { who, what } => News::Promised {
            who: swap(who).unwrap_or_else(|| who.clone()), what },
        News::IssueIs { .. } => news.clone(),
    }
}

/// The line a piece of inside news is told in
pub fn line_of(n: &News) -> String {
    match n {
        News::MeetOn { days, at } => format!(
            "there's a meet at {}'s house in {} days' time - keep it close", at, days),
        News::IssueIs { name } => format!("word is the cause's set on {}", name),
        News::Promised { who, what } => format!("{}'s promised {}, so they say", who, what),
    }
}
