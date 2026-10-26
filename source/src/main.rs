#![windows_subsystem = "windows"]
//! HEAD OF STATE – 3D Catch-Wrestling in Rust
mod anim;
mod arena;
mod audio;
mod game;
mod hud;
mod mesh;
mod model;
mod render;
mod wrestler;

use arena::*;
use game::*;
use hud::*;
use macroquad::prelude::*;
use wrestler::*;

fn conf() -> macroquad::conf::Conf {
    let shot = std::env::var("HOS_SHOT").is_ok();
    macroquad::conf::Conf {
        miniquad_conf: macroquad::miniquad::conf::Conf {
            window_title: "Head of State".to_string(),
            window_width: if shot { 1280 } else { 1600 },
            window_height: if shot { 720 } else { 900 },
            high_dpi: true,
            sample_count: if shot { 1 } else { 4 },
            window_resizable: true,
            ..Default::default()
        },
        draw_call_vertex_capacity: 40000,
        draw_call_index_capacity: 120000,
        ..Default::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    VsCpu(usize),
    TwoPlayer,
    Watch,
}

const DIFF: [f32; 3] = [0.6, 1.0, 1.45];

fn new_game(mode: Mode) -> Game {
    let mut g = match mode {
        Mode::VsCpu(d) => Game::new(Ctrl::Human(Keys::p1()), Ctrl::Cpu(Ai::new(DIFF[d]))),
        Mode::TwoPlayer => Game::new(Ctrl::Human(Keys::p1()), Ctrl::Human(Keys::p2())),
        Mode::Watch => Game::new(Ctrl::Cpu(Ai::new(1.2)), Ctrl::Cpu(Ai::new(1.2))),
    };
    g.cam.yaw = 0.25;
    g
}

fn new_demo() -> Game {
    let mut g = Game::new(Ctrl::Cpu(Ai::new(1.3)), Ctrl::Cpu(Ai::new(1.3)));
    g.intro = 0.0;
    g.bells.clear();
    g
}

#[macroquad::main(conf)]
async fn main() {
    if let Ok(d) = std::env::var("HOS_WAVDUMP") {
        audio::dump(&d);
        return;
    }
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/Poppins-Bold.ttf")).expect("Schrift");
    let gfx = render::load_gfx();
    let tex = make_textures(&font).await;
    gfx.shadow.set_texture("MatTex", tex.mat.clone());
    if let Ok(d) = std::env::var("HOS_TEXDUMP") {
        tex.mat.get_texture_data().export_png(&format!("{}/tex_mat.png", d));
        tex.apron.get_texture_data().export_png(&format!("{}/tex_apron.png", d));
    }
    let arena = build_arena(&tex);
    let sfx = audio::Sfx::load().await;
    if let Some(s) = &sfx {
        s.start_ambience();
    }

    let shot_dir = std::env::var("HOS_SHOT").ok();
    if let Some(dir) = shot_dir {
        screenshot_run(&dir, &gfx, &arena, &font).await;
        return;
    }

    let mut menu = MenuState { sel: 0, diff: 1, show_controls: false };
    let mut demo = new_demo();
    let mut demo_reset = 0.0f32;
    let mut game: Option<Game> = None;
    let mut mode = Mode::VsCpu(1);
    let mut paused = false;
    let mut help = false;

    loop {
        let dt = get_frame_time().min(0.05);
        if let Some(g) = &mut game {
            // ------------------------------------------------ Kampf
            if is_key_pressed(KeyCode::F1) {
                help = !help;
            }
            if is_key_pressed(KeyCode::P) && g.over.is_none() {
                paused = !paused;
            }
            let over_ready = g.over.map(|(_, t)| t > 1.5).unwrap_or(false);
            if is_key_pressed(KeyCode::Escape) {
                game = None;
                paused = false;
                next_frame().await;
                continue;
            }
            if over_ready && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter)) {
                *g = new_game(mode);
            }
            if !paused {
                g.update(dt, &sfx);
            }
            g.update_camera(dt, false);
            g.draw_world(&gfx, &arena, dt);
            set_default_camera();
            draw_hud(g, &font, help);
            if paused {
                draw_rectangle(0., 0., screen_width(), screen_height(), Color::new(0., 0., 0., 0.5));
                text_shadow_c("PAUSE", &font, screen_width() / 2., screen_height() / 2., 90., WHITE);
            }
        } else {
            // ------------------------------------------------ Menü
            if menu.show_controls {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Backspace) {
                    menu.show_controls = false;
                }
            } else {
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    menu.sel = (menu.sel + 1) % MENU.len();
                }
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    menu.sel = (menu.sel + MENU.len() - 1) % MENU.len();
                }
                if menu.sel == 0 {
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                        menu.diff = (menu.diff + 2) % 3;
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                        menu.diff = (menu.diff + 1) % 3;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) || is_key_pressed(KeyCode::Space) {
                    match menu.sel {
                        0 => mode = Mode::VsCpu(menu.diff),
                        1 => mode = Mode::TwoPlayer,
                        2 => mode = Mode::Watch,
                        3 => {
                            menu.show_controls = true;
                        }
                        _ => break,
                    }
                    if menu.sel <= 2 {
                        game = Some(new_game(mode));
                        help = false;
                    }
                }
            }
            demo.update(dt, &None);
            if let Some(s) = &sfx {
                s.ambience(0.15);
            }
            if demo.over.is_some() {
                demo_reset += dt;
                if demo_reset > 6.0 {
                    demo = new_demo();
                    demo_reset = 0.0;
                }
            }
            demo.update_camera(dt, true);
            demo.draw_world(&gfx, &arena, dt);
            set_default_camera();
            draw_menu(&menu, &font, &demo);
        }
        next_frame().await;
    }
}

/// Automatischer Testlauf: rendert Szenen und speichert Screenshots
async fn screenshot_run(dir: &str, gfx: &render::Gfx, arena: &Arena, font: &Font) {
    let force = std::env::var("HOS_FORCE").unwrap_or_default();
    let dt = 1.0 / 30.0;
    let _ = std::fs::create_dir_all(dir);
    let save = |name: &str| {
        let img = get_screen_data();
        img.export_png(&format!("{}/{}.png", dir, name));
    };
    if force.is_empty() {
        let mut demo = new_demo();
        let menu = MenuState { sel: 0, diff: 1, show_controls: false };
        for f in 0..40 {
            demo.update(dt, &None);
            demo.update_camera(dt, true);
            demo.draw_world(gfx, arena, dt);
            set_default_camera();
            draw_menu(&menu, font, &demo);
            if f == 39 {
                save("menu");
            }
            next_frame().await;
        }
        let mut g = new_game(Mode::Watch);
        let shots = [2.0f32, 5.0, 9.0, 14.0, 20.0, 27.0, 35.0, 45.0, 60.0, 75.0, 90.0];
        let mut k = 0;
        let mut frame = 0;
        while k < shots.len() {
            g.update(dt, &None);
            g.update_camera(dt, false);
            g.draw_world(gfx, arena, dt);
            set_default_camera();
            draw_hud(&g, font, false);
            frame += 1;
            if g.time >= shots[k] {
                save(&format!("fight_{:02}", k));
                println!("t={:.1} st0={:?} st1={:?} hp={:.0}/{:.0}", g.time, g.w[0].st, g.w[1].st, g.w[0].health, g.w[1].health);
                k += 1;
            }
            if frame > 4000 {
                break;
            }
            next_frame().await;
        }
    } else {
        // Erzwungener Wurf zur Kontrolle der Animation
        let mut g = new_game(Mode::Watch);
        g.intro = 0.0;
        g.w[0].pos = vec2(0.4, 0.0);
        g.w[0].yaw = -std::f32::consts::FRAC_PI_2;
        g.w[1].pos = vec2(0.4 - wrestler_lock_d(), 0.0);
        g.w[1].yaw = std::f32::consts::FRAC_PI_2;
        g.cam.yaw = 0.0;
        g.cam.pitch = std::env::var("HOS_PITCH").ok().and_then(|v| v.parse().ok()).unwrap_or(0.2);
        g.cam.zoom = std::env::var("HOS_ZOOM").ok().and_then(|v| v.parse().ok()).unwrap_or(0.75);
        if let Ctrl::Cpu(ai) = &mut g.w[0].ctrl {
            ai.cd = 100.0;
        }
        if let Ctrl::Cpu(ai) = &mut g.w[1].ctrl {
            ai.cd = 100.0;
            ai.diff = 0.0;
        }
        g.referee.pos = vec2(2.6, -2.6);
        g.debug_force(&force);
        let mut t = 0.0;
        let mut n = 0;
        while t < 3.2 {
            g.update(dt, &None);
            g.update_camera(dt, false);
            g.draw_world(gfx, arena, dt);
            set_default_camera();
            draw_hud(&g, font, false);
            t += dt;
            let step = (t / 0.2).floor() as i32;
            if step > n {
                n = step;
                save(&format!("{}_{:02}", force, n));
            }
            next_frame().await;
        }
    }
}

fn wrestler_lock_d() -> f32 {
    anim::LOCK_D
}
