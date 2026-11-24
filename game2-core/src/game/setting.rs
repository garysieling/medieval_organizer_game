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
}

/// The street, unmarked: the face every talk wears by default
pub const STREET: Setting = Setting {
    place: Place::Street, law_earshot: false, invited: false,
};

/// The warm backstage, no ears about: what a trusted talk at the
/// alehouse or the kitchen table carries
pub const BACKSTAGE: Setting = Setting {
    place: Place::Alehouse, law_earshot: false, invited: true,
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
        v
    }
}
