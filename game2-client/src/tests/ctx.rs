use crate::*;

fn overlap(a: &Button, b: &Button) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// Every contextual action at once: the band must never let two
/// rects meet, whatever the mix of things the sim offers
#[test]
fn test_ctx_buttons_never_overlap() {
    let acts = [
        CtxAct::Errand, CtxAct::PairUp, CtxAct::Matchmake,
        CtxAct::Delegate, CtxAct::Strike, CtxAct::Townhall, CtxAct::Petition,
        CtxAct::Gather, CtxAct::Pact, CtxAct::PetCat,
        CtxAct::Snowballs, CtxAct::Douse,
        CtxAct::PickCaps, CtxAct::ShareFindings, CtxAct::TakeCache,
        CtxAct::Tame, CtxAct::Give,
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
    // Talk rides the band's next cell - after the last ctx button,
    // overlapping none (#1599)
    let talk = talk_button(m.ctx.len());
    for b in &btns {
        assert!(!overlap(&talk, b), "Talk meets {}", b.label);
    }
    if let Some(last) = btns.last() {
        if m.ctx.len() % 2 == 1 {
            assert_eq!(talk.y, last.y, "an open right cell takes Talk beside the last");
            assert!(talk.x > last.x);
        } else {
            assert!(talk.y > last.y, "a filled row puts Talk on the next");
        }
    } else {
        assert_eq!(talk.y, CTX_BAND_Y, "no ctx buttons: Talk takes the first cell");
    }
}

/// The wire's own wording wins: `ctx_labels` overrides the static text,
/// a short or absent list (an older server) falls back to `ctx_button`'s
#[test]
fn test_ctx_labels_override_the_static_wording() {
    let acts = [CtxAct::Errand, CtxAct::Tame];
    let btns = ctx_buttons_labeled(&acts, &["Errand".to_string(), "Pet the sheep".to_string()]);
    assert_eq!(btns[1].label, "Pet the sheep");
    let btns = ctx_buttons_labeled(&acts, &[]);
    assert_eq!(btns[1].label, "Pet it", "the static label fills the gap");
}

/// The cause's verbs left the panel (#1337): a campaign about does not
/// draw a Meet, a March or a hall-mend on the card - the cadence's own
/// date sits the floor instead, and the motions carry the rest
#[test]
fn test_the_campaign_verbs_stay_off_the_card() {
    let mut game = Game::new_seeded(MAP_SEED);
    // a stretch of frames so the campaign and the world both get a say
    // in what the card offers
    for _ in 0..900 {
        game.update();
        game.update_ctx_actions();
    }
    let m = game.ui_model();
    assert!(!m.ctx.iter().any(|a| matches!(a,
        CtxAct::Strike | CtxAct::Townhall)),
        "a street motion drew as a button: {:?}", m.ctx);
}
