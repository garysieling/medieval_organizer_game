//! HUD text and overlays: the status line, help and pause boxes, the title card.

use super::*;

/// The top-left status line: season, day, the village food store, weather, speed
pub(crate) fn hud_text(season: Season, day: u64, food: impl std::fmt::Display, weather: Option<&str>, paused: bool, speed: u32) -> String {
    format!("{:?} | day {} | food store {}{}{}", season, day, food,
        weather.map(|w| format!(" | {}", w)).unwrap_or_default(),
        if paused { " [PAUSED]" } else if speed > 1 { " [4x]" } else { "" })
}

/// The panel's standing reminder of the keys - the only on-screen pointer
/// to the help overlay once the title screen is gone
pub(crate) const PANEL_HINT: &str = "1-8 select villager   H help";

pub(crate) const HELP_LINES: [&str; 9] = [
    "click button / key: take action",
    "click villager: select + inspect",
    "bright name: close enough to talk",
    "1-8: select villager",
    "T talk  R recruit  I invest",
    "O organize  B build",
    "WASD: move   arrows: pan   +/-: zoom",
    "Space: pause   0: speed 4x",
    "F12: screenshot   Esc: quit",
];

pub(crate) const HELP_W: i32 = 420;

pub(crate) const HELP_ROW: i32 = 22;

#[cfg(test)] // only the tests check against it
/// Where the help box sits: centred on the map view, as tall as its text
pub(crate) fn help_rect() -> (i32, i32, i32, i32) {
    overlay_rect(HELP_W, HELP_LINES.len(), HELP_ROW, true)
}

pub(crate) fn draw_help(buf: &mut Buffer) {
    draw_overlay(buf, HELP_W, "CONTROLS", (255, 220, 130), &HELP_LINES, HELP_ROW, Some("press H to close"));
}

pub(crate) const PAUSE_LINES: [&str; 4] = ["Space: resume", "0: speed 1x/4x", "Esc: quit", "1-8: select villager"];

pub(crate) const PAUSE_W: i32 = 260;

pub(crate) const PAUSE_ROW: i32 = 20;

#[cfg(test)]
pub(crate) fn pause_rect() -> (i32, i32, i32, i32) {
    overlay_rect(PAUSE_W, PAUSE_LINES.len(), PAUSE_ROW, false)
}

/// A modal box over the map (help, pause): centred on the map view and
/// sized to its lines, which are centred as a block under a scale-2 title
pub(crate) fn overlay_rect(w: i32, lines: usize, row: i32, footer: bool) -> (i32, i32, i32, i32) {
    let h = 55 + lines as i32 * row + if footer { 30 } else { 12 };
    (MAP_W as i32 / 2 - w / 2, H as i32 / 2 - h / 2, w, h)
}

pub(crate) fn draw_overlay(buf: &mut Buffer, w: i32, title: &str, title_rgb: (u8, u8, u8), lines: &[&str], row: i32, footer: Option<&str>) {
    let (x, y, w, h) = overlay_rect(w, lines.len(), row, footer.is_some());
    let cx = x + w / 2;
    buf.rect(x, y, w, h, 20, 20, 35);
    buf.rect(x, y, w, 2, 200, 200, 220);
    draw_centered_at(buf, cx, y + 15, title, 2, title_rgb);
    // the block of lines is centred as a whole, left-aligned within it
    let block = lines.iter().map(|l| text_width(l, 1)).max().unwrap_or(0);
    for (i, l) in lines.iter().enumerate() {
        draw_text(buf, cx - block / 2, y + 55 + i as i32 * row, l, 210, 210, 210);
    }
    if let Some(f) = footer { draw_centered_at(buf, cx, y + h - 22, f, 1, (150, 150, 170)); }
}

/// The panel's status line: your organization by name, your gold, and the
/// rival merchants' guild by size (it was "Org 5 | Guilds 1 | ... | lead at
/// 2.0" - a count of guild objects, and the goal the bar already shows)
pub(crate) fn status_line(org: &str, leader: Option<&str>, members: usize, gold: f32, guild: Option<usize>) -> String {
    let rival = match guild { Some(n) => format!("Merchant Guild {}", n), None => "no guild yet".into() };
    // the line fits ~55 glyphs before the screen edge: the count and the
    // guild must always survive, so the name takes what's left
    let tail = format!(" | Gold {:.0} | {}", gold, rival);
    let room = 55i32 - (org.trim_start_matches("The ").len() + 3 + tail.len()) as i32;
    let head = leader.filter(|_| room >= 13).map(|l| {
        let cap = (room - 9) as usize; // " led by " is 8 glyphs + the name
        if l.len() <= cap { format!(" led by {}", l) } else { format!(" led by {}", &l[..cap]) }
    }).unwrap_or_default();
    format!("{} {}{}{}", org.trim_start_matches("The "), members, head, tail)
}

/// Label for who's in your organization, on the villager card ("Org:" never
/// said which, and the status line calls it the Harvesters)
pub(crate) const MEMBERS_LABEL: &str = "Joined";

/// Characters that fit after the label, inside the villager card
pub(crate) const MEMBERS_CHARS: usize = 36;

/// The card's member list, cut at a whole name with ".." when it won't fit
pub(crate) fn members_line(names: &[&str]) -> String {
    if names.is_empty() { return "nobody yet - recruit!".into(); }
    let all = names.join(", ");
    if all.len() <= MEMBERS_CHARS { return all; }
    let mut line = names[0].to_string();
    for n in &names[1..] {
        let next = format!("{}, {}", line, n);
        if next.len() + 3 > MEMBERS_CHARS { break; }
        line = next;
    }
    format!("{} ..", line)
}

/// `n` things called `one`/`many`: "1 follower", "2 followers"
fn counted(n: usize, one: &str, many: &str) -> String {
    if n == 1 { format!("1 {}", one) } else { format!("{} {}", n, many) }
}

/// The BONDS block under the action buttons: who the seat has spoken
/// to, then the count at each stage the bond has climbed. The first
/// line is the headline; the breakdown only shows once there's anyone
/// to break down
pub(crate) fn bond_lines(s: &SocialV) -> Vec<String> {
    let mut lines = vec![if s.met == 1 {
        "1 person you've spoken to".to_string()
    } else {
        format!("{} folk you've spoken to", s.met)
    }];
    if s.met > 0 {
        lines.push(format!("{} · {}",
            counted(s.acquaintances, "acquaintance", "acquaintances"),
            counted(s.known, "known", "known")));
        lines.push(format!("{} · {}",
            counted(s.followers, "follower", "followers"),
            counted(s.coleaders, "co-leader", "co-leaders")));
    }
    lines
}

/// A yeses list cut to one sidebar row: names joined `sep`, whatever
/// won't fit counted as "+n" - so a long crowd can't push the roles off
fn capped_list(label: &str, items: &[String], sep: &str, cap: usize) -> String {
    let head = if label.is_empty() { String::new() } else { format!("{}: ", label) };
    let mut kept = items.len();
    loop {
        let rest = items.len() - kept;
        let mut s = format!("{}{}", head, items[..kept].join(sep));
        if rest > 0 { s.push_str(&format!(" +{}", rest)); }
        if s.len() <= cap || kept == 0 { return s; }
        kept -= 1;
    }
}

/// The house-meeting plan's yeses as sidebar lines, one row apiece:
/// who's said they'd come, who'll bring friends, and the promises
/// already filled - capped so the roles always show
pub(crate) fn meeting_lines(m: &MeetingPlanV) -> Vec<String> {
    let mut out = vec![];
    if !m.coming.is_empty() { out.push(capped_list("coming", &m.coming, ", ", 40)); }
    if !m.spreaders.is_empty() { out.push(capped_list("bringing friends", &m.spreaders, ", ", 40)); }
    if !m.roles.is_empty() { out.push(capped_list("", &m.roles, " · ", 40)); }
    out
}

/// Width in pixels of `text` at `scale`: each glyph is 5px plus 1px
/// spacing, and the last space doesn't count
pub(crate) fn text_width(text: &str, scale: usize) -> i32 {
    (text.chars().count() * 6 * scale) as i32 - scale as i32
}

/// Draw `text` horizontally centred on x = `cx` at row `y`
pub(crate) fn draw_centered_at(buf: &mut Buffer, cx: i32, y: i32, text: &str, scale: usize, (r, g, b): (u8, u8, u8)) {
    draw_text_scaled(buf, cx - text_width(text, scale) / 2, y, text, scale, r, g, b);
}

/// Draw `text` horizontally centred on the screen at row `y`
pub(crate) fn draw_centered(buf: &mut Buffer, y: i32, text: &str, scale: usize, rgb: (u8, u8, u8)) {
    draw_centered_at(buf, W as i32 / 2, y, text, scale, rgb);
}

/// The title screen shown until the player presses Enter: what the game
/// is, how organizing works, the main keys, and how to start
pub fn render_title() -> Buffer {
    let mut tbuf = Buffer::new(W, H);
    tbuf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
    draw_centered(&mut tbuf, 160, "THE WANDERER", 4, (255, 220, 130));
    draw_centered(&mut tbuf, 230, "an outsider builds power", 2, (200, 200, 200));
    draw_centered(&mut tbuf, 290, "Talk with the villagers, win their trust, organize.", 1, (190, 190, 205));
    draw_centered(&mut tbuf, 306, "Win them over, then found a campaign at a house meeting.", 1, (190, 190, 205));
    draw_centered(&mut tbuf, 340, "WASD move   T talk   R recruit   O organize   H help", 1, (140, 140, 160));
    draw_centered(&mut tbuf, 400, "press ENTER to begin", 2, (255, 220, 130));
    tbuf
}
