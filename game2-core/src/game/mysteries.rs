//! Village mysteries: the gossip a seat can piece together a fact at a
//! time. Each is a question somebody in the village wants answered and
//! five facts folk pass on in talk; hear them all and the truth's yours
//! to carry back to whoever asked.

use crate::game::vocation::Vocation;

/// Facts to a mystery
pub const FACTS: usize = 5;

/// One mystery doing the rounds: the question, the gossip, and the
/// payoff when it's told to whoever wanted to know
#[derive(Debug)]
pub struct Mystery {
    /// What the asker wonders, in a line ("who's been leaving gifts on doorsteps")
    pub title: &'static str,
    /// The gossip pieces, one learned per telling - five makes the whole
    pub facts: [&'static str; FACTS],
    /// The trade most likely to be asking, when one fits - else any folk
    pub asker: Option<Vocation>,
    /// What they say when the truth's told
    pub wonders: &'static str,
    /// And their thanks, once it's delivered
    pub thanks: &'static str,
}

/// The mysteries pre-written for the village. A seat meets one at a
/// time, drawn off this table.
pub const MYSTERIES: &[Mystery] = &[
    Mystery {
        title: "who keeps leaving gifts on doorsteps",
        facts: [
            "Somebody left a twist of wool on Matilda's step before first light.",
            "The miller's boy found a pouch of flour outside his door, still warm.",
            "A carved bird appeared on the church door - nobody claims it.",
            "Old Wystan swears it's the hermit's doing, out of the dark.",
            "The widow's cow gave better milk the week the hay came.",
        ],
        asker: None,
        wonders: "So it was someone kind, all along. I'd half convinced myself the village had gone hard.",
        thanks: "Bless you - truly. You've given me back my faith in the place, and I'm not one for saying so lightly.",
    },
    Mystery {
        title: "why the mill wheel sticks on market days",
        facts: [
            "The wheel always jams the morning the tax man rides in.",
            "Fresh mud was found packed in the race on a Thursday.",
            "The apprentice swears he saw fingers on the wheel before it stuck.",
            "The miller laughs it off, but I've seen him count the toll twice.",
            "A crow's feather turned up in the sluice, plain as day.",
        ],
        asker: None,
        wonders: "Then it's sabotage, and somebody's playing me for a fool. I'll know who to watch now.",
        thanks: "You saw what I couldn't. That's a rare eye - the village owes you one.",
    },
    Mystery {
        title: "who rang the chapel bell at midnight",
        facts: [
            "The bell rang out over the street in the dead of night.",
            "The rope was cut clean through and tied back short.",
            "The whole street woke; the dogs wouldn't stop barking at the church.",
            "Father Anselm swears he never left his bed.",
            "Boot-prints in the frost led from the graveyard to the door.",
        ],
        asker: Some(Vocation::Priest),
        wonders: "From the graveyard, you say. Then it wasn't a prank - somebody wanted to be seen coming.",
        thanks: "The truth of it's a weight lifted. Come - my door's open to you now, more than most.",
    },
    Mystery {
        title: "where the reeve's tithe-cart went",
        facts: [
            "The cart turned up by the creek, empty and cold.",
            "Its wheels were found far upstream, by the ford.",
            "A boy saw two men arguing by the mill that evening.",
            "The reeve hasn't ridden out alone since.",
            "Cart-marks on its side-board match a farm up the hollow.",
        ],
        asker: None,
        wonders: "So it was moved in daylight, by folk who knew the roads. That narrows it to half the village.",
        thanks: "Careful work, that - asking after instead of grabbing. You've a friend in me, and the reeve will hear of it.",
    },
    Mystery {
        title: "the face in the elder tree",
        facts: [
            "A face has appeared in the elder oak, weeping sap when it rains.",
            "Children leave flowers at its roots and swear it watches them.",
            "The woodcutter swears he never touched it.",
            "The hermit says the tree remembers what the village forgets.",
            "New lines showed on the face after the lightning strike.",
        ],
        asker: None,
        wonders: "It remembers, does it? Then it's right - the village needs something that does.",
        thanks: "You've an eye for what's hidden. Walk with us a while - the village talks different when you're near.",
    },
    Mystery {
        title: "why the alewife's ale has turned sour",
        facts: [
            "Her ale's gone off three batches running.",
            "The water-boy swears he saw her tasting it at dusk.",
            "A new lad started helping her this month.",
            "The cooper found a bad cask in her stack.",
            "She laughs it off, but she won't say where the water comes from.",
        ],
        asker: None,
        wonders: "A new hand and a bad cask - that's a beginning. She's been patient with it longer than I knew.",
        thanks: "Sharp eyes. Most folk'd have drunk it and complained later - you've a friend's welcome at my bench.",
    },
];
