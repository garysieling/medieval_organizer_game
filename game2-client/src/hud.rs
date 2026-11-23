//! HUD text and overlays: the status line, help and pause boxes, the title card.

use super::*;

/// The top-left status line: season, day, the village food store, weather, speed
pub(crate) fn hud_text(season: Season, day: u64, food: impl std::fmt::Display, weather: Option<&str>, paused: bool,
    food_yield: f32, harsh_winter: bool) -> String {
    // two independent reads: what the climate does to the fields, and
    // whether a harsh winter's also running - one can hide neither
    let yield_word = format!("{}{}",
        if food_yield > 0.0 && food_yield < 0.95 { " | lean fields" }
        else if food_yield > 1.05 { " | rich fields" } else { "" },
        if harsh_winter { " | the store thins" } else { "" });
    format!("{:?} | day {} | food store {}{}{}{}", season, day, food,
        weather.map(|w| format!(" | {}", w)).unwrap_or_default(),
        yield_word,
        if paused { " [PAUSED]" } else { "" })
}

/// The seat's own bag and belly, as one strip under the HUD line:
/// "satchel: breadx2, cheesex1 | hungry" - empty when there's nothing
/// to say (an empty bag and a full belly)
pub(crate) fn satchel_strip(m: &UiModel) -> String {
    let bag: Vec<String> = m.player_satchel.iter().map(|(g, n)| format!("{}x{}", g, n)).collect();
    let mut s = if bag.is_empty() { String::new() } else { format!("satchel: {}", bag.join(", ")) };
    if m.player_hunger < 40.0 {
        if !s.is_empty() { s.push_str(" | "); }
        s.push_str("hungry");
    }
    s
}

/// The panel's standing reminder of the keys - the only on-screen pointer
/// to the help overlay once the title screen is gone
pub(crate) const PANEL_HINT: &str = "1-8 pick villager - H help";

pub(crate) const HELP_LINES: [&str; 10] = [
    "click button / key: take action",
    "click villager: select + inspect",
    "bright name: close enough to talk",
    "1-8: select villager",
    "T talk  R recruit  I invest",
    "O organize",
    "WASD: move   arrows: pan   +/-: zoom",
    "Space: pause   0: speed 4x",
    "F12: screenshot   Esc: menu",
    "F11: fullscreen",
];

pub(crate) const HELP_W: i32 = 460;

pub(crate) const HELP_ROW: i32 = 22;

#[cfg(test)] // only the tests check against it
/// Where the help box sits: centred on the map view, as tall as its text
pub(crate) fn help_rect() -> (i32, i32, i32, i32) {
    overlay_rect(HELP_W, HELP_LINES.len(), HELP_ROW, true)
}

pub(crate) fn draw_help(buf: &mut Buffer) {
    draw_overlay(buf, HELP_W, "CONTROLS", (255, 220, 130), &HELP_LINES, HELP_ROW, Some("press H to close"));
}

pub(crate) const PAUSE_LINES: [&str; 4] = ["Space: resume", "0: speed 1x/4x", "Esc: menu", "1-8: select villager"];

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

/// A posting wraps inside the 480px board box (~41 glyphs leave the frame)
pub(crate) const NOTICE_CHARS: usize = 41;

/// As many wrapped rows as the box holds before a "+n more" tail: 24
/// rows at NOTICE_ROW is a ~420px box, well inside the map view (#1349)
pub(crate) const NOTICE_MAX: usize = 24;

/// The postings' row pitch - a wrapped list, not a menu (it was 120: the
/// whole screen's worth a handful of notices in)
pub(crate) const NOTICE_ROW: i32 = LINE_H;

/// The board's postings as display rows: each `[kind] text` wrapped to
/// the box with its continuation indented, the list capped at NOTICE_MAX
/// rows with a "+n more" tail so the panel can never outgrow the screen
pub(crate) fn notice_lines(notices: &[(String, String)]) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    for (k, t) in notices {
        for (i, w) in wrap_text(&format!("[{}] {}", k, t), NOTICE_CHARS).into_iter().enumerate() {
            lines.push(if i == 0 { w } else { format!("   {}", w) });
        }
    }
    if lines.len() > NOTICE_MAX {
        let more = lines.len() - NOTICE_MAX + 1;
        lines.truncate(NOTICE_MAX - 1);
        lines.push(format!("+{} more", more));
    }
    lines
}

/// The panel's status line: your organization by name, your gold, and the
/// rival merchants' guild by size (it was "Org 5 | Guilds 1 | ... | lead at
/// 2.0" - a count of guild objects, and the goal the bar already shows)
/// The org row of the header: the cause's name, its count, its head
/// when one's named - the wider face gives the row ~29 glyphs, so a
/// long leader's name clips rather than run off the panel
pub(crate) fn org_line(org: &str, leader: Option<&str>, members: usize) -> String {
    let lead = format!("{} {}", org.trim_start_matches("The "), members);
    let room = 29i32 - lead.len() as i32;
    let head = leader.map(|l| {
        if l.len() + 8 <= room.max(0) as usize { format!(" led by {}", l) }
        else if room >= 11 { format!(" led by {}", &l[..room.max(0) as usize - 8]) }
        else { String::new() }
    }).unwrap_or_default();
    format!("{}{}", lead, head)
}

/// The purse row of the header: gold held and the rival guild's roll
pub(crate) fn purse_line(gold: f32, guild: Option<usize>) -> String {
    let rival = match guild { Some(n) => format!("Merchant Guild {}", n), None => "no guild yet".into() };
    format!("{:.0}g | {}", gold, rival)
}

/// The powers' standing words on the org, joined to the one line the
/// panel gives them (same ~55-glyph budget as the status line)
pub(crate) fn org_status_line(words: &[String]) -> String {
    words.join(" \u{b7} ")
}

/// Label for who's in your organization, on the villager card ("Org:" never
/// said which, and the status line calls it the Harvesters)
pub(crate) const MEMBERS_LABEL: &str = "Joined";

/// Characters that fit after the label, inside the villager card
pub(crate) const MEMBERS_CHARS: usize = 28;

/// The card's member list, cut at a whole name with ".." when it won't fit
/// The cause's own sign: the org's blocks only make sense once the
/// cause's founded - the members' roll non-empty is the marker
pub(crate) fn org_started(m: &crate::UiModel) -> bool { !m.panel.members.is_empty() }

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
    (text.chars().count() * GLYPH_W as usize * scale) as i32 - scale as i32
}

/// How wide the muted-speaker glyph's box is - the HUD strip grows by
/// this much while `View.mute` is on (#1505)
pub(crate) const MUTE_GLYPH_W: i32 = 16;

/// Where the muted-speaker glyph starts in the HUD strip (#1505):
/// right after the season line's text, on the icon row. Render and
/// the draw test agree through here.
pub(crate) fn mute_glyph_x(hud: &str, icon_w: i32) -> i32 {
    10 + icon_w + text_width(hud, 1) + 4
}

/// The muted-speaker mark (#1505): a little horn crossed out, drawn at
/// the end of the HUD strip while `View.mute` is on. 12 px wide on the
/// icon row, in the strip's own palette.
pub(crate) fn draw_mute_glyph(buf: &mut Buffer, x: i32, y: i32) {
    // the horn: a speaker box with a wedge opening right
    buf.rect(x, y + 4, 3, 6, 255, 232, 160);
    for i in 0..4 { buf.rect(x + 3 + i, y + 3 - i, 1, 8 + i * 2, 255, 232, 160); }
    // the slash, two px wide so it reads at this size
    for i in 0..12 {
        buf.set(x + i, y + 11 - i, 220, 90, 70);
        buf.set(x + i, y + 10 - i, 220, 90, 70);
    }
}

/// Draw `text` horizontally centred on x = `cx` at row `y`
pub(crate) fn draw_centered_at(buf: &mut Buffer, cx: i32, y: i32, text: &str, scale: usize, (r, g, b): (u8, u8, u8)) {
    draw_text_scaled(buf, cx - text_width(text, scale) / 2, y, text, scale, r, g, b);
}

/// Draw `text` horizontally centred on the screen at row `y`
pub(crate) fn draw_centered(buf: &mut Buffer, y: i32, text: &str, scale: usize, rgb: (u8, u8, u8)) {
    draw_centered_at(buf, W as i32 / 2, y, text, scale, rgb);
}

/// What a pause-menu row does when it's pressed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuPick {
    /// Close the menu and go on with the day
    Resume,
    /// Open the Sound overlay over the menu (#1506)
    Sound,
    /// Leave the day entirely - the window closes clean
    Quit,
}

pub(crate) const MENU_W: i32 = 220;
pub(crate) const MENU_ROW: i32 = 24;

/// The pause menu's box and its three rows, centred on the map view
#[cfg(test)]
pub(crate) fn pause_menu_box() -> (i32, i32, i32, i32) {
    overlay_rect(MENU_W, 3, MENU_ROW, false)
}

/// The pause menu's row rects, in draw order: Resume, Sound, Quit
pub fn pause_menu_rows() -> [(i32, i32, i32, i32, MenuPick); 3] {
    let (x, y, w, _) = overlay_rect(MENU_W, 3, MENU_ROW, false);
    [
        (x + 8, y + 52, w - 16, MENU_ROW, MenuPick::Resume),
        (x + 8, y + 52 + MENU_ROW + 2, w - 16, MENU_ROW, MenuPick::Sound),
        (x + 8, y + 52 + (MENU_ROW + 2) * 2, w - 16, MENU_ROW, MenuPick::Quit),
    ]
}

/// Draw the pause menu over the day's frame: three bordered rows like the
/// conversation box's choices, a hover lighting the one under the mouse
pub fn draw_pause_menu(buf: &mut Buffer, mouse: (i32, i32), title: &str) {
    let (x, y, w, h) = overlay_rect(MENU_W, 3, MENU_ROW, false);
    buf.rect(x, y, w, h, 20, 20, 35);
    buf.rect(x, y, w, 2, 200, 200, 220);
    draw_centered_at(buf, x + w / 2, y + 15, title, 2, (255, 220, 130));
    for &(rx, ry, rw, rh, pick) in &pause_menu_rows() {
        let hover = mouse.0 >= rx && mouse.0 < rx + rw && mouse.1 >= ry && mouse.1 < ry + rh;
        let (r, g, b) = if hover { (52, 64, 54) } else { (36, 48, 38) };
        buf.rect(rx, ry, rw, rh, r, g, b);
        let edge = if hover { (150, 230, 150) } else { (70, 110, 70) };
        buf.rect(rx, ry, rw, 1, edge.0, edge.1, edge.2);
        buf.rect(rx, ry + rh - 1, rw, 1, edge.0, edge.1, edge.2);
        buf.rect(rx, ry, 1, rh, edge.0, edge.1, edge.2);
        buf.rect(rx + rw - 1, ry, 1, rh, edge.0, edge.1, edge.2);
        let (label, lr, lg, lb) = match pick {
            MenuPick::Resume => ("Resume  (Esc)", 210, 230, 210),
            MenuPick::Sound => ("Sound...  (V)", 215, 200, 150),
            MenuPick::Quit => ("Quit the day  (Q)", 220, 160, 140),
        };
        draw_centered_at(buf, rx + rw / 2, ry + 8, label, 1, (lr, lg, lb));
    }
}

/// Which row a click at (mx, my) presses, if it lands on one
pub fn pause_menu_click(mx: i32, my: i32) -> Option<MenuPick> {
    pause_menu_rows().into_iter()
        .find(|&(x, y, w, h, _)| mx >= x && mx < x + w && my >= y && my < y + h)
        .map(|r| r.4)
}

/// Where a frame's Escape goes when the pause menu isn't already up:
/// a Talk's leave first (it goes out through `read_input`), then the
/// board's postings, then a conversation box that isn't a Talk just
/// holds the press - and only a bare Escape opens the menu
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EscRoute {
    /// A Talk is up: the leave goes out as Command::TalkPick(None)
    TalkLeave,
    /// The board's postings are up: they close as Command::Board(false)
    Board,
    /// A conversation box that isn't a Talk holds the press
    Swallow,
    /// Nothing conversational has it: the pause menu opens
    Pause,
}

/// The route a frame's Escape takes, read off the seat's model
pub fn esc_route(m: &UiModel) -> EscRoute {
    let leave = m.convo.as_ref().map(|p| p.leave).unwrap_or(false) || m.talk.in_talk;
    if leave { EscRoute::TalkLeave }
    else if m.notice_panel && !m.notices.is_empty() { EscRoute::Board }
    else if m.convo.is_some() { EscRoute::Swallow }
    else { EscRoute::Pause }
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

/// The prologue card after the picker: what this seed's world holds,
/// until any key lets the game begin
pub fn render_intro(facts: &[String]) -> Buffer {
    let mut buf = Buffer::new(W, H);
    buf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
    draw_centered(&mut buf, 90, "THE WANDERER", 4, (255, 220, 130));
    draw_centered(&mut buf, 160, "the village you're come to", 2, (200, 200, 200));
    let mut y = 240;
    for f in facts {
        for line in wrap_text(f, 80) {
            draw_centered(&mut buf, y, &line, 1, (190, 190, 205));
            y += LINE_H;
        }
        y += 10;
    }
    draw_centered(&mut buf, H as i32 - 60, "press ENTER", 2, (255, 220, 130));
    buf
}
