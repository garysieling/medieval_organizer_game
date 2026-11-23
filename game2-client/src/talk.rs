//! The TALK panel (under the minimap) and the conversation box (top-right
//! of the map): where they sit, how their rows lay out, drawing them, and
//! mapping a click back to a pick. Reads the panel models (`TalkPanelV`,
//! `ConvoPanelV`) that `session::dialogue` builds.

use super::*;

// ---------- the TALK panel ----------

/// The TALK box: under the find box, the side panel's foot
pub const TALK_X: i32 = UI_X as i32 + 10;
pub const TALK_Y: i32 = 848;
pub const TALK_W: i32 = 320;
pub const TALK_H: i32 = 108;
pub(crate) const ROW_H: i32 = LINE_H;
/// Characters to a row: the box less its padding
pub const TALK_CHARS: usize = ((TALK_W - 12) / GLYPH_W) as usize;
/// Rows the box holds under its title
pub(crate) const TALK_ROWS: usize = ((TALK_H - 18) / ROW_H) as usize;
/// An option takes at most this many rows; a longer one is cut with "..."
pub(crate) const OPTION_ROWS: usize = 2;

/// Where each clickable reply sits: (x, y, w, h, the pick - None is Leave)
pub type OptionRect = (i32, i32, i32, i32, Option<usize>);
/// A drawn row: (y, text, colour, the speaker's name leading it in its
/// colour, rubric - day rules draw tracked capitals, the H2 tier #1321)
pub type TalkRow = (i32, String, (u8, u8, u8), Option<(String, (u8, u8, u8))>, bool);

/// Word-wrap a reply as "1) text", at most OPTION_ROWS rows
pub(crate) fn option_rows(n: usize, text: &str) -> Vec<String> {
    let mut rows = super::wrap_text(&format!("{}) {}", n + 1, text), TALK_CHARS - 1);
    if rows.len() > OPTION_ROWS {
        rows.truncate(OPTION_ROWS);
        let last = rows.last_mut().unwrap();
        while last.len() + 3 > TALK_CHARS - 1 { last.pop(); }
        last.push_str("...");
    }
    rows
}

/// The rows the box draws, bottom-anchored like a chat - the newest line
/// sits just above the replies - and where the replies are, for clicks
pub fn talk_layout(p: &TalkPanelV) -> (Vec<TalkRow>, Vec<OptionRect>) {
    let mut opt_rows: Vec<(String, Option<usize>)> = vec![];
    for (i, o) in p.options.iter().enumerate() {
        for r in option_rows(i, o) { opt_rows.push((r, Some(i))); }
    }
    if p.in_talk { opt_rows.push(("x) Take your leave".into(), None)); }
    let room = TALK_ROWS.saturating_sub(opt_rows.len());
    // each line wrapped; the speaker's name leads the first row in its colour
    let mut lines: Vec<(String, (u8, u8, u8), Option<(String, (u8, u8, u8))>, bool)> = vec![];
    // which rows start a line: the view never opens on a line's tail
    let mut starts: Vec<bool> = vec![];
    for r in &p.rows {
        let before = lines.len();
        if r.rule { lines.push((r.text.clone(), (110, 110, 135), None, true)); starts.push(true); continue; }
        let body = if r.gist { (255, 215, 120) } else if r.name.is_none() { (150, 150, 170) } else { (215, 215, 200) };
        let ncol = if r.you { (255, 255, 120) } else { (150, 200, 255) };
        match &r.name {
            Some(n) => {
                let lead = format!("{}: ", n.to_uppercase());
                for (k, w) in super::wrap_text(&format!("{}{}", lead, r.text), TALK_CHARS).into_iter().enumerate() {
                    match w.strip_prefix(lead.trim_end()) {
                        Some(rest) if k == 0 => lines.push((rest.trim_start().to_string(), body, Some((lead.clone(), ncol)), false)),
                        _ => lines.push((w, body, None, false)),
                    }
                }
            }
            None => for w in super::wrap_text(&r.text, TALK_CHARS) { lines.push((w, body, None, false)); },
        }
        starts.extend((before..lines.len()).map(|k| k == before));
    }
    let mut skip = lines.len().saturating_sub(room);
    while skip < lines.len() && !starts[skip] { skip += 1; }
    // no speaker's start fits the room left: show the tail anyway rather
    // than a box of replies with nothing said
    if skip == lines.len() && room > 0 { skip = lines.len() - room.min(lines.len()); }
    let top = TALK_Y + 16;
    let mut out: Vec<TalkRow> = vec![];
    for (k, (text, rgb, name, rubric)) in lines.into_iter().skip(skip).enumerate() {
        out.push((top + k as i32 * ROW_H, text, rgb, name, rubric));
    }
    // the replies sit right under the last line said
    let opt_top = top + out.len() as i32 * ROW_H + 2;
    let mut rects: Vec<OptionRect> = vec![];
    for (k, (text, pick)) in opt_rows.into_iter().enumerate() {
        let y = opt_top + k as i32 * ROW_H;
        out.push((y, text, if pick.is_some() { (140, 230, 140) } else { (200, 150, 130) }, None, false));
        // a reply's rows click as one
        match rects.last_mut() {
            Some(r) if r.4 == pick && pick.is_some() => r.3 += ROW_H,
            _ => rects.push((TALK_X + 2, y - 2, TALK_W - 4, ROW_H, pick)),
        }
    }
    (out, rects)
}

/// Draw the TALK box; a reply under the mouse lights up
pub fn draw_talk_panel(buf: &mut Buffer, p: &TalkPanelV, mouse: (i32, i32)) {
    buf.rect(TALK_X, TALK_Y, TALK_W, TALK_H, 25, 25, 40);
    draw_text(buf, TALK_X + 4, TALK_Y + 4, &p.title, 150, 150, 180);
    let (rows, rects) = talk_layout(p);
    for &(x, y, w, h, _) in &rects {
        if mouse.0 >= x && mouse.0 < x + w && mouse.1 >= y && mouse.1 < y + h {
            buf.rect(x, y, w, h, 45, 60, 50);
        }
    }
    for (y, text, (r, g, b), name, rubric) in rows {
        let mut x = TALK_X + 6;
        if let Some((n, (nr, ng, nb))) = name { x += draw_text(buf, x, y, &n, nr, ng, nb); } // draw_text returns the width
        if rubric { draw_tier(buf, x, y, &text, TextTier::H2, r, g, b); }
        else { draw_text(buf, x, y, &text, r, g, b); }
    }
}

/// Which reply (if any) a click at (mx, my) on the TALK box picks:
/// Some(Some(i)) a reply, Some(None) Leave, None not a reply
pub fn talk_click(p: &TalkPanelV, mx: i32, my: i32) -> Option<Option<usize>> {
    talk_layout(p).1.into_iter()
        .find(|&(x, y, w, h, _)| mx >= x && mx < x + w && my >= y && my < y + h)
        .map(|r| r.4)
}

// ---------- the conversation box (top-right of the map) ----------

/// Where the box sits: up at the map's top-right, over the ground
pub const CONVO_X: i32 = MAP_W as i32 - 344;
pub const CONVO_Y: i32 = 46;
pub const CONVO_W: i32 = 336;
/// Characters to a row inside a button: the box less its padding
pub(crate) const CONVO_CHARS: usize = ((CONVO_W - 24) / GLYPH_W) as usize;
pub(crate) const CONVO_ROW_H: i32 = LINE_H;
/// The tallest the box gets before the oldest lines drop off the top:
/// most of the window's height (#1317)
pub(crate) const CONVO_MAX_H: i32 = 660;
/// Frames a new line takes to fade up to full colour (#1318)
pub(crate) const CONVO_FADE: u32 = 10;
/// The speaker icon's slot left of a name lead: ~10px for the head,
/// the rest a gap before the words (#1319); the mood mark #1320 hangs
/// on this slot
pub(crate) const CONVO_PORT: i32 = 16;
/// `CONVO_PORT` in glyphs, off a speaker row's wrap width
pub(crate) const CONVO_PORT_CHARS: usize = (CONVO_PORT as usize) / GLYPH_W as usize;
/// The drop cap's decorated box, opening a talk or meeting (#1321)
pub(crate) const CONVO_CAP: i32 = 28;
/// Rows that sit beside the cap before the text takes the full width
pub(crate) const CONVO_CAP_ROWS: usize = 2;
/// `CONVO_CAP` plus its gap, in glyphs off the opener's wrap width
pub(crate) const CAP_CHARS: usize = (CONVO_CAP as usize + 4) / GLYPH_W as usize;

/// The conversation box's scroll pin (#1317): which panel the anchor
/// belongs to and the first line still shown, so a line keeps the y it
/// drew at until it drops off the top - and what's dropped doesn't come
/// back. It realigns by each row's signature, not its index: the talk's
/// record slides gist lines in under the top, which would drift a bare
/// index. `borns` remembers the frame each shown signature first drew
/// at, for the new lines' fade-in
#[derive(Default)]
pub struct ConvoPin {
    /// hash of the panel's title + first row - a different panel starts over
    id: u64,
    /// (signature, last index) of the first row still shown
    top: Option<(u64, usize)>,
    /// frame each shown row's signature first drew at
    borns: game2_core::engine::det::HashMap<u64, u32>,
}

/// A drawn colour faded toward the dark of a line just landed: dim at
/// first sight, full after CONVO_FADE frames
fn fade_rgb((r, g, b): (u8, u8, u8), age: u32) -> (u8, u8, u8) {
    let s = 0.2 + 0.8 * (age.min(CONVO_FADE) as f32 / CONVO_FADE as f32);
    ((r as f32 * s) as u8, (g as f32 * s) as u8, (b as f32 * s) as u8)
}

/// A row's signature: what was said, by whom, in which voice - enough to
/// find it again after the talk's record inserts a line over it
fn row_sig(r: &TalkRowV) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut mix = |b: u8| { h ^= b as u64; h = h.wrapping_mul(0x0000_0100_0000_01b3); };
    for &b in r.name.as_deref().unwrap_or("").as_bytes().iter().chain(r.text.as_bytes()) { mix(b); }
    for b in [r.gist as u8, r.you as u8, r.rule as u8] { mix(b); }
    for b in r.who.unwrap_or(usize::MAX).to_le_bytes() { mix(b); }
    h
}

/// What panel this is: its title and first line. A new Talk, the
/// meeting's call, a survey - each its own pin
fn panel_id(p: &ConvoPanelV) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in p.title.as_bytes() { h ^= b as u64; h = h.wrapping_mul(0x0000_0100_0000_01b3); }
    h ^ p.rows.first().map(row_sig).unwrap_or(0).wrapping_mul(0x9e37_79b9_7f4a_7c15)
}

/// What a click on one of the box's buttons means
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConvoClick {
    /// Select the speaker (a villager index)
    Speaker(usize),
    /// A Talk reply; None is Take your leave
    Reply(Option<usize>),
    /// The founding call after a house meeting
    Decide(usize),
}

/// One bordered button in the box: a speaker's line (its wrapped rows
/// together) or a clickable choice under the talk
pub struct ConvoBox {
    pub x: i32, pub y: i32, pub w: i32, pub h: i32,
    /// (y, text, colour, the speaker's name leading the first row)
    pub rows: Vec<TalkRow>,
    /// What a click does - a named line selects its speaker
    pub click: Option<ConvoClick>,
    /// A choice button, drawn raised, vs a line of what was said
    pub choice: bool,
    /// The speaker's mood mark (#1320): 0 neutral, 1 warm, 2 cold,
    /// 3 angry - a badge pip on the head slot, 0 draws none
    pub mood: u8,
    /// The opener's drop cap (#1321): its letter drawn decorated on the
    /// first line's left, the first rows sitting beside it
    pub cap: Option<char>,
}

/// Wrap each conversation line and choice into its own button: the box
/// grows down as lines arrive, pinning what it already drew (the `pin`
/// holds the first shown row across calls); past CONVO_MAX_H the oldest
/// lines drop off the top and the survivors scroll up as one block. New
/// lines fade up over CONVO_FADE model frames (`frame` is the model's
/// own counter, so a recording ages like the window). The choices sit
/// under the lines. Returns the boxes and how tall the panel needs to be
pub fn convo_layout(p: &ConvoPanelV, pin: &mut ConvoPin, frame: u32) -> (Vec<ConvoBox>, i32) {
    // the choices' space is held first - lines give way to them
    let mut opts: Vec<Vec<String>> = vec![];
    let opt_count = p.options.len() + p.leave as usize;
    for (i, o) in p.options.iter().enumerate() {
        let label = format!("{}) {}", i + 1, o);
        let mut ws = super::wrap_text(&label, CONVO_CHARS);
        ws.truncate(3);
        opts.push(ws);
    }
    if p.leave {
        opts.push(vec!["x) Take your leave".into()]);
    }
    // the opener's drop cap: the first real line's first letter,
    // decorated - a day rule isn't the opener, your own words aren't
    // either. A named line caps its lead word (the name), so the words
    // themselves stay whole (#1321)
    let cap_at = p.rows.iter().position(|r| !r.you && !r.rule);
    let cap = cap_at.and_then(|i| {
        let r = &p.rows[i];
        r.name.as_deref().unwrap_or(&r.text).chars().next()
    }).filter(|c| c.is_alphanumeric());
    // wrap every line, then keep the newest that fit beside the choices
    let wrap_line = |r: &TalkRowV, cap: Option<char>| -> Vec<(String, (u8, u8, u8), Option<(String, (u8, u8, u8))>, bool)> {
        let body = if r.gist { (255, 215, 120) } else if r.name.is_none() { (150, 150, 170) } else { (215, 215, 200) };
        let ncol = if r.you { (255, 255, 120) } else { (150, 200, 255) };
        // a speaker's words leave room for their head beside the lead;
        // the opener leaves the cap's width beside it too
        let width = CONVO_CHARS - r.who.is_some() as usize * CONVO_PORT_CHARS
            - cap.is_some() as usize * CAP_CHARS;
        // a named line's cap comes off its lead ("ELIZA: Good morrow"
        // reads whole); an unowned line's comes off its first word
        let text = cap.filter(|_| r.name.is_none())
            .and_then(|c| r.text.strip_prefix(c)).unwrap_or(&r.text).trim_start();
        let mut ws = vec![];
        match &r.name {
            Some(n) => {
                let capped: String = if cap.is_some() { n.chars().skip(1).collect() } else { n.clone() };
                let lead = format!("{}: ", capped.to_uppercase());
                for (k, w) in super::wrap_text(&format!("{}{}", lead, text), width).into_iter().enumerate() {
                    match w.strip_prefix(lead.trim_end()) {
                        Some(rest) if k == 0 => ws.push((rest.trim_start().to_string(), body, Some((lead.clone(), ncol)), false)),
                        _ => ws.push((w, body, None, false)),
                    }
                }
            }
            None => for w in super::wrap_text(text, width) { ws.push((w, body, None, r.rule)) },
        }
        ws
    };
    let lines: Vec<Vec<(String, (u8, u8, u8), Option<(String, (u8, u8, u8))>, bool)>> =
        p.rows.iter().enumerate().map(|(i, r)| wrap_line(r, cap.filter(|_| Some(i) == cap_at))).collect();
    // room for lines = the box less the tiered title and the choice buttons
    let opt_h = opts.iter().map(|ws| ws.len() as i32 * CONVO_ROW_H + 6).sum::<i32>() + 3 * opt_count as i32;
    let room = CONVO_MAX_H - 38 - opt_h;
    // the newest that fit beside the choices: how far back the tail
    // reaches this call - the pin can only shorten it, never lengthen
    let mut fit = 0usize;
    {
        let mut room = room;
        for (i, ws) in lines.iter().enumerate().rev() {
            let h = (ws.len() as i32 * CONVO_ROW_H + 9)
                .max(if cap.is_some() && Some(i) == cap_at { CONVO_CAP + 11 } else { 0 });
            if h > room && i != lines.len() - 1 { break; }
            room -= h;
            fit = i;
        }
    }
    // the pin: a new panel starts over; a standing one holds its first
    // row, realigned by signature since the record inserts under the top.
    // Overflow advances the anchor - a dropped row doesn't come back
    let id = panel_id(p);
    if pin.id != id { pin.id = id; pin.top = None; pin.borns.clear(); }
    let sigs: Vec<u64> = p.rows.iter().map(row_sig).collect();
    let pinned = pin.top.and_then(|(s, idx)|
        (0..sigs.len()).filter(|&i| sigs[i] == s).min_by_key(|&i| i.abs_diff(idx)));
    let anchor = fit.max(pinned.unwrap_or(fit));
    pin.top = p.rows.get(anchor).map(|r| (row_sig(r), anchor));
    // new signatures get this frame as their first sight; sigs gone from
    // the record let the map go - dead rows can't come back anyway
    for &s in &sigs[anchor.min(sigs.len())..] { pin.borns.entry(s).or_insert(frame); }
    pin.borns.retain(|&s, _| sigs.contains(&s));
    // lay the buttons out top to bottom, the title's tier above them
    let mut boxes: Vec<ConvoBox> = vec![];
    let mut y = CONVO_Y + 34;
    for (i, ws) in lines.iter().enumerate().skip(anchor) {
        let h = (ws.len() as i32 * CONVO_ROW_H + 6)
            .max(if cap.is_some() && Some(i) == cap_at { CONVO_CAP + 8 } else { 0 });
        let age = frame.saturating_sub(pin.borns.get(&sigs[i]).copied().unwrap_or(frame));
        let rows = ws.iter().enumerate()
            .map(|(k, (t, c, n, rub))| (y + 3 + k as i32 * CONVO_ROW_H, t.clone(), fade_rgb(*c, age),
                n.clone().map(|(s, col)| (s, fade_rgb(col, age))), *rub))
            .collect();
        let click = p.rows[i].who.map(ConvoClick::Speaker);
        boxes.push(ConvoBox { x: CONVO_X + 4, y, w: CONVO_W - 8, h, rows, click, choice: false,
            mood: p.rows[i].mood, cap: cap.filter(|_| Some(i) == cap_at) });
        y += h + 3;
    }
    for (i, ws) in opts.into_iter().enumerate() {
        let h = ws.len() as i32 * CONVO_ROW_H + 6;
        let pick = if p.decides { ConvoClick::Decide(i) }
            else if i < p.options.len() { ConvoClick::Reply(Some(i)) }
            else { ConvoClick::Reply(None) };
        let colour = if matches!(pick, ConvoClick::Reply(None)) { (200, 150, 130) } else { (140, 230, 140) };
        let rows = ws.iter().enumerate()
            .map(|(k, t)| (y + 3 + k as i32 * CONVO_ROW_H, t.clone(), colour, None, false))
            .collect();
        boxes.push(ConvoBox { x: CONVO_X + 4, y, w: CONVO_W - 8, h, rows, click: Some(pick), choice: true, mood: 0, cap: None });
        y += h + 3;
    }
    (boxes, y - CONVO_Y + 4)
}

/// Draw the conversation box; a button under the mouse lights up.
/// `portrait` resolves a speaker's villager index to their head still -
/// where it answers None the slot stays empty and the name lead carries
/// the line alone (#1319). `mf` is the marginalia set (#1323): its pieces
/// border the box, the season's vine corners it, and a fleuron marks
/// who's speaking - what's missing falls back to a drawn gilt rule
pub fn draw_convo_panel<'a>(buf: &mut Buffer, p: &ConvoPanelV, mouse: (i32, i32), pin: &mut ConvoPin, frame: u32,
        portrait: &'a dyn Fn(usize) -> Option<&'a Sprite>, mf: &UiFrame, season: Season) {
    let (boxes, h) = convo_layout(p, pin, frame);
    buf.rect(CONVO_X - 2, CONVO_Y - 2, CONVO_W + 4, h + 2, 20, 20, 32);
    buf.rect(CONVO_X, CONVO_Y, CONVO_W, h, 25, 25, 40);
    // the manuscript border: edge runs and corner blocks round the box,
    // the season's vine flourishes over them, the speaker's fleuron
    // beside the title (#1323)
    mf.draw_around(buf, CONVO_X - 2, CONVO_Y - 2, CONVO_W + 4, h + 2);
    mf.draw_vines(buf, season, CONVO_X - 2, CONVO_Y - 2, CONVO_W + 4, h + 2);
    if let Some(a) = mf.accent(&p.rows) {
        a.blit(buf, CONVO_X + CONVO_W - ACCENT_PX as i32 - 7, CONVO_Y + 5);
    }
    // the doubled title; a long call's name falls back to 1x rather
    // than spilling into the sidebar
    if p.title.len() as i32 * (2 * GLYPH_W) <= CONVO_W - 12 {
        draw_tier(buf, CONVO_X + 6, CONVO_Y + 3, &p.title, TextTier::H1, 150, 150, 180);
    } else {
        draw_text(buf, CONVO_X + 6, CONVO_Y + 10, &p.title, 150, 150, 180);
    }
    for b in &boxes {
        let hover = b.click.is_some()
            && mouse.0 >= b.x && mouse.0 < b.x + b.w && mouse.1 >= b.y && mouse.1 < b.y + b.h;
        let (r, g, bcol) = if hover { (52, 64, 54) } else if b.choice { (36, 48, 38) } else { (31, 31, 47) };
        buf.rect(b.x, b.y, b.w, b.h, r, g, bcol);
        // the button's edge: choices get a frame, lines a left rule
        if b.choice {
            let edge = if hover { (150, 230, 150) } else { (70, 110, 70) };
            buf.rect(b.x, b.y, b.w, 1, edge.0, edge.1, edge.2);
            buf.rect(b.x, b.y + b.h - 1, b.w, 1, edge.0, edge.1, edge.2);
            buf.rect(b.x, b.y, 1, b.h, edge.0, edge.1, edge.2);
            buf.rect(b.x + b.w - 1, b.y, 1, b.h, edge.0, edge.1, edge.2);
        } else {
            buf.rect(b.x, b.y, 2, b.h, 90, 110, 140);
        }
        // a speaker's head in the slot beside their lead; the words
        // start after it whether the still resolved or not (#1319)
        if let Some(ConvoClick::Speaker(i)) = b.click {
            crate::draw::draw_icon(buf, portrait(i), b.x + 3, b.y + 2, CONVO_PORT - 4);
            // the mood mark: a badge pip on the slot's bottom corner -
            // gold warm, steel-blue cold, red angry (#1320)
            if b.mood != 0 {
                let (mr, mg, mb) = match b.mood {
                    1 => (236, 178, 88),
                    3 => (222, 82, 62),
                    _ => (142, 176, 220),
                };
                buf.rect(b.x + 11, b.y + 10, 5, 5, 15, 15, 22);
                buf.rect(b.x + 12, b.y + 11, 3, 3, mr, mg, mb);
            }
        }
        let words_x = CONVO_X + 10
            + if matches!(b.click, Some(ConvoClick::Speaker(_))) { CONVO_PORT } else { 0 };
        // the opener's decorated initial: a bordered box with corner
        // flecks, the letter at 3x inside - the first rows sit beside it
        if let Some(c) = b.cap {
            buf.rect(words_x - 1, b.y + 1, CONVO_CAP, CONVO_CAP + 1, 172, 136, 68);
            buf.rect(words_x, b.y + 2, CONVO_CAP - 2, CONVO_CAP - 1, 30, 24, 36);
            for &(fx, fy) in &[(words_x - 2, b.y), (words_x + CONVO_CAP - 1, b.y),
                    (words_x - 2, b.y + CONVO_CAP + 1), (words_x + CONVO_CAP - 1, b.y + CONVO_CAP + 1)] {
                buf.rect(fx, fy, 2, 2, 210, 172, 96);
            }
            draw_text_scaled(buf, words_x + 4, b.y + 2, &c.to_uppercase().to_string(), 2, 226, 186, 104);
        }
        let cap_x = if b.cap.is_some() { CONVO_CAP + 4 } else { 0 };
        for (k, (y, text, (r, g, b), name, rubric)) in b.rows.iter().enumerate() {
            let mut x = words_x + if k < CONVO_CAP_ROWS { cap_x } else { 0 };
            if let Some((n, (nr, ng, nb))) = name { x += draw_text(buf, x, *y, n, *nr, *ng, *nb); }
            if *rubric { draw_tier(buf, x, *y, text, TextTier::H2, *r, *g, *b); }
            else { draw_text(buf, x, *y, text, *r, *g, *b); }
        }
    }
}

/// What a click at (mx, my) on the conversation box means, if anything
pub fn convo_click(p: &ConvoPanelV, mx: i32, my: i32, pin: &mut ConvoPin, frame: u32) -> Option<ConvoClick> {
    convo_layout(p, pin, frame).0.into_iter()
        .find(|b| mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h)
        .and_then(|b| b.click)
}

/// The command a digit key makes: the conversation box's numbered row
/// while one's open - a Talk reply, or the call's framing when the box
/// offers the founding call - and the villager pick while it isn't. A
/// digit past the drawn rows is swallowed: the numbers you see are the
/// keys that work, nothing slips through to a villager under the box
pub fn digit_cmd(convo: Option<&ConvoPanelV>, n: usize) -> Option<Command> {
    match convo {
        Some(p) if n < p.options.len() =>
            Some(if p.decides { Command::Decide(n) } else { Command::TalkPick(Some(n)) }),
        Some(_) => None,
        None => Some(Command::Select(n)),
    }
}
