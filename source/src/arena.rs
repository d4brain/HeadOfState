//! Ring, Seile, Halle, Publikum, Licht-Traversen, Bühne.
use crate::mesh::*;
use crate::model::shade;
use macroquad::models::Mesh;
use macroquad::prelude::*;

pub const MAT_H: f32 = 1.2;
pub const ROPE: f32 = 3.05;
pub const APRON: f32 = 3.65;
pub const POST: f32 = 3.22;
pub const ROPE_H: [f32; 3] = [0.52, 0.94, 1.36];

pub struct Arena {
    pub statics: Vec<Mesh>,
    pub textured: Vec<Mesh>,
    pub crowd: Vec<Mesh>,
    pub flash_spots: Vec<Vec3>,
}

pub struct Textures {
    pub _rts: Vec<RenderTarget>,
    pub mat: Texture2D,
    pub apron: Texture2D,
    pub tron: Texture2D,
}

fn rt_begin(w: u32, h: u32) -> RenderTarget {
    let rt = render_target(w, h);
    rt.texture.set_filter(FilterMode::Linear);
    let mut cam = Camera2D::from_display_rect(Rect::new(0., 0., w as f32, h as f32));
    cam.render_target = Some(rt.clone());
    set_camera(&cam);
    rt
}

fn centered(text: &str, font: &Font, cx: f32, y: f32, size: u16, color: Color) {
    let d = measure_text(text, Some(font), size, 1.0);
    draw_text_ex(text, cx - d.width / 2., y, TextParams { font: Some(font), font_size: size, color, ..Default::default() });
}

pub async fn make_textures(font: &Font) -> Textures {
    // Matte mit Logo
    let mat = rt_begin(1024, 1024);
    clear_background(Color::new(0.50, 0.50, 0.49, 1.0));
    for k in 0..40 {
        let a = k as f32 * 0.7;
        draw_circle(512. + a.sin() * 300. * (k as f32 * 0.37).cos(), 512. + a.cos() * 280., 30. + (k % 7) as f32 * 9., Color::new(0.44, 0.44, 0.43, 0.2));
    }
    draw_rectangle_lines(18., 18., 988., 988., 10., Color::new(0.10, 0.12, 0.25, 0.85));
    draw_poly(512., 512., 96, 300., 0., Color::new(0.08, 0.10, 0.22, 0.92));
    draw_poly(512., 512., 96, 284., 0., Color::new(0.62, 0.06, 0.09, 0.95));
    draw_poly(512., 512., 96, 262., 0., Color::new(0.08, 0.10, 0.22, 0.98));
    for k in 0..5 {
        let a = -std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::TAU / 5.0;
        let p = vec2(512. + a.cos() * 236., 512. + a.sin() * 236.);
        draw_poly(p.x, p.y, 5, 12., a.to_degrees(), Color::new(0.85, 0.68, 0.25, 1.0));
    }
    centered("HEAD", font, 512., 470., 150, Color::new(0.93, 0.93, 0.9, 1.0));
    centered("OF STATE", font, 512., 600., 110, Color::new(0.85, 0.68, 0.25, 1.0));
    centered("WORLD CATCH FEDERATION", font, 512., 680., 34, Color::new(0.9, 0.9, 0.9, 0.9));

    set_default_camera();
    next_frame().await;
    // Ring-Schürze
    let apron = rt_begin(1024, 160);
    clear_background(Color::new(0.03, 0.03, 0.05, 1.0));
    draw_rectangle(0., 0., 1024., 10., Color::new(0.62, 0.06, 0.09, 1.0));
    draw_rectangle(0., 150., 1024., 10., Color::new(0.10, 0.18, 0.55, 1.0));
    centered("HEAD OF STATE", font, 512., 112., 84, Color::new(0.95, 0.95, 0.95, 1.0));
    centered("★", font, 120., 108., 60, Color::new(0.85, 0.68, 0.25, 1.0));
    centered("★", font, 904., 108., 60, Color::new(0.85, 0.68, 0.25, 1.0));
    draw_poly(120., 85., 5, 26., -90., Color::new(0.85, 0.68, 0.25, 1.0));
    draw_poly(904., 85., 5, 26., -90., Color::new(0.85, 0.68, 0.25, 1.0));

    set_default_camera();
    next_frame().await;
    // Videowand
    let tron = rt_begin(1024, 448);
    clear_background(Color::new(0.02, 0.02, 0.06, 1.0));
    for k in 0..14 {
        let y = k as f32 * 34.;
        draw_rectangle(0., y, 1024., 2., Color::new(0.3, 0.1, 0.5, 0.25));
    }
    draw_triangle(vec2(0., 448.), vec2(380., 448.), vec2(0., 120.), Color::new(0.55, 0.05, 0.08, 0.9));
    draw_triangle(vec2(1024., 448.), vec2(644., 448.), vec2(1024., 120.), Color::new(0.08, 0.16, 0.6, 0.9));
    centered("HEAD OF STATE", font, 512., 210., 120, Color::new(1.0, 1.0, 1.0, 1.0));
    centered("DER PRÄSIDENT  vs  DER KANZLER", font, 512., 300., 46, Color::new(0.95, 0.75, 0.3, 1.0));
    centered("LIVE · MAIN EVENT", font, 512., 370., 32, Color::new(0.8, 0.8, 0.9, 1.0));
    set_default_camera();
    next_frame().await;
    Textures { mat: mat.texture.clone(), apron: apron.texture.clone(), tron: tron.texture.clone(), _rts: vec![mat, apron, tron] }
}

struct R(u32);
impl R {
    fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32
    }
}

pub fn build_arena(tx: &Textures) -> Arena {
    let mut m = MB::new();
    let mut tm: Vec<Mesh> = Vec::new();

    // Hallenboden
    m.boxc(vec3(0., -0.05, 0.), vec3(70., 0.05, 70.), hex(0x16161a, 0.2));
    // Schutzmatten
    m.boxc(vec3(0., 0.02, 0.), vec3(7.0, 0.02, 7.0), hex(0x0b1020, 0.15));
    // Ringkörper
    m.boxc(vec3(0., MAT_H * 0.5, 0.), vec3(APRON - 0.01, MAT_H * 0.5 - 0.005, APRON - 0.01), hex(0x08080c, 0.2));
    // Matten-Rand (Polsterkante)
    for s in [-1.0f32, 1.0] {
        m.tube(vec3(-APRON, MAT_H - 0.03, s * APRON), vec3(APRON, MAT_H - 0.03, s * APRON), 0.035, 8, hex(0x0a0a10, 0.3));
        m.tube(vec3(s * APRON, MAT_H - 0.03, -APRON), vec3(s * APRON, MAT_H - 0.03, APRON), 0.035, 8, hex(0x0a0a10, 0.3));
    }

    // Ringpfosten und Spannschlösser
    let steel = hex(0xb8bcc4, 0.95);
    let corner_cols = [hex(0x9c0f18, 0.45), hex(0xe8e8e8, 0.4), hex(0x123a8c, 0.45), hex(0xe8e8e8, 0.4)];
    let corners = [vec2(1., 1.), vec2(-1., 1.), vec2(-1., -1.), vec2(1., -1.)];
    for (k, c) in corners.iter().enumerate() {
        let p = vec3(c.x * POST, 0.0, c.y * POST);
        m.cylinder(p, MAT_H + 1.55, 0.06, 14, steel);
        m.cylinder(p + vec3(0., MAT_H + 1.55, 0.), 0.04, 0.075, 14, hex(0x202020, 0.6));
        let inward = -vec3(c.x, 0., c.y).normalize();
        let rot = Quat::from_rotation_y(inward.x.atan2(inward.z));
        for h in ROPE_H {
            let pc = p + vec3(0., MAT_H + h, 0.) + inward * 0.12;
            m.ellipsoid(Mat4::from_rotation_translation(rot, pc), Vec3::ZERO, vec3(0.13, 0.11, 0.09), 14, 8, &|_| corner_cols[k]);
            m.tube(p + vec3(0., MAT_H + h, 0.), pc, 0.02, 6, steel);
        }
    }

    // Ringtreppen an den neutralen Ecken
    for c in [vec2(-1., 1.), vec2(1., -1.)] {
        let dir = vec3(c.x, 0., c.y).normalize();
        let rot = Quat::from_rotation_y(dir.x.atan2(dir.z));
        for st in 0..3 {
            let h = 0.4 * (st + 1) as f32;
            let d = 4.35 + 0.0 - st as f32 * 0.35;
            let center = vec3(c.x * POST, 0., c.y * POST) * 0.0 + dir * (d + 0.6);
            let center = vec3(center.x, h * 0.5, center.z);
            m.boxm(Mat4::from_rotation_translation(rot, center), vec3(0.55, h * 0.5, 0.18), hex(0x2a2d33, 0.6));
        }
    }

    // Absperrgitter
    let bar = 7.6;
    for side in 0..4 {
        for k in -7..7 {
            let a = k as f32 * 1.08 + 0.54;
            let (cx, cz, rot) = match side {
                0 => (a, bar, 0.0),
                1 => (a, -bar, 0.0),
                2 => (bar, a, std::f32::consts::FRAC_PI_2),
                _ => (-bar, a, std::f32::consts::FRAC_PI_2),
            };
            if side == 1 && a.abs() < 1.2 {
                continue; // Einlauf
            }
            let mm = Mat4::from_rotation_translation(Quat::from_rotation_y(rot), vec3(cx, 0.55, cz));
            m.boxm(mm, vec3(0.52, 0.5, 0.03), hex(0x1d1f24, 0.7));
            m.boxm(mm * Mat4::from_translation(vec3(0., 0.53, 0.)), vec3(0.54, 0.03, 0.05), hex(0x7d828c, 0.95));
        }
    }

    // Kommentatorentisch
    m.boxc(vec3(0., 0.4, 5.6), vec3(1.6, 0.4, 0.4), hex(0x0d0f16, 0.4));
    m.boxc(vec3(0., 0.82, 5.6), vec3(1.65, 0.02, 0.45), hex(0x22252e, 0.7));
    for x in [-1.0f32, 0.0, 1.0] {
        m.boxm(Mat4::from_rotation_translation(Quat::from_rotation_x(-0.3), vec3(x, 0.98, 5.75)), vec3(0.22, 0.14, 0.015), hex(0x111111, 0.9));
        m.w = -1.0;
        m.boxm(Mat4::from_rotation_translation(Quat::from_rotation_x(-0.3), vec3(x, 0.98, 5.73)), vec3(0.2, 0.12, 0.01), hex(0x3050a0, 0.0));
        m.w = 0.0;
    }

    // Tribünen
    let mut crowd = MB::new();
    let mut rng = R(1234567);
    let shirts = [0x8c1c24u32, 0x1e3f8a, 0x222222, 0xdddddd, 0x3c6e3c, 0xc8a02a, 0x5a2a6e, 0x777777, 0x2a6e8c, 0xaa5522, 0x101010];
    let skins = [0xe0b090u32, 0xc89070, 0x8a5a3a, 0x5a3a28, 0xf0c8a8];
    let mut flash_spots = Vec::new();
    for side in 0..4 {
        for row in 0..13 {
            let d = 9.2 + row as f32 * 0.9;
            let h = 0.45 + row as f32 * 0.52;
            let half = d + 0.4;
            // Stufe
            let (c, hs) = match side {
                0 => (vec3(0., h * 0.5, d + 0.35), vec3(half + 0.5, h * 0.5, 0.45)),
                1 => (vec3(0., h * 0.5, -d - 0.35), vec3(half + 0.5, h * 0.5, 0.45)),
                2 => (vec3(d + 0.35, h * 0.5, 0.), vec3(0.45, h * 0.5, half + 0.5)),
                _ => (vec3(-d - 0.35, h * 0.5, 0.), vec3(0.45, h * 0.5, half + 0.5)),
            };
            m.boxc(c, hs, hex(0x1a1b22, 0.1));
            let n = (2.0 * half / 0.62) as i32;
            for k in 0..n {
                let a = -half + (k as f32 + 0.5) * 0.62;
                if rng.f() < 0.08 {
                    continue;
                }
                if side == 1 && a.abs() < 3.5 {
                    continue;
                }
                let (p, face) = match side {
                    0 => (vec3(a, h, d + 0.3), 180.0f32),
                    1 => (vec3(a, h, -d - 0.3), 0.0),
                    2 => (vec3(d + 0.3, h, a), -90.0),
                    _ => (vec3(-d - 0.3, h, a), 90.0),
                };
                let p = p + vec3((rng.f() - 0.5) * 0.12, 0., (rng.f() - 0.5) * 0.12);
                let rot = Quat::from_rotation_y((face + (rng.f() - 0.5) * 20.0).to_radians());
                let shirt = hex(shirts[(rng.f() * shirts.len() as f32) as usize % shirts.len()], 0.1);
                let skin = hex(skins[(rng.f() * skins.len() as f32) as usize % skins.len()], 0.3);
                let hair = hex([0x1a1410u32, 0x3a2a1a, 0x8a6a3a, 0x555555, 0x0a0a0a][(rng.f() * 5.0) as usize % 5], 0.2);
                let sc = 0.92 + rng.f() * 0.16;
                let stand = rng.f() < 0.3;
                crowd.w = 1.0 + rng.f() * 50.0;
                let base = Mat4::from_scale_rotation_translation(Vec3::splat(sc), rot, p);
                let ty = if stand { 1.15 } else { 0.62 };
                crowd.ellipsoid(base, vec3(0., ty, 0.), vec3(0.2, 0.3, 0.13), 7, 5, &|_| shirt);
                crowd.ellipsoid(base, vec3(0., ty + 0.42, 0.01), vec3(0.095, 0.12, 0.105), 6, 5, &move |q: Vec3| {
                    if q.y > p.y + (ty + 0.46) * sc { hair } else { skin }
                });
                if stand {
                    crowd.ellipsoid(base, vec3(0., 0.45, 0.), vec3(0.15, 0.45, 0.11), 6, 4, &|_| hex(0x20242c, 0.1));
                    if rng.f() < 0.45 {
                        // Arme hoch
                        for s in [-1.0f32, 1.0] {
                            crowd.ellipsoid(base, vec3(s * 0.2, ty + 0.45, 0.04), vec3(0.05, 0.3, 0.05), 5, 4, &|_| shirt);
                        }
                        if rng.f() < 0.35 {
                            let sign = hex([0xf0f0f0u32, 0xf2d24a, 0xe04848, 0x48a0e0][(rng.f() * 4.0) as usize % 4], 0.1);
                            crowd.boxm(base * Mat4::from_translation(vec3(0., ty + 0.85, 0.06)), vec3(0.32, 0.2, 0.01), sign);
                        }
                    }
                } else {
                    crowd.ellipsoid(base, vec3(0., 0.42, 0.15), vec3(0.14, 0.1, 0.22), 6, 4, &|_| hex(0x20242c, 0.1));
                }
                if rng.f() < 0.12 {
                    flash_spots.push(p + vec3(0., ty * sc + 0.5, 0.));
                }
            }
        }
    }
    crowd.w = 0.0;

    // Licht-Traverse
    let truss = hex(0x2a2c30, 0.8);
    let ty = 10.0;
    for s in [-1.0f32, 1.0] {
        m.boxc(vec3(0., ty, s * 5.0), vec3(5.3, 0.18, 0.18), truss);
        m.boxc(vec3(s * 5.0, ty, 0.), vec3(0.18, 0.18, 5.3), truss);
    }
    for c in [vec2(1., 1.), vec2(-1., 1.), vec2(1., -1.), vec2(-1., -1.)] {
        let p = vec3(c.x * 5.0, ty - 0.5, c.y * 5.0);
        let dir = (vec3(0., 1.0, 0.) - p).normalize();
        let rot = Quat::from_rotation_arc(Vec3::NEG_Y, dir);
        let mm = Mat4::from_rotation_translation(rot, p);
        m.lathe(mm, &[(-0.25, 0.0, 0.0), (-0.24, 0.24, 0.24), (0.3, 0.2, 0.2), (0.32, 0.0, 0.0)], 14, &|_| hex(0x15161a, 0.7));
        m.w = -1.0;
        m.lathe(mm, &[(-0.27, 0.0, 0.0), (-0.26, 0.2, 0.2)], 14, &|_| Color::new(1.0, 0.97, 0.88, 1.0));
        m.w = 0.0;
        m.tube(p, vec3(c.x * 5.0, ty, c.y * 5.0), 0.04, 6, truss);
    }
    // Ketten zur Hallendecke
    for c in [vec2(1., 1.), vec2(-1., 1.), vec2(1., -1.), vec2(-1., -1.)] {
        m.tube(vec3(c.x * 5.0, ty, c.y * 5.0), vec3(c.x * 5.0, 24.0, c.y * 5.0), 0.03, 5, hex(0x333333, 0.6));
    }

    // Bühne mit Videowand
    m.boxc(vec3(0., 0.9, -27.0), vec3(9.0, 0.9, 3.0), hex(0x101015, 0.4));
    m.boxc(vec3(0., 0.6, -16.5), vec3(1.5, 0.6, 7.6), hex(0x14141a, 0.5));
    m.boxc(vec3(0., 6.6, -28.6), vec3(8.4, 4.4, 0.25), hex(0x08080a, 0.5));
    for x in [-8.6f32, 8.6] {
        m.boxc(vec3(x, 5.5, -28.0), vec3(0.3, 5.5, 0.3), truss);
    }
    m.w = -1.0;
    for x in [-7.5f32, -2.5, 2.5, 7.5] {
        m.ellipsoid(Mat4::IDENTITY, vec3(x, 11.3, -28.3), vec3(0.25, 0.12, 0.25), 10, 6, &|_| Color::new(0.75, 0.55, 1.0, 1.0));
    }
    m.w = 0.0;

    // Texturierte Flächen
    let mut mat = MB::new();
    let y = MAT_H + 0.004;
    mat.quad(
        [vec3(-APRON, y, -APRON), vec3(-APRON, y, APRON), vec3(APRON, y, APRON), vec3(APRON, y, -APRON)],
        [vec2(0., 1.), vec2(0., 0.), vec2(1., 0.), vec2(1., 1.)],
        Color::new(1.0, 1.0, 1.0, 0.18),
    );
    tm.extend(mat.build(Some(tx.mat.clone())));

    let mut ap = MB::new();
    let a = APRON + 0.002;
    let (y0, y1) = (0.25, MAT_H - 0.08);
    let wc = Color::new(1.0, 1.0, 1.0, 0.2);
    let uvs = [vec2(0., 0.), vec2(1., 0.), vec2(1., 1.), vec2(0., 1.)];
    ap.quad([vec3(-a + 0.4, y0, a), vec3(a - 0.4, y0, a), vec3(a - 0.4, y1, a), vec3(-a + 0.4, y1, a)], uvs, wc);
    ap.quad([vec3(a - 0.4, y0, -a), vec3(-a + 0.4, y0, -a), vec3(-a + 0.4, y1, -a), vec3(a - 0.4, y1, -a)], uvs, wc);
    ap.quad([vec3(a, y0, a - 0.4), vec3(a, y0, -a + 0.4), vec3(a, y1, -a + 0.4), vec3(a, y1, a - 0.4)], uvs, wc);
    ap.quad([vec3(-a, y0, -a + 0.4), vec3(-a, y0, a - 0.4), vec3(-a, y1, a - 0.4), vec3(-a, y1, -a + 0.4)], uvs, wc);
    tm.extend(ap.build(Some(tx.apron.clone())));

    let mut tr = MB::new();
    tr.w = -1.0;
    let z = -28.3;
    tr.quad(
        [vec3(-8.0, 2.6, z), vec3(8.0, 2.6, z), vec3(8.0, 10.6, z), vec3(-8.0, 10.6, z)],
        uvs,
        Color::new(1.0, 1.0, 1.0, 1.0),
    );
    tm.extend(tr.build(Some(tx.tron.clone())));

    Arena { statics: m.build(None), textured: tm, crowd: crowd.build(None), flash_spots }
}

/// Seile: Federsystem je Seite (0:+z, 1:-z, 2:+x, 3:-x)
pub struct Ropes {
    pub disp: [f32; 4],
    pub vel: [f32; 4],
    pub center: [f32; 4],
}

impl Ropes {
    pub fn new() -> Self {
        Ropes { disp: [0.; 4], vel: [0.; 4], center: [0.; 4] }
    }

    pub fn impulse(&mut self, side: usize, v: f32, along: f32) {
        self.vel[side] += v;
        self.center[side] = along;
    }

    pub fn push(&mut self, side: usize, d: f32, along: f32) {
        if d > self.disp[side] {
            self.disp[side] = d;
            self.vel[side] = self.vel[side].max(0.0);
            self.center[side] = along;
        }
    }

    pub fn update(&mut self, dt: f32) {
        for k in 0..4 {
            let acc = -160.0 * self.disp[k] - 6.0 * self.vel[k];
            self.vel[k] += acc * dt;
            self.disp[k] += self.vel[k] * dt;
            self.disp[k] = self.disp[k].clamp(-0.12, 0.45);
        }
    }

    pub fn mesh(&self) -> Mesh {
        let mut mb = MB::new();
        let cols = [hex(0x1a3aa0, 0.55), hex(0xeeeeee, 0.5), hex(0xb01020, 0.55)];
        let e = POST - 0.06;
        for side in 0..4 {
            for (k, h) in ROPE_H.iter().enumerate() {
                let y = MAT_H + h;
                let n = 14;
                let mut pts = Vec::new();
                for i in 0..=n {
                    let s = i as f32 / n as f32;
                    let a = -e + 2.0 * e * s;
                    let c = self.center[side].clamp(-e * 0.7, e * 0.7);
                    // Auslenkung mit Spitze nahe dem Kontaktpunkt
                    let u = if a < c { (a + e) / (c + e) } else { (e - a) / (e - c) };
                    let shape = (u.clamp(0., 1.) * std::f32::consts::FRAC_PI_2).sin();
                    let dd = self.disp[side] * shape * (0.6 + 0.2 * k as f32);
                    let sag = -0.015 * (std::f32::consts::PI * s).sin();
                    let p = match side {
                        0 => vec3(a, y + sag, e + dd),
                        1 => vec3(a, y + sag, -e - dd),
                        2 => vec3(e + dd, y + sag, a),
                        _ => vec3(-e - dd, y + sag, a),
                    };
                    pts.push(p);
                }
                // Röhre entlang der Polylinie
                for w in pts.windows(2) {
                    mb.tube(w[0], w[1], 0.022, 8, cols[k]);
                }
            }
        }
        let _ = shade;
        mb.build1()
    }
}
