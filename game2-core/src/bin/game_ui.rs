//! Game UI: windowed front-end for the village game.
//! The simulation and rendering live in game2_core::session - this shell
//! opens a window, feeds input to the Game, and blits what View draws.

use minifb::{Key, Window, WindowOptions, MouseMode};
use game2_core::engine::render::Buffer;
use game2_core::game::player::{PlayerCharacter, LEAD_INFLUENCE};
use game2_core::session::{
    self, Art, Game, View, ACTION_LABELS, CtxAct, MAP_TILES,
};

const W: usize = session::W;
const H: usize = session::H;
const FPS: u32 = session::FPS;

fn present(window: &mut Window, buf: &Buffer) {
    // RGBA -> 0xRRGGBB u32 pixels for minifb
    let px: Vec<u32> = buf.pixels.chunks_exact(4)
        .map(|c| session::rgb(c[0], c[1], c[2])).collect();
    window.update_with_buffer(&px, W, H).unwrap();
}

fn main() {
    // headless verification: --dump-frame N renders tick N to BMP and exits
    let args: Vec<String> = std::env::args().collect();
    let dump_at: Option<u64> = args.iter().position(|a| a == "--dump-frame")
        .and_then(|i| args.get(i+1)).and_then(|v| v.parse().ok());
    // optional --out PATH for the dump (default evidence/frame-check-N.bmp)
    let out: Option<String> = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).cloned();
    // optional --zoom Z for the dump (default 1)
    let zoom: f32 = args.iter().position(|a| a == "--zoom").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(1.0);
    // optional --cam X,Y to pan the dump
    let cam: Option<(i32, i32)> = args.iter().position(|a| a == "--cam").and_then(|i| args.get(i + 1))
        .and_then(|v| v.split_once(',')).and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)));
    // optional --seed N: which perturbed map to draw (default: a fresh one
    // each run; GAME2_SEED does the same for launches)
    let seed = args.iter().position(|a| a == "--seed").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or_else(session::run_seed);
    let art = Art::load_fs("assets", seed);
    if let Some(target) = dump_at {
        // --weather <drizzle|light|showers|rain|heavy|flurries|snow|blizzard|rainbow|clear>
        let weather = args.iter().position(|a| a == "--weather").and_then(|i| args.get(i + 1)).cloned();
        session::run_headless(art, target, out, zoom, cam, weather.as_deref(), seed);
        return;
    }

    let mut window = Window::new("The Wanderer - Medieval Village", W, H, WindowOptions::default())
        .expect("window failed");
    let mut game = Game::new_seeded(seed);
    let mut view = View::new(art);
    let mut started = false; // title screen until Enter
    let mut mouse_was_down = false;
    let frame_len = std::time::Duration::from_millis(1000 / FPS as u64);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_at = std::time::Instant::now();
        // TITLE SCREEN until Enter
        if !started {
            present(&mut window, &session::render_title());
            if window.is_key_down(Key::Enter) { started = true; }
            continue;
        }

        game.update();

        // --- RENDER into Buffer ---
        let (mxf, myf) = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let (mx, my) = (mxf as i32, myf as i32);
        let mut buf = view.render(&game.ui_model(), (mx, my));

        // blit buffer to framebuffer: this is also what pumps the OS event
        // queue, so input read just after is the freshest there is
        present(&mut window, &buf);

        // --- INPUT ---
        // Polled once per frame right behind the event pump (the window and
        // its events can only live on the main thread, so the fix for
        // dropped input is fast frames + fresh state, not a second thread).
        // Actions fire once per click / key press, not every frame the
        // input is held
        let (mxf, myf) = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let (mx, my) = (mxf as i32, myf as i32);
        let mouse_down = window.get_mouse_down(minifb::MouseButton::Left);
        let clicked = mouse_down && !mouse_was_down;
        mouse_was_down = mouse_down;
        let mut action: Option<usize> = None;
        let mut ctx_action: Option<usize> = None;
        if clicked {
            for (i, b) in session::action_buttons().iter().enumerate() {
                if mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h { action = Some(i); }
            }
            let ctx_specs: Vec<session::Button> = game.ctx_actions.iter().map(|a| session::ctx_button(*a)).collect();
            for (i, b) in ctx_specs.iter().enumerate() {
                if mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h { ctx_action = Some(i); }
            }
        }
        const KEYS: [Key; 5] = [Key::T, Key::R, Key::I, Key::O, Key::B];
        for (i, k) in KEYS.iter().enumerate() {
            if window.is_key_pressed(*k, minifb::KeyRepeat::No) { action = Some(i); }
        }
        if let Some(i) = action { game.act(i); }
        if let Some(i) = ctx_action { game.ctx_act(i); }
        // holding the mouse on the minimap moves the view there
        let mut on_minimap = false;
        if mouse_down {
            if let Some((cx, cy)) = game.minimap_look(mx, my) { (view.cam_x, view.cam_y) = (cx, cy); on_minimap = true; }
        }
        if !on_minimap && clicked && action.is_none() && ctx_action.is_none() {
            game.click_map(mx, my, view.cam_x, view.cam_y, view.zoom);
        }
        if window.is_key_down(Key::Key1) { game.selected_target = 0; game.picked = true; }
        if window.is_key_down(Key::Key2) { game.selected_target = 1; game.picked = true; }
        if window.is_key_down(Key::Key3) { game.selected_target = 2; game.picked = true; }
        if window.is_key_down(Key::Key4) { game.selected_target = 3; game.picked = true; }
        if window.is_key_down(Key::Key5) { game.selected_target = 4; game.picked = true; }
        if window.is_key_down(Key::Key6) { game.selected_target = 5; game.picked = true; }
        if window.is_key_down(Key::Key7) { game.selected_target = 6; game.picked = true; }
        if window.is_key_down(Key::Key8) { game.selected_target = 7; game.picked = true; }
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) { game.paused = !game.paused; }
        if window.is_key_pressed(Key::H, minifb::KeyRepeat::No) { view.help_open = !view.help_open; }
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) { view.zoom = (view.zoom + 0.006).min(2.0); }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) { view.zoom = (view.zoom - 0.006).max(0.5); }
        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) { game.speed = if game.speed == 1 { 4 } else { 1 }; }
        // WASD walk along the grid, so the player can follow a road or a
        // row of tiles: W/S up-right and down-left, A/D up-left and down-right
        let (mut dx, mut dy) = (0.0, 0.0);
        if window.is_key_down(Key::W) { dy -= 1.0; }
        if window.is_key_down(Key::S) { dy += 1.0; }
        if window.is_key_down(Key::A) { dx -= 1.0; }
        if window.is_key_down(Key::D) { dx += 1.0; }
        if dx != 0.0 || dy != 0.0 {
            // the player is walking themselves: drop any walk over to talk
            game.approach = None;
            game.picked = false;
            game.walk(dx, dy);
        }
        // arrows pan the camera
        if window.is_key_down(Key::Left)  { view.cam_x += 2; }
        if window.is_key_down(Key::Right) { view.cam_x -= 2; }
        if window.is_key_down(Key::Up)    { view.cam_y += 2; }
        if window.is_key_down(Key::Down)  { view.cam_y -= 2; }

        if window.is_key_pressed(Key::F12, minifb::KeyRepeat::No) { buf.save_bmp(&format!("evidence/ui-shot-{}.bmp", game.frame_n)).ok(); }

        let p = game.world.get_component::<PlayerCharacter>(game.player).unwrap();
        if p.influence > LEAD_INFLUENCE {
            // win banner over the village, then the summary screen
            session::draw_win_banner(&mut buf);
            present(&mut window, &buf);
            std::thread::sleep(std::time::Duration::from_secs(3));
            present(&mut window, &session::render_summary(&game));
            std::thread::sleep(std::time::Duration::from_secs(5));
            break;
        }
        // pace the frame: idle out whatever's left of the frame budget
        // rather than a flat sleep on top of the work (which dropped real
        // fps well under the target whenever a frame ran long)
        let spent = frame_at.elapsed();
        if spent < frame_len { std::thread::sleep(frame_len - spent); }
    }

    let _ = (ACTION_LABELS, CtxAct::Work, MAP_TILES); // referenced re-exports
}
