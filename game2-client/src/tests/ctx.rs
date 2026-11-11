use crate::*;

fn overlap(a: &Button, b: &Button) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// Every contextual action at once: the band must never let two
/// rects meet, whatever the mix of things the sim offers
#[test]
fn test_ctx_buttons_never_overlap() {
    let acts = [
        CtxAct::Work, CtxAct::Errand, CtxAct::PairUp, CtxAct::Matchmake,
        CtxAct::Meet, CtxAct::Cadence, CtxAct::March, CtxAct::MendHall,
        CtxAct::Delegate, CtxAct::Strike, CtxAct::Townhall, CtxAct::Petition,
        CtxAct::Gather, CtxAct::TakeJob, CtxAct::Pact, CtxAct::PetCat,
        CtxAct::Snowballs, CtxAct::Douse, CtxAct::Flyers, CtxAct::Recruit,
        CtxAct::PickCaps, CtxAct::ShareFindings, CtxAct::TakeCache,
        CtxAct::Give, CtxAct::Buy, CtxAct::Tame, CtxAct::School, CtxAct::Brigade,
    ];
    let btns = ctx_buttons(&acts);
    assert_eq!(btns.len(), acts.len(), "one button per action, in order");
    for (i, b) in btns.iter().enumerate() {
        assert_eq!(b.label, ctx_button(acts[i]).label, "the order is the list's");
        for other in &btns[..i] {
            assert!(!overlap(b, other), "{:?} meets {}", acts[i], other.label);
        }
        // inside the panel's band - the all-actions list is longer than
        // any real frame offers, so only the columns are pinned here
        assert!(b.x >= UI_X as i32 + 10 && b.x + b.w <= W as i32 - 10);
    }
}

/// A real game's card: whatever the sim offers, the draw's rects are
/// the click's rects and none of them intersect
#[test]
fn test_ctx_buttons_on_a_real_frame() {
    let mut game = Game::new_seeded(MAP_SEED);
    // a stretch of frames so fires, cats, patches, plans and the
    // campaign verbs all get their turn on the card
    for _ in 0..900 {
        game.update();
        game.update_ctx_actions();
    }
    let m = game.ui_model();
    let btns = ctx_buttons(&m.ctx);
    assert_eq!(btns.len(), m.ctx.len());
    for (i, b) in btns.iter().enumerate() {
        for other in &btns[..i] {
            assert!(!overlap(b, other), "{:?} meets {}", m.ctx[i], other.label);
        }
        // a real frame's list fits the band: clear of BONDS above and
        // the campaign text below
        assert!(b.y >= CTX_BAND_Y && b.y + b.h <= 452, "{:?} sits in the band", m.ctx[i]);
    }
}

/// "The school" was a mystery button (#1071): hovering it tells what
/// the press does - and a hinted button isn't greyed like a blocked one
#[test]
fn test_the_school_button_shows_its_hint_on_hover() {
    use game2_core::game::villager::Pos;
    use game2_core::sim::community::Organization;
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    let mp = game.world.get_component::<Pos>(game.market_food).copied().unwrap();
    game.world.get_component_mut::<Organization>(game.org).unwrap().resources = 100.0;
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: mp.x + 1.0, y: mp.y };
    game.update_ctx_actions();
    let m = game.ui_model();
    let i = m.ctx.iter().position(|a| *a == CtxAct::School).expect("no school ask beside the market");
    let btns = ctx_buttons(&m.ctx);
    let b = &btns[i];
    let bottom = btns.iter().map(|b| b.y + b.h).max().unwrap();
    // unhovered the button keeps its live colour - a hint is not a block
    let buf = view.render(&m, (-1, -1));
    let px = ((b.y + 4) as usize * W + (b.x + 4) as usize) * 4;
    assert_eq!((buf.pixels[px], buf.pixels[px + 1], buf.pixels[px + 2]), b.color, "a hinted button drew greyed");
    // hovering draws the hint line under the band
    let buf = view.render(&m, (b.x + 4, b.y + 4));
    let mut hint_ink = false;
    for y in (bottom + 4)..(bottom + 12) {
        for x in (UI_X as i32 + 18)..(UI_X as i32 + 300) {
            let i2 = (y as usize * W + x as usize) * 4;
            if (buf.pixels[i2], buf.pixels[i2 + 1], buf.pixels[i2 + 2]) == (255, 180, 140) { hint_ink = true; }
        }
    }
    assert!(hint_ink, "no hint line drawn under the band on hover");
}
