//! Prozeduraler Mesh-Builder: Drehkörper (Lathe), Ellipsoide, Boxen, Röhren.
use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

pub fn c8(c: Color) -> [u8; 4] {
    [
        (c.r.clamp(0., 1.) * 255.) as u8,
        (c.g.clamp(0., 1.) * 255.) as u8,
        (c.b.clamp(0., 1.) * 255.) as u8,
        (c.a.clamp(0., 1.) * 255.) as u8,
    ]
}

/// Farbe mit Glanzwert (Alpha-Kanal = Spekular-Stärke im Shader)
pub fn col(r: f32, g: f32, b: f32, spec: f32) -> Color {
    Color::new(r, g, b, spec)
}

pub fn hex(h: u32, spec: f32) -> Color {
    Color::new(
        ((h >> 16) & 255) as f32 / 255.,
        ((h >> 8) & 255) as f32 / 255.,
        (h & 255) as f32 / 255.,
        spec,
    )
}

pub fn mixc(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

pub struct MB {
    pub v: Vec<Vertex>,
    pub i: Vec<u32>,
    /// w-Komponente der Normalen: 0 normal, >0.5 Publikum (Phase), <-0.5 emissiv
    pub w: f32,
}

impl MB {
    pub fn new() -> Self {
        MB { v: Vec::new(), i: Vec::new(), w: 0.0 }
    }

    fn vert(&self, p: Vec3, n: Vec3, c: Color, uv: Vec2) -> Vertex {
        Vertex { position: p, uv, color: c8(c), normal: vec4(n.x, n.y, n.z, self.w) }
    }

    fn smooth_normals(&mut self, v0: usize, i0: usize) {
        let mut acc = vec![Vec3::ZERO; self.v.len() - v0];
        for t in self.i[i0..].chunks(3) {
            let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
            let pa = self.v[a].position;
            let n = (self.v[b].position - pa).cross(self.v[c].position - pa);
            acc[a - v0] += n;
            acc[b - v0] += n;
            acc[c - v0] += n;
        }
        for (k, n) in acc.iter().enumerate() {
            let n = n.normalize_or_zero();
            let w = self.v[v0 + k].normal.w;
            self.v[v0 + k].normal = vec4(n.x, n.y, n.z, w);
        }
    }

    /// Drehkörper um die lokale Y-Achse. Profil: (y, rx, rz); rx=rz=0 => Pol.
    pub fn lathe(&mut self, m: Mat4, prof: &[(f32, f32, f32)], segs: usize, colf: &dyn Fn(Vec3) -> Color) {
        let mut prof: Vec<(f32, f32, f32)> = prof.to_vec();
        if prof.first().unwrap().0 > prof.last().unwrap().0 {
            prof.reverse();
        }
        let v0 = self.v.len();
        let i0 = self.i.len();
        let mut rings: Vec<(u32, bool)> = Vec::new();
        for &(y, rx, rz) in &prof {
            let start = self.v.len() as u32;
            if rx.abs() < 1e-6 && rz.abs() < 1e-6 {
                let p = m.transform_point3(vec3(0., y, 0.));
                let c = colf(p);
                let vx = self.vert(p, Vec3::Y, c, Vec2::ZERO);
                self.v.push(vx);
                rings.push((start, true));
            } else {
                for s in 0..segs {
                    let a = s as f32 / segs as f32 * TAU;
                    let p = m.transform_point3(vec3(a.sin() * rx, y, a.cos() * rz));
                    let c = colf(p);
                    let vx = self.vert(p, Vec3::Y, c, Vec2::ZERO);
                    self.v.push(vx);
                }
                rings.push((start, false));
            }
        }
        let sg = segs as u32;
        for k in 0..rings.len() - 1 {
            let (a, pa) = rings[k];
            let (b, pb) = rings[k + 1];
            if pa && pb {
                continue;
            }
            for s in 0..sg {
                let s1 = (s + 1) % sg;
                if pa {
                    self.i.extend_from_slice(&[a, b + s1, b + s]);
                } else if pb {
                    self.i.extend_from_slice(&[a + s, a + s1, b]);
                } else {
                    self.i.extend_from_slice(&[a + s, a + s1, b + s1, a + s, b + s1, b + s]);
                }
            }
        }
        self.smooth_normals(v0, i0);
    }

    pub fn ellipsoid(&mut self, m: Mat4, c: Vec3, r: Vec3, segs: usize, rings: usize, colf: &dyn Fn(Vec3) -> Color) {
        let mut prof = Vec::with_capacity(rings + 1);
        for k in 0..=rings {
            let phi = PI * k as f32 / rings as f32;
            let s = if k == 0 || k == rings { 0.0 } else { phi.sin() };
            prof.push((-phi.cos() * r.y, r.x * s, r.z * s));
        }
        self.lathe(m * Mat4::from_translation(c), &prof, segs, colf);
    }

    pub fn ell(&mut self, c: Vec3, r: Vec3, color: Color) {
        self.ellipsoid(Mat4::IDENTITY, c, r, 14, 10, &|_| color);
    }

    pub fn ell_rot(&mut self, c: Vec3, r: Vec3, rot: Quat, color: Color) {
        self.ellipsoid(Mat4::from_rotation_translation(rot, c), Vec3::ZERO, r, 14, 10, &|_| color);
    }

    /// Glattes Profil per Catmull-Rom-Interpolation, mit Polkappen.
    pub fn smooth_limb(&mut self, m: Mat4, pts: &[(f32, f32, f32)], sub: usize, segs: usize, colf: &dyn Fn(Vec3) -> Color) {
        let n = pts.len();
        let mut prof = Vec::new();
        let first = pts[0];
        let last = pts[n - 1];
        let dir = if last.0 < first.0 { -1.0 } else { 1.0 };
        // Kappe am Anfang
        prof.push((first.0 - dir * first.1.min(first.2) * 0.75, 0.0, 0.0));
        prof.push((first.0 - dir * first.1.min(first.2) * 0.45, first.1 * 0.8, first.2 * 0.8));
        for k in 0..n - 1 {
            let p0 = pts[if k == 0 { 0 } else { k - 1 }];
            let p1 = pts[k];
            let p2 = pts[k + 1];
            let p3 = pts[(k + 2).min(n - 1)];
            for s in 0..sub {
                let t = s as f32 / sub as f32;
                let cr = |a: f32, b: f32, c: f32, d: f32| {
                    0.5 * ((2. * b) + (-a + c) * t + (2. * a - 5. * b + 4. * c - d) * t * t + (-a + 3. * b - 3. * c + d) * t * t * t)
                };
                prof.push((cr(p0.0, p1.0, p2.0, p3.0), cr(p0.1, p1.1, p2.1, p3.1), cr(p0.2, p1.2, p2.2, p3.2)));
            }
        }
        prof.push(last);
        prof.push((last.0 + dir * last.1.min(last.2) * 0.45, last.1 * 0.8, last.2 * 0.8));
        prof.push((last.0 + dir * last.1.min(last.2) * 0.75, 0.0, 0.0));
        self.lathe(m, &prof, segs, colf);
    }

    /// Quader mit harten Kanten, zentriert im Ursprung von m.
    pub fn boxm(&mut self, m: Mat4, h: Vec3, color: Color) {
        let faces: [(Vec3, Vec3, Vec3); 6] = [
            (Vec3::X, Vec3::Y, Vec3::Z),
            (-Vec3::X, Vec3::Y, -Vec3::Z),
            (Vec3::Y, Vec3::Z, Vec3::X),
            (-Vec3::Y, Vec3::Z, -Vec3::X),
            (Vec3::Z, Vec3::Y, -Vec3::X),
            (-Vec3::Z, Vec3::Y, Vec3::X),
        ];
        for (n, u, v) in faces {
            let base = self.v.len() as u32;
            let c = n * h;
            let uu = u * (u.abs().dot(h));
            let vv = v * (v.abs().dot(h));
            let corners = [c - uu - vv, c + uu - vv, c + uu + vv, c - uu + vv];
            let nw = m.transform_vector3(n).normalize();
            for (k, p) in corners.iter().enumerate() {
                let uv = [vec2(0., 1.), vec2(1., 1.), vec2(1., 0.), vec2(0., 0.)][k];
                let vx = self.vert(m.transform_point3(*p), nw, color, uv);
                self.v.push(vx);
            }
            self.i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    pub fn boxc(&mut self, c: Vec3, h: Vec3, color: Color) {
        self.boxm(Mat4::from_translation(c), h, color);
    }

    /// Röhre zwischen zwei Punkten (ohne Deckel)
    pub fn tube(&mut self, p0: Vec3, p1: Vec3, r: f32, segs: usize, color: Color) {
        let d = p1 - p0;
        let len = d.length();
        if len < 1e-5 {
            return;
        }
        let rot = Quat::from_rotation_arc(Vec3::Y, d / len);
        let m = Mat4::from_rotation_translation(rot, p0);
        self.lathe(m, &[(0.0, r, r), (len, r, r)], segs, &|_| color);
    }

    /// Zylinder mit leicht gefasten Deckeln
    pub fn cylinder(&mut self, base: Vec3, h: f32, r: f32, segs: usize, color: Color) {
        let b = (r * 0.12).min(0.03);
        let prof = [(0.0, 0.0, 0.0), (0.0, r - b, r - b), (b, r, r), (h - b, r, r), (h, r - b, r - b), (h, 0.0, 0.0)];
        self.lathe(Mat4::from_translation(base), &prof, segs, &|_| color);
    }

    /// Texturiertes Viereck (p0..p3 gegen den Uhrzeigersinn), UV (0,0) oben links
    pub fn quad(&mut self, p: [Vec3; 4], uv: [Vec2; 4], color: Color) {
        let n = (p[1] - p[0]).cross(p[3] - p[0]).normalize();
        let base = self.v.len() as u32;
        for k in 0..4 {
            let vx = self.vert(p[k], n, color, uv[k]);
            self.v.push(vx);
        }
        self.i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn append(&mut self, o: &MB) {
        let base = self.v.len() as u32;
        self.v.extend_from_slice(&o.v);
        self.i.extend(o.i.iter().map(|k| k + base));
    }

    /// In macroquad-Meshes (u16-Indizes) aufteilen
    pub fn build(&self, tex: Option<Texture2D>) -> Vec<Mesh> {
        let mut out = Vec::new();
        let mut map: HashMap<u32, u16> = HashMap::new();
        let mut cv: Vec<Vertex> = Vec::new();
        let mut ci: Vec<u16> = Vec::new();
        for t in self.i.chunks(3) {
            if cv.len() + 3 >= 30000 || ci.len() + 3 >= 90000 {
                out.push(Mesh { vertices: std::mem::take(&mut cv), indices: std::mem::take(&mut ci), texture: tex.clone() });
                map.clear();
            }
            for &k in t {
                let idx = *map.entry(k).or_insert_with(|| {
                    cv.push(self.v[k as usize]);
                    (cv.len() - 1) as u16
                });
                ci.push(idx);
            }
        }
        if !ci.is_empty() {
            out.push(Mesh { vertices: cv, indices: ci, texture: tex });
        }
        out
    }

    pub fn build1(&self) -> Mesh {
        let mut v = self.build(None);
        if v.is_empty() {
            return Mesh { vertices: vec![], indices: vec![], texture: None };
        }
        v.swap_remove(0)
    }
}
