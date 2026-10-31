//! Herbology: witchy apothecary medicine tree.
//! From backlog: "simple witchy apothecary style medicine tree"
//! Plants/natural stuff that is pseudo-medical.

use crate::engine::ecs::{Entity, World};
use crate::game::household::{self, Building};
use crate::sim::illness::{Disease, Stage, Treatment};

/// A medicinal herb/plant
#[derive(Debug)]
pub struct Herb {
    pub name: String,
    pub potency: f32,       // 0-1 strength
    pub effect: HerbEffect,
    pub preparation: PrepMethod,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HerbEffect {
    Healing,
    Calming,
    Energizing,
    PainRelief,
    Purifying,
    Vision,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrepMethod {
    Tea,        // steeped in water
    Poultice,   // crushed, applied
    Tincture,   // alcohol extract
    Salve,      // infused in fat
    Smoke,      // burned, inhaled
}

/// Herbal remedy = herb + preparation
#[derive(Debug)]
pub struct Remedy {
    pub herb: Entity,
    pub method: PrepMethod,
    pub efficacy: f32,
}

/// Apothecary knowledge (learned herbs/recipes)
#[derive(Debug)]
pub struct Apothecary {
    pub known_herbs: Vec<String>,
    pub remedies: Vec<Remedy>,
    pub skill: f32,
}

/// Herb tree recipes (witchy medicine progression)
pub fn herb_tree() -> Vec<(String, HerbEffect, PrepMethod, f32)> {
    vec![
        ("chamomile".into(), HerbEffect::Calming, PrepMethod::Tea, 0.4),
        ("willow_bark".into(), HerbEffect::PainRelief, PrepMethod::Tea, 0.6),
        ("yarrow".into(), HerbEffect::Healing, PrepMethod::Poultice, 0.5),
        ("lavender".into(), HerbEffect::Calming, PrepMethod::Salve, 0.5),
        ("peppermint".into(), HerbEffect::Energizing, PrepMethod::Tea, 0.4),
        ("mugwort".into(), HerbEffect::Vision, PrepMethod::Smoke, 0.3),
        ("sage".into(), HerbEffect::Purifying, PrepMethod::Smoke, 0.6),
        ("feverfew".into(), HerbEffect::Healing, PrepMethod::Tincture, 0.55),
    ]
}

/// Spawn an herb
pub fn spawn_herb(world: &mut World, name: &str, potency: f32, effect: HerbEffect, method: PrepMethod) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Herb {
        name: name.into(),
        potency,
        effect,
        preparation: method,
    });
    e
}

/// Skill an apothecary gains each time they prepare a remedy
pub const PRACTICE_GAIN: f32 = 0.02;

/// Whether an herb's effect treats illness (vs. mood/energy/visions)
pub fn treats_illness(effect: HerbEffect) -> bool {
    matches!(effect, HerbEffect::Healing | HerbEffect::PainRelief | HerbEffect::Purifying)
}

/// The herbs that heal rather than soothe or stir visions: what the
/// herb-wife stocks her hut with and what a forage trip is for
pub fn medicinal_herbs() -> Vec<(&'static str, PrepMethod, f32)> {
    herb_tree().into_iter()
        .filter(|(_, effect, _, _)| treats_illness(*effect))
        .map(|(name, _, prep, potency)| {
            let name: &'static str = match name.as_str() {
                "willow_bark" => "willow_bark",
                "yarrow" => "yarrow",
                "sage" => "sage",
                "feverfew" => "feverfew",
                _ => "sage",
            };
            (name, prep, potency)
        })
        .collect()
}

/// Whether an apothecary knows a medicinal cure for this herb
pub fn knows_cure(world: &World, apothecary: Entity, herb: &str) -> bool {
    let known = world.get_component::<Apothecary>(apothecary)
        .map(|a| a.known_herbs.contains(&herb.to_string())).unwrap_or(false);
    known && medicinal_herbs().iter().any(|(n, _, _)| *n == herb)
}

/// Brew one of the hut's raw herbs into a remedy on the apothecary's
/// shelf: one stock unit out, one Remedy recorded. A tincture steeps
/// the herb in the water of life - it takes a measure of spirits off
/// the shelf too, and draws the cure out stronger. Returns the remedy's
/// efficacy (potency x skill x traditional-prep fit), or None when the
/// hut has no herb she knows a cure for
pub fn brew_from_stock(world: &mut World, apothecary: Entity, hut: Entity) -> Option<f32> {
    // the stock keys she has a known cure for, tree order (strongest
    // first) - a tincture's only on when there's spirit to steep it in
    let pick = medicinal_herbs().iter().copied()
        .filter(|(name, prep, _)| knows_cure(world, apothecary, name)
            && (*prep != PrepMethod::Tincture || world.get_component::<Building>(hut)
                .map(|b| b.stock.get("spirits").copied().unwrap_or(0) > 0).unwrap_or(false)))
        .find(|(name, _, _)| {
            world.get_component::<Building>(hut)
                .map(|b| b.stock.get(*name).copied().unwrap_or(0) > 0).unwrap_or(false)
        })?;
    let (name, prep, potency) = pick;
    {
        let mut b = world.get_component_mut::<Building>(hut)?;
        if household::take_stock(&mut b, name, 1) < 1 { return None; }
        if prep == PrepMethod::Tincture { household::take_stock(&mut b, "spirits", 1); }
    }
    let herb = spawn_herb(world, name, potency, herb_tree().into_iter()
        .find(|(n, _, _, _)| n == name).map(|(_, e, _, _)| e).unwrap_or(HerbEffect::Healing), prep);
    let treatment = prepare_remedy(world, apothecary, herb, prep)?;
    // the spirit draws the virtue out: a tincture keeps and works better
    if prep == PrepMethod::Tincture {
        if let Some(ap) = world.get_component_mut::<Apothecary>(apothecary) {
            if let Some(r) = ap.remedies.last_mut() { r.efficacy *= 1.3; }
        }
        return Some(treatment.efficacy * 1.3);
    }
    Some(treatment.efficacy)
}

/// The strongest remedy she has on the shelf: take it off and hand its
/// efficacy over (None when the shelf is empty)
pub fn best_remedy(world: &mut World, apothecary: Entity) -> Option<f32> {
    let ap = world.get_component_mut::<Apothecary>(apothecary)?;
    let best = ap.remedies.iter().enumerate()
        .max_by(|(_, a), (_, b)| a.efficacy.partial_cmp(&b.efficacy).unwrap())?
        .0;
    Some(ap.remedies.remove(best).efficacy)
}

/// How many remedies sit on her shelf
pub fn remedies_on_hand(world: &World, apothecary: Entity) -> usize {
    world.get_component::<Apothecary>(apothecary).map(|a| a.remedies.len()).unwrap_or(0)
}

/// Apply a remedy to a patient: each disease they're fighting loses
/// `efficacy` severity; a disease beaten outright moves to Recovering,
/// and a little of the treatment's strength goes into the body's own
/// condition. Returns whether there was anything to treat
pub fn apply_efficacy(world: &mut World, patient: Entity, efficacy: f32) -> bool {
    let mut treated = false;
    for d in world.query::<Disease>() {
        let Some(dis) = world.get_component_mut::<Disease>(d) else { continue };
        if dis.patient != patient || dis.stage == Stage::Recovering { continue; }
        dis.severity -= efficacy;
        treated = true;
        if dis.severity <= 0.0 { dis.stage = Stage::Recovering; }
    }
    if treated {
        if let Some(h) = world.get_component_mut::<crate::sim::illness::Health>(patient) {
            h.condition = (h.condition + efficacy * 0.2).min(1.0);
        }
    }
    treated
}

/// An apothecary prepares `herb` by `method` into a treatment for the sick.
/// They must know the herb, and it must be medicinal. Efficacy is the
/// herb's potency scaled by the apothecary's skill, halved when it isn't
/// prepared the traditional way. Each preparation is recorded and is
/// practice that improves the apothecary's skill.
pub fn prepare_remedy(world: &mut World, apothecary: Entity, herb: Entity, method: PrepMethod) -> Option<Treatment> {
    let (name, potency, effect, traditional) = world.get_component::<Herb>(herb)
        .map(|h| (h.name.clone(), h.potency, h.effect, h.preparation))?;
    if !treats_illness(effect) { return None; }
    let ap = world.get_component_mut::<Apothecary>(apothecary)?;
    if !ap.known_herbs.contains(&name) { return None; }
    let fit = if method == traditional { 1.0 } else { 0.5 };
    let efficacy = potency * ap.skill * fit;
    ap.remedies.push(Remedy { herb, method, efficacy });
    ap.skill = (ap.skill + PRACTICE_GAIN).min(1.0);
    Some(Treatment {
        name: format!("{} {:?}", name, method).to_lowercase(),
        dose: 1.0, absorption: 0.5, clearance: 0.1, efficacy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(w: &mut World, known: &[&str], skill: f32) -> Entity {
        let a = w.create_entity();
        w.add_component(a, Apothecary { known_herbs: known.iter().map(|s| s.to_string()).collect(), remedies: vec![], skill });
        a
    }

    #[test]
    fn test_known_herb_makes_remedy_and_builds_skill() {
        let mut w = World::new();
        let ap = setup(&mut w, &["willow_bark"], 0.5);
        let willow = spawn_herb(&mut w, "willow_bark", 0.6, HerbEffect::PainRelief, PrepMethod::Tea);
        let t = prepare_remedy(&mut w, ap, willow, PrepMethod::Tea).unwrap();
        assert!((t.efficacy - 0.3).abs() < 1e-6);
        let a = w.get_component::<Apothecary>(ap).unwrap();
        assert_eq!(a.remedies.len(), 1);
        assert!(a.skill > 0.5);
    }

    #[test]
    fn test_wrong_preparation_is_weaker() {
        let mut w = World::new();
        let ap = setup(&mut w, &["yarrow"], 1.0);
        let yarrow = spawn_herb(&mut w, "yarrow", 0.5, HerbEffect::Healing, PrepMethod::Poultice);
        let right = prepare_remedy(&mut w, ap, yarrow, PrepMethod::Poultice).unwrap().efficacy;
        let wrong = prepare_remedy(&mut w, ap, yarrow, PrepMethod::Smoke).unwrap().efficacy;
        assert!(wrong < right);
    }

    #[test]
    fn test_a_tincture_takes_the_spirit_and_draws_stronger() {
        let mut w = World::new();
        let ap = setup(&mut w, &["feverfew"], 1.0);
        let hut = household::spawn_building(&mut w,
            household::BuildingKind::Shop(crate::game::vocation::Vocation::Mason),
            "the hut", 0.0, 0.0, 20);
        // no spirit, no tincture: she brews nothing at all
        w.get_component_mut::<Building>(hut).unwrap().stock.insert("feverfew".into(), 1);
        assert!(brew_from_stock(&mut w, ap, hut).is_none(),
            "a tincture needs the water of life to steep in");
        // with the spirit on the shelf it goes in the cure - and the
        // alcohol draws the virtue out stronger than the bare herb
        w.get_component_mut::<Building>(hut).unwrap().stock.insert("spirits".into(), 1);
        let e = brew_from_stock(&mut w, ap, hut).unwrap();
        assert!((e - 0.55 * 1.3).abs() < 1e-6, "the spirit draws it stronger");
        assert_eq!(w.get_component::<Building>(hut).unwrap()
            .stock.get("spirits").copied().unwrap_or(0), 0, "a measure went in");
        assert_eq!(w.get_component::<Apothecary>(ap).unwrap().remedies.len(), 1);
    }

    #[test]
    fn test_unknown_or_non_medicinal_herb_gives_nothing() {
        let mut w = World::new();
        let ap = setup(&mut w, &["chamomile"], 1.0);
        let willow = spawn_herb(&mut w, "willow_bark", 0.6, HerbEffect::PainRelief, PrepMethod::Tea);
        let chamomile = spawn_herb(&mut w, "chamomile", 0.4, HerbEffect::Calming, PrepMethod::Tea);
        assert!(prepare_remedy(&mut w, ap, willow, PrepMethod::Tea).is_none(), "unknown herb");
        assert!(prepare_remedy(&mut w, ap, chamomile, PrepMethod::Tea).is_none(), "calming isn't medicine");
    }
}
