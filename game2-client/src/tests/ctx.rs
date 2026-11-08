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
