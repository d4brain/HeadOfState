//! HUD, Menü und Overlays
use crate::game::*;
use crate::wrestler::*;
use macroquad::prelude::*;

pub fn text(t: &str, font: &Font, x: f32, y: f32, size: f32, color: Color) {
    draw_text_ex(t, x, y, TextParams { font: Some(font), font_size: size as u16, color, ..Default::default() });
}

pub fn text_c(t: &str, font: &Font, cx: f32, y: f32, size: f32, color: Color) {
    let d = measure_text(t, Some(font), size as u16, 1.0);
    text(t, font, cx - d.width / 2.0, y, size, color);
}

pub fn text_shadow_c(t: &str, font: &Font, cx: f32, y: f32, size: f32, color: Color) {
    text_c(t, font, cx + size * 0.04, y + size * 0.05, size, Color::new(0., 0., 0., 0.7 * color.a));
    text_c(t, font, cx, y, size, color);
}

fn bar(x: f32, y: f32, w: f32, h: f32, v: f32, col: Color, right: bool, glow: bool) {
    let v = v.clamp(0.0, 1.0);
    draw_rectangle(x - 2., y - 2., w + 4., h + 4., Color::new(0., 0., 0., 0.65));
    draw_rectangle(x, y, w, h, Color::new(0.12, 0.12, 0.14, 0.9));
    let fw = w * v;
    let fx = if right { x + w - fw } else { x };
    draw_rectangle(fx, y, fw, h, col);
    draw_rectangle(fx, y, fw, h * 0.35, Color::new(1., 1., 1., 0.18));
    if glow {
        let a = 0.35 + 0.35 * (get_time() as f32 * 8.0).sin();
        draw_rectangle_lines(x - 3., y - 3., w + 6., h + 6., 3.0, Color::new(1.0, 0.85, 0.3, a));
    }
}

pub fn draw_hud(g: &Game, font: &Font, help: bool) {
    let sw = screen_width();
    let sh = screen_height();
    let s = (sh / 900.0).max(0.6);
    let bw = 520.0 * s;
    let margin = 36.0 * s;
    for (i, w) in g.w.iter().enumerate() {
        let right = i == 1;
        let x = if right { sw - margin - bw } else { margin };
        let y = 34.0 * s;
        let hc = if w.health > 50.0 {
            Color::new(0.85, 0.15, 0.17, 1.0)
        } else if w.health > 25.0 {
            Color::new(0.95, 0.45, 0.1, 1.0)
        } else {
            Color::new(1.0, 0.85, 0.2, 1.0)
        };
        let name_y = y + 2.0;
        let nm = &w.name;
        let ctrl = match &w.ctrl {
            Ctrl::Human(_) => if i == 0 { "P1" } else { "P2" },
            Ctrl::Cpu(_) => "CPU",
        };
        let label = format!("{}  ·  {}", nm, ctrl);
        let d = measure_text(&label, Some(font), (24.0 * s) as u16, 1.0);
        let tx = if right { x + bw - d.width } else { x };
        text(&label, font, tx + 2., name_y + 2., 24.0 * s, Color::new(0., 0., 0., 0.7));
        text(&label, font, tx, name_y, 24.0 * s, WHITE);
        draw_rectangle(if right { x + bw - 6. * s } else { x }, name_y + 8. * s, 6. * s, 4. * s, w.color);
        bar(x, y + 16.0 * s, bw, 22.0 * s, w.health / 100.0, hc, right, false);
        bar(x, y + 44.0 * s, bw * 0.62 + 0.0, 8.0 * s, w.stamina / 100.0, Color::new(0.95, 0.8, 0.25, 1.0), false, false);
        let sx = if right { x } else { x };
        let _ = sx;
        let sp_full = w.special >= 100.0;
        let spx = if right { x } else { x + bw * 0.66 };
        bar(spx, y + 44.0 * s, bw * 0.34, 8.0 * s, w.special / 100.0, if sp_full { Color::new(1.0, 0.78, 0.2, 1.0) } else { Color::new(0.35, 0.55, 1.0, 1.0) }, right, sp_full);
        if sp_full {
            let lbl = "FINISHER BEREIT";
            let d = measure_text(lbl, Some(font), (16.0 * s) as u16, 1.0);
            let lx = if right { x } else { x + bw - d.width };
            text(lbl, font, lx, y + 74.0 * s, 16.0 * s, Color::new(1.0, 0.85, 0.3, 1.0));
        }
    }
    // Uhr / Titel
    text_shadow_c("HEAD OF STATE", font, sw / 2.0, 42.0 * s, 26.0 * s, Color::new(1.0, 1.0, 1.0, 0.9));
    let mins = (g.time / 60.0) as i32;
    let secs = (g.time % 60.0) as i32;
    text_shadow_c(&format!("{:02}:{:02}", mins, secs), font, sw / 2.0, 72.0 * s, 22.0 * s, Color::new(0.85, 0.85, 0.9, 0.85));

    // Meldungen
    let mut y_small = sh * 0.24;
    for m in &g.msgs {
        let a = if m.t < 1.7 { 1.0 } else { 1.0 - (m.t - 1.7) / 0.5 };
        let pop = 1.0 + (1.0 - (m.t * 6.0).min(1.0)) * 0.35;
        let col = Color::new(m.color.r, m.color.g, m.color.b, a.max(0.0));
        if m.big {
            text_shadow_c(&m.text, font, sw / 2.0, sh * 0.42, 74.0 * s * pop, col);
        } else {
            text_shadow_c(&m.text, font, sw / 2.0, y_small, 34.0 * s * pop.min(1.15), col);
            y_small += 44.0 * s;
        }
    }

    // Pin-Zählung
    if let Some(pin) = &g.pin {
        if pin.count > 0 {
            let n = pin.count.min(3);
            let size = 150.0 * s * (1.0 + g.count_pop * 0.5);
            text_shadow_c(&format!("{}", n), font, sw / 2.0, sh * 0.5, size, Color::new(1.0, 1.0, 1.0, 0.95));
        }
        let v = 1 - pin.att;
        if g.w[v].is_human() {
            let x = sw / 2.0 - 220.0 * s;
            let y = sh * 0.62;
            text_shadow_c("TASTEN HÄMMERN ZUM BEFREIEN!", font, sw / 2.0, y - 14.0 * s, 28.0 * s, Color::new(1.0, 0.85, 0.3, 1.0));
            bar(x, y, 440.0 * s, 18.0 * s, pin.kick / 100.0, Color::new(0.3, 0.9, 0.4, 1.0), false, false);
        }
    }
    for (i, w) in g.w.iter().enumerate() {
        if w.is_human() && matches!(w.st, St::Down { .. }) && g.over.is_none() && g.pin.is_none() {
            let lbl = if g.w.iter().filter(|w| w.is_human()).count() > 1 { format!("P{}: Tasten hämmern zum Aufstehen", i + 1) } else { "Tasten hämmern zum Aufstehen".to_string() };
            text_shadow_c(&lbl, font, sw / 2.0, sh * 0.72 + i as f32 * 34.0 * s, 24.0 * s, Color::new(0.9, 0.9, 1.0, 0.8 + 0.2 * (get_time() as f32 * 6.0).sin()));
        }
    }

    if g.intro > 0.0 {
        let a = (g.intro / 0.6).min(1.0);
        text_shadow_c("KAMPF!", font, sw / 2.0, sh * 0.45, 110.0 * s * (1.0 + (2.2 - g.intro) * 0.1), Color::new(1.0, 0.85, 0.3, a));
    }

    if let Some((winner, t)) = g.over {
        if t > 1.5 {
            let a = ((t - 1.5) / 0.6).min(1.0);
            draw_rectangle(0., sh * 0.62, sw, 120.0 * s, Color::new(0., 0., 0., 0.55 * a));
            text_shadow_c(&format!("{} GEWINNT DURCH PINFALL", g.w[winner].name), font, sw / 2.0, sh * 0.62 + 55.0 * s, 40.0 * s, Color::new(1.0, 0.85, 0.3, a));
            text_c("ENTER  Revanche     ·     ESC  Hauptmenü", font, sw / 2.0, sh * 0.62 + 98.0 * s, 22.0 * s, Color::new(1.0, 1.0, 1.0, a * 0.85));
        }
    }

    if help {
        draw_help(g.w.iter().filter(|w| w.is_human()).count() > 1, font);
    } else {
        text("F1  Steuerung   ·   Q/E oder rechte Maus  Kamera   ·   Mausrad  Zoom   ·   P  Pause   ·   ESC  Menü", font, 16.0 * s, sh - 14.0 * s, 16.0 * s, Color::new(1., 1., 1., 0.45));
    }
}

pub fn draw_help(two: bool, font: &Font) {
    let sw = screen_width();
    let sh = screen_height();
    let s = (sh / 900.0).max(0.6);
    let w = if two { 1080.0 } else { 600.0 } * s;
    let h = 400.0 * s;
    let x = (sw - w) / 2.0;
    let y = sh - h - 40.0 * s;
    draw_rectangle(x, y, w, h, Color::new(0.02, 0.02, 0.05, 0.82));
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(1.0, 0.85, 0.3, 0.6));
    let rows = [
        ("Bewegen", "W A S D", "Pfeiltasten"),
        ("Rennen", "Shift links", "Shift rechts"),
        ("Schlag / Stampfer / Clothesline", "J", "Num1 / ,"),
        ("Tritt / Knee Drop", "K", "Num2 / ."),
        ("Griff (Clinch) / Irish Whip", "L", "Num3 / -"),
        ("Pin / Posen / Finisher im Clinch", "Leertaste", "Num0 / Ctrl rechts"),
        ("Blocken (halten)", "I / Ctrl links", "Num5 / M"),
    ];
    text("STEUERUNG", font, x + 24. * s, y + 42. * s, 30. * s, Color::new(1.0, 0.85, 0.3, 1.0));
    text("P1", font, x + 370. * s, y + 42. * s, 22. * s, WHITE);
    if two {
        text("P2", font, x + 600. * s, y + 42. * s, 22. * s, WHITE);
    }
    for (k, (a, b, c)) in rows.iter().enumerate() {
        let yy = y + 80. * s + k as f32 * 32. * s;
        text(a, font, x + 24. * s, yy, 19. * s, Color::new(0.85, 0.85, 0.9, 1.0));
        text(b, font, x + 370. * s, yy, 19. * s, WHITE);
        if two {
            text(c, font, x + 600. * s, yy, 19. * s, WHITE);
        }
    }
    let yy = y + 80. * s + 7.0 * 32. * s + 10. * s;
    text("Im Clinch:  Schlag = Body Slam · Tritt = Suplex · Griff = Irish Whip", font, x + 24. * s, yy, 17. * s, Color::new(1.0, 0.85, 0.3, 0.9));
    text("Finisher-Leiste voll + Pin-Taste im Clinch = HEAD OF STATE (DDT)", font, x + 24. * s, yy + 26. * s, 17. * s, Color::new(1.0, 0.85, 0.3, 0.9));
}

pub struct MenuState {
    pub sel: usize,
    pub diff: usize,
    pub show_controls: bool,
}

pub const MENU: [&str; 5] = ["1 SPIELER  vs  CPU", "2 SPIELER  (LOKAL)", "CPU  vs  CPU", "STEUERUNG", "BEENDEN"];
pub const DIFFS: [&str; 3] = ["LEICHT", "MITTEL", "SCHWER"];

pub fn draw_menu(m: &MenuState, font: &Font, demo: &Game) {
    let sw = screen_width();
    let sh = screen_height();
    let s = (sh / 900.0).max(0.6);
    // Vignette
    draw_rectangle(0., 0., sw * 0.46, sh, Color::new(0., 0., 0., 0.55));
    draw_rectangle(sw * 0.46, 0., 60. * s, sh, Color::new(0., 0., 0., 0.3));
    let x = 80.0 * s;
    text("HEAD", font, x + 4., 190. * s + 5., 120. * s, Color::new(0., 0., 0., 0.6));
    text("HEAD", font, x, 190. * s, 120. * s, WHITE);
    text("OF STATE", font, x + 4., 280. * s + 5., 84. * s, Color::new(0., 0., 0., 0.6));
    text("OF STATE", font, x, 280. * s, 84. * s, Color::new(0.85, 0.68, 0.25, 1.0));
    text("CATCH-WRESTLING  ·  3D", font, x + 4., 322. * s, 22. * s, Color::new(0.8, 0.8, 0.85, 0.85));
    draw_rectangle(x, 340. * s, 300. * s, 4. * s, Color::new(0.62, 0.06, 0.09, 1.0));
    for (k, item) in MENU.iter().enumerate() {
        let y = 420. * s + k as f32 * 56. * s;
        let sel = k == m.sel;
        let col = if sel { Color::new(1.0, 0.85, 0.3, 1.0) } else { Color::new(0.85, 0.85, 0.9, 0.85) };
        if sel {
            draw_rectangle(x - 22. * s, y - 30. * s, 8. * s, 36. * s, Color::new(1.0, 0.85, 0.3, 1.0));
        }
        let label = if k == 0 { format!("{}   < {} >", item, DIFFS[m.diff]) } else { item.to_string() };
        text(&label, font, x, y, if sel { 34. * s } else { 30. * s }, col);
    }
    text("PFEILE hoch/runter wählen   ·   links/rechts Schwierigkeit   ·   ENTER starten", font, x, sh - 40. * s, 18. * s, Color::new(1., 1., 1., 0.5));
    text(&format!("{}  vs  {}", demo.w[0].name, demo.w[1].name), font, sw - 520. * s, sh - 40. * s, 20. * s, Color::new(1., 1., 1., 0.45));
    if m.show_controls {
        draw_help(true, font);
    }
}
