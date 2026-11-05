//! Secret inner lives: the wants people carry (Maslow-flavoured, sampled
//! per vocation and class) and the problems that stand in their way.
//! Neither shows in the UI until revealed: a 1-1 has a small chance, and
//! at a house meeting each parallel complaint already voiced makes the
//! next speaker likelier to share their own. The organizer keeps what he
//! hears in a ledger that expires after a year; three or more people with
//! versions of the same trouble, heard inside the year, surface an issue.

use crate::engine::ecs::{Entity, World};
use crate::game::vocation::{self, Craft, Kid, SocialClass, TownRole};

/// Days in a year (4 seasons x 90 days); heard complaints expire after one
pub const YEAR_DAYS: u64 = 360;

/// Base chance a 1-1 or a meeting turn reveals one of someone's secrets
pub const REVEAL_BASE: f32 = 0.15;
/// Extra chance per parallel complaint already voiced at the meeting
pub const REVEAL_PARALLEL: f32 = 0.10;

/// An issue kind: complaints in the same family, however people put them
/// What an issue's about. Trade-flavored problems carry the trade too -
/// "no master (mason)" only echoes among masons; the commons (tax,
/// broke, the bad bread) carry no tag and echo for everyone
pub type IssueTag = String;

/// What a person wants, inspired by Maslow's hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Want {
    GoodFood,      // buys bread
    Finery,        // buys garments
    Faith,         // buys a service at the church
    Shelter,       // have a home (a problem when unhoused)
    Security,      // buys the watch's protection
    Mastery,       // rise in the craft
    Music,         // hear music played
    Instrument,    // own a lute
    BetterTools,   // own tools (works faster)
    DifferentTrade,// the assigned vocation bores them
    Love,          // a close bond
    Joy,           // play - mostly the children's want
    Drink,         // a cup at the alehouse - company in a bowl
}

/// What's in someone's way. Most are computed from their situation each
/// day; a few (a taxing) stick on them until it fades
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Problem {
    NoHome,
    NoWorkplace,
    /// A journeyman or apprentice with no master to learn from
    NoMaster,
    /// A day's input they can't lay hands on
    NoInput(&'static str),
    /// An empty shop near home that sours the street
    DerelictNearby,
    /// Working under an overseer who takes a cut
    OverseerCut,
    /// Paid the tax collector within the year
    Taxed,
    /// Under three gold to their name
    Broke,
    /// Carries a smell: the flock's, the cauldron's smoke - honest work
    /// hangs on them
    Smelly,
    /// Wolves got into the fold - stock lost in the night
    Loss,
    /// The village says their bread brought the fire
    Blamed,
    /// Ate spoiled stock - a sour gut for a poor soul
    SourGut,
    /// The lord took his fifth of what the dead one left
    DeathDuty,
    /// The wood's thin this side of the hill - gathers outpace regrowth
    ThinWood,
    /// The worked pit drowned - no stone to be had
    DryPit,
    /// The reach below town stinks - the leavings settle in it
    StinkingReach,
    /// The reckoning didn't add up - shorted in a foreign count
    ShortChanged,
    /// The circus's pull: the road's calling and the days here are gray
    Tempted,
    /// The bailiff's writ: a levy, a fee, a summons from the castle
    WritBorne,
    /// The winter runs long and the stores run thin
    LongWinter,
    /// No fuel for the hearth - the house eats cold all winter
    ColdHearth,
    /// The hermit's want: the dark keeps them
    TheDark,
    /// What they watched and can't unsee - the hanging's weight
    Grief,
    /// The good stock walked off in the night - a cellar worth robbing
    /// is a cellar worth watching
    Robbed,
}

/// One secret a person can share: a want or a problem
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reveal {
    Want(Want),
    Problem(Problem),
}

/// A person's hidden inner life: the wants they carry, the sticky
/// grievances picked up along the way, and which wants are currently met
/// (with the day each stops counting as met)
#[derive(Debug)]
pub struct InnerLife {
    pub wants: Vec<Want>,
    /// (problem, the day it stops counting) - e.g. a taxing, fresh for a year
    pub grievances: Vec<(Problem, u64)>,
    /// (want, the day its satisfaction wears off)
    pub satisfied: Vec<(Want, u64)>,
}

/// What the organizer has learned about this person, piece by piece
#[derive(Debug, Default)]
pub struct Known {
    pub revealed: Vec<Reveal>,
}

/// The organizer's ledger: what was heard, from whom, and when. Entries
/// expire after a year; three or more live entries in a family surface an
/// issue (kept in `issues` once found)
#[derive(Debug, Default)]
pub struct Organizer {
    pub heard: Vec<(IssueTag, Entity, u64)>,
    pub issues: Vec<IssueTag>,
}

/// Mark a want satisfied until `until` days from now (or refresh it)
pub fn satisfy(world: &mut World, e: Entity, want: Want, until: u64) {
    let Some(life) = world.get_component_mut::<InnerLife>(e) else { return };
    match life.satisfied.iter_mut().find(|(w, _)| *w == want) {
        Some((_, u)) => *u = (*u).max(until),
        None => life.satisfied.push((want, until)),
    }
}

/// Whether a want currently counts as met
pub fn is_satisfied(world: &World, e: Entity, want: Want, tick: u64) -> bool {
    world.get_component::<InnerLife>(e)
        .map(|l| l.satisfied.iter().any(|(w, u)| *w == want && tick <= *u))
        .unwrap_or(false)
}

/// Pin a grievance on someone until `until` (e.g. a taxing: the year out)
pub fn add_grievance(world: &mut World, e: Entity, p: Problem, until: u64) {
    let Some(life) = world.get_component_mut::<InnerLife>(e) else { return };
    match life.grievances.iter_mut().find(|(g, _)| *g == p) {
        Some((_, u)) => *u = (*u).max(until),
        None => life.grievances.push((p, until)),
    }
}

/// The issue family a reveal belongs to - parallel complaints share one
pub fn issue_tag(world: &World, e: Entity, r: &Reveal) -> IssueTag {
    let base: &'static str = match r {
        Reveal::Want(Want::GoodFood) => "hunger",
        Reveal::Want(Want::Finery) => "comfort",
        Reveal::Want(Want::Faith) => "faith",
        Reveal::Want(Want::Shelter) | Reveal::Problem(Problem::NoHome) => "housing",
        Reveal::Want(Want::Security) => "security",
        Reveal::Want(Want::Mastery) => "mastery",
        Reveal::Want(Want::Music) | Reveal::Want(Want::Instrument) => "music",
        Reveal::Want(Want::BetterTools) => "tools",
        Reveal::Want(Want::DifferentTrade) => "restlessness",
        Reveal::Want(Want::Love) => "loneliness",
        Reveal::Want(Want::Joy) => "joy",
        Reveal::Want(Want::Drink) => "the drink",
        Reveal::Problem(Problem::NoWorkplace) => "workplaces",
        Reveal::Problem(Problem::NoMaster) => "apprenticeship",
        Reveal::Problem(Problem::NoInput(_)) => "materials",
        Reveal::Problem(Problem::DerelictNearby) => "empty shops",
        Reveal::Problem(Problem::OverseerCut) => "the overseers' cut",
        Reveal::Problem(Problem::Taxed) => "the taxes",
        Reveal::Problem(Problem::Broke) => "poverty",
        Reveal::Problem(Problem::Smelly) => "honest work's stink",
        Reveal::Problem(Problem::Loss) => "the wolves",
        Reveal::Problem(Problem::Blamed) => "the bad bread",
        Reveal::Problem(Problem::SourGut) => "the sour gut",
        Reveal::Problem(Problem::DeathDuty) => "the death duty",
        Reveal::Problem(Problem::ThinWood) => "the shrinking wood",
        Reveal::Problem(Problem::ColdHearth) => "cold hearths",
        Reveal::Problem(Problem::DryPit) => "the drowned pit",
        Reveal::Problem(Problem::StinkingReach) => "the stinking reach",
        Reveal::Problem(Problem::ShortChanged) => "the reckoning that didn't add up",
        Reveal::Problem(Problem::Tempted) => "the road's pull",
        Reveal::Problem(Problem::WritBorne) => "the bailiff's writ",
        Reveal::Problem(Problem::LongWinter) => "the long winter",
        Reveal::Problem(Problem::TheDark) => "the dark",
        Reveal::Problem(Problem::Grief) => "the grief",
        Reveal::Problem(Problem::Robbed) => "the thieves",
    };
    // a trade's grievance only echoes among the trade - the commons
    // (tax, poverty, the bad bread, the sour gut) echo for everyone
    let scoped = matches!(r,
        Reveal::Problem(Problem::NoWorkplace) | Reveal::Problem(Problem::NoMaster)
        | Reveal::Problem(Problem::NoInput(_)) | Reveal::Problem(Problem::DerelictNearby)
        | Reveal::Problem(Problem::OverseerCut) | Reveal::Problem(Problem::Smelly)
        | Reveal::Problem(Problem::Loss) | Reveal::Want(Want::BetterTools));
    if scoped {
        if let Some(v) = world.get_component::<crate::game::vocation::Craft>(e)
            .map(|c| crate::game::vocation::vocation_name(c.vocation).to_lowercase()) {
            return format!("{} ({})", base, v);
        }
    }
    base.into()
}

/// What the issue is called once the organizer sees the pattern. A
/// trade-scoped tag names its trade: "materials (baker)" reads "the
/// bakers' workshops starved of materials"
pub fn issue_name(tag: IssueTag) -> String {
    if let Some((base, voc)) = tag.strip_suffix(')').and_then(|s| s.split_once(" (")) {
        return format!("the {}s' {}", voc, issue_name(base.to_string()));
    }
    match tag.as_str() {
        "hunger" => "the bread shortage".into(),
        "comfort" => "plain living".into(),
        "faith" => "the neglected faith".into(),
        "housing" => "folk without a roof".into(),
        "security" => "folk who feel unsafe".into(),
        "mastery" => "trades without a path up".into(),
        "music" => "a village with no music".into(),
        "tools" => "folk short of tools".into(),
        "restlessness" => "trades that don't fit".into(),
        "loneliness" => "folk on their own".into(),
        "joy" => "children with nothing to do".into(),
        "workplaces" => "trades without a workshop".into(),
        "apprenticeship" => "youngsters with no master".into(),
        "materials" => "workshops starved of materials".into(),
        "empty shops" => "the boarded-up shops".into(),
        "the overseers' cut" => "the overseers' cut".into(),
        "the taxes" => "the tax collector's cut".into(),
        "poverty" => "folk without a coin".into(),
        "honest work's stink" => "the stink of honest work".into(),
        "the wolves" => "wolves at the fold".into(),
        "the bad bread" => "the cursed bread".into(),
        "the sour gut" => "spoiled stock in the larder".into(),
        "the death duty" => "the lord's death-duty".into(),
        "the shrinking wood" => "the shrinking wood".into(),
        "the drowned pit" => "the drowned pit".into(),
        "the stinking reach" => "the stinking reach".into(),
        "the reckoning that didn't add up" => "the reckoning that didn't add up".into(),
        "the road's pull" => "the road's pull".into(),
        "the bailiff's writ" => "the bailiff's writ".into(),
        "the long winter" => "the long winter".into(),
        "the dark" => "the dark".into(),
        "the grief" => "the grief".into(),
        "the drink" => "a dry village".into(),
        "the thieves" => "thieves in the night".into(),
        _ => tag,
    }
}

/// How a want reads on a character card once revealed
pub fn want_label(w: Want) -> &'static str {
    match w {
        Want::GoodFood => "wants good bread",
        Want::Finery => "wants fine clothes",
        Want::Faith => "wants the faith",
        Want::Shelter => "wants a home",
        Want::Security => "wants to feel safe",
        Want::Mastery => "wants to rise in the trade",
        Want::Music => "wants to hear music",
        Want::Instrument => "wants an instrument",
        Want::BetterTools => "wants better tools",
        Want::DifferentTrade => "bored with the trade",
        Want::Love => "wants love",
        Want::Joy => "wants to play",
        Want::Drink => "wants a drink",
    }
}

/// How a problem reads on a character card once revealed
pub fn problem_label(p: &Problem) -> String {
    match p {
        Problem::NoHome => "has no home".into(),
        Problem::NoWorkplace => "has no workshop".into(),
        Problem::NoMaster => "has no master".into(),
        Problem::NoInput(g) => format!("can't get {}", g),
        Problem::DerelictNearby => "lives by an empty shop".into(),
        Problem::OverseerCut => "the overseer takes a cut".into(),
        Problem::Taxed => "paid the tax collector".into(),
        Problem::Broke => "is nearly broke".into(),
        Problem::Smelly => "smells of honest work".into(),
        Problem::Loss => "lost stock to wolves".into(),
        Problem::Blamed => "is blamed for the sickness".into(),
        Problem::SourGut => "ate spoiled stock".into(),
        Problem::DeathDuty => "paid the lord's fifth on the dead".into(),
        Problem::ThinWood => "the wood's thin this side of the hill".into(),
        Problem::DryPit => "the pit's drowned - no stone to be had".into(),
        Problem::StinkingReach => "the reach below town stinks".into(),
        Problem::ShortChanged => "shorted in a foreign count".into(),
        Problem::Tempted => "the road's calling".into(),
        Problem::WritBorne => "a writ came down from the castle".into(),
        Problem::LongWinter => "the winter runs long and the stores run thin".into(),
        Problem::ColdHearth => "no fuel for the hearth".into(),
        Problem::TheDark => "the dark keeps me".into(),
        Problem::Grief => "I watched it and can't unsee it".into(),
        Problem::Robbed => "was robbed in the night".into(),
    }
}

/// What a revealed secret sounds like said aloud: (what the log saw them
/// do, what they said in the bubble)
pub fn reveal_speech(name: &str, r: &Reveal) -> (String, String) {
    match r {
        Reveal::Want(Want::GoodFood) => (format!("{} admits missing good bread", name), "A crust of real bread would be something.".into()),
        Reveal::Want(Want::Finery) => (format!("{} wishes for fine clothes", name), "One decent garment. Is that so much?".into()),
        Reveal::Want(Want::Faith) => (format!("{} misses the faith", name), "It's been too long since I felt anything.".into()),
        Reveal::Want(Want::Shelter) => (format!("{} longs for a home", name), "A roof of my own. That's all I ask.".into()),
        Reveal::Want(Want::Security) => (format!("{} feels unsafe", name), "I bar my door at night and still don't sleep.".into()),
        Reveal::Want(Want::Mastery) => (format!("{} wants to rise in the trade", name), "I'm better than this. Someone should see it.".into()),
        Reveal::Want(Want::Music) => (format!("{} misses music", name), "Not a song in the village for a year.".into()),
        Reveal::Want(Want::Instrument) => (format!("{} covets an instrument", name), "If I had a lute I'd never put it down.".into()),
        Reveal::Want(Want::BetterTools) => (format!("{} needs better tools", name), "My hands do the work my tools won't.".into()),
        Reveal::Want(Want::DifferentTrade) => (format!("{} is bored with the trade", name), "This isn't the life I pictured.".into()),
        Reveal::Want(Want::Love) => (format!("{} confides they're lonely", name), "Everyone's got someone. Not me.".into()),
        Reveal::Want(Want::Joy) => (format!("{} wants to play", name), "I just want to play a while.".into()),
        Reveal::Want(Want::Drink) => (format!("{} misses a cup", name), "A cup of something would sit well about now.".into()),
        Reveal::Problem(Problem::NoHome) => (format!("{} has no home", name), "Nowhere to lay my head. Nowhere.".into()),
        Reveal::Problem(Problem::NoWorkplace) => (format!("{} has no workshop", name), "No bench, no hearth - I work in the street.".into()),
        Reveal::Problem(Problem::NoMaster) => (format!("{} has no master", name), "No one to learn from. I'll be a journeyman forever.".into()),
        Reveal::Problem(Problem::NoInput(g)) => (format!("{} can't get {}", name, g), format!("No {} to be had anywhere.", g)),
        Reveal::Problem(Problem::DerelictNearby) => (format!("{} hates the empty shop", name), "That boarded shop on the corner. It sits there like a sore.".into()),
        Reveal::Problem(Problem::OverseerCut) => (format!("{} resents the overseer's cut", name), "I sweat and he takes his share for watching.".into()),
        Reveal::Problem(Problem::Taxed) => (format!("{} curses the tax collector", name), "The collector took his cut again. The lord eats well.".into()),
        Reveal::Problem(Problem::Broke) => (format!("{} is nearly broke", name), "Two coins left. Two.".into()),
        Reveal::Problem(Problem::Smelly) => (format!("{} stinks of honest work", name), "Aye, honest work hangs on - it'll wash off.".into()),
        Reveal::Problem(Problem::Loss) => (format!("{} lost stock to wolves", name), "The wolf came in the night. I found what's left of her.".into()),
        Reveal::Problem(Problem::Blamed) => (format!("{} is blamed for the sickness", name), "They say my bread did it. My bread.".into()),
        Reveal::Problem(Problem::SourGut) => (format!("{} ate spoiled stock", name), "It was all there was. Still smells.".into()),
        Reveal::Problem(Problem::DeathDuty) => (format!("{} paid the lord's fifth", name), "A fifth of all he had. For dying.".into()),
        Reveal::Problem(Problem::ThinWood) => (format!("{}: the wood's thin", name), "Walked half a day for a bundle. It wasn't always so.".into()),
        Reveal::Problem(Problem::DryPit) => (format!("{}: the pit's drowned", name), "Water in the cut. No stone comes out of a pond.".into()),
        Reveal::Problem(Problem::StinkingReach) => (format!("{}: the reach stinks", name), "You can smell it on a still night. That wasn't always so.".into()),
        Reveal::Problem(Problem::ShortChanged) => (format!("{}: the reckoning's off", name), "He counted it his way. My way it's short.".into()),
        Reveal::Problem(Problem::Tempted) => (format!("{}'s feet itch", name), "The road looks shorter from the gate than the days look here.".into()),
        Reveal::Problem(Problem::WritBorne) => (format!("{} bore the writ", name), "Sealed by the castle. No arguing with wax.".into()),
        Reveal::Problem(Problem::LongWinter) => (format!("{}'s stores are thin", name), "Second winter running. The barrel knows it.".into()),
        Reveal::Problem(Problem::ColdHearth) => (format!("{} eats cold", name), "No faggots, no turf, not even the dry leavings. The pot's cold and the bed's colder.".into()),
        Reveal::Problem(Problem::TheDark) => (format!("{} keeps to the dark", name), "The village is loud with light. Down here it's honest.".into()),
        Reveal::Problem(Problem::Grief) => (format!("{} saw it", name), "They strung him up where we could see. I can't unsee it.".into()),
        Reveal::Problem(Problem::Robbed) => (format!("{} was robbed", name), "In the night, quiet as a cat. Took the good stock.".into()),
    }
}

/// What a person can be made to reveal: their unsatisfied wants, their
/// live problems, and their fresh grievances - skipping anything already
/// in `known`
pub fn unrevealed(world: &World, e: Entity, tick: u64) -> Vec<Reveal> {
    let known = world.get_component::<Known>(e)
        .map(|k| k.revealed.clone()).unwrap_or_default();
    let mut out: Vec<Reveal> = problems_of(world, e, tick).into_iter()
        .map(Reveal::Problem).collect();
    if let Some(life) = world.get_component::<InnerLife>(e) {
        for w in &life.wants {
            if is_satisfied(world, e, *w, tick) { continue; }
            out.push(Reveal::Want(*w));
        }
    }
    out.retain(|r| !known.contains(r));
    out
}

/// Pick one secret to share: problems first (they're what's pressing),
/// then wants. `roll` is 0..1 - callers give it a boosted chance
pub fn next_reveal(world: &World, e: Entity, tick: u64, seed: u64) -> Option<Reveal> {
    let pool = unrevealed(world, e, tick);
    if pool.is_empty() { return None; }
    Some(pool[(seed % pool.len() as u64) as usize])
}

/// Record that the organizer heard `r` from `person`: mark it known on
/// them, journal it on the organizer, prune entries older than a year, and
/// return the new issue if three or more people have now said versions of
/// the same thing inside the year
pub fn hear(world: &mut World, organizer: Entity, person: Entity, r: Reveal, tick: u64) -> Option<String> {
    {
        let mut known = world.get_component_mut::<Known>(person);
        match &mut known {
            Some(k) if !k.revealed.contains(&r) => k.revealed.push(r),
            Some(_) => return None,
            None => {
                drop(known);
                world.add_component(person, Known { revealed: vec![r] });
            }
        }
    }
    let tag = issue_tag(world, person, &r);
    let org = world.get_component_mut::<Organizer>(organizer)?;
    org.heard.retain(|(_, _, day)| tick <= *day + YEAR_DAYS);
    org.heard.push((tag.clone(), person, tick));
    if org.issues.contains(&tag) { return None; }
    let voices: Vec<Entity> = org.heard.iter()
        .filter(|(t, _, _)| *t == tag).map(|(_, e, _)| *e).collect();
    let mut dedup = voices.clone();
    dedup.sort_unstable();
    dedup.dedup();
    if dedup.len() >= 3 {
        org.issues.push(tag.clone());
        Some(issue_name(tag))
    } else {
        None
    }
}

/// Add an issue directly (e.g. a house meeting where 3+ voiced it) - the
/// same as hearing it out three times over
pub fn found_issue(world: &mut World, organizer: Entity, tag: IssueTag) -> Option<String> {
    let org = world.get_component_mut::<Organizer>(organizer)?;
    if org.issues.contains(&tag) { return None; }
    org.issues.push(tag.clone());
    Some(issue_name(tag))
}

/// A person's live problems, worked out from the world: where they live,
/// where they work, who teaches them, what they can't buy, the state of
/// their street, and what they've been through lately
pub fn problems_of(world: &World, e: Entity, tick: u64) -> Vec<Problem> {
    use crate::game::household::{self, Building};
    use crate::game::villager::Pos;
    let mut out = vec![];
    // a roof: they must be an occupant of some Home
    let housed = world.query::<Building>().iter().any(|&b| {
        world.get_component::<Building>(b)
            .map(|bb| bb.kind == household::BuildingKind::Home && bb.occupants.contains(&e))
            .unwrap_or(false)
    });
    if !housed { out.push(Problem::NoHome); }
    if let Some(c) = world.get_component::<Craft>(e) {
        if c.workplace.is_none() { out.push(Problem::NoWorkplace); }
        if c.master.is_none() && vocation::level_of(c.skill) < vocation::Level::Master {
            out.push(Problem::NoMaster);
        }
        // the fold's smell hangs on a shepherd; folk notice
        if c.vocation == vocation::Vocation::Shepherd { out.push(Problem::Smelly); }
    }
    // a derelict shop (workplace kind with no worker) within a stroll of home
    if let Some(p) = world.get_component::<Pos>(e).copied() {
        let near_derelict = world.query::<Building>().iter().any(|&b| {
            let Some(bb) = world.get_component::<Building>(b) else { return false };
            if !household::derelict(bb) { return false; }
            world.get_component::<Pos>(b)
                .map(|bp| (bp.x - p.x).abs() + (bp.y - p.y).abs() < 6.0)
                .unwrap_or(false)
        });
        if near_derelict { out.push(Problem::DerelictNearby); }
    }
    if let Some(life) = world.get_component::<InnerLife>(e) {
        out.extend(life.grievances.iter()
            .filter(|(_, until)| tick <= *until).map(|(p, _)| *p));
    }
    out
}

/// Sample a person's wants for seeding: `n` draws from a pool weighted by
/// class and vocation. Deterministic via `rng` (a splitmix64 next() value
/// per draw).
pub fn sample_wants(class: SocialClass, voc: Option<vocation::Vocation>, is_kid: bool, rng: &mut impl FnMut() -> u64) -> Vec<Want> {
    if is_kid {
        let mut w = vec![Want::Joy];
        if rng() % 3 == 0 { w.push(Want::Instrument); }
        return w;
    }
    // every want's weight for this person: base + vocation/class flavour
    let weight = |w: Want| -> u64 {
        let base: u64 = match w {
            Want::GoodFood => 4, Want::Security => 3, Want::Mastery => 3,
            Want::Love => 3, Want::Faith => 2, Want::Music => 2,
            Want::Finery => 2, Want::BetterTools => 2, Want::Shelter => 2,
            Want::DifferentTrade => 2, Want::Instrument => 1, Want::Joy => 1,
            Want::Drink => 3,
        };
        let bonus: u64 = match (w, voc) {
            (Want::Faith, Some(vocation::Vocation::Priest)) => 6,
            (Want::Music, Some(vocation::Vocation::Luthier)) => 6,
            (Want::Instrument, Some(vocation::Vocation::Luthier)) => 4,
            (Want::GoodFood, Some(vocation::Vocation::Baker)) => 2,
            (Want::BetterTools, Some(vocation::Vocation::Mason)) => 3,
            (Want::BetterTools, Some(vocation::Vocation::Toolmaker)) => 2,
            (Want::Mastery, Some(vocation::Vocation::Knight)) => 3,
            (Want::Finery, Some(vocation::Vocation::Tailor)) => 4,
            (Want::Finery, Some(vocation::Vocation::Glazier)) => 2,
            _ => 0,
        };
        let class_bonus: u64 = match (w, class) {
            (Want::Finery, SocialClass::ProfessionalMiddle) => 2,
            (Want::Finery, SocialClass::PetitBourgeois) => 3,
            (Want::Mastery, SocialClass::Working) => 2,
            (Want::Security, SocialClass::ProfessionalMiddle) => 0,
            _ => 0,
        };
        base + bonus + class_bonus
    };
    const ALL: [Want; 12] = [
        Want::GoodFood, Want::Finery, Want::Faith, Want::Shelter, Want::Security,
        Want::Mastery, Want::Music, Want::Instrument, Want::BetterTools,
        Want::DifferentTrade, Want::Love, Want::Drink,
    ];
    let mut pool: Vec<Want> = ALL.iter().copied()
        .flat_map(|w| std::iter::repeat(w).take(weight(w) as usize)).collect();
    // mostly two wants, sometimes one or three
    let n = match rng() % 10 { 0..=1 => 1, 2..=7 => 2, _ => 3 };
    let mut out = vec![];
    for _ in 0..n {
        if pool.is_empty() { break; }
        let i = (rng() % pool.len() as u64) as usize;
        let w = pool[i];
        pool.retain(|x| *x != w);
        out.push(w);
    }
    out
}

/// Whether a `Kid` marker is on the entity
pub fn is_kid(world: &World, e: Entity) -> bool {
    world.get_component::<Kid>(e).is_some()
}

/// What the posting is called over the village's tongue
pub fn town_role_name(role: TownRole) -> &'static str {
    match role {
        TownRole::Overseer => "Overseer",
        TownRole::TaxCollector => "Tax Collector",
        TownRole::Trader => "Trader",
        TownRole::Herbologist => "Herb-wife",
        TownRole::Matchmaker => "Heart-helper",
        TownRole::Hermit => "Hermit",
    }
}

/// Their posting, for the inspector: the trade or PMC role they hold
pub fn occupation_of(world: &World, e: Entity) -> Option<String> {
    if let Some(c) = world.get_component::<Craft>(e) {
        let lvl = vocation::level_name(vocation::level_of(c.skill));
        return Some(format!("{} {}", lvl, vocation::vocation_name(c.vocation)));
    }
    if world.get_component::<Kid>(e).is_some() { return Some("Child".into()); }
    world.get_component::<TownRole>(e).map(|r| town_role_name(*r).to_string())
}

/// The class a character reads as: PMC roles and PMC vocations read PMC;
/// masters who own their workshop read petit bourgeois
pub fn class_of(world: &World, e: Entity) -> SocialClass {
    if let Some(role) = world.get_component::<TownRole>(e) {
        // the herb-wife works her hedge like any labourer; the rest of the
        // postings carry the lord's or the profession's standing
        return match role { TownRole::Herbologist | TownRole::Matchmaker => SocialClass::Working, _ => SocialClass::ProfessionalMiddle };
    }
    if let Some(c) = world.get_component::<Craft>(e) {
        if vocation::is_pmc(c.vocation) { return SocialClass::ProfessionalMiddle; }
        if vocation::level_of(c.skill) == vocation::Level::Master && c.workplace.is_some() {
            return SocialClass::PetitBourgeois;
        }
    }
    SocialClass::Working
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::household::{Building, BuildingKind};
    use crate::game::villager::Pos;
    use crate::game::vocation::craft;

    fn person(w: &mut World, wants: &[Want]) -> Entity {
        let e = w.create_entity();
        w.add_component(e, Pos { x: 0.0, y: 0.0 });
        w.add_component(e, InnerLife { wants: wants.to_vec(), grievances: vec![], satisfied: vec![] });
        e
    }

    fn housed(w: &mut World, e: Entity) {
        let b = w.create_entity();
        w.add_component(b, Pos { x: 0.0, y: 0.0 });
        w.add_component(b, Building {
            kind: BuildingKind::Home, name: "a house".into(),
            occupants: vec![e], worker: None, overseer: None,
            stock: Default::default(), capacity: 20,
        });
    }

    #[test]
    fn test_problems_come_from_the_situation() {
        let mut w = World::new();
        let e = person(&mut w, &[]);
        assert_eq!(problems_of(&w, e, 0), vec![Problem::NoHome]);
        housed(&mut w, e);
        assert!(problems_of(&w, e, 0).is_empty());
        // an unpaired journeyman with no shop has two more problems
        let c = craft(vocation::Vocation::Mason, 0.2);
        w.add_component(e, c);
        let probs = problems_of(&w, e, 0);
        assert!(probs.contains(&Problem::NoWorkplace) && probs.contains(&Problem::NoMaster));
    }

    #[test]
    fn test_hearing_journals_and_discovers_the_pattern() {
        let mut w = World::new();
        let org = w.create_entity();
        w.add_component(org, Organizer::default());
        let (a, b, c) = (person(&mut w, &[]), person(&mut w, &[]), person(&mut w, &[]));
        let r = Reveal::Problem(Problem::Taxed);
        assert_eq!(hear(&mut w, org, a, r, 100), None);
        assert_eq!(hear(&mut w, org, b, r, 200), None);
        // the same person twice doesn't make a pattern
        assert_eq!(hear(&mut w, org, a, Reveal::Problem(Problem::Broke), 210), None);
        assert_eq!(hear(&mut w, org, c, r, 300).as_deref(), Some("the tax collector's cut"));
        // and it isn't discovered twice
        assert_eq!(hear(&mut w, org, c, r, 300), None);
        let known = w.get_component::<Known>(a).unwrap();
        assert!(known.revealed.contains(&r) && known.revealed.contains(&Reveal::Problem(Problem::Broke)));
    }

    #[test]
    fn test_heard_entries_expire_after_a_year() {
        let mut w = World::new();
        let org = w.create_entity();
        w.add_component(org, Organizer::default());
        let (a, b, c) = (person(&mut w, &[]), person(&mut w, &[]), person(&mut w, &[]));
        let r = Reveal::Problem(Problem::OverseerCut);
        hear(&mut w, org, a, r, 10);
        hear(&mut w, org, b, r, 20);
        // at day 371 the oldest complaint is stale (10 + a year), the
        // second one still counts - so c makes two voices, not three
        assert_eq!(hear(&mut w, org, c, r, 371), None,
            "a fresh complaint plus a stale one shouldn't surface an issue");
        // the fourth voice inside the year completes the pattern
        let d = person(&mut w, &[]);
        assert_eq!(hear(&mut w, org, d, r, 372).as_deref(), Some("the overseers' cut"));
    }

    #[test]
    fn test_unrevealed_skips_known_and_satisfied() {
        let mut w = World::new();
        let e = person(&mut w, &[Want::GoodFood, Want::Love]);
        // NoHome is the only live problem; both wants unrevealed
        let pool = unrevealed(&w, e, 0);
        assert_eq!(pool.len(), 3);
        let org = w.create_entity();
        w.add_component(org, Organizer::default());
        hear(&mut w, org, e, Reveal::Problem(Problem::NoHome), 0);
        satisfy(&mut w, e, Want::GoodFood, 30);
        let pool = unrevealed(&w, e, 10);
        assert_eq!(pool, vec![Reveal::Want(Want::Love)]);
    }

    #[test]
    fn test_wants_are_weighted_but_sparse() {
        let mut seed = 7u64;
        let mut rng = || { seed = seed.wrapping_add(0x9E3779B97F4A7C15); seed };
        for _ in 0..50 {
            let w = sample_wants(SocialClass::Working, Some(vocation::Vocation::Priest), false, &mut rng);
            assert!((1..=3).contains(&w.len()));
        }
        let kids = sample_wants(SocialClass::Working, None, true, &mut rng);
        assert!(kids.contains(&Want::Joy));
    }

    #[test]
    fn test_classes_read_off_the_components() {
        let mut w = World::new();
        let e = person(&mut w, &[]);
        assert_eq!(class_of(&w, e), SocialClass::Working);
        w.add_component(e, craft(vocation::Vocation::Knight, 0.5));
        assert_eq!(class_of(&w, e), SocialClass::ProfessionalMiddle);
        w.add_component(e, craft(vocation::Vocation::Baker, 0.9));
        assert_eq!(class_of(&w, e), SocialClass::Working, "a master with no shop is still working class");
    }
}

/// Lift a grievance: the load is answered, not just waited out - a
/// grievance that stops counting right now (a grant, a fix, a fight won)
pub fn lift_grievance(world: &mut World, e: Entity, p: Problem) {
    if let Some(life) = world.get_component_mut::<InnerLife>(e) {
        life.grievances.retain(|(g, _)| *g != p);
    }
}
