//! Two transcripts (E5-9, #1446): Scott's pair. Every talk has a
//! setting - the place it stands and the ears that reach it - and the
//! setting picks which transcript is spoken: the public one, deferential
//! where power can hear, or the hidden one kept for backstage. Lines
//! carry their channel as a tag, so the talk's record reads which face
//! each line was said in.
//!
//! The exposure rule: backstage is a place, but it's also a privilege -
//! the law's ear turns a tavern public again, and a stranger gets the
//! street voice even at the kitchen table. What a warm bond buys is the
//! invitation: the same villager who defers in the market bites at the
//! alehouse, once they've let you in.

/// The named place a talk stands in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// The market square, a stall round the cross - everyone hears
    Market,
    /// The church door after the service - the parish assembled
    ChurchDoor,
    /// The street or an open door - the reeve's men could be about
    Street,
    /// A field at dusk, the wood's edge - far from the manor's ears
    Field,
    /// The alehouse corner - the village's own room
    Alehouse,
    /// The kitchen table - inside the house, the warmest room
    Kitchen,
}

/// Which transcript a place speaks by default - before ears and bonds
/// weigh in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Said where power can hear: the deference, the careful word
    Public,
    /// Said among their own: the hidden transcript, the bite
    Backstage,
}

impl Place {
    /// The channel this place speaks when nothing modulates it
    pub fn channel(self) -> Channel {
        match self {
            Place::Alehouse | Place::Kitchen | Place::Field => Channel::Backstage,
            _ => Channel::Public,
        }
    }

    /// The fact-slug the criteria world knows it by (`pair:place:*`)
    pub fn slug(self) -> &'static str {
        match self {
            Place::Market => "market",
            Place::ChurchDoor => "churchdoor",
            Place::Street => "street",
            Place::Field => "field",
            Place::Alehouse => "alehouse",
            Place::Kitchen => "kitchen",
        }
    }
}

/// The village's standing weather (#1696): what the whole parish lives
/// under this stretch, whoever's talking - empty stores, a fever in the
/// beds, the lord's squeeze, fresh graves, the long snow. The talk
/// carries it the way it carries the day's sky: the trees read the
/// `village:*` facts and bend their topics toward the weight. A
/// bitmask - a famine can stand in a hard winter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pressure(pub u8);

impl Pressure {
    /// Nothing amiss
    pub const NONE: Pressure = Pressure(0);
    /// The stores are out, the bellies know it
    pub const FAMINE: Pressure = Pressure(1);
    /// The sickness is in more than one bed
    pub const FEVER: Pressure = Pressure(2);
    /// The cold's settled in for the season
    pub const WINTER: Pressure = Pressure(4);
    /// Grievances abroad - taxed, blamed, shorted
    pub const UNREST: Pressure = Pressure(8);
    /// Fresh graves in the churchyard
    pub const BEREFT: Pressure = Pressure(16);
    /// A fat year - full purses, full cellar
    pub const FLUSH: Pressure = Pressure(32);

    pub fn contains(self, p: Pressure) -> bool { self.0 & p.0 != 0 }

    /// The weights a talk can open on. `village:hard` is the umbrella
    /// every hard axis shares, so one tree hears them all
    pub fn facts(self) -> Vec<String> {
        let mut v: Vec<String> = vec![];
        let mut put = |bit: Pressure, slug: &str| if self.contains(bit) {
            v.push(format!("village:{slug}"));
        };
        put(Pressure::FAMINE, "famine");
        put(Pressure::FEVER, "fever");
        put(Pressure::WINTER, "winter");
        put(Pressure::UNREST, "unrest");
        put(Pressure::BEREFT, "bereft");
        put(Pressure::FLUSH, "flush");
        if self.0 & (Pressure::FAMINE.0 | Pressure::FEVER.0 | Pressure::WINTER.0
            | Pressure::UNREST.0 | Pressure::BEREFT.0) != 0
        {
            v.push("village:hard".into());
        }
        v
    }
}

/// A talk's setting: the place, the ears, the bond (E5-9, #1446)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Setting {
    /// Where it stands
    pub place: Place,
    /// The law's ear is on it - an overseer about, the tax man in the
    /// room, the watch walking past. Their presence turns backstage
    /// public again
    pub law_earshot: bool,
    /// The listener's been let past the public face. The invitation is
    /// a trust milestone: a cold bond hears the street voice anywhere
    pub invited: bool,
    /// The village's standing weather (#1696): the weight the whole
    /// parish lives under - the talk's topics bend toward it
    pub pressure: Pressure,
}

/// The street, unmarked: the face every talk wears by default
pub const STREET: Setting = Setting {
    place: Place::Street, law_earshot: false, invited: false,
    pressure: Pressure::NONE,
};

/// The warm backstage, no ears about: what a trusted talk at the
/// alehouse or the kitchen table carries
pub const BACKSTAGE: Setting = Setting {
    place: Place::Alehouse, law_earshot: false, invited: true,
    pressure: Pressure::NONE,
};

impl Setting {
    /// Which transcript the talk speaks: backstage only while the place
    /// keeps it, the law can't hear, and the bond's bought the way in
    pub fn channel(&self) -> Channel {
        if self.place.channel() == Channel::Backstage
            && !self.law_earshot && self.invited {
            Channel::Backstage
        } else {
            Channel::Public
        }
    }

    /// The pair-facts the setting contributes: `pair:place:*` always,
    /// `pair:channel:backstage` when the hidden transcript's the one
    /// being spoken, `pair:earshot:law` while their ear's on it
    pub fn facts(&self) -> Vec<String> {
        let mut v = vec![
            format!("pair:place:{}", self.place.slug()),
            format!("pair:channel:{}", match self.channel() {
                Channel::Public => "public",
                Channel::Backstage => "backstage",
            }),
        ];
        if self.law_earshot { v.push("pair:earshot:law".into()); }
        v.extend(self.pressure.facts());
        v
    }
}
