use crate::session::*;

#[test]
fn test_generated_ground_set_and_villager_walks_load() {
    let art = Art::load_fs("../assets", 0);
    // green seasons: every road and river piece generated, in the pack's
    // geometry (so BLOCK_FACE still fits); winter is still the pack's
    for season in [0, 1] {
        let t = &art.tiles[season];
        assert_eq!((t.roads.len(), t.rivers.len()), (15, 17), "season {}: full road and river sets", season);
        let g = t.grass.as_ref().unwrap();
        assert_eq!((g.width, g.height), (95, 97), "the composed grass block keeps the pack's size");
    }
    // winter: snow, roads, rivers and ice all generated too
    let w = &art.tiles[2];
    assert_eq!((w.roads.len(), w.rivers.len(), w.rivers_ice.len()), (15, 17, 17));
    // crop fields: both growth stages of wheat and corn
    assert_eq!((art.tiles[0].wheat.len(), art.tiles[0].corn.len()), (2, 2));
    // every villager look, the watch and the priest walk; the street is
    // the generated cottages, with snowy versions for winter
    assert!(art.villager_walks.iter().all(|w| w.is_some()), "every villager look has a walk sheet");
    assert!(art.guard_walks.len() == 4 && art.guard_walks.iter().all(|w| w.is_some()));
    assert!(art.priest_walk.is_some());
    assert_eq!(art.houses.len(), 9);
    assert_eq!(art.houses_winter.len(), art.houses.len());
    let ws = art.villager_walks[0].as_ref().unwrap();
    assert_eq!((ws.frames.len(), ws.frames[0].len()), (4, 4));
    assert!((28..=40).contains(&(ws.feet - ws.head)), "a grown-up is ~35px tall, not {}", ws.feet - ws.head);
}

#[test]
fn test_cottages_stand_flush_on_their_tile() {
    // the generated cottages came on a grass slab with an earth edge;
    // with the edge on, the slab (and house) floated above the ground.
    // Exported with rd_slice --flush-base, a slab's bottom row is its
    // grass (the stone-foundation cottage has no slab: left as it was)
    let art = Art::load_fs("../assets", 0);
    let green_bottom = art.houses.iter().flatten().filter(|s| {
        let y = s.height - 1;
        (0..s.width).any(|x| { let p = &s.pixels[(y * s.width + x) * 4..][..4]; p[3] > 128 && p[1] > p[0] && p[1] >= p[2] })
    }).count();
    assert!(green_bottom >= 8, "only {} of {} cottages end in grass", green_bottom, art.houses.len());
}

#[test]
fn test_generated_props_and_castle_load_at_1x() {
    let art = Art::load_fs("../assets", 0);
    // woods: broadleaf and fir in the green seasons and autumn, bare and
    // snowed-on in winter, all drawn at 1x (not the pack's block scale)
    for (season, n) in [(0, 8), (1, 8), (2, 4)] {
        let t = &art.tiles[season];
        assert_eq!(t.trees.len(), n, "season {}: generated trees", season);
        assert_eq!(t.trees_k, 1.0);
        assert!(t.trees.iter().all(|s| (40..=62).contains(&s.height)), "a tree is about a cottage tall");
    }
    assert_eq!((art.stones.len(), art.stone_grounds.len(), art.stones_k), (12, 6, 1.0));
    // the town wall is generated as a set, the church too
    assert!(art.castle_rd);
    assert!(art.walls.iter().chain(&art.wall_posts).all(|s| s.is_some()) && art.tower_s.is_some());
    assert!(art.church_s.as_ref().is_some_and(|c| c.width < 70), "the generated church, not the pack's keep");
    // the wall pieces stand on the map's own ground: no grass slab left
    // on their bottom rows to show green on snow or rock
    let greenish = |s: &Sprite| (s.height - 3..s.height).flat_map(|y| (0..s.width).map(move |x| (x, y)))
        .filter(|&(x, y)| { let p = &s.pixels[(y * s.width + x) * 4..][..4]; p[3] > 128 && p[1] > p[0] + 8 && p[1] > p[2] + 4 }).count();
    for s in art.walls.iter().chain(&art.wall_posts).chain([&art.tower_s]).flatten() {
        assert!(greenish(s) <= 2, "a wall piece still has its grass slab ({} green px)", greenish(s));
    }
}

#[test]
fn test_falls_back_to_pack_props_and_castle() {
    // without assets/rd/{trees,stones,castle} the pack's art draws, at
    // the pack's scale
    let fetch = |rel: &str| -> Option<Vec<u8>> {
        if ["rd/trees", "rd/stones", "rd/castle", "rd/buildings/church"].iter().any(|p| rel.starts_with(p)) { return None; }
        std::fs::read(format!("../assets/{}", rel)).ok()
    };
    let art = Art::load(&fetch, 0);
    assert_eq!((art.tiles[0].trees.len(), art.tiles[0].trees_k), (4, PACK_PROP_K));
    assert_eq!((art.stones.len(), art.stones_k), (3, PACK_PROP_K));
    assert!(!art.castle_rd && art.walls.iter().all(|s| s.is_some()));
    assert!(art.church_s.as_ref().is_some_and(|c| c.width > 90), "the pack's keep stands in for the church");
}

#[test]
fn test_round4_art_loads_and_falls_back() {
    // generated: tufts at 1x (gilded for autumn, a snowy winter set),
    // the windmill and its four sail frames, the trade halls and the
    // rival village with snowed-over winter looks, and walk cycles for
    // the rivals and the travellers
    let art = Art::load_fs("../assets", 0);
    assert_eq!((art.details[0].len(), art.details[1].len(), art.details[2].len(), art.details_k), (14, 14, 12, 1.0));
    assert!(art.mill_rd && art.mill_frames.len() == 4 && art.mill_tower.as_ref().is_some_and(|t| t.width < 40));
    assert!(art.trades_winter.iter().all(|s| s.is_some()));
    assert!(art.rival_walks.len() == 3 && art.rival_walks.iter().all(|w| w.is_some()));
    assert!(art.traveller_walk.is_some());
    // without them: the pack's, at the pack's scale, and no winter looks
    let fetch = |rel: &str| -> Option<Vec<u8>> {
        if ["rd/tufts", "rd/windmill", "rd/rivals", "rd/traveller", "rd/buildings/smithy", "rd/buildings/guildhall",
            "rd/buildings/rival"].iter().any(|p| rel.starts_with(p)) { return None; }
        std::fs::read(format!("../assets/{}", rel)).ok()
    };
    let art = Art::load(&fetch, 0);
    assert_eq!(art.details_k, PACK_PROP_K);
    assert!(!art.details[0].is_empty());
    assert!(!art.mill_rd && art.mill_frames.len() == 4 && art.mill_tower.is_some());
    assert!(art.blacksmith_s.is_some() && art.guildhall_s.is_some() && art.rival_house.is_some());
    assert!(art.trades_winter.iter().all(|s| s.is_none()));
    assert!(art.rival_walks.iter().all(|w| w.is_none()) && art.traveller_walk.is_none());
}

#[test]
fn test_rivals_and_travellers_get_poses() {
    // the rivals walk a cycle like villagers: they need a facing each
    let mut g = Game::new_seeded(7);
    for _ in 0..30 { g.update(); }
    let m = g.ui_model();
    assert!(!g.rivals.is_empty());
    assert!(g.rivals.iter().all(|r| m.poses.iter().any(|p| p.0 == *r)), "a pose for every rival");
    if g.traveller_pos().is_some() {
        assert!(m.poses.iter().any(|p| p.0 == 0 && p.2), "the courier rides with a facing");
    }
}

#[test]
fn test_standing_pieces_sit_on_their_tile() {
    // a 1x generated piece is centred on the tile and its bottom (the
    // footprint's front corner) a quarter of its width below the centre
    let s = Sprite { width: 40, height: 50, pixels: vec![255; 40 * 50 * 4] };
    let mut buf = Buffer::new(200, 200);
    draw_standing(&mut buf, &s, 100, 100, 1.0);
    let lit = |x: usize, y: usize| buf.pixels[(y * 200 + x) * 4] == 255;
    assert!(lit(100, 109) && !lit(100, 110), "bottom row at the front corner (centre + 10)");
    assert!(lit(80, 100) && !lit(79, 100) && lit(119, 100) && !lit(120, 100), "centred");
    assert!(lit(100, 60) && !lit(100, 59), "50px tall");
}

#[test]
fn test_walking_villagers_face_where_they_go() {
    let mut g = Game::new_seeded(7);
    for _ in 0..200 { g.update(); }
    let m = g.ui_model();
    assert!(g.villagers.iter().all(|v| m.poses.iter().any(|p| p.0 == *v)), "a pose for every villager");
    assert!(m.poses.iter().all(|p| p.1 < 4), "facings are walk-sheet rows");
    // the model straight after an update: whoever moved in that update
    // is mid-stride, facing that way; whoever didn't is standing (no
    // one-frame lag)
    let mut moved_some = false;
    for _ in 0..50 {
        let before: Vec<(f32, f32)> = g.villagers.iter().map(|&v| g.world.get_component::<Pos>(v).map(|p| (p.x, p.y)).unwrap_or_default()).collect();
        g.update();
        let m = g.ui_model();
        for (i, &v) in g.villagers.iter().enumerate() {
            let Some(p) = g.world.get_component::<Pos>(v) else { continue };
            let (dx, dy) = (p.x - before[i].0, p.y - before[i].1);
            let moved = dx.abs() + dy.abs() > 1e-4;
            let pose = m.poses.iter().find(|q| q.0 == v).unwrap();
            assert_eq!(pose.2, moved, "villager {}: walking shows this frame's movement", i);
            if moved { assert_eq!(pose.1, facing_on_screen(dx, dy)); moved_some = true; }
        }
    }
    assert!(moved_some, "someone walked in 50 frames");
}

#[test]
fn test_walking_turns_and_strides_the_player() {
    let mut g = Game::new_seeded(7);
    while g.tick < 5 { g.update(); }
    g.update();
    assert!(!g.ui_model().player_pose.1, "standing still isn't walking");
    // each way across the map shows its own row of the walk sheet
    let mut seen = vec![];
    for (dx, dy) in [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        for _ in 0..4 { g.command(Command::Walk(dx, dy)); g.update(); }
        let (facing, walking) = g.ui_model().player_pose;
        assert!(walking, "walking ({}, {}) shows the walk cycle", dx, dy);
        seen.push(facing);
    }
    assert_eq!(seen, vec![1, 2, 3, 0], "right, down, left, up on screen");
    g.update();
    assert!(!g.ui_model().player_pose.1, "stops striding when they stop");
}

#[test]
fn test_generated_animals_load_and_are_animal_sized() {
    // the Retro Diffusion sprites (assets/rd/farm-animals) replace the
    // hand-drawn ones: they must be found, and stay smaller than a
    // villager drawn on screen (a 32px frame at 2x, ~24px of figure)
    let art = Art::load_fs("../assets", 0);
    let dog = art.dog.as_ref().unwrap();
    assert!(dog.width != 22 || dog.height != 14, "still the doubled hand-drawn dog: rd/farm-animals didn't load");
    assert_eq!(art.sheep.len(), 2, "both sheep looks load");
    assert_eq!(art.goats.len(), 2, "both goat looks load");
    assert_eq!(art.hens.len(), 3, "hen, hen, rooster");
    for s in [dog, art.cat.as_ref().unwrap()].into_iter().chain(&art.hens).chain(&art.sheep).chain(&art.goats) {
        assert!((10..=22).contains(&s.height), "{}x{} animal - too big or too small next to a villager", s.width, s.height);
        assert!(s.pixels.chunks(4).any(|p| p[3] == 0), "background keyed out");
    }
}

#[test]
fn test_animals_fall_back_to_hand_drawn_art() {
    // the generated files missing (the packs still there): the
    // hand-drawn pets and hen, doubled
    let art = Art::load(&|rel: &str| if rel.starts_with("rd/") { None } else { std::fs::read(std::path::Path::new("../assets").join(rel)).ok() }, 0);
    assert_eq!(art.dog.as_ref().map(|d| (d.width, d.height)), Some((22, 14)));
    assert_eq!(art.hens.len(), 1);
    assert!(art.sheep.is_empty() && art.goats.is_empty(), "draw_beast covers the fold");
}

#[test]
fn test_the_dog_is_a_proper_little_sprite() {
    let dog = dog_sprite();
    assert_eq!((dog.width, dog.height), (11, 7));
    assert!(DOG_ART.iter().all(|r| r.len() == dog.width), "every row of the art is the same width");
    let coat = dog.pixels.chunks(4).filter(|p| p[3] > 0 && (p[0], p[1], p[2]) == DOG).count();
    // drawn 2x that's 4x as many on screen; the old two rectangles were 33 px
    assert!(coat * 4 >= 2 * 33, "only {} coat pixels ({} on screen)", coat, coat * 4);
}
