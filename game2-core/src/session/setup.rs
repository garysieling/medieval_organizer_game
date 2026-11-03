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

        let mut terrain = Terrain::generate_country(MAP_TILES, 0.7, seed);
        let _ = crate::game::landforms::shape_terrain(&mut terrain, seed);
        let climate = climate::spawn_climate(&mut world);
        // the felt-worth ledger rides beside the climate - what the
        // shelves show leans what a swap is worth, quietly
        goods::spawn_worths(&mut world);
        // tell the sim where the tile edge is, or its fixed bound pinches
        // anyone homed past the old 33-tile map
        world.add_component(climate, villager::WorldBounds { w: MAP_TILES as f32, h: MAP_TILES as f32 });
        let market_food = villager::spawn_food_source(&mut world, 16.0, 16.0, 500);

        // the river winds south between the towns to the sea; each waypoint
        // on its course drifts a tile or two, so it meanders differently
        // every map (kept east of the wall and west of the rivals')
        let river: Vec<(i32, i32)> = [3, 9, 15, 21, 27, 33, 39, 45].iter()
            .map(|&y| (24 + rng.below(3), y)).collect();
        // the castle suits the river: the east face keeps a tile clear of
        // its west bank and the rest of the rectangle breathes a tile
        // either way (every building still sits well inside). Only the
        // waypoints level with the wall set it - the castle doesn't grow
        // just because the river now runs further south
        let wall = (8 + rng.around(1), 7 + rng.around(1),
            river.iter().filter(|w| w.1 <= 30).map(|w| w.0).min().unwrap() - 1, 24 + rng.around(1));
        // the walled fields run out of the south gate to a little farm
        let (gate_x, farm_y) = ((wall.0 + wall.2) / 2, wall.3 + 2);

        // fields beside the food store, and strips either side of the lane
        // that runs out of the south gate, down through the farm - most of
        // the land the map grew is worked
        let mut fields: Vec<(i32, i32, Tile)> = vec![
            (13, 14, Tile::Wheat), (14, 14, Tile::Wheat), (13, 15, Tile::Wheat), (14, 15, Tile::Wheat),
            (17, 14, Tile::Corn), (18, 14, Tile::Corn), (18, 15, Tile::Corn),
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
            // homes around the market (the first three have chimney smoke)
            ((13, 12), BSprite::House(0)), ((19, 12), BSprite::House(1)), ((12, 18), BSprite::House(2)),
            ((19, 19), BSprite::Blacksmith),
            // church (religion centre) + guild hall (merchant base)
            ((14, 9), BSprite::Church), ((17, 10), BSprite::Guildhall),
            // rival village across the river (red-roofed foreign houses)
            ((28, 13), BSprite::RivalHouse), ((30, 15), BSprite::RivalWoodhouse),
        ];
        let mut herb_patches: Vec<HerbPatch> = vec![];
        let mut layout = Layout {
            // our village and the rivals' each stand on dry land, the neck
            // between them never floods, and the farm below the south gate
            // gets a broad pocket of level field
            settlements: vec![(16, 16, 9), (29, 14, 4), (25, 15, 3), (gate_x, farm_y + 4, 6)],
            river: river.clone(),
            bridge_near: (river[2].0, 15),
            town_wall: Some(wall),
            fields,
            buildings: planned.iter().map(|(p, _)| *p).collect(),
            road_hub: (16, 16),
            // the lane out of the south gate runs down through the strips
            // to the farmstead, beside the road to the rivals' village
            road_ends: vec![(29, 14), (gate_x, farm_y + 8)],
            // two remote keeps and two satellite towns make the enlarged
            // country read as a settled region while the main village keeps
            // its existing simulation and political systems.
            castles: vec![(126, 58, 5), (181, 164, 5)],
            towns: vec![(92, 48, 8), (190, 111, 8)],
            settlement_houses: [
                (89, 46), (92, 44), (95, 46), (89, 50), (95, 50),
                (187, 109), (190, 107), (193, 109), (187, 113), (193, 113),
            ].to_vec(),
        };
        layout.buildings.push((MILL_TILE.0 as i32, MILL_TILE.1 as i32));
        let mut map = TileMap::generate(&mut terrain, &layout);
        let mut buildings: Vec<(f32, f32, BSprite)> = planned.iter().zip(&map.buildings)
            .map(|((_, s), (x, y))| (*x as f32, *y as f32, *s)).collect();

        // out-of-town townage: a farmhand's cottage by the south fields and
        // a smallholder's at the far end of the strips, a woodcutter's
        // clearing at the woods' edge, a fisher's hut on the shore - each
        // gets its road run out to it
        for (site, sprite) in [
            (Some((gate_x + 2, farm_y)), BSprite::House(6)),
            (Some((gate_x - 3, farm_y + 9)), BSprite::House(9)),
            (shore_site(&map, (26, 30)), BSprite::House(7)),
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
        for gy in (wall.1 + 3..wall.3 - 1).step_by(3) {
            for gx in (wall.0 + 3..wall.2 - 1).step_by(3) {
                if let Some((bx, by)) = map.nearest_buildable(gx + rng.around(1), gy + rng.around(1)) {
                    map.build_at(bx, by);
                    map.connect((bx, by));
                    buildings.push((bx as f32, by as f32, BSprite::House(rng.below(97) as usize)));
                }
            }
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
        for kind in shop_kinds.iter().copied() {
            // near the market for shops; farms by the south fields; the
            // fish-smoke house goes down to the water's edge
            let (cx, cy) = if kind == BuildingKind::Shop(Vocation::Farmer) { (gate_x + 2, farm_y + 1) }
                else if kind == BuildingKind::Shop(Vocation::Fisher) { (35, 18) }
                else { (16 + rng.around(4), 15 + rng.around(4)) };
            let Some((bx, by)) = map.nearest_buildable(cx, cy) else { continue };
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
        // the trader's stall is the guildhall slot (index 5), stocked with imports
        let trader_stall = building_entities[5];
        for g in ["cloth", "paper", "ink", "pigment"] {
            world.get_component_mut::<Building>(trader_stall).unwrap().stock.insert(g.to_string(), 4);
        }

        let stones: Vec<(i32, i32)> = [(6, 19), (20, 23), (34, 30), (14, 38), (42, 18)].iter()
            .filter_map(|&(x, y)| map.nearest_buildable(x, y)).collect();
        for &(x, y) in &stones { map.occupy(x, y); }

        // the land keeps score: canopy a little thinned already, the pit
        // already wet, the reach already carrying the town's leavings
        let mut land = crate::sim::environment::LandScore {
            tree_cover: map.trees.iter().map(|&t| (t, 0.85f32)).collect(),
            pit_water: 0.5, effluent: Default::default(),
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
            .filter(|&(x, y)| (x - 16).abs() + (y - 16).abs() <= 16
                && map.get(x, y) == Some(Tile::Grass) && map.passable(x, y) && !in_wall(x, y))
            .collect();
        let step = (herb_sites.len() / PATCH_HERBS.len().max(1)).max(1);
        for (pi, &herb) in PATCH_HERBS.iter().enumerate() {
            if let Some(&tile) = herb_sites.get(pi * step) {
                herb_patches.push(HerbPatch { tile, herb, next_ready: (pi * 7 % 45) as u64 });
            }
        }

        // the watch stands just inside each gate; the priest keeps by the
        // church; a courier rides the road to the rivals' and back; gulls
        // hang about the open sea
        let guards: Vec<(f32, f32)> = map.walls.iter()
            .filter(|&((x, y), piece)| matches!(piece, WallPiece::GateAlongX | WallPiece::GateAlongY)
                && (wall.0..=wall.2).contains(x) && (wall.1..=wall.3).contains(y))
            .map(|&((x, y), piece)| match piece {
                WallPiece::GateAlongX => (x as f32, y as f32 + if y == wall.1 { 0.9 } else { -0.9 }),
                _ => (x as f32 + if x == wall.0 { 0.9 } else { -0.9 }, y as f32),
            }).collect();
        let priest = (14.7, 9.6);
        let traveller_path = map.walk_path((16, 16), (29, 14)).unwrap_or_default();
        let mut gulls = vec![];
        for y in 0..map.height as i32 { for x in 0..map.width as i32 {
            if map.get(x, y) == Some(Tile::Water) && noise(x, y, 21) > 0.85 { gulls.push((x as f32, y as f32)); }
        }}
        gulls.truncate(4);

        let mut villagers = vec![];
        // 23 vocation adults (0..23), two overseers (23,24), the trader
        // (25), three children (26..29) and the tax collector (29)
        for i in 0..NAMES.len() {
            let v = villager::spawn_villager(&mut world, 14.0 + (i % 6) as f32 * 0.8, 14.0 + (i % 4) as f32 * 0.8);
            world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
            world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32 % 6, warmth_episodes: 4 - i as u32 % 5, episodes: vec![] });
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
            villagers.push(v);
        }
        for i in 0..3 {
            let m = world.create_entity();
            world.add_component(m, Merchant { name: format!("Merchant{}",i), wealth: 60.0, trade_routes: vec!["north_road".into()], vulnerable: i < 2 });
        }

        // rival village: foreign settlement across the river, with its own
        // granary so its people eat at home instead of crossing to our market
        villager::spawn_food_source(&mut world, 30.0, 13.0, 500);
        let mut rivals = vec![];
        let mut rival_homes = vec![];
        for i in 0..3 {
            let home = (28.0 + i as f32 * 2.0, 12.0 + (i % 2) as f32 * 3.0);
            let r = villager::spawn_villager(&mut world, home.0, home.1);
            world.add_component(r, Health { condition: 0.9, immunity: 0.5 });
            rivals.push(r);
            rival_homes.push(home);
        }

        let player = world.create_entity();
        world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        world.add_component(player, Pos { x: 10.0, y: 10.0 }); // player has a position
        let org = world.create_entity();
        world.add_component(org, Organization { name: "The Harvesters".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into() });
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
            for k in 0..homes.len().max(1) {
                if placed { break; }
                let h = homes[(i * 7 + k) % homes.len()];
                if world.get_component::<Building>(h).map(|b| b.occupants.len() < 4).unwrap_or(false) {
                    world.get_component_mut::<Building>(h).unwrap().occupants.push(v);
                    placed = true;
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
                .unwrap_or((16 + (i % 3) as i32, 17 + (i % 2) as i32));
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
                world.add_component(villagers[i], villager::Condition { kind });
            }
        }
        // and the recluse keeps to the cave at the map's edge
        let hermit_e = villagers.get(22).copied();
        if let Some(h) = hermit_e {
            world.add_component(h, TownRole::Hermit);
            world.add_component(h, Pos { x: 36.0, y: 33.0 });
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
        mush_patches.push(MushPatch { tile: (35, 33), kind: MushKind::UnderCap, next_ready: 0 });
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
            let voc = world.get_component::<Craft>(v).map(|c| c.vocation);
            let wants = desires::sample_wants(class, voc, is_kid, &mut || g.next());
            world.add_component(v, InnerLife { wants, grievances: vec![], satisfied: vec![] });
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

        Game {
            world, sched, map, climate, villagers, rivals,
            seats: vec![Seat { player, org, ..Seat::default() }], pets, market_food, buildings, stones,
            town_wall: wall,
            guards,
            posted: vec![],
            priest,
            traveller_path,
            gulls,
            chicken_at: (gate_x as f32 + 0.4, farm_y as f32 + 0.3),
            rng,
            built_count: 0,
            leader_named: false,
            works: vec![],
            rival_homes,
            toasts: vec![],
            last_guild_count: 0,
            tick: 0,
            paused: false,
            speed: 1,
            log: vec!["Welcome to the village.".into(),
                format!("The village keeps {} - and the tavern on the green, where it all gets drunk.",
                    vocation::trade_name(drink_voc))],
            log_dropped: 0,
            bubbles: vec![],
            meeting: None,
            near: vec![],
            last_chat: vec![],
            talk_log: vec![],
            talk_next_id: 0,
            was_sick: vec![None; NAMES.len()],
            died_on: vec![None; NAMES.len()],
            frame_n: 0,
            precip: None,
            rainbow_from: None,
            building_entities,
            collector: Some(collector),
            trader_stall: Some(trader_stall),
            travels: vec![],
            minstrel: None,
            next_minstrel: 250 + seed % 200,
            pedlar: None,
            next_pedlar: 60 + seed % 60,
            circus: None,
            next_circus: 300 + seed % 180,
            ink_boiler: None,
            next_ink_boiler: 400 + seed % 200,
            fire: Fire::new(),
            derelict: Default::default(),
            voiced: vec![],
            herbologist: Some(herbologist),
            herb_hut,
            herb_patches,
            remedy_visits: vec![],
            fold,
            shepherd,
            flock,
            graze_at,
            flock_wander: std::collections::HashMap::new(),
            next_graze: 0,
            hens,
            hen_houses,
            matchmaker,
            match_tried: vec![],
            mush_patches,
            shroom_breaks: vec![],
            shroom_lore: vec![],
            ailments: vec![],
            shroom_rng: Rng(seed ^ 0xCA45),
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
            cave_pos: Some((36, 33)),
            player_passed: false,
            martyr_grave: None,
            path_cache: std::collections::HashMap::new(),
            villager_poses: std::collections::HashMap::new(),
            path_tick: u64::MAX,
            sfx: vec![],
            dungeon: vec![],
            rare: vec![],
            next_rare: events::initial_due(seed),
            rare_fired: vec![],
            rare_seed: seed ^ 0xAA55_5EED,
        }
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
