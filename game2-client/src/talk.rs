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
pub(crate) const ROW_H: i32 = 10;
/// Characters to a row: the box less its padding
pub const TALK_CHARS: usize = ((TALK_W - 12) / GLYPH_W) as usize;
/// Rows the box holds under its title
pub(crate) const TALK_ROWS: usize = ((TALK_H - 18) / ROW_H) as usize;
/// An option takes at most this many rows; a longer one is cut with "..."
pub(crate) const OPTION_ROWS: usize = 2;

/// Where each clickable reply sits: (x, y, w, h, the pick - None is Leave)
pub type OptionRect = (i32, i32, i32, i32, Option<usize>);
/// A drawn row: (y, text, colour, the speaker's name leading it in its colour)
pub type TalkRow = (i32, String, (u8, u8, u8), Option<(String, (u8, u8, u8))>);

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
    let mut lines: Vec<(String, (u8, u8, u8), Option<(String, (u8, u8, u8))>)> = vec![];
    // which rows start a line: the view never opens on a line's tail
    let mut starts: Vec<bool> = vec![];
    for r in &p.rows {
        let before = lines.len();
        if r.rule { lines.push((r.text.clone(), (110, 110, 135), None)); starts.push(true); continue; }
        let body = if r.gist { (255, 215, 120) } else if r.name.is_none() { (150, 150, 170) } else { (215, 215, 200) };
        let ncol = if r.you { (255, 255, 120) } else { (150, 200, 255) };
        match &r.name {
            Some(n) => {
                let lead = format!("{}: ", n);
                for (k, w) in super::wrap_text(&format!("{}{}", lead, r.text), TALK_CHARS).into_iter().enumerate() {
                    match w.strip_prefix(lead.trim_end()) {
                        Some(rest) if k == 0 => lines.push((rest.trim_start().to_string(), body, Some((lead.clone(), ncol)))),
                        _ => lines.push((w, body, None)),
                    }
                }
            }
            None => for w in super::wrap_text(&r.text, TALK_CHARS) { lines.push((w, body, None)); },
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
    for (k, (text, rgb, name)) in lines.into_iter().skip(skip).enumerate() {
        out.push((top + k as i32 * ROW_H, text, rgb, name));
    }
    // the replies sit right under the last line said
    let opt_top = top + out.len() as i32 * ROW_H + 2;
    let mut rects: Vec<OptionRect> = vec![];
    for (k, (text, pick)) in opt_rows.into_iter().enumerate() {
        let y = opt_top + k as i32 * ROW_H;
        out.push((y, text, if pick.is_some() { (140, 230, 140) } else { (200, 150, 130) }, None));
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
    for (y, text, (r, g, b), name) in rows {
        let mut x = TALK_X + 6;
        if let Some((n, (nr, ng, nb))) = name { x += draw_text(buf, x, y, &n, nr, ng, nb); } // draw_text returns the width
        draw_text(buf, x, y, &text, r, g, b);
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
pub(crate) const CONVO_ROW_H: i32 = 10;
/// The tallest the box gets before the oldest lines drop off the top
pub(crate) const CONVO_MAX_H: i32 = 300;

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
}

/// Wrap each conversation line and choice into its own button: the lines
/// keep the tail that fits, the choices sit under them. Returns the
/// boxes and how tall the panel needs to be
pub fn convo_layout(p: &ConvoPanelV) -> (Vec<ConvoBox>, i32) {
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
    // wrap every line, then keep the newest that fit beside the choices
    let wrap_line = |r: &TalkRowV| -> Vec<(String, (u8, u8, u8), Option<(String, (u8, u8, u8))>)> {
        let body = if r.gist { (255, 215, 120) } else if r.name.is_none() { (150, 150, 170) } else { (215, 215, 200) };
        let ncol = if r.you { (255, 255, 120) } else { (150, 200, 255) };
        let mut ws = vec![];
        match &r.name {
            Some(n) => {
                let lead = format!("{}: ", n);
                for (k, w) in super::wrap_text(&format!("{}{}", lead, r.text), CONVO_CHARS).into_iter().enumerate() {
                    match w.strip_prefix(lead.trim_end()) {
                        Some(rest) if k == 0 => ws.push((rest.trim_start().to_string(), body, Some((lead.clone(), ncol)))),
                        _ => ws.push((w, body, None)),
                    }
                }
            }
            None => for w in super::wrap_text(&r.text, CONVO_CHARS) { ws.push((w, body, None)) },
        }
        ws
    };
    let lines: Vec<Vec<(String, (u8, u8, u8), Option<(String, (u8, u8, u8))>)>> =
        p.rows.iter().map(|r| wrap_line(r)).collect();
    // room for lines = the box less the title and the choice buttons
    let opt_h = opts.iter().map(|ws| ws.len() as i32 * CONVO_ROW_H + 6).sum::<i32>() + 3 * opt_count as i32;
    let mut kept: Vec<(usize, &Vec<(String, (u8, u8, u8), Option<(String, (u8, u8, u8))>)>)> = vec![];
    let mut room = CONVO_MAX_H - 18 - opt_h;
    for (i, ws) in lines.iter().enumerate().rev() {
        let h = ws.len() as i32 * CONVO_ROW_H + 9;
        if h > room && !kept.is_empty() { break; }
        room -= h;
        kept.push((i, ws));
    }
    kept.reverse();
    // lay the buttons out top to bottom
    let mut boxes: Vec<ConvoBox> = vec![];
    let mut y = CONVO_Y + 16;
    for (i, ws) in kept {
        let h = ws.len() as i32 * CONVO_ROW_H + 6;
        let rows = ws.iter().enumerate()
            .map(|(k, (t, c, n))| (y + 3 + k as i32 * CONVO_ROW_H, t.clone(), *c, n.clone()))
            .collect();
        let click = p.rows[i].who.map(ConvoClick::Speaker);
        boxes.push(ConvoBox { x: CONVO_X + 4, y, w: CONVO_W - 8, h, rows, click, choice: false });
        y += h + 3;
    }
    for (i, ws) in opts.into_iter().enumerate() {
        let h = ws.len() as i32 * CONVO_ROW_H + 6;
        let pick = if p.decides { ConvoClick::Decide(i) }
            else if i < p.options.len() { ConvoClick::Reply(Some(i)) }
            else { ConvoClick::Reply(None) };
        let colour = if matches!(pick, ConvoClick::Reply(None)) { (200, 150, 130) } else { (140, 230, 140) };
        let rows = ws.iter().enumerate()
            .map(|(k, t)| (y + 3 + k as i32 * CONVO_ROW_H, t.clone(), colour, None))
            .collect();
        boxes.push(ConvoBox { x: CONVO_X + 4, y, w: CONVO_W - 8, h, rows, click: Some(pick), choice: true });
        y += h + 3;
    }
    (boxes, y - CONVO_Y + 4)
}

/// Draw the conversation box; a button under the mouse lights up
pub fn draw_convo_panel(buf: &mut Buffer, p: &ConvoPanelV, mouse: (i32, i32)) {
    let (boxes, h) = convo_layout(p);
    buf.rect(CONVO_X - 2, CONVO_Y - 2, CONVO_W + 4, h + 2, 20, 20, 32);
    buf.rect(CONVO_X, CONVO_Y, CONVO_W, h, 25, 25, 40);
    draw_text(buf, CONVO_X + 6, CONVO_Y + 4, &p.title, 150, 150, 180);
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
        for (y, text, (r, g, b), name) in &b.rows {
            let mut x = CONVO_X + 10;
            if let Some((n, (nr, ng, nb))) = name { x += draw_text(buf, x, *y, n, *nr, *ng, *nb); }
            draw_text(buf, x, *y, text, *r, *g, *b);
        }
    }
}

/// What a click at (mx, my) on the conversation box means, if anything
pub fn convo_click(p: &ConvoPanelV, mx: i32, my: i32) -> Option<ConvoClick> {
    convo_layout(p).0.into_iter()
        .find(|b| mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h)
        .and_then(|b| b.click)
}
