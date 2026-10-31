//! The render model: everything a frame needs that isn't the player's own
//! art, camera or mouse. Produced server-side (or in-process for solo play)
//! by `Game::ui_model`, consumed by `View::render`. Serializable so a remote
//! client draws the exact same frame as a local one.

use serde::{Deserialize, Serialize};
use crate::engine::ecs::Entity;
use crate::game::tilemap::TileMap;
use crate::game::pets::Species;
use crate::sim::climate::Season;
use crate::sim::precipitation::PrecipKind;
use super::{BSprite, CtxAct};

/// A villager, rival, pet or other figure standing on the map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonV {
    /// The person entity (for bubble/name lookups)
    pub e: Entity,
    /// What they are on the map
    pub kind: PKind,
    pub pos: (f32, f32),
    /// Name over the head (villagers/rivals only)
    pub label: Option<String>,
    /// Draw a grave instead of a figure
    pub dead: bool,
    /// Small figure + "(child)" label
    pub kid: bool,
    /// Ring highlights
    pub selected: bool,
    pub in_reach: bool,
    /// Hunger/illness indicators next to the name
    pub hungry: bool,
    pub sick: bool,
    /// Activity marker: 0 idle, 1 seeking food, 2 busy
    pub act_mark: u8,
    /// A pet bonded strongly to its owner (the <3 label)
    pub bond: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PKind {
    /// Sprite index into art.villager_sprites
    Villager(usize),
    /// Sprite index into art.rival_sprites
    Rival(usize),
    Player,
    Pet(Species),
    /// Sprite index into art.guards
    Guard(usize),
    Priest,
    Traveller,
    Chicken,
    Gull,
    /// An animal of the fold, grazing the meadow
    Sheep,
    Goat,
}

/// A building on the map
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BldgV {
    pub x: f32,
    pub y: f32,
    pub sprite: BSprite,
    /// 0..1 while it rises (under construction)
    pub risen: f32,
    pub derelict: bool,
    /// Chimney smoke (the cottages nearest the market)
    pub smoke: bool,
}

/// A speech or thought bubble, already placed over the speaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleV {
    pub e: Entity,
    pub pos: (f32, f32),
    pub text: String,
    pub thought: bool,
    /// Speaker's name (to find the label height under the bubble)
    pub name: Option<String>,
}

/// A line of inspector text, pre-formatted server-side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub rgb: (u8, u8, u8),
}

/// The inspector card for the selection (or none when nothing's picked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorV {
    /// Villager sprite index for the portrait; None for a building card or grave
    pub fig: Option<usize>,
    /// Draw a grave portrait (died on `died_day`)
    pub dead: bool,
    pub died_day: Option<u64>,
    pub lines: Vec<Line>,
}

/// The right-hand panel's player/org data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelV {
    pub name: String,
    pub standing: String,
    pub influence: f32,
    pub gold: f32,
    pub org: String,
    pub members: Vec<String>,
    pub issues: Vec<String>,
    pub guilds: usize,
    /// Market servings (the HUD's "food")
    pub food: u32,
}

/// Which of the five action buttons is blocked and why
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub target: usize,
    pub picked: bool,
    pub building: Option<usize>,
}

/// Everything `View::render` draws that comes from the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiModel {
    pub tick: u64,
    /// Animation frame count (mill sails, smoke, the courier's ride)
    pub frame: u32,
    pub map: TileMap,
    pub town_wall: (i32, i32, i32, i32),
    pub season: Season,
    pub precip: Option<PrecipKind>,
    /// Frame the last rain stopped (rainbow fades from there)
    pub rainbow_from: Option<u32>,
    pub paused: bool,
    /// Sim speed when local control is allowed (always 1 when networked)
    pub speed: u32,
    pub stones: Vec<(i32, i32)>,
    pub buildings: Vec<BldgV>,
    pub people: Vec<PersonV>,
    pub bubbles: Vec<BubbleV>,
    pub toasts: Vec<String>,
    pub log: Vec<String>,
    pub sel: Selection,
    /// Villager indices within talk/meeting reach
    pub reach: Vec<usize>,
    /// Whom Talk/Recruit would go to now (a villager index)
    pub reach_target: usize,
    /// Disabled reason per action button
    pub blocked: [Option<String>; 5],
    pub ctx: Vec<CtxAct>,
    pub inspector: Option<InspectorV>,
    pub panel: PanelV,
    /// The player won (influence over the village lead)
    pub won: bool,
    /// Mill stands at MILL_TILE when the art has a mill
    pub mill: Option<(f32, f32)>,
    /// Wild herb tufts that are ready to pick
    pub herbs: Vec<(f32, f32)>,
    /// Ready mushroom patches in the shade
    pub mush: Vec<(f32, f32)>,
    /// The skeps of the apiary, and a swarm out following someone
    pub skeps: Vec<(f32, f32)>,
    pub swarm: Option<(f32, f32)>,
    /// Where feet have worn the ground in (x, y, strength)
    pub trails: Vec<(f32, f32, f32)>,
    /// The land's score: stump tiles, the foul reach, the drowned pit
    pub stumps: Vec<(f32, f32)>,
    pub murk: Vec<(f32, f32)>,
    pub pond: Option<(f32, f32)>,
    /// Kitchen-garden beds and how full they've grown (x, y, growth)
    pub gardens: Vec<(f32, f32, f32)>,
    /// Something riding over the walls tonight
    pub sky: Option<String>,
    /// The painted halls - color is clout that fades
    pub painted: Vec<(f32, f32)>,
}
