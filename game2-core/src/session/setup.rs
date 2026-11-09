//! World generation: the map a seed draws, the village laid out on it, and
//! the saved scenarios. Runs once per game; everything after is the sim.

use crate::session::*;

impl Game {
    /// The classic map (MAP_SEED): tests, dumps and tooling stay put
    #[cfg(test)]
    pub(in crate::session) fn new() -> Game { Game::new_seeded(MAP_SEED) }

    /// A game on the map `seed` draws. The country is perturbed - the coast
    /// shifts, the river meanders a tile or two - and the castle suits it:
    /// the wall's east face keeps clear of the river's west bank. Outside it
    /// the map's grown a little townage of its own: a farm below the south
    /// gate, a woodcutter's clearing and a fisher's hut on the shore.
    pub fn new_seeded(seed: u64) -> Game {
        let mut rng = Rng(seed ^ 0x9E3779B9);
        let mut world = World::new();
        let mut sched = Scheduler::new();
        sched.add_system(villager::HungerSystem);
        sched.add_system(villager::SeekFoodSystem);
        sched.add_system(pets::PetBondSystem);
        sched.add_system(SeasonSystem);
        sched.add_system(GuildFormationSystem);
        sched.add_system(IllnessSystem); // diseases progress, spread to neighbours, and heal
        sched.add_system(crate::game::feuds::FeudSystem); // bad blood worsens till it's talked down
        sched.add_system(crate::game::contracts::ContractSystem); // kept terms build a name for keeping them

        let mut terrain = Terrain::generate_country(MAP_TILES, 0.7, seed);
        // the ground's own shape: a named landform picked by seed shapes
        // the heightfield - keep its name for the telling
        let landform = crate::game::landforms::shape_terrain(&mut terrain, seed)
            .map(|p| p.display_name.clone());
        // where this seed's village stands: dry country the whole cluster
        // fits inside, on land the country plan can walk back to
        let (vx, vy) = village_origin(&terrain, seed);
        let climate = climate::spawn_climate(&mut world);
        // the felt-worth ledger rides beside the climate - what the
        // shelves show leans what a swap is worth, quietly
        goods::spawn_worths(&mut world);
        let music = crate::game::music::spawn_music(&mut world);
        let weather = crate::sim::weather::spawn_weather(&mut world,
            crate::sim::weather::Weather::Clear, 0.0, 0);
        // tell the sim where the tile edge is, or its fixed bound pinches
        // anyone homed past the old 33-tile map
        world.add_component(climate, villager::WorldBounds { w: MAP_TILES as f32, h: MAP_TILES as f32 });
        let market_food = villager::spawn_food_source(&mut world, vx as f32, vy as f32, 500);
        // the street's market book: empty till a price's fixed on a good
        world.add_component(market_food, crate::sim::economics::Market::default());
        // the fields go in with the country - the map's made by then,
        // and a spot on water or a roof would feed no one

        // the river winds south between the towns to the sea; each waypoint
        // on its course drifts a tile or two, so it meanders differently
        // every map (kept east of the wall and west of the rivals')
        let river: Vec<(i32, i32)> = (0..8).map(|i| (vx + 8 + rng.below(3), vy - 13 + i * 6)).collect();
        // the castle suits the river: the east face keeps a tile clear of
        // its west bank and the rest of the rectangle breathes a tile
        // either way (every building still sits well inside). Only the
        // waypoints level with the wall set it - the castle doesn't grow
        // just because the river now runs further south
        let wall = (vx - 8 + rng.around(1), vy - 9 + rng.around(1),
            river.iter().filter(|w| w.1 <= vy + 14).map(|w| w.0).min().unwrap() - 1, vy + 8 + rng.around(1));
        // the walled fields run out of the south gate to a little farm
        let (gate_x, farm_y) = ((wall.0 + wall.2) / 2, wall.3 + 2);
        // the country beyond: where this seed's towns and castles stand
        // (its own stream, so the village's rolls land as they always have)
        let country_plan = country::plan_country(&terrain, &river, (vx, vy), &mut Rng(seed ^ 0x5173_C0DE));

        // fields beside the food store, and strips either side of the lane
        // that runs out of the south gate, down through the farm - most of
        // the land the map grew is worked
        let mut fields: Vec<(i32, i32, Tile)> = vec![
            (gate_x - 2, wall.3 + 1, Tile::Wheat), (gate_x - 1, wall.3 + 1, Tile::Wheat),
            (gate_x - 2, wall.3 + 2, Tile::Wheat), (gate_x - 1, wall.3 + 2, Tile::Corn),
            (gate_x, wall.3 + 2, Tile::Corn), (gate_x + 1, wall.3 + 2, Tile::Wheat),
        ];
        for dy in 2..10 {
            for &dx in &[-6i32, -5, -4, -3, 3, 4, 5, 6] {
                let (fx, fy) = (gate_x + dx, farm_y + dy);
                if noise(fx, fy, 17) < 0.15 { continue; } // a ragged field edge
                fields.push((fx, fy, if dy % 3 == 1 { Tile::Corn } else { Tile::Wheat }));
            }
        }

        // where each building wants to stand; the map moves one onto dry land if it must
        let planned: Vec<((i32, i32), BSprite)> = vec![
            // the better homes on the square's corners (the first three
            // have chimney smoke)
            ((vx - 2, vy - 3), BSprite::House(0)), ((vx + 2, vy - 3), BSprite::House(1)), ((vx - 2, vy + 3), BSprite::House(2)),
            // the smithy keeps its fire by the south gate
            ((vx + 1, vy + 5), BSprite::Blacksmith),
            // the church stands on the market square's north side, the
            // guild hall on its east
            ((vx, vy - 3), BSprite::Church), ((vx + 3, vy - 1), BSprite::Guildhall),
            // rival village across the river (red-roofed foreign houses)
            ((vx + 12, vy - 3), BSprite::RivalHouse), ((vx + 14, vy - 1), BSprite::RivalWoodhouse),
        ];
        let mut herb_patches: Vec<HerbPatch> = vec![];
        let mut layout = Layout {
            // our village and the rivals' each stand on dry land, the neck
            // between them never floods, and the farm below the south gate
            // gets a broad pocket of level field
            settlements: vec![(vx, vy, 9), (vx + 13, vy - 2, 4), (vx + 9, vy - 1, 3), (gate_x, farm_y + 4, 6)],
            river: river.clone(),
            bridge_near: river[2],
            town_wall: Some(wall),
            fields,
            buildings: planned.iter().map(|(p, _)| *p).collect(),
            road_hub: (vx, vy),
            streets: town_streets(wall, (vx, vy), [(vx, vy - 3), (vx - 6, vy - 7)]),
            // the lane out of the south gate runs down through the strips
            // to the farmstead, beside the road to the rivals' village
            road_ends: vec![(vx + 13, vy - 2), (gate_x, farm_y + 8)],
            // two remote keeps and two satellite towns make the enlarged
            // country read as a settled region while the main village keeps
            // its existing simulation and political systems.
            castles: country_plan.castles.clone(),
            towns: country_plan.towns.clone(),
            settlement_houses: country_plan.houses.clone(),
        };
        layout.buildings.push((vx - 6, vy - 7));
        let heights = terrain.heights.clone();
        let mut map = TileMap::generate(&mut terrain, &layout);
        // where the mill wanted to stand is where it ended up (or nearly):
        // the last layout building, after the eight planned ones
        let mill_pos = map.buildings.get(8).map(|&(x, y)| (x as f32, y as f32))
            .unwrap_or((vx as f32 - 6.0, vy as f32 - 7.0));
        let mut buildings: Vec<(f32, f32, BSprite)> = planned.iter().zip(&map.buildings)
            .map(|((_, s), (x, y))| (*x as f32, *y as f32, *s)).collect();

        // out-of-town townage: a farmhand's cottage by the south fields and
        // a smallholder's at the far end of the strips, a woodcutter's
        // clearing at the woods' edge, a fisher's hut on the shore - each
        // gets its road run out to it
        for (site, sprite) in [
            (Some((gate_x + 2, farm_y)), BSprite::House(6)),
            (Some((gate_x - 3, farm_y + 9)), BSprite::House(9)),
            (shore_site(&map, (vx + 10, vy + 14)), BSprite::House(7)),
            (woods_edge_site(&map, wall, (wall.0, (wall.1 + wall.3) / 2)), BSprite::House(8)),
        ] {
            let Some((bx, by)) = site.and_then(|(x, y)| map.nearest_buildable(x, y)) else { continue };
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, sprite));
        }

        // a fuller town: homes on a loose grid inside the wall (so ~30
        // people have somewhere to live), the odd cottage outside, and a
        // workshop for most of the vocations that get rolled - the ~85%
        // coverage leaves the odd trade without a shop to work out of
        // the plots between the streets, nearest the square first: the
        // workshops take the inner ring (the market's trade), the homes the
        // outer, out to the wall - the rolls still come in the old order
        let plots = town_plots(&map, wall, (vx, vy));
        let n_homes = ((wall.1 + 3..wall.3 - 1).step_by(3).count()) * ((wall.0 + 3..wall.2 - 1).step_by(3).count());
        // the inner ring (enough for every trade) stays for the workshops;
        // the homes spread evenly over the rest, a yard or two apart
        let mut outer: Vec<(i32, i32)> = plots.iter().copied().skip(TRADE_PLOTS).collect();
        outer.sort_by_key(|&(x, y)| (y, x));
        let stride = (outer.len() as f32 / n_homes.max(1) as f32).max(1.0);
        let picks: Vec<(i32, i32)> = (0..n_homes).filter_map(|k| outer.get((k as f32 * stride) as usize).copied()).collect();
        // a company town zones its homes by where they stand: the
        // manager's row by the square, twins west, rowhouses east, the
        // barracks at the south gate. The buildings and beds land the
        // same either way - only the looks (and the river works below)
        // differ; the other seeds keep the loose scatter. Its own rng
        // stream, so the village's rolls land as they always have.
        let mut town_rng = Rng(seed ^ 0xC0FFEE);
        let company = town_rng.below(100) < 40;
        let mid_x = (wall.0 + wall.2) / 2;
        let mut twin: Option<usize> = None;
        // the manager's row is one street of grand houses, not the whole
        // quarter: the first few picks by the square wear it, the rest
        // fall through to the twins and the terraces
        let mut mansions = 0;
        let looks: Vec<BSprite> = picks.iter().map(|&(x, y)| {
            let d_sq = (x - vx).abs().max((y - vy).abs());
            let d_gate = (x - gate_x).abs().max((y - wall.3).abs());
            if d_gate <= 5 { BSprite::Barracks(((x + y) as usize) % 2) }
            else if d_sq <= 6 && mansions < 4 { mansions += 1; BSprite::Mansion }
            else if x < mid_x {
                if let Some(i) = twin.take() { BSprite::House(i) }
                else { let i = town_rng.below(97) as usize; twin = Some(i); BSprite::House(i) }
            } else { BSprite::Rowhouse(((y * 37 + 11).rem_euclid(97)) as usize) }
        }).collect();
        let mut homes = picks.into_iter().zip(looks);
        for _ in 0..n_homes {
            let _ = (rng.around(1), rng.around(1));
            let Some(((bx, by), look)) = homes.by_ref().find(|&((x, y), _)| map.buildable(x, y)) else { break };
            map.build_at(bx, by);
            map.connect((bx, by));
            // the cottage roll is spent either way, so a company town's
            // folk land the same as a loose one's on the same seed
            let cottage = rng.below(97) as usize;
            buildings.push((bx as f32, by as f32, if company { look } else { BSprite::House(cottage) }));
        }
        for &(x, y) in &[(wall.0 - 3, wall.3 + 3), (wall.2 + 4, wall.3 + 2), (wall.0 - 5, (wall.1 + wall.3) / 2)] {
            if let Some((bx, by)) = map.nearest_buildable(x, y) {
                map.build_at(bx, by);
                map.connect((bx, by));
                buildings.push((bx as f32, by as f32, BSprite::House(rng.below(97) as usize)));
            }
        }
        // the herb-wife's hut stands apart in the meadow below the west end
        // of the wall, nearer the wild herbs than the market square
        let herb_hut_site = map.nearest_buildable(wall.0 + 1, wall.3 + 3);
        if let Some((bx, by)) = herb_hut_site {
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, BSprite::House(5)));
        }
        // the apiary: the beekeeper's yard at the meadow's edge, east of the fold
        let apiary_site = map.nearest_buildable(wall.0 + (wall.2 - wall.0) / 2 + 8, wall.3 + 5);
        if let Some((bx, by)) = apiary_site {
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, BSprite::House(9)));
        }
        // the fold: the shepherd's pen in the meadow below the town
        let fold_site = map.nearest_buildable(wall.0 + (wall.2 - wall.0) / 2 + 3, wall.3 + 4);
        if let Some((bx, by)) = fold_site {
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, BSprite::House(7)));
        }

        // ---- the people ----
        // vocations rolled by weight; knights and priests are PMC
        // workshops for the vocations that get rolled (~85% coverage);
        // shops go on buildable tiles inside the wall, farms just outside
        let mut vocs_present: Vec<Vocation> = vec![];
        let mut adult_vocs: Vec<Vocation> = (0..23).map(|_| roll_vocation(&mut rng)).collect();
        // the fold always has a keeper: one hand from a trade the village
        // has twice (never the last farmer) keeps the flock instead - a
        // later duplicate is taken so the shops roll lands the same
        let swap = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or_else(|| adult_vocs.iter().rposition(|&v| v != Vocation::Farmer).unwrap_or(5));
        adult_vocs[swap] = Vocation::Shepherd;
        // and the apiary: another doubled trade hands the bees over
        let swap2 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap);
        adult_vocs[swap2] = Vocation::Beekeeper;
        // and the loom: the wool needs somewhere to go
        let swap3 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && v != Vocation::Beekeeper
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap2);
        adult_vocs[swap3] = Vocation::Weaver;
        // and the still: the gardens' flowers need somewhere to go
        let swap4 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && v != Vocation::Beekeeper && v != Vocation::Weaver
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap3);
        adult_vocs[swap4] = Vocation::Perfumer;
        // and the paint shop: pigment to clout
        let swap5 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && v != Vocation::Beekeeper && v != Vocation::Weaver && v != Vocation::Perfumer
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap4);
        adult_vocs[swap5] = Vocation::Painter;
        // and the river's edge needs its fisher
        let swap6 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && v != Vocation::Beekeeper && v != Vocation::Weaver && v != Vocation::Perfumer
                && v != Vocation::Painter && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap5);
        adult_vocs[swap6] = Vocation::Fisher;
        // and the drink trade: every village keeps one - this village's
        // is rolled off the seed (alehouse, brewery, vintner's or still).
        // A later duplicate of an unforced trade gives way first; failing
        // that, any unforced trade does - the forced benches keep theirs
        let drink_voc = DRINK_TRADES[rng.below(DRINK_TRADES.len() as i32) as usize];
        let forced = |v: Vocation| matches!(v,
            Vocation::Farmer | Vocation::Shepherd | Vocation::Beekeeper | Vocation::Weaver
            | Vocation::Perfumer | Vocation::Painter | Vocation::Fisher | Vocation::Taverner
            | Vocation::Alewife | Vocation::Brewer | Vocation::Vintner | Vocation::Distiller);
        let swap7 = adult_vocs.iter().rposition(|&v| !forced(v)
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .or_else(|| adult_vocs.iter().rposition(|&v| !forced(v)))
            .unwrap_or(swap6);
        adult_vocs[swap7] = drink_voc;
        // and the tavern on the green: whoever brews here, the village
        // drinks at one table. The keeper brews nothing - he buys the
        // drink trade out and pours it by the cup, and keeps a bed for
        // whoever's on the road
        let swap8 = adult_vocs.iter().rposition(|&v| !forced(v)
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .or_else(|| adult_vocs.iter().rposition(|&v| !forced(v)))
            .unwrap_or(swap7);
        adult_vocs[swap8] = Vocation::Taverner;
        for &v in &adult_vocs { if !vocs_present.contains(&v) { vocs_present.push(v); } }
        let mut shop_kinds: Vec<BuildingKind> = vec![];
        for v in vocs_present {
            // the fold and the apiary stand apart, seeded on their own
            if v == Vocation::Shepherd || v == Vocation::Beekeeper { continue; }
            // the trades a feature hangs on always keep their house
            let must = matches!(v, Vocation::Weaver | Vocation::Perfumer | Vocation::Painter
                | Vocation::Fisher | Vocation::Alewife | Vocation::Brewer | Vocation::Vintner
                | Vocation::Distiller | Vocation::Taverner);
            if must || rng.below(100) < 85 { shop_kinds.push(household::kind_for(v)); }
        }
        // the smithy, church and guildhall already stand: shop for the
        // toolmaker, the priest's church, and the trader's stall
        for (x, y) in [(19, 19), (14, 9)] { let _ = (x, y); }
        // the company town's works stands on the riverbank east of the
        // wall: the weaver's bench moves out there to the fulling mill
        let factory_site = if company { shore_site(&map, (wall.2, (wall.1 + wall.3) / 2)) } else { None };
        let mut factory_built = false;
        for kind in shop_kinds.iter().copied() {
            // near the market for shops; farms by the south fields; the
            // fish-smoke house goes down to the water's edge
            let (cx, cy) = if kind == BuildingKind::Shop(Vocation::Farmer) { (gate_x + 2, farm_y + 1) }
                else if kind == BuildingKind::Shop(Vocation::Fisher) { (vx + 19, vy + 2) }
                else { (vx + rng.around(4), vy - 1 + rng.around(4)) };
            let in_town = !matches!(kind, BuildingKind::Shop(Vocation::Farmer | Vocation::Fisher));
            // the works takes the riverbank when it's got one
            let site = if company && kind == BuildingKind::Shop(Vocation::Weaver) {
                factory_site.filter(|&(x, y)| map.buildable(x, y))
            } else { None };
            // a workshop takes the free plot nearest the square
            let plot = if in_town && site.is_none() { plots.iter().copied().find(|&(x, y)| map.buildable(x, y)) } else { None };
            let Some((bx, by)) = site.or(plot).or_else(|| map.nearest_buildable(cx, cy)) else { continue };
            factory_built |= site.is_some();
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, BSprite::House(rng.below(97) as usize)));
        }
        // every building gets an entity: homes hold occupants, workplaces
        // hold a worker and a stockpile
        let mut building_entities: Vec<Entity> = vec![];
        let rival_from = buildings.len().saturating_sub(buildings.len()); // marker, computed below
        let n_planned = 8; // the `planned` list
        for (i, &(bx, by, _)) in buildings.iter().enumerate() {
            let (kind, name, cap) = if i < 3 { (BuildingKind::Home, "a cottage", 20) }
                else if i == 3 { (BuildingKind::Shop(Vocation::Toolmaker), "the smithy", 20) }
                else if i == 4 { (BuildingKind::Church, "the church", 20) }
                else if i == 5 { (BuildingKind::TraderStall, "the trader's stall", 12) }
                else if i < n_planned { (BuildingKind::Home, "a foreign house", 20) }
                else if i >= buildings.len() - shop_kinds.len() {
                    // the freshly seeded workshops, in shop_kinds order
                    let k = shop_kinds[i - (buildings.len() - shop_kinds.len())];
                    (k, match k { BuildingKind::Shop(v) => vocation::trade_name(v), BuildingKind::Church => "the church", BuildingKind::WatchPost => "the watch house", _ => "a workshop" }, 20)
                }
                else { (BuildingKind::Home, "a house", 20) };
            building_entities.push(household::spawn_building(&mut world, kind, name, bx, by, cap));
        }
        let _ = rival_from;
        // the herb hut: the Home-kind building standing on the site picked
        // earlier (Pos-matched so shop-seeding order can't mislabel it)
        let herb_hut = herb_hut_site.and_then(|(hx, hy)| {
            let e = building_entities.iter().copied().find(|&e| {
                world.get_component::<Pos>(e)
                    .map(|p| p.x.round() as i32 == hx && p.y.round() as i32 == hy).unwrap_or(false)
            })?;
            if let Some(b) = world.get_component_mut::<Building>(e) { b.name = "the herb hut".into(); }
            Some(e)
        });
        // the apiary: the building on the site picked earlier, re-kinded
        let apiary = apiary_site.and_then(|(hx, hy)| {
            let e = building_entities.iter().copied().find(|&e| {
                world.get_component::<Pos>(e)
                    .map(|p| p.x.round() as i32 == hx && p.y.round() as i32 == hy).unwrap_or(false)
            })?;
            if let Some(b) = world.get_component_mut::<Building>(e) {
                b.kind = household::BuildingKind::Apiary;
                b.name = "the apiary".into();
            }
            Some(e)
        });
        // the cheese cellar: a dark room where wheels age - this
        // village's is a stone one, every village's cheese is its own
        let cellar_trait = "stone-cellar";
        let cellar = fold_site.and_then(|(fx, _)| {
            let b = building_entities.iter().copied().find(|&e| {
                world.get_component::<Building>(e).map(|bb|
                    bb.kind == household::BuildingKind::Home).unwrap_or(false)
                    && world.get_component::<Pos>(e)
                        .map(|p| (p.x.round() as i32 - fx).abs() < 6).unwrap_or(false)
            })?;
            if let Some(bb) = world.get_component_mut::<Building>(b) {
                bb.kind = household::BuildingKind::Cellar;
                bb.name = "the cheese cellar".into();
            }
            Some(b)
        });
        // the fold: the building on the site picked earlier, re-kinded to
        // a pasture (Pos-matched so shop-seeding order can't mislabel it)
        let fold = fold_site.and_then(|(hx, hy)| {
            let e = building_entities.iter().copied().find(|&e| {
                world.get_component::<Pos>(e)
                    .map(|p| p.x.round() as i32 == hx && p.y.round() as i32 == hy).unwrap_or(false)
            })?;
            if let Some(b) = world.get_component_mut::<Building>(e) {
                b.kind = household::BuildingKind::Pasture;
                b.name = "the fold".into();
            }
            Some(e)
        });
        // a workplace wears its own look, not a cottage's (the index still
        // picks the stand-in cottage when that look has no art)
        for (i, &e) in building_entities.iter().enumerate() {
            let Some(look) = world.get_component::<Building>(e).and_then(|b| WorkLook::of(b.kind)) else { continue };
            if let BSprite::House(h) = buildings[i].2 { buildings[i].2 = BSprite::Work(look, h); }
        }
        // the company town's river works: the weaver's bench moved out to
        // the bank - it answers to the factory's name and wears its look
        if factory_built {
            if let Some(i) = building_entities.iter().position(|&e| {
                world.get_component::<Building>(e)
                    .map(|b| b.kind == BuildingKind::Shop(Vocation::Weaver)).unwrap_or(false)
            }) {
                if let Some(b) = world.get_component_mut::<Building>(building_entities[i]) {
                    b.name = "the factory".into();
                }
                if let BSprite::Work(_, h) = buildings[i].2 {
                    buildings[i].2 = BSprite::Work(WorkLook::Factory, h);
                }
            }
        }
        // the trader's stall is the guildhall slot (index 5), stocked with imports
        let trader_stall = building_entities[5];
        for g in ["cloth", "paper", "ink", "pigment"] {
            world.get_component_mut::<Building>(trader_stall).unwrap().stock.insert(g.to_string(), 4);
        }

        // boulders out in the fields, scattered round the village the way
        // they always were round (16, 16)
        let stones: Vec<(i32, i32)> = [(-10, 3), (4, 7), (18, 14), (-2, 22), (26, 2)].iter()
            .filter_map(|&(dx, dy)| map.nearest_buildable(vx + dx, vy + dy)).collect();
        for &(x, y) in &stones { map.occupy(x, y); }

        // the land keeps score: canopy a little thinned already, the pit
        // already wet, the reach already carrying the town's leavings
        let mut land = crate::sim::environment::LandScore {
            tree_cover: map.trees.iter().map(|&t| (t, 0.85f32)).collect(),
            pit_water: 0.5, effluent: Default::default(), filth: Default::default(),
        };
        let pit = stones.first().copied();
        // the reach: river tiles below the south wall take what the town dumps
        let reach: Vec<(i32, i32)> = (0..map.height as i32).flat_map(|y| (0..map.width as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| map.get(x, y) == Some(Tile::River) && y > wall.3).collect();
        for &t in &reach { land.effluent.insert(t, 0.15); }

        // kitchen gardens: a soil bed beside some of the homes
        let homes_for_gardens: Vec<Entity> = building_entities.iter().copied()
            .filter(|&b| world.get_component::<Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false))
            .take(6).collect();
        let mut gardens: Vec<(Entity, (i32, i32), f32, bool)> = vec![];
        for home in homes_for_gardens {
            if let Some(hp) = world.get_component::<Pos>(home).copied() {
                let (hx, hy) = (hp.x.round() as i32, hp.y.round() as i32);
                if let Some(bed) = [(hx + 2, hy), (hx - 2, hy), (hx, hy + 2), (hx, hy - 2),
                                    (hx + 2, hy + 1), (hx - 2, hy - 1)]
                    .iter().copied().find(|&(x, y)| map.passable(x, y)) {
                    // every other bed keeps a kitchen garden - roots and
                    // peas for the pot beside the herb-wife's flowers
                    let kitchen = gardens.len() % 2 == 1;
                    gardens.push((home, bed, (gardens.len() as f32 * 0.3) % 0.9, kitchen));
                }
            }
        }

        // herb patches: wild tufts out in the meadow past the wall, weighted
        // to what cures - the herb-wife keeps them picked over
        const PATCH_HERBS: [&str; 13] = ["willow_bark", "yarrow", "sage", "willow_bark",
            "yarrow", "sage", "chamomile", "lavender", "willow_bark", "yarrow",
            "peppermint", "mugwort", "feverfew"];
        let in_wall = |x: i32, y: i32| (wall.0..=wall.2).contains(&x) && (wall.1..=wall.3).contains(&y);
        let herb_sites: Vec<(i32, i32)> = (0..map.height as i32)
            .flat_map(|y| (0..map.width as i32).map(move |x| (x, y)))
            // Foraging is a village activity: keep the wild patches within
            // the herb-wife's walking range instead of scattering them over
            // the newly enlarged country.
            .filter(|&(x, y)| (x - vx).abs() + (y - vy).abs() <= 16
                && map.get(x, y) == Some(Tile::Grass) && map.passable(x, y) && !in_wall(x, y))
            .collect();
        let step = (herb_sites.len() / PATCH_HERBS.len().max(1)).max(1);
        for (pi, &herb) in PATCH_HERBS.iter().enumerate() {
            if let Some(&tile) = herb_sites.get(pi * step) {
                herb_patches.push(HerbPatch { tile, herb, next_ready: (pi * 7 % 45) as u64 });
            }
        }

        // the priest keeps by the church (its actual spot, if it moved);
        // a courier rides the road to the rivals' and back; gulls hang
        // about the open sea
        let guards: Vec<(f32, f32)> = map.walls.iter()
            .filter(|&((x, y), piece)| matches!(piece, WallPiece::GateAlongX | WallPiece::GateAlongY)
                && (wall.0..=wall.2).contains(x) && (wall.1..=wall.3).contains(y))
            .map(|&((x, y), piece)| match piece {
                WallPiece::GateAlongX => (x as f32, y as f32 + if y == wall.1 { 0.9 } else { -0.9 }),
                _ => (x as f32 + if x == wall.0 { 0.9 } else { -0.9 }, y as f32),
            }).collect();
        let priest = map.buildings.get(4).map(|&(bx, by)| (bx as f32 + 0.7, by as f32 + 0.6))
            .unwrap_or((vx as f32 + 0.7, vy as f32 - 2.4));
        let traveller_path = map.walk_path((vx, vy), (vx + 13, vy - 2)).unwrap_or_default();
        let mut gulls = vec![];
        for y in 0..map.height as i32 { for x in 0..map.width as i32 {
            if map.get(x, y) == Some(Tile::Water) && noise(x, y, 21) > 0.85 { gulls.push((x as f32, y as f32)); }
        }}
        gulls.truncate(4);

        let mut villagers = vec![];
        // 23 vocation adults (0..23), two overseers (23,24), the trader
        // (25), three children (26..29) and the tax collector (29)
        for i in 0..NAMES.len() {
            let v = villager::spawn_villager(&mut world,
                vx as f32 - 2.0 + (i % 6) as f32 * 0.8, vy as f32 - 2.0 + (i % 4) as f32 * 0.8);
            world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
            world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32 % 6, warmth_episodes: 4 - i as u32 % 5, episodes: vec![] });
            world.add_component(v, crate::sim::psychology::personality_from_seed(i as u64));
            world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
            world.add_component(v, personality_from_seed(i as u64 * 7 + 3));
            world.add_component(v, Satchel::default());
            world.add_component(v, Known::default());
            if i < 23 {
                // a working adult: the level roll sets the skill band
                let skill = match rng.below(100) {
                    r if r < 55 => 0.05 + rng.below(30) as f32 / 100.0,
                    r if r < 85 => 0.40 + rng.below(30) as f32 / 100.0,
                    _ => 0.75 + rng.below(20) as f32 / 100.0,
                };
                world.add_component(v, vocation::craft(adult_vocs[i], skill));
            } else if i < 25 {
                world.add_component(v, TownRole::Overseer);
            } else if i == 25 {
                world.add_component(v, TownRole::Trader);
            } else if i < 29 {
                world.add_component(v, Kid);
                // staggered ages - they come of age one by one, not in a wave
                world.add_component(v, KidGrowth { age_days: (i as u64 * 180) % 700, errands: vec![] });
            } else if i == 29 {
                world.add_component(v, TownRole::TaxCollector);
            } else {
                // the herb-wife: wise to the hedgerow cures, out in the meadow
                world.add_component(v, TownRole::Herbologist);
                world.add_component(v, Apothecary {
                    known_herbs: herbology::medicinal_herbs().iter().map(|(n, _, _)| n.to_string()).collect(),
                    remedies: vec![], skill: 0.55 });
            }
            // a soul to tend: some come devout, some half-hearted
            world.add_component(v, crate::sim::religion::Faith {
                religion: "the parish".into(), devotion: 0.35 + (villagers.len() % 6) as f32 * 0.09,
                traditions: vec![] });
            villagers.push(v);
        }
        // the parish holds them all; the priest keeps the office
        let congregation = {
            let c = world.create_entity();
            world.add_component(c, crate::sim::religion::Congregation {
                cultish: false,
                name: "the village parish".into(), members: villagers.clone(),
                leader: villagers.iter().copied().find(|&v| world.get_component::<Craft>(v)
                    .map(|cr| cr.vocation == Vocation::Priest).unwrap_or(false)),
                orthodoxy: 0.6, services: vec![] });
            // the parish saint's relic: the church's prestige in a box
            world.add_component(c, crate::game::relic::Relic::new(
                "the parish saint's knuckle", 0.6, 0.5));
            c
        };
        // rival village: foreign settlement across the river, with its own
        // granary so its people eat at home instead of crossing to our market
        villager::spawn_food_source(&mut world, (vx + 14) as f32, (vy - 3) as f32, 500);
        let mut rivals = vec![];
        let mut rival_homes = vec![];
        for i in 0..3 {
            let home = ((vx + 12) as f32 + i as f32 * 2.0, (vy - 4) as f32 + (i % 2) as f32 * 3.0);
            let r = villager::spawn_villager(&mut world, home.0, home.1);
            world.add_component(r, Health { condition: 0.9, immunity: 0.5 });
            rivals.push(r);
            rival_homes.push(home);
        }
        // and the foreign houses are theirs: each rival keeps the nearest
        // one across the river, so "home" is a place, not a pull-point
        for (&r, &(hx, hy)) in rivals.iter().zip(&rival_homes) {
            let nearest = building_entities.iter()
                .filter(|&&e| world.get_component::<Building>(e)
                    .map(|b| b.kind == BuildingKind::Home).unwrap_or(false))
                .min_by(|&&a, &&b| {
                    let da = world.get_component::<Pos>(a).map(|p| (p.x - hx).abs() + (p.y - hy).abs()).unwrap_or(99.0);
                    let db = world.get_component::<Pos>(b).map(|p| (p.x - hx).abs() + (p.y - hy).abs()).unwrap_or(99.0);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                }).copied();
            if let Some(b) = nearest {
                world.get_component_mut::<Building>(b).unwrap().occupants.push(r);
            }
        }

        let player = world.create_entity();
        world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), resources: 100.0 });
        world.add_component(player, crate::sim::psychology::AttachmentHistory {
            anxiety_episodes: 0, warmth_episodes: 0, episodes: vec![] });
            world.add_component(player, crate::sim::psychology::personality_from_seed(0xBEEF));
        world.add_component(player, Pos { x: (vx - 6) as f32, y: (vy - 6) as f32 }); // player has a position
        let org = world.create_entity();
        world.add_component(org, Organization { name: "The Harvesters".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into() , successors: vec![] });
        world.add_component(org, crate::sim::organized_power::OrganizedPower::new());
        let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
        pets::adopt(&mut world, dog, villagers[0]);
        let cat = pets::spawn_pet(&mut world, "Mog", Species::Cat, PetRole::Hunting);
        pets::adopt(&mut world, cat, villagers[1]);
        let mut pets = vec![(dog, Species::Dog, (0.8, 0.5)), (cat, Species::Cat, (0.5, 0.3))];

        // ---- settle the town ----
        // the player is just another character: a class (never the
        // landowner), a trade, gold to match, and an inner life
        let pvoc = roll_vocation(&mut rng);
        let pclass = match rng.below(10) { 0..=6 => SocialClass::Working, 7..=8 => SocialClass::PetitBourgeois, _ => SocialClass::ProfessionalMiddle };
        let pvoc = if pclass == SocialClass::ProfessionalMiddle && !vocation::is_pmc(pvoc) {
            if rng.below(2) == 0 { Vocation::Knight } else { Vocation::Priest }
        } else { pvoc };
        let pskill = if pclass == SocialClass::PetitBourgeois { 0.8 } else { 0.1 + rng.below(20) as f32 / 100.0 };
        world.add_component(player, vocation::craft(pvoc, pskill));
        world.add_component(player, Satchel::default());
        // the wanderer hungers like anyone - pressure without lethality:
        // no Health, so the nourishment step never lets it bite
        world.add_component(player, Hunger { value: 100.0 });
        world.add_component(player, Purse { gold: livelihood::seed_gold(pclass, false, (rng.next() % 1000) as f32 / 1000.0) });
        world.add_component(player, Organizer::default());
        {
            let pwants = desires::sample_wants(pclass, Some(pvoc), false, &mut || rng.next());
            world.add_component(player, InnerLife { wants: pwants, grievances: vec![], satisfied: vec![] });
            world.get_component_mut::<PlayerCharacter>(player).unwrap().resources =
                world.get_component::<Purse>(player).unwrap().gold;
        }
        // beds: homes hold 1-4; the unlucky have NoHome
        let homes: Vec<Entity> = building_entities.iter().copied().filter(|&b| {
            world.get_component::<Building>(b)
                // the herb hut is hers alone - nobody else takes a bed in it
                .map(|bb| bb.kind == BuildingKind::Home && bb.name != "a foreign house" && bb.name != "the herb hut").unwrap_or(false)
        }).collect();
        for (i, &v) in villagers.iter().enumerate() {
            if i == HERB_IDX {
                // the herb-wife keeps her own roof, out past the wall
                if let Some(hut) = herb_hut {
                    world.get_component_mut::<Building>(hut).unwrap().occupants.push(v);
                }
                continue;
            }
            let prefer = if KID_IDXS.contains(&i) {
                // a child lodges with the adult a few places before him
                household::home_of(&world, villagers[(i + 20) % 23])
            } else { None };
            let mut placed = false;
            if let Some(h) = prefer {
                if world.get_component::<Building>(h).map(|b| b.occupants.len() < 4).unwrap_or(false) {
                    world.get_component_mut::<Building>(h).unwrap().occupants.push(v);
                    placed = true;
                }
            }
            // farmers and herders take the cottages out by the fields, the
            // gentry the houses inside the wall; whoever's left over
            // (or finds their side full) takes any bed going
            let wants_out = country::prefers_outside(&world, v);
            for pass in 0..2 {
                for k in 0..homes.len().max(1) {
                    if placed { break; }
                    let h = homes[(i * 7 + k) % homes.len()];
                    let inside = world.get_component::<Pos>(h)
                        .map(|p| (wall.0 as f32..=wall.2 as f32).contains(&p.x) && (wall.1 as f32..=wall.3 as f32).contains(&p.y))
                        .unwrap_or(true);
                    if pass == 0 && wants_out.map(|out| out == inside).unwrap_or(false) { continue; }
                    if world.get_component::<Building>(h).map(|b| b.occupants.len() < 4).unwrap_or(false) {
                        world.get_component_mut::<Building>(h).unwrap().occupants.push(v);
                        placed = true;
                    }
                }
            }
            let _ = placed;
        }
        // the player finds a bed too, like anyone else
        for k in 0..homes.len().max(1) {
            let h = homes[(k * 3) % homes.len()];
            if world.get_component::<Building>(h).map(|b| b.occupants.len() < 4).unwrap_or(false) {
                world.get_component_mut::<Building>(h).unwrap().occupants.push(player);
                break;
            }
        }
        // everyone sleeps where they live - standing beside the door (the
        // home tile itself is occupied, and would trap them there); the
        // unhomed get a clear tile off the market square
        for (i, &v) in villagers.iter().enumerate() {
            let stand = household::home_of(&world, v)
                .and_then(|h| world.get_component::<Pos>(h).copied())
                .map(|hp| (hp.x.round() as i32, hp.y.round() as i32))
                .unwrap_or((vx + (i % 3) as i32, vy + 1 + (i % 2) as i32));
            let door = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (stand.0 + dx, stand.1 + dy)))
                .find(|&(x, y)| map.passable(x, y))
                .unwrap_or(stand);
            let p = world.get_component_mut::<Pos>(v).unwrap();
            p.x = door.0 as f32 + ((i % 3) as f32 - 1.0) * 0.3;
            p.y = door.1 as f32 + ((i % 2) as f32) * 0.3;
        }
        // workshops: one worker each; trades with no shop (or more hands
        // than benches) leave folk working the street
        for &v in &villagers {
            let Some(voc) = world.get_component::<Craft>(v).map(|c| c.vocation) else { continue };
            let want = household::kind_for(voc);
            if let Some(b) = building_entities.iter().copied().find(|&b| {
                world.get_component::<Building>(b).map(|bb| bb.kind == want && bb.worker.is_none()).unwrap_or(false)
            }) {
                world.get_component_mut::<Building>(b).unwrap().worker = Some(v);
                world.get_component_mut::<Craft>(v).unwrap().workplace = Some(b);
            }
        }
        // and if the player's trade has a bench free, it's theirs
        if world.get_component::<Craft>(player).unwrap().workplace.is_none() {
            let want = household::kind_for(pvoc);
            if let Some(b) = building_entities.iter().copied().find(|&b| {
                world.get_component::<Building>(b).map(|bb| bb.kind == want && bb.worker.is_none()).unwrap_or(false)
            }) {
                world.get_component_mut::<Building>(b).unwrap().worker = Some(player);
                world.get_component_mut::<Craft>(player).unwrap().workplace = Some(b);
            }
        }
        world.get_component_mut::<Building>(trader_stall).unwrap().worker = Some(villagers[25]);
        // a few carry a difference: the village walks a little slower
        // for them, and theirs check on them
        for (i, kind) in [(7usize, "a lame leg"), (11, "failing sight"), (15, "a mind that needs quiet")] {
            if i < villagers.len() {
                world.add_component(villagers[i], villager::Condition { kind, until: u64::MAX });
            }
        }
        // and the recluse keeps to the cave at the map's edge - the same
        // offset from the village as the classic seed's corner, nudged
        // onto passable ground if the seed's terrain doesn't have it there
        let cave_pos = map.nearest_passable(vx + 20, vy + 17).unwrap_or((vx + 20, vy + 17));
        let hermit_e = villagers.get(22).copied();
        if let Some(h) = hermit_e {
            world.add_component(h, TownRole::Hermit);
            world.add_component(h, Pos { x: cave_pos.0 as f32, y: cave_pos.1 as f32 });
        }
        // the heart-helper: one elder who keeps an eye on lonely hearts
        let matchmaker = villagers.get(27).copied();
        if let Some(mm) = matchmaker { world.add_component(mm, TownRole::Matchmaker); }
        // the shepherd keeps the fold, and a sheepdog keeps him
        let shepherd = villagers.iter().copied().find(|&v|
            world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Shepherd).unwrap_or(false));
        // the beekeeper keeps the apiary, and the skeps stand round it
        let beekeeper = villagers.iter().copied().find(|&v|
            world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Beekeeper).unwrap_or(false));
        let church = Some(building_entities[4]);
        // the tavern: the taverner's own bench, named for what it is.
        // A keeper's shelf starts with a barrel on it so the first
        // thirsty night doesn't find a dry house
        let tavern = building_entities.iter().copied().find(|&b|
            world.get_component::<Building>(b)
                .map(|bb| bb.kind == BuildingKind::Shop(Vocation::Taverner)).unwrap_or(false));
        if let Some(t) = tavern {
            if let Some(b) = world.get_component_mut::<Building>(t) {
                b.name = "the tavern".into();
                household::add_stock(b, "ale", 4);
                household::add_stock(b, "lodging", 2);
            }
        }
        let mut hives = vec![];
        if let (Some(ay), Some((ax, ay2))) = (apiary, apiary_site) {
            for i in 0..4 {
                let (dx, dy) = ((i % 2) as i32 * 2 - 1, (i / 2) as i32 * 2 - 1);
                let (hx, hy) = (1..=4).map(|r| (ax + dx * r, ay2 + dy * r))
                    .find(|&(x, y)| map.passable(x, y)).unwrap_or((ax + 1, ay2 + 1));
                hives.push(apiary::spawn_hive(&mut world, if i % 2 == 0 { HiveKind::Skep } else { HiveKind::Log }, ay, hx as f32 + 0.5, hy as f32 + 0.5));
            }
        }
        if let Some(sh) = shepherd {
            let d = pets::spawn_pet(&mut world, "the sheepdog", Species::Dog, PetRole::Herding);
            world.get_component_mut::<Pet>(d).unwrap().owner = Some(sh);
            pets.push((d, Species::Dog, (0.7, 0.5)));
        }
        // the flock: sheep in the main, a few goats, out round the fold
        let mut flock: Vec<Entity> = vec![];
        if let (Some(fold), Some((fx, fy))) = (fold, fold_site) {
            for a in 0..10 {
                let kind = if a < 7 { LivestockKind::Sheep } else { LivestockKind::Goat };
                let (dx, dy) = ((a % 4) as i32 - 1, (a / 4) as i32 - 1);
                let (ax, ay) = (1..=6).map(|r| (fx + dx * r, fy + dy * r))
                    .find(|&(x, y)| map.passable(x, y)).unwrap_or((fx + 1, fy + 1));
                flock.push(livestock::spawn_animal(&mut world, kind, fold, ax as f32 + 0.35, ay as f32 + 0.45));
            }
        }
        let graze_at = fold.and_then(|f| world.get_component::<Pos>(f).map(|p| (p.x, p.y))).unwrap_or((0.0, 0.0));
        // mushroom patches, each kind true to its ground: the wood's shade
        // keeps the edibles and their death-traps side by side, the pasture
        // keeps the little nipple-tipped caps (and the little brown killers
        // beside them), the fly agaric stays under birch, and the best
        // mushrooms grow in the under
        let mut mush_patches: Vec<MushPatch> = vec![];
        let wood_kinds = [MushKind::Chanterelle, MushKind::DeathCap, MushKind::FieldMushroom,
                          MushKind::JackOLantern, MushKind::Galerina, MushKind::Chanterelle];
        for (mi, &(tx, ty)) in map.trees.iter().step_by(7).take(6).enumerate() {
            if let Some(t) = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (tx + dx, ty + dy)))
                .find(|&(x, y)| map.passable(x, y)) {
                mush_patches.push(MushPatch { tile: t, kind: wood_kinds[mi % wood_kinds.len()], next_ready: (mi * 11) as u64 });
            }
        }
        // the fly agaric keeps to birch: a patch off a deeper tree
        if let Some(&(tx, ty)) = map.trees.iter().nth(11) {
            if let Some(t) = (-3..=3).flat_map(|dy| (-3..=3).map(move |dx| (tx + dx, ty + dy)))
                .find(|&(x, y)| map.passable(x, y)) {
                mush_patches.push(MushPatch { tile: t, kind: MushKind::FlyAgaric, next_ready: 30 });
            }
        }
        // liberty caps in the wet grassland by the fold - the shepherd's
        // secret - with a galerina a tile off: the little brown mushroom
        // that looks just like them is the one that kills
        let patch_near = |mush_patches: &mut Vec<MushPatch>, map: &TileMap,
                              (cx, cy): (i32, i32), kind: MushKind, ready: u64| {
            if let Some(t) = (-4..=4).flat_map(|dy| (-4..=4).map(move |dx| (cx + dx, cy + dy)))
                .find(|&(x, y)| map.passable(x, y) && !mush_patches.iter().any(|p| p.tile == (x, y))) {
                mush_patches.push(MushPatch { tile: t, kind, next_ready: ready });
            }
        };
        let graze_tile = (graze_at.0.round() as i32, graze_at.1.round() as i32);
        patch_near(&mut mush_patches, &map, graze_tile, MushKind::LibertyCap, 40);
        if let Some(lc) = mush_patches.iter().find(|p| p.kind == MushKind::LibertyCap).map(|p| p.tile) {
            patch_near(&mut mush_patches, &map, (lc.0 + 1, lc.1 + 1), MushKind::Galerina, 45);
        }
        // the cave's deep patch: the best mushrooms grow in the under -
        // pushed last so the wood's poisoned patches keep their numbers
        let undercap_tile = map.nearest_passable(cave_pos.0 - 1, cave_pos.1).unwrap_or((cave_pos.0 - 1, cave_pos.1));
        mush_patches.push(MushPatch { tile: undercap_tile, kind: MushKind::UnderCap, next_ready: 0 });
        // hens scratch about a few of the cottages' yards
        let mut hens = vec![];
        let mut hen_houses: Vec<Entity> = vec![];
        for &h in homes.iter().step_by(5).take(4) {
            let Some(hp) = world.get_component::<Pos>(h).copied() else { continue };
            hen_houses.push(h);
            for a in 0..3 {
                let (ax, ay) = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (hp.x.round() as i32 + dx + a % 2, hp.y.round() as i32 + dy)))
                    .find(|&(x, y)| map.passable(x, y)).unwrap_or((hp.x.round() as i32 + 1, hp.y.round() as i32 + 1));
                hens.push(livestock::spawn_animal(&mut world, LivestockKind::Hen, h, ax as f32 + 0.4, ay as f32 + 0.5));
            }
        }
        // an overseer drives each farm and mason's yard
        let mut oi = 23usize;
        for &b in &building_entities {
            if oi >= 25 { break; }
            let drives = world.get_component::<Building>(b)
                .map(|bb| matches!(bb.kind, BuildingKind::Shop(Vocation::Farmer | Vocation::Mason))).unwrap_or(false);
            if drives {
                world.get_component_mut::<Building>(b).unwrap().overseer = Some(villagers[oi]);
                oi += 1;
            }
        }
        // masters pair juniors: half the journeymen/apprentices find a
        // same-vocation master to learn from
        for i in 0..23 {
            let v = villagers[i];
            let (voc, skill) = match world.get_component::<Craft>(v) { Some(c) => (c.vocation, c.skill), None => continue };
            if vocation::level_of(skill) == Level::Master || rng.below(100) >= 50 { continue; }
            let master = villagers[..23].iter().copied().find(|&m| {
                m != v && world.get_component::<Craft>(m)
                    .map(|c| c.vocation == voc && vocation::level_of(c.skill) == Level::Master).unwrap_or(false)
            });
            if let Some(m) = master {
                world.get_component_mut::<Craft>(v).unwrap().master = Some(m);
                relationships::add_relationship(&mut world, relationships::Relationship {
                    from: v, other: m, kind: relationships::RelKind::Apprentice, strength: 0.4, interactions: 3 });
            }
        }
        // purses and inner lives now that classes are settled
        let mut g = Rng(seed ^ 0xA11CE);
        for (i, &v) in villagers.iter().enumerate() {
            let class = desires::class_of(&world, v);
            let is_kid = KID_IDXS.contains(&i);
            let gold = livelihood::seed_gold(class, is_kid, (g.next() % 1000) as f32 / 1000.0);
            world.add_component(v, Purse { gold });
            // the years they're carrying - a kid's grow from the days
            // already on their growth-clock - and the span they're given
            let years = if is_kid {
                world.get_component::<KidGrowth>(v).map(|kg| kg.age_days as f32 / 52.0).unwrap_or(5.0)
            } else { 19.0 + (g.next() % 42) as f32 };
            world.add_component(v, crate::game::lifecycle::Age {
                years, stage: crate::game::lifecycle::stage_for(years),
                born_day: (v as u16) % 360 });
            // and where their soul stands on Fowler's scale - a seeded
            // draw weighted by the years they're carrying
            world.add_component(v, crate::sim::religion::FaithStage::for_years(years,
                crate::sim::religion::stage_roll(v, 23)));
            world.add_component(v, crate::game::lifecycle::Vitality {
                health: 1.0, max_age: 58.0 + (g.next() % 35) as f32, cause_of_death: None });
            let voc = world.get_component::<Craft>(v).map(|c| c.vocation);
            let wants = desires::sample_wants(class, voc, is_kid, &mut || g.next());
            world.add_component(v, InnerLife { wants, grievances: vec![], satisfied: vec![] });
        }
        // the traders the guild forms of: the trade-vocation folk and
        // the stall's trader - purses seeded now, their stake the purse
        // plus the stock and tools the writs can strip
        for (i, &v) in villagers.iter().enumerate() {
            if subsystems::society::is_trader(&world, v) {
                subsystems::society::tag_trader(&mut world, v, vill_name(i));
            }
        }
        // the keeper opens with a float. His trade is the only one in the
        // village that buys before it sells, so a journeyman's purse
        // deadlocks him on day one: no coin, no barrel; no barrel, no
        // takings; no takings, no coin. A taverner's stock-in-trade is
        // capital, and this is it
        if let Some(k) = tavern.and_then(|t| world.get_component::<Building>(t)).and_then(|b| b.worker) {
            if let Some(p) = world.get_component_mut::<Purse>(k) {
                p.gold = p.gold.max(TAVERN_FLOAT);
            }
        }
        // the dark keeps the hermit - their one big want, now that they
        // have an inner life to hold it in
        if let Some(h) = hermit_e {
            desires::add_grievance(&mut world, h, desires::Problem::TheDark, u64::MAX);
        }
        // the player's coins and their Purse are the same purse - see
        // sync_purse, called at the end of every day's business
        sync_purse(&mut world, player, true);
        // the collector starts his yearly round in a few weeks
        let collector = villagers[COLLECTOR_IDX];
        let herbologist = villagers[HERB_IDX];
        {
            let mut roster: Vec<Entity> = villagers.clone();
            roster.push(player);
            livelihood::tax_round(&mut world, collector, &roster, 25, 11);
        }

        let mut g = Game {
            world, sched, map, heights, flood: crate::game::flood::Flood::new(),
            climate, music, weather, congregation, villagers, rivals,
            splinter: None, reformation_spark: None,
            seats: vec![Seat { player, org, ..Seat::default() }], pets, market_food, buildings, stones,
            town_wall: wall,
            village: (vx, vy),
            mill: mill_pos,
            guards,
            posted: vec![],
            priest,
            traveller_path,
            gulls,
            chicken_at: (gate_x as f32 + 0.4, farm_y as f32 + 0.3),
            rng,
            seed,
            built_count: 0,
            works: vec![],
            tech: crate::game::tech_tree::extended_printmaking_tree(),
            flyers: vec![],
            rival_homes,
            toasts: Toasts::default(),
            last_guild_count: 0,
            tick: 0,
            paused: false,
            speed: 1,
            log: {
                let mut l = gamelog::GameLog::default();
                l.push("Welcome to the village.".into());
                l.push(format!("The village keeps {} - and the tavern on the green, where it all gets drunk.",
                    vocation::trade_name(drink_voc)));
                if let Some(n) = &landform {
                    l.push(format!("The village makes its home in {}.", n));
                }
                l
            },
            log_dropped: 0,
            bubbles: vec![],
            meeting: None,
            snowball_day: u64::MAX,
            march_day: u64::MAX,
            cadence_day: u64::MAX,
            townhall_day: u64::MAX,
            petition_day: u64::MAX,
            assembly_day: u64::MAX,
            snowball_cued: false,
            snowball_at: (0.0, 0.0),
            snowballs: vec![],
            near: vec![],
            last_chat: vec![],
            talk_log: vec![],
            talk_next_id: 0,
            was_sick: vec![None; NAMES.len()],
            died_on: vec![None; NAMES.len()],
            graveyard: None,
            frame_n: 0,
            precip: None,
            rainbow_from: None,
            building_entities,
            collector: Some(collector),
            trader_stall: Some(trader_stall),
            travels: vec![],
            minstrel: None,
            // one early visit's owed to every kind - then the long game
            next_minstrel: 60 + seed % 30,
            pedlar: None,
            next_pedlar: 60 + seed % 60,
            circus: None,
            next_circus: 60 + (seed / 3) % 30,
            ink_boiler: None,
            next_ink_boiler: 60 + (seed / 9) % 30,
            visit_misses: 0,
            fire: Fire::new(),
            fire_crew: vec![],
            fire_spent: Default::default(),
            fire_banks: None,
            fire_line_called: false,
            brigade: None,
            brigade_gear: 0,
            cairns: vec![],
            monuments: vec![],
            imperial_project: None,
            doctor: None,
            stocks_tile: None,
            lord_works: vec![],
            attended_days: vec![],
            journal: crate::session::journal::Journal::new(seed),
            notices: vec![],
            dilemma: None, dilemma_debts: vec![],
            boats: vec![(10, 5), (20, 8)],
            construction_sites: vec![],
            law_book: crate::game::laws::LawBook::new(),
            sites: vec![],
            rebel_camps: vec![],
            rebel_stores: vec![],
            village_gaol: None,
            castle_gaol: None,
            suffrage_won: false,
            ordinance: None,
            petition: None,
            derelict: Default::default(),
            ruined: Default::default(),
            cutaways: Default::default(),
            voiced: vec![],
            herbologist: Some(herbologist),
            herb_hut,
            herb_patches,
            remedy_visits: vec![],
            fold,
            shepherd,
            flock,
            graze_at,
            flock_wander: crate::engine::det::HashMap::default(),
            pet_cool: crate::engine::det::HashMap::default(),
            formation: None,
            cells: vec![],
            cell_mail: vec![],
            school: None,
            teacher: None,
            post_offices: vec![],
            couriers: vec![],
            next_graze: 0,
            hens,
            hen_houses,
            matchmaker,
            match_tried: vec![],
            courtships: vec![],
            mush_patches,
            shroom_breaks: vec![],
            shroom_lore: vec![],
            ailments: vec![],
            shroom_rng: Rng(seed ^ 0xCA45),
            forest_rng: Rng(seed ^ 0xF12E),
            plague_rng: Rng(seed ^ 0x9A6E),
            shroom_disclaimed: false,
            tavern,
            tavern_table: vec![],
            apiary,
            beekeeper,
            hives,
            church,
            swarm: None,
            succeeded: vec![],
            trails: Default::default(),
            land,
            pit,
            reach,
            sanctioned: vec![],
            protests: vec![],
            feud_told: Default::default(),
            produced: Default::default(),
            tales: Default::default(),
            forest: {
                let mut f = crate::sim::emergence::Automata::new(24, 24);
                f.seed();
                f
            },
            grimoire: crate::sim::magic::Grimoire { discoveries: vec![],
                domains_known: Default::default(), total_power: 0.0 },
            tales_told: Default::default(),
            cascade_tale: vec![],
            cascade_named_at: None,
            cascade_levels: 0,
            dead_since: Default::default(),
            crop_noted: false,
            crop_fields: vec![],
            harvest_year: u64::MAX,
            landform,
            shop_flow: Default::default(),
            wolf_coming: false,
            takings: 0.0,
            fox_coming: false,
            gardens,
            cellar,
            cheeses: vec![],
            cellar_trait,
            lord_esteem: 0.0,
            numeracy: if seed % 10 < 6 { Numeracy::Tally } else { Numeracy::RomanBoard },
            translator: None,
            translator_days: 0,
            hermit: hermit_e,
            reckonings: 0,
            sky: None,
            pending_discovery: None,
            disclosure_made: false,
            cured: 0,
            repression_debt: 0,
            harsh_winter: false,
            painted: vec![],
            heirlooms: vec![],
            letters: vec![],
            cave_pos: Some(cave_pos),
            cave_zone: Some(((cave_pos.0 - 3, cave_pos.1 - 3), (cave_pos.0 + 3, cave_pos.1 + 3))),
            seat_offer: None,
            plant: None,
            proposal: None,
            emergency: None,
            witnesses: 0,
            counter_intel: 0,
            suppression_arc: 0,
            player_passed: false,
            martyr_grave: None,
            path_cache: crate::engine::det::HashMap::default(),
            villager_poses: crate::engine::det::HashMap::default(),
            path_tick: u64::MAX,
            sfx: vec![],
            dungeon: vec![],
            rare: vec![],
            next_rare: events::initial_due(seed),
            rare_fired: vec![],
            rare_seed: seed ^ 0xAA55_5EED,
            pending_quests: vec![],
            quest_rest: Default::default(),
            intro_victim: None,
        };
        // and the country's folk, out in its towns and castles
        g.settle_country(&country_plan, seed);
        // and the woods' bands, if the seed dealt any
        g.settle_rebels(seed);
        // one villager marked by a seeded injustice - the Wanderer's
        // first job is to hear them out
        g.pick_intro_victim();
        g
    }

    /// A village already mid-story: one of the saved scenarios - the
    /// fever year, the circus winter, the widow's shop
    pub fn new_scenario(s: Scenario) -> Game {
        let mut g = Game::new_seeded(MAP_SEED);
        match s {
            Scenario::FeverYear => {
                // the scarlatina's already in three beds when you arrive
                for (i, &v) in g.villagers.iter().take(3).enumerate() {
                    let d = g.world.create_entity();
                    g.world.add_component(d,
                        Disease::new(DiseaseKind::ScarletFever, v, 0.6 - i as f32 * 0.1));
                }
                g.log.push(format!("t{}: the scarlatina year - it's already in three beds.", g.tick));
            }
            Scenario::CircusWinter => {
                // a hard winter, and the tents already pitched outside
                g.tick = 400; // inside the harsh window
                g.spawn_circus();
                g.log.push("t400: the circus winter - wonders, and thin stores.".to_string());
            }
            Scenario::WidowsShop => {
                // a shop stands derelict, a widow weighs the keeping of it
                let b = g.building_entities.iter().copied().find(|&b|
                    g.world.get_component::<Building>(b)
                        .map(|bb| bb.worker.is_some() && matches!(bb.kind, household::BuildingKind::Shop(_)))
                        .unwrap_or(false));
                if let Some(b) = b {
                    if let Some(w) = g.world.get_component::<Building>(b).and_then(|bb| bb.worker) {
                        g.world.add_component(w, villager::Dead);
                        g.succession_tick(1);
                        g.log.push("t1: the widow's shop - who's to keep it now?".to_string());
                    }
                }
            }
        }
        g
    }
}

/// Where this seed's village stands. The classic map keeps its corner
/// spot so tests and tooling stay put; every other seed roams the middle
/// country for dry ground the whole cluster - wall, fields, the rivals'
/// bank - fits inside, on land the country plan can walk back to.
fn village_origin(t: &Terrain, seed: u64) -> (i32, i32) {
    if seed == MAP_SEED { return (16, 16); }
    let n = t.width as i32;
    let mut rng = Rng(seed ^ 0x71D6_9E5D);
    for _ in 0..800 {
        let (vx, vy) = (n / 4 + rng.below(n / 2), n / 4 + rng.below(n / 2));
        if !village_ground(t, vx, vy) { continue; }
        // the settlement pockets dry the village out, but a pocket cut off
        // from the mainland is an islet the map's sinker would drown - and
        // the country plan would find nowhere to put its towns
        let land = country::reachable_land(t, (vx, vy));
        if land.iter().filter(|&&l| l).count() < 20_000 { continue; }
        return (vx, vy);
    }
    (n / 2, n / 2)
}

/// Whether the country round (vx, vy) is dry enough for the village
/// cluster: sampled over the fields and the rivals' bank, well past the
/// walls. The settlement pockets flatten the core either way - what must
/// already be dry is the ground the roads and the country walk over.
fn village_ground(t: &Terrain, vx: i32, vy: i32) -> bool {
    use crate::game::tilemap::WATER_LEVEL;
    let n = t.width as i32;
    let (x0, x1, y0, y1) = (vx - 12, vx + 20, vy - 12, vy + 24);
    if x0 < 4 || y0 < 4 || x1 >= n - 4 || y1 >= n - 4 { return false; }
    let (mut dry, mut total) = (0, 0);
    for y in (y0..=y1).step_by(3) {
        for x in (x0..=x1).step_by(3) {
            total += 1;
            if t.get(x as usize, y as usize) >= WATER_LEVEL + 0.02 { dry += 1; }
        }
    }
    dry * 10 >= total * 8
}
