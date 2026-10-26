//! Skelett, Posen, prozedurale Bewegung und Keyframe-Animationen.
use macroquad::prelude::*;
use std::f32::consts::PI;
use std::sync::OnceLock;

pub const NJ: usize = 19;
pub const PELVIS: usize = 0;
pub const SPINE: usize = 1;
pub const CHEST: usize = 2;
pub const NECK: usize = 3;
pub const HEAD: usize = 4;
pub const CLAV_L: usize = 5;
pub const UARM_L: usize = 6;
pub const FARM_L: usize = 7;
pub const HAND_L: usize = 8;
pub const CLAV_R: usize = 9;
pub const UARM_R: usize = 10;
pub const FARM_R: usize = 11;
pub const HAND_R: usize = 12;
pub const THIGH_L: usize = 13;
pub const SHIN_L: usize = 14;
pub const FOOT_L: usize = 15;
pub const THIGH_R: usize = 16;
pub const SHIN_R: usize = 17;
pub const FOOT_R: usize = 18;

pub const PARENT: [i32; NJ] = [-1, 0, 1, 2, 3, 2, 5, 6, 7, 2, 9, 10, 11, 0, 13, 14, 0, 16, 17];
const MIRROR: [usize; NJ] = [0, 1, 2, 3, 4, 9, 10, 11, 12, 5, 6, 7, 8, 16, 17, 18, 13, 14, 15];

pub const DEG: f32 = PI / 180.0;
pub const LIE_Y: f32 = 0.14;
/// Abstand der Körperzentren im Clinch
pub const LOCK_D: f32 = 0.74;

pub fn offsets() -> [Vec3; NJ] {
    [
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 0.10, 0.0),
        vec3(0.0, 0.20, 0.0),
        vec3(0.0, 0.24, 0.0),
        vec3(0.0, 0.10, 0.0),
        vec3(0.03, 0.20, -0.01),
        vec3(0.17, -0.01, 0.0),
        vec3(0.0, -0.30, 0.0),
        vec3(0.0, -0.27, 0.0),
        vec3(-0.03, 0.20, -0.01),
        vec3(-0.17, -0.01, 0.0),
        vec3(0.0, -0.30, 0.0),
        vec3(0.0, -0.27, 0.0),
        vec3(0.105, -0.05, 0.0),
        vec3(0.0, -0.45, 0.0),
        vec3(0.0, -0.43, 0.0),
        vec3(-0.105, -0.05, 0.0),
        vec3(0.0, -0.45, 0.0),
        vec3(0.0, -0.43, 0.0),
    ]
}

pub fn qx(d: f32) -> Quat {
    Quat::from_rotation_x(d * DEG)
}
pub fn qy(d: f32) -> Quat {
    Quat::from_rotation_y(d * DEG)
}
pub fn qyx(y: f32, x: f32) -> Quat {
    qy(y) * qx(x)
}

#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub j: [Vec3; NJ],
    pub root: Vec3,
    pub rq: Quat,
}

impl Pose {
    pub fn new() -> Pose {
        Pose { j: [Vec3::ZERO; NJ], root: vec3(0.0, 1.0, 0.0), rq: Quat::IDENTITY }
    }
    /// Gelenkrotation in Grad setzen (Euler YXZ)
    pub fn r(mut self, k: usize, x: f32, y: f32, z: f32) -> Pose {
        self.j[k] = vec3(x, y, z) * DEG;
        self
    }
    pub fn add(mut self, k: usize, x: f32, y: f32, z: f32) -> Pose {
        self.j[k] += vec3(x, y, z) * DEG;
        self
    }
    pub fn at(mut self, x: f32, y: f32, z: f32) -> Pose {
        self.root = vec3(x, y, z);
        self
    }
    pub fn q(mut self, q: Quat) -> Pose {
        self.rq = q;
        self
    }
    /// Arme symmetrisch: Oberarm (x, z), Unterarm x
    pub fn arms(self, ux: f32, uz: f32, fx: f32) -> Pose {
        self.r(UARM_L, ux, 0., uz).r(UARM_R, ux, 0., -uz).r(FARM_L, fx, 0., 0.).r(FARM_R, fx, 0., 0.)
    }
    /// Beine symmetrisch: Oberschenkel, Knie, Fuss
    pub fn legs(self, th: f32, sh: f32, ft: f32) -> Pose {
        self.r(THIGH_L, th, 0., 3.).r(THIGH_R, th, 0., -3.).r(SHIN_L, sh, 0., 0.).r(SHIN_R, sh, 0., 0.).r(FOOT_L, ft, 0., 0.).r(FOOT_R, ft, 0., 0.)
    }
    pub fn lerp(a: &Pose, b: &Pose, t: f32) -> Pose {
        let mut o = *a;
        for k in 0..NJ {
            o.j[k] = a.j[k].lerp(b.j[k], t);
        }
        o.root = a.root.lerp(b.root, t);
        o.rq = a.rq.slerp(b.rq, t).normalize();
        o
    }
    /// Links/rechts spiegeln
    pub fn mirror(&self) -> Pose {
        let mut o = *self;
        for k in 0..NJ {
            let s = self.j[MIRROR[k]];
            o.j[k] = vec3(s.x, -s.y, -s.z);
        }
        o.root.x = -self.root.x;
        o.rq = Quat::from_xyzw(self.rq.x, -self.rq.y, -self.rq.z, self.rq.w);
        o
    }
}

pub fn qe(e: Vec3) -> Quat {
    Quat::from_euler(EulerRot::YXZ, e.y, e.x, e.z)
}

/// Vorwärtskinematik: (Rotation, Position) je Gelenk im Charakterraum
pub fn fk(p: &Pose, off: &[Vec3; NJ]) -> [(Quat, Vec3); NJ] {
    let mut out = [(Quat::IDENTITY, Vec3::ZERO); NJ];
    out[0] = (p.rq * qe(p.j[0]), p.root);
    for k in 1..NJ {
        let par = PARENT[k] as usize;
        let (pr, pp) = out[par];
        out[k] = (pr * qe(p.j[k]), pp + pr * off[k]);
    }
    out
}

/// Pose so verschieben, dass der tiefste Fuss den Boden berührt
pub fn ground(mut p: Pose, off: &[Vec3; NJ]) -> Pose {
    let f = fk(&p, off);
    let mut lo = f32::MAX;
    for k in [FOOT_L, FOOT_R] {
        let (q, pos) = f[k];
        // Ferse und Zehe des Schuhs prüfen
        for pt in [vec3(0., -0.075, -0.04), vec3(0., -0.075, 0.17)] {
            lo = lo.min((pos + q * pt).y);
        }
    }
    p.root.y -= lo;
    p
}

// ---------------------------------------------------------------- Prozedurale Posen

pub fn stance(t: f32) -> Pose {
    let b = (t * 2.3).sin();
    Pose::new()
        .at(0., 0.95, 0.)
        .r(SPINE, 12. + b * 1.2, 0., 0.)
        .r(CHEST, 6. + b * 1.8, 0., 0.)
        .r(NECK, -6., 0., 0.)
        .r(HEAD, -8., 0., 0.)
        .r(CLAV_L, 0., 0., 4. + b)
        .r(CLAV_R, 0., 0., -4. - b)
        .r(UARM_L, -36., 0., 22.)
        .r(FARM_L, -98., 0., 0.)
        .r(HAND_L, -10., 0., 0.)
        .r(UARM_R, -40., 0., -22.)
        .r(FARM_R, -94., 0., 0.)
        .r(HAND_R, -10., 0., 0.)
        .r(THIGH_L, -28., 0., 7.)
        .r(SHIN_L, 36., 0., 0.)
        .r(FOOT_L, -8., 0., -7.)
        .r(THIGH_R, -16., 0., -7.)
        .r(SHIN_R, 30., 0., 0.)
        .r(FOOT_R, -14., 0., 7.)
}

pub fn walk(phase: f32, run: f32) -> Pose {
    let s = phase.sin();
    let c = phase.cos();
    let amp = 26. + 22. * run;
    let kl = 10. + (45. + 45. * run) * c.max(0.).powf(1.3);
    let kr = 10. + (45. + 45. * run) * (-c).max(0.).powf(1.3);
    let tl = -s * amp - 8. - 10. * run;
    let tr = s * amp - 8. - 10. * run;
    let arm = 22. + 34. * run;
    Pose::new()
        .at(0., 1.0, 0.)
        .r(PELVIS, 0., s * 7., 0.)
        .r(SPINE, 6. + 14. * run, -s * 5., 0.)
        .r(CHEST, 2. + 4. * run, -s * 6., 0.)
        .r(HEAD, -4. - 8. * run, s * 4., 0.)
        .r(UARM_L, s * arm - 5., 0., 12.)
        .r(UARM_R, -s * arm - 5., 0., -12.)
        .r(FARM_L, -30. - 70. * run + s * 10., 0., 0.)
        .r(FARM_R, -30. - 70. * run - s * 10., 0., 0.)
        .r(THIGH_L, tl, 0., 3.)
        .r(THIGH_R, tr, 0., -3.)
        .r(SHIN_L, kl, 0., 0.)
        .r(SHIN_R, kr, 0., 0.)
        .r(FOOT_L, -(tl + kl) * 0.55, 0., 0.)
        .r(FOOT_R, -(tr + kr) * 0.55, 0., 0.)
}

/// Seitwärts-/Rückwärtsschritte im Kampfstand
pub fn shuffle(phase: f32, amt: f32, t: f32) -> Pose {
    let st = stance(t);
    let s = phase.sin();
    let c = phase.cos();
    let mut p = st;
    p.j[THIGH_L].x += (-s * 16. - 4. * c.max(0.)) * DEG * amt;
    p.j[THIGH_R].x += (s * 16. - 4. * (-c).max(0.)) * DEG * amt;
    p.j[SHIN_L].x += 26. * c.max(0.) * DEG * amt;
    p.j[SHIN_R].x += 26. * (-c).max(0.) * DEG * amt;
    p.j[PELVIS].y += s * 5. * DEG * amt;
    p
}

pub fn lie_up() -> Pose {
    Pose::new()
        .at(0., LIE_Y, 0.)
        .q(qx(-90.))
        .r(HEAD, 8., 10., 0.)
        .r(UARM_L, 8., 0., 28.)
        .r(UARM_R, 4., 0., -34.)
        .r(FARM_L, -25., 0., 0.)
        .r(FARM_R, -15., 0., 0.)
        .r(THIGH_L, -8., 0., 8.)
        .r(THIGH_R, -3., 0., -10.)
        .r(SHIN_L, 14., 0., 0.)
        .r(SHIN_R, 5., 0., 0.)
        .r(FOOT_L, 45., 0., 10.)
        .r(FOOT_R, 50., 0., -10.)
}

pub fn lie_down() -> Pose {
    Pose::new()
        .at(0., LIE_Y + 0.01, 0.)
        .q(qx(90.))
        .r(HEAD, -10., 70., 0.)
        .r(UARM_L, -20., 0., 45.)
        .r(UARM_R, -10., 0., -40.)
        .r(FARM_L, -50., 0., 0.)
        .r(FARM_R, -40., 0., 0.)
        .r(THIGH_L, 4., 0., 10.)
        .r(THIGH_R, 2., 0., -8.)
        .r(SHIN_L, 6., 0., 0.)
        .r(SHIN_R, 10., 0., 0.)
        .r(FOOT_L, 70., 0., 0.)
        .r(FOOT_R, 65., 0., 0.)
}

pub fn lockup(t: f32) -> Pose {
    let s = (t * 5.0).sin();
    Pose::new()
        .at(0.0, 0.93, 0.06)
        .r(SPINE, 24., s * 3., 0.)
        .r(CHEST, 10., 0., 0.)
        .r(NECK, -8., 0., 0.)
        .r(HEAD, -14., -10., 0.)
        .r(UARM_L, -72., 0., 18.)
        .r(FARM_L, -55., 0., 0.)
        .r(UARM_R, -58., 0., -12.)
        .r(FARM_R, -75., 0., 0.)
        .r(THIGH_L, -32. + s * 3., 0., 4.)
        .r(SHIN_L, 40., 0., 0.)
        .r(FOOT_L, -8., 0., 0.)
        .r(THIGH_R, 12. - s * 3., 0., -4.)
        .r(SHIN_R, 28., 0., 0.)
        .r(FOOT_R, -40., 0., 0.)
}

/// Pose aus dem eigenen Raum des Opfers in den Raum des Angreifers (Opfer steht vor ihm, Blick zurück)
pub fn in_front(p: Pose, d: f32) -> Pose {
    let mut o = p;
    o.root = vec3(-p.root.x, p.root.y, d - p.root.z);
    o.rq = qy(180.) * p.rq;
    o
}

pub fn kneel_cover() -> Pose {
    Pose::new()
        .at(0., 0.50, 0.0)
        .r(SPINE, 62., 0., 0.)
        .r(CHEST, 28., 0., 0.)
        .r(NECK, -20., 0., 0.)
        .r(HEAD, -10., 40., 0.)
        .r(UARM_L, -75., 0., 25.)
        .r(FARM_L, -30., 0., 0.)
        .r(UARM_R, -40., 0., -35.)
        .r(FARM_R, -50., 0., 0.)
        .r(THIGH_L, -5., 0., 12.)
        .r(THIGH_R, -10., 0., -10.)
        .r(SHIN_L, 95., 0., 0.)
        .r(SHIN_R, 92., 0., 0.)
        .r(FOOT_L, 45., 0., 0.)
        .r(FOOT_R, 45., 0., 0.)
}

pub fn ref_count(slap: f32) -> Pose {
    // slap 0..1: 0 = Arm oben, 1 = Hand auf der Matte
    let ux = -165. + slap * 110.;
    Pose::new()
        .at(0., 0.47, 0.0)
        .r(SPINE, 50., 0., 0.)
        .r(CHEST, 25., 0., 0.)
        .r(HEAD, -25., 0., 0.)
        .r(UARM_L, -55., 0., 30.)
        .r(FARM_L, -20., 0., 0.)
        .r(UARM_R, ux, 0., -15.)
        .r(FARM_R, -10. - (1. - slap) * 20., 0., 0.)
        .r(THIGH_L, -5., 0., 10.)
        .r(THIGH_R, -5., 0., -10.)
        .r(SHIN_L, 95., 0., 0.)
        .r(SHIN_R, 95., 0., 0.)
        .r(FOOT_L, 45., 0., 0.)
        .r(FOOT_R, 45., 0., 0.)
}

pub fn celebrate(t: f32) -> Pose {
    let pump = (t * 6.0).sin().max(0.0);
    Pose::new()
        .at(0., 1.0, 0.)
        .r(SPINE, -6., 0., 0.)
        .r(CHEST, -8., 0., 0.)
        .r(HEAD, -22., (t * 0.8).sin() * 20., 0.)
        .r(UARM_L, -20., 0., 150. + pump * 15.)
        .r(UARM_R, -20., 0., -150. - pump * 15.)
        .r(FARM_L, 0., 0., 20. + pump * 25.)
        .r(FARM_R, 0., 0., -20. - pump * 25.)
        .legs(-4., 6., -2.)
}

pub fn ref_stand(t: f32) -> Pose {
    let b = (t * 1.7).sin();
    Pose::new()
        .at(0., 1.0, 0.)
        .r(SPINE, 4., 0., 0.)
        .r(CHEST, 2. + b, 0., 0.)
        .r(HEAD, -2., 0., 0.)
        .r(UARM_L, -8., 0., 10.)
        .r(UARM_R, -8., 0., -10.)
        .r(FARM_L, -35., 0., 0.)
        .r(FARM_R, -35., 0., 0.)
        .r(THIGH_L, -6., 0., 5.)
        .r(THIGH_R, -6., 0., -5.)
        .r(SHIN_L, 10., 0., 0.)
        .r(SHIN_R, 10., 0., 0.)
        .r(FOOT_L, -4., 0., 0.)
        .r(FOOT_R, -4., 0., 0.)
}

// ---------------------------------------------------------------- Keyframe-Clips

#[derive(Clone)]
pub struct Clip {
    pub keys: Vec<(f32, Pose)>,
    pub ground: bool,
}

impl Clip {
    pub fn new(keys: Vec<(f32, Pose)>, ground: bool) -> Clip {
        Clip { keys, ground }
    }
    pub fn dur(&self) -> f32 {
        self.keys.last().unwrap().0
    }
    pub fn sample(&self, t: f32) -> Pose {
        let k = &self.keys;
        if t <= k[0].0 {
            return k[0].1;
        }
        for w in k.windows(2) {
            let (t0, ref p0) = w[0];
            let (t1, ref p1) = w[1];
            if t <= t1 {
                let u = ((t - t0) / (t1 - t0)).clamp(0., 1.);
                let u = u * u * (3. - 2. * u);
                return Pose::lerp(p0, p1, u);
            }
        }
        k.last().unwrap().1
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum End {
    Stand,
    LieUp,
    LieDown,
}

pub struct PairClip {
    pub a: Clip,
    pub v: Clip,
    pub impact: f32,
    pub dmg: f32,
    pub a_end: End,
    pub v_end: End,
    pub name: &'static str,
}

impl PairClip {
    pub fn dur(&self) -> f32 {
        self.a.dur().max(self.v.dur())
    }
}

pub struct Clips {
    pub punch: Clip,
    pub kick: Clip,
    pub lariat: Clip,
    pub stomp: Clip,
    pub kneedrop: Clip,
    pub grab: Clip,
    pub whip: Clip,
    pub taunt: Clip,
    pub hit_high: Clip,
    pub hit_body: Clip,
    pub block: Clip,
    pub fall_back: Clip,
    pub fall_front: Clip,
    pub getup_back: Clip,
    pub getup_front: Clip,
    pub rope: Clip,
    pub pushed: Clip,
    pub slam: PairClip,
    pub suplex: PairClip,
    pub ddt: PairClip,
}

static CLIPS: OnceLock<Clips> = OnceLock::new();

pub fn clips() -> &'static Clips {
    CLIPS.get_or_init(build_clips)
}

fn st() -> Pose {
    stance(0.0)
}

fn build_clips() -> Clips {
    let stand = st();

    // --- Rechter Schlag
    let punch = Clip::new(
        vec![
            (0.0, stand),
            (0.12, stand.r(CHEST, 4., 28., 0.).r(SPINE, 10., 8., 0.).r(UARM_R, -25., 0., -35.).r(FARM_R, -125., 0., 0.)),
            (0.22, stand
                .at(0., 0.93, 0.10)
                .r(SPINE, 16., -12., 0.)
                .r(CHEST, 6., -34., 0.)
                .r(UARM_R, -86., 0., -6.)
                .r(FARM_R, -6., 0., 0.)
                .r(UARM_L, -55., 0., 18.)
                .r(FARM_L, -120., 0., 0.)),
            (0.32, stand.at(0., 0.94, 0.08).r(CHEST, 6., -20., 0.).r(UARM_R, -70., 0., -10.).r(FARM_R, -40., 0., 0.)),
            (0.52, stand.at(0., 0.95, 0.06)),
        ],
        true,
    );

    // --- Big Boot
    let kick = Clip::new(
        vec![
            (0.0, stand),
            (0.20, stand
                .at(0., 0.97, 0.0)
                .r(SPINE, -8., 0., 0.)
                .r(UARM_L, -20., 0., 38.)
                .r(UARM_R, -10., 0., -40.)
                .r(THIGH_R, -112., 0., -4.)
                .r(SHIN_R, 112., 0., 0.)
                .r(FOOT_R, -10., 0., 0.)
                .r(THIGH_L, -6., 0., 6.)
                .r(SHIN_L, 10., 0., 0.)),
            (0.33, stand
                .at(0., 0.98, 0.06)
                .r(SPINE, -26., 0., 0.)
                .r(CHEST, -10., 0., 0.)
                .r(UARM_L, -10., 0., 50.)
                .r(UARM_R, 10., 0., -50.)
                .r(FARM_L, -30., 0., 0.)
                .r(FARM_R, -30., 0., 0.)
                .r(THIGH_R, -96., 0., -2.)
                .r(SHIN_R, 6., 0., 0.)
                .r(FOOT_R, 15., 0., 0.)
                .r(THIGH_L, 4., 0., 4.)
                .r(SHIN_L, 6., 0., 0.)
                .r(FOOT_L, -8., 0., 0.)),
            (0.48, stand.at(0., 0.96, 0.10).r(THIGH_R, -60., 0., -4.).r(SHIN_R, 60., 0., 0.)),
            (0.72, stand.at(0., 0.95, 0.14)),
        ],
        true,
    );

    // --- Clothesline (Lariat)
    let lariat = Clip::new(
        vec![
            (0.0, stand),
            (0.12, stand.r(CHEST, 0., 32., 0.).r(UARM_R, -25., 0., -95.).r(FARM_R, -10., 0., 0.)),
            (0.24, stand
                .at(0., 0.94, 0.30)
                .r(SPINE, 18., -14., 0.)
                .r(CHEST, 4., -42., 0.)
                .r(UARM_R, -78., 0., -88.)
                .r(FARM_R, -5., 0., 0.)),
            (0.40, stand.at(0., 0.94, 0.45).r(CHEST, 4., -55., 0.).r(UARM_R, -80., 0., -60.).r(FARM_R, -20., 0., 0.)),
            (0.62, stand.at(0., 0.95, 0.55)),
        ],
        true,
    );

    // --- Stampfer auf den liegenden Gegner
    let stomp = Clip::new(
        vec![
            (0.0, stand),
            (0.24, stand
                .r(SPINE, 6., 0., 0.)
                .r(HEAD, 20., 0., 0.)
                .r(UARM_L, -20., 0., 35.)
                .r(UARM_R, -20., 0., -35.)
                .r(THIGH_R, -100., 0., -4.)
                .r(SHIN_R, 100., 0., 0.)
                .r(FOOT_R, 0., 0., 0.)
                .r(THIGH_L, -10., 0., 5.)
                .r(SHIN_L, 14., 0., 0.)),
            (0.36, stand
                .at(0., 0.9, 0.08)
                .r(SPINE, 26., 0., 0.)
                .r(HEAD, 30., 0., 0.)
                .r(UARM_L, -30., 0., 30.)
                .r(UARM_R, -30., 0., -30.)
                .r(THIGH_R, -38., 0., -4.)
                .r(SHIN_R, 8., 0., 0.)
                .r(FOOT_R, -10., 0., 0.)),
            (0.68, stand.at(0., 0.95, 0.06)),
        ],
        true,
    );

    // --- Kniefall (Knee Drop)
    let kneeling = Pose::new()
        .at(0., 0.52, 0.62)
        .r(SPINE, 22., 0., 0.)
        .r(HEAD, 25., 0., 0.)
        .r(UARM_L, -40., 0., 40.)
        .r(UARM_R, -30., 0., -45.)
        .r(FARM_L, -30., 0., 0.)
        .r(FARM_R, -40., 0., 0.)
        .r(THIGH_L, -92., 0., 8.)
        .r(SHIN_L, 92., 0., 0.)
        .r(THIGH_R, -2., 0., -4.)
        .r(SHIN_R, 92., 0., 0.)
        .r(FOOT_R, 40., 0., 0.);
    let kneedrop = Clip::new(
        vec![
            (0.0, stand),
            (0.25, stand.at(0., 0.76, 0.08).legs(-62., 92., -28.).r(SPINE, 20., 0., 0.).arms(-10., 25., -40.)),
            (0.48, Pose::new()
                .at(0., 1.38, 0.42)
                .r(SPINE, 8., 0., 0.)
                .r(HEAD, 18., 0., 0.)
                .r(UARM_L, -10., 0., 120.)
                .r(UARM_R, -10., 0., -120.)
                .r(THIGH_L, -55., 0., 8.)
                .r(SHIN_L, 110., 0., 0.)
                .r(THIGH_R, -20., 0., -4.)
                .r(SHIN_R, 125., 0., 0.)),
            (0.66, kneeling),
            (0.95, kneeling.at(0., 0.54, 0.62).r(HEAD, 0., 0., 0.)),
            (1.35, stand.at(0., 0.95, 0.62)),
        ],
        false,
    );

    // --- Griff ansetzen
    let grab = Clip::new(
        vec![
            (0.0, stand),
            (0.16, stand.at(0., 0.91, 0.16).r(SPINE, 26., 0., 0.).arms(-82., 16., -24.)),
            (0.46, stand.at(0., 0.95, 0.10)),
        ],
        true,
    );

    // --- Irish Whip (Angreifer)
    let lk = lockup(0.0);
    let whip = Clip::new(
        vec![
            (0.0, lk),
            (0.28, lk.r(CHEST, 6., 38., 0.).r(UARM_R, -60., 0., -40.).r(FARM_R, -25., 0., 0.)),
            (0.48, lk.r(CHEST, 6., -48., 0.).r(SPINE, 18., -10., 0.).r(UARM_R, -88., 0., -75.).r(FARM_R, 0., 0., 0.)),
            (0.80, stand),
        ],
        true,
    );

    // --- Posing (Doppelbizeps)
    let flex = Pose::new()
        .at(0., 1.0, 0.)
        .r(SPINE, -4., 0., 0.)
        .r(CHEST, -10., 0., 0.)
        .r(HEAD, -18., 0., 0.)
        .r(UARM_L, -8., 0., 86.)
        .r(UARM_R, -8., 0., -86.)
        .r(FARM_L, 0., 0., 105.)
        .r(FARM_R, 0., 0., -105.)
        .legs(-6., 8., -2.);
    let taunt = Clip::new(
        vec![
            (0.0, stand),
            (0.3, flex),
            (0.75, flex.r(CHEST, -14., 0., 0.).r(FARM_L, 0., 0., 118.).r(FARM_R, 0., 0., -118.)),
            (1.15, flex),
            (1.45, stand),
        ],
        true,
    );

    // --- Treffer-Reaktionen
    let hit_high = Clip::new(
        vec![
            (0.0, stand),
            (0.08, stand
                .at(0., 0.95, -0.10)
                .r(HEAD, -36., 18., 0.)
                .r(NECK, -18., 0., 0.)
                .r(CHEST, -14., 10., 0.)
                .r(SPINE, -8., 0., 0.)
                .r(UARM_L, -20., 0., 45.)
                .r(UARM_R, -15., 0., -40.)
                .r(FARM_L, -40., 0., 0.)
                .r(FARM_R, -40., 0., 0.)),
            (0.22, stand.at(0., 0.95, -0.16).r(HEAD, -10., 8., 0.)),
            (0.42, stand.at(0., 0.95, -0.18)),
        ],
        true,
    );
    let hit_body = Clip::new(
        vec![
            (0.0, stand),
            (0.10, stand
                .at(0., 0.86, -0.06)
                .r(SPINE, 44., 0., 0.)
                .r(CHEST, 22., 0., 0.)
                .r(HEAD, 12., 0., 0.)
                .r(UARM_L, -38., 0., 12.)
                .r(UARM_R, -38., 0., -12.)
                .r(FARM_L, -95., 0., 0.)
                .r(FARM_R, -95., 0., 0.)
                .legs(-40., 60., -20.)),
            (0.32, stand.at(0., 0.9, -0.12).r(SPINE, 30., 0., 0.).arms(-40., 14., -95.)),
            (0.52, stand.at(0., 0.95, -0.14)),
        ],
        true,
    );
    let block = Clip::new(
        vec![
            (0.0, stand.arms(-70., 10., -140.)),
            (0.08, stand.at(0., 0.95, -0.08).arms(-75., 8., -145.).r(HEAD, 15., 0., 0.).r(SPINE, 18., 0., 0.)),
            (0.25, stand.at(0., 0.95, -0.12).arms(-70., 10., -140.)),
        ],
        true,
    );

    // --- Stürze
    let fall_back = Clip::new(
        vec![
            (0.0, stand),
            (0.15, stand
                .at(0., 0.95, -0.15)
                .r(SPINE, -25., 0., 0.)
                .r(CHEST, -10., 0., 0.)
                .r(HEAD, -25., 0., 0.)
                .arms(-30., 70., -30.)
                .r(THIGH_L, -22., 0., 6.)),
            (0.45, Pose::new()
                .at(0., 0.62, -0.45)
                .q(qx(-42.))
                .r(HEAD, 22., 0., 0.)
                .arms(-35., 80., -40.)
                .legs(-35., 35., 0.)),
            (0.70, Pose::new()
                .at(0., LIE_Y, -0.72)
                .q(qx(-90.))
                .r(HEAD, 10., 0., 0.)
                .arms(-30., 65., -20.)
                .legs(-35., 40., 20.)),
            (0.82, lie_up().at(0., LIE_Y + 0.03, -0.74).legs(-20., 25., 30.)),
            (1.05, lie_up().at(0., LIE_Y, -0.75)),
        ],
        false,
    );
    let fall_front = Clip::new(
        vec![
            (0.0, stand),
            (0.15, stand.at(0., 0.86, 0.06).r(SPINE, 40., 0., 0.).r(HEAD, 10., 0., 0.).arms(-30., 15., -60.)),
            (0.45, Pose::new().at(0., 0.6, 0.32).q(qx(42.)).arms(-70., 20., -20.).legs(-20., 40., 0.)),
            (0.70, Pose::new().at(0., LIE_Y + 0.02, 0.56).q(qx(90.)).r(HEAD, -25., 30., 0.).arms(-100., 30., -30.).legs(5., 10., 50.)),
            (1.05, lie_down().at(0., LIE_Y + 0.01, 0.6)),
        ],
        false,
    );

    // --- Aufstehen aus Rückenlage
    let kneel1 = Pose::new()
        .at(0., 0.50, 0.30)
        .r(SPINE, 22., 0., 0.)
        .r(CHEST, 6., 0., 0.)
        .r(HEAD, -10., 0., 0.)
        .r(UARM_L, -45., 0., 12.)
        .r(FARM_L, -25., 0., 0.)
        .r(UARM_R, -15., 0., -18.)
        .r(FARM_R, -30., 0., 0.)
        .r(THIGH_L, -92., 0., 6.)
        .r(SHIN_L, 92., 0., 0.)
        .r(FOOT_L, 0., 0., 0.)
        .r(THIGH_R, -4., 0., -4.)
        .r(SHIN_R, 94., 0., 0.)
        .r(FOOT_R, 40., 0., 0.);
    let getup_back = Clip::new(
        vec![
            (0.0, lie_up()),
            (0.45, Pose::new()
                .at(0., 0.13, 0.0)
                .q(qx(-15.))
                .r(SPINE, 26., 0., 0.)
                .r(CHEST, 10., 0., 0.)
                .r(HEAD, -5., 0., 0.)
                .r(UARM_L, 38., 0., 22.)
                .r(UARM_R, 38., 0., -22.)
                .r(FARM_L, -10., 0., 0.)
                .r(FARM_R, -10., 0., 0.)
                .r(THIGH_L, -76., 0., 8.)
                .r(THIGH_R, -70., 0., -8.)
                .r(SHIN_L, 30., 0., 0.)
                .r(SHIN_R, 12., 0., 0.)
                .r(FOOT_L, 10., 0., 0.)),
            (0.9, kneel1),
            (1.25, stand.at(0., 0.80, 0.40).legs(-45., 70., -25.).r(SPINE, 26., 0., 0.)),
            (1.6, stand.at(0., 0.95, 0.42)),
        ],
        false,
    );
    let getup_front = Clip::new(
        vec![
            (0.0, lie_down()),
            (0.40, Pose::new()
                .at(0., 0.40, -0.05)
                .q(qx(62.))
                .r(HEAD, -35., 0., 0.)
                .r(UARM_L, -88., 0., 14.)
                .r(UARM_R, -88., 0., -14.)
                .r(FARM_L, -5., 0., 0.)
                .r(FARM_R, -5., 0., 0.)
                .legs(4., 8., 60.)),
            (0.80, Pose::new()
                .at(0., 0.52, -0.22)
                .q(qx(18.))
                .r(SPINE, 30., 0., 0.)
                .r(HEAD, -30., 0., 0.)
                .arms(-62., 12., -5.)
                .legs(-18., 92., 40.)),
            (1.1, kneel1.at(0., 0.50, -0.12)),
            (1.5, stand.at(0., 0.95, 0.0)),
        ],
        false,
    );

    let rope = Clip::new(
        vec![
            (0.0, walk(0.0, 1.0)),
            (0.12, stand.at(0., 0.95, -0.12).r(SPINE, -28., 0., 0.).r(CHEST, -12., 0., 0.).r(HEAD, -10., 0., 0.).arms(35., 80., -20.)),
            (0.36, stand.at(0., 0.95, -0.05).r(SPINE, 10., 0., 0.).arms(10., 40., -30.)),
        ],
        true,
    );

    let cov = kneel_cover();
    let pushed = Clip::new(
        vec![
            (0.0, cov),
            (0.25, cov.at(0., 0.42, -0.25).r(SPINE, 10., 0., 0.).r(CHEST, 0., 0., 0.).arms(20., 30., -20.)),
            (0.7, stand.at(0., 0.95, -0.45)),
        ],
        false,
    );

    // ============================================================ Würfe (Paar-Clips)
    let lk_a = lockup(0.0);
    let lk_v = in_front(lockup(0.0), LOCK_D);

    // --- Body Slam
    let held = qyx(90., -90.);
    let slam_a = Clip::new(
        vec![
            (0.0, lk_a),
            (0.35, stand
                .at(0., 0.78, 0.16)
                .r(SPINE, 36., 0., 0.)
                .r(CHEST, 10., 0., 0.)
                .r(UARM_L, -62., 0., 12.)
                .r(FARM_L, -40., 0., 0.)
                .r(UARM_R, -72., 0., -22.)
                .r(FARM_R, -30., 0., 0.)
                .legs(-58., 82., -24.)),
            (0.80, stand
                .at(0., 0.98, 0.04)
                .r(SPINE, -10., 0., 0.)
                .r(CHEST, -6., 0., 0.)
                .r(HEAD, -10., 0., 0.)
                .r(UARM_L, -150., 0., 22.)
                .r(FARM_L, -40., 0., 0.)
                .r(UARM_R, -140., 0., -20.)
                .r(FARM_R, -50., 0., 0.)
                .legs(-16., 24., -8.)),
            (1.15, stand
                .at(0., 0.80, 0.18)
                .r(SPINE, 45., 0., 0.)
                .r(CHEST, 14., 0., 0.)
                .r(HEAD, 15., 0., 0.)
                .arms(-80., 10., -10.)
                .legs(-50., 72., -22.)),
            (1.45, stand.at(0., 0.9, 0.12).r(SPINE, 30., 0., 0.)),
            (1.75, stand),
        ],
        true,
    );
    let slam_v = Clip::new(
        vec![
            (0.0, lk_v),
            (0.35, in_front(stand.at(0., 1.0, -0.04).r(SPINE, 12., 0., 0.).arms(-50., 30., -20.).legs(-10., 15., -5.), LOCK_D - 0.1)),
            (0.58, Pose::new()
                .at(0.12, 1.55, 0.42)
                .q(qy(180.).slerp(held, 0.5))
                .arms(-20., 40., -30.)
                .legs(-25., 30., 10.)),
            (0.80, Pose::new()
                .at(0.28, 1.92, 0.22)
                .q(held)
                .r(HEAD, -15., 0., 0.)
                .arms(55., 35., -20.)
                .legs(-20., 30., 20.)),
            (1.15, Pose::new()
                .at(0.30, LIE_Y, 0.92)
                .q(held)
                .r(HEAD, 25., 0., 0.)
                .arms(20., 70., -10.)
                .legs(-30., 22., 30.)),
            (1.35, lie_up().at(0.30, LIE_Y + 0.07, 0.92).q(held).legs(-18., 20., 30.)),
            (1.75, lie_up().at(0.30, LIE_Y, 0.92).q(held)),
        ],
        false,
    );

    // --- Vertical Suplex
    let suplex_a = Clip::new(
        vec![
            (0.0, lk_a),
            (0.35, stand
                .at(0., 0.86, 0.10)
                .r(SPINE, 22., 0., 0.)
                .r(UARM_R, -100., 0., -40.)
                .r(FARM_R, -90., 0., 0.)
                .r(UARM_L, -62., 0., 20.)
                .r(FARM_L, -60., 0., 0.)
                .legs(-44., 60., -16.)),
            (0.80, stand
                .at(0., 0.96, -0.05)
                .r(SPINE, -24., 0., 0.)
                .r(CHEST, -14., 0., 0.)
                .r(HEAD, -10., 0., 0.)
                .r(UARM_R, -150., 0., -20.)
                .r(FARM_R, -40., 0., 0.)
                .r(UARM_L, -160., 0., 20.)
                .r(FARM_L, -30., 0., 0.)
                .legs(-10., 15., -5.)),
            (1.10, Pose::new()
                .at(0., 0.68, -0.30)
                .q(qx(-42.))
                .r(SPINE, -8., 0., 0.)
                .arms(-150., 20., -40.)
                .legs(-42., 30., 10.)),
            (1.35, Pose::new()
                .at(0., LIE_Y, -0.36)
                .q(qx(-90.))
                .r(HEAD, 10., 0., 0.)
                .arms(-165., 20., -20.)
                .legs(-35., 45., 20.)),
            (2.0, lie_up().at(0., LIE_Y, -0.36)),
        ],
        false,
    );
    let suplex_v = Clip::new(
        vec![
            (0.0, lk_v),
            (0.35, Pose::new()
                .at(0., 0.96, 0.56)
                .q(qyx(180., 25.))
                .r(SPINE, 25., 0., 0.)
                .r(HEAD, 20., 0., 0.)
                .arms(-50., 25., -20.)
                .legs(-25., 30., -5.)),
            (0.80, Pose::new()
                .at(0., 2.25, 0.12)
                .q(qyx(180., 178.))
                .r(HEAD, -15., 0., 0.)
                .arms(-20., 95., -10.)
                .legs(-5., 5., 20.)),
            (1.10, Pose::new()
                .at(0., 1.55, -0.95)
                .q(qyx(180., 228.))
                .r(HEAD, 20., 0., 0.)
                .arms(-10., 90., -10.)
                .legs(-10., 10., 20.)),
            (1.35, Pose::new()
                .at(0., LIE_Y, -1.85)
                .q(qyx(180., 270.))
                .r(HEAD, 15., 0., 0.)
                .arms(-10., 70., -10.)
                .legs(-30., 25., 30.)),
            (1.55, lie_up().at(0., LIE_Y + 0.07, -1.85).q(qyx(180., 270.)).legs(-16., 12., 30.)),
            (2.0, lie_up().at(0., LIE_Y, -1.85).q(qyx(180., 270.))),
        ],
        false,
    );

    // --- HEAD OF STATE (DDT-Finisher)
    let ddt_a = Clip::new(
        vec![
            (0.0, lk_a),
            (0.30, stand
                .at(0., 0.90, 0.05)
                .r(SPINE, 30., 0., 0.)
                .r(UARM_R, -82., 0., -28.)
                .r(FARM_R, -100., 0., 0.)
                .r(UARM_L, -70., 0., 30.)
                .r(FARM_L, -90., 0., 0.)),
            (0.72, stand
                .at(0., 0.84, 0.0)
                .r(SPINE, 16., 0., 0.)
                .r(UARM_R, -82., 0., -28.)
                .r(FARM_R, -105., 0., 0.)
                .r(UARM_L, -70., 0., 30.)
                .r(FARM_L, -95., 0., 0.)
                .legs(-38., 55., -17.)),
            (0.88, Pose::new()
                .at(0., 1.05, -0.15)
                .q(qx(-30.))
                .r(UARM_R, -80., 0., -25.)
                .r(FARM_R, -100., 0., 0.)
                .r(UARM_L, -70., 0., 30.)
                .r(FARM_L, -95., 0., 0.)
                .legs(-60., 30., 10.)),
            (1.05, Pose::new()
                .at(0., LIE_Y, -0.40)
                .q(qx(-90.))
                .r(UARM_R, -60., 0., -20.)
                .r(FARM_R, -100., 0., 0.)
                .r(UARM_L, -40., 0., 40.)
                .r(FARM_L, -60., 0., 0.)
                .legs(-25., 30., 20.)),
            (1.9, lie_up().at(0., LIE_Y, -0.40)),
        ],
        false,
    );
    let ddt_v = Clip::new(
        vec![
            (0.0, lk_v),
            (0.30, Pose::new()
                .at(0.05, 0.86, 0.80)
                .q(qyx(180., 68.))
                .r(SPINE, 10., 0., 0.)
                .r(NECK, 10., 0., 0.)
                .arms(-55., 25., -30.)
                .legs(-66., 8., -10.)),
            (0.72, Pose::new()
                .at(0.05, 0.86, 0.76)
                .q(qyx(180., 72.))
                .r(SPINE, 12., 0., 0.)
                .arms(-30., 40., -40.)
                .legs(-70., 10., -10.)),
            (0.88, Pose::new()
                .at(0.35, 1.0, 0.45)
                .q(qyx(180., 112.))
                .arms(-10., 70., -20.)
                .legs(-20., 15., 0.)),
            (1.05, Pose::new()
                .at(0.55, 0.69, -0.22)
                .q(qyx(180., 150.))
                .r(HEAD, 20., 0., 0.)
                .arms(-60., 70., -10.)
                .legs(-10., 10., 20.)),
            (1.30, Pose::new()
                .at(0.70, 0.35, 0.05)
                .q(qyx(180., 108.))
                .arms(-80., 50., -10.)
                .legs(0., 15., 30.)),
            (1.55, lie_down().at(0.72, LIE_Y + 0.04, 0.15).q(qyx(180., 90.))),
            (1.9, lie_down().at(0.72, LIE_Y + 0.01, 0.15).q(qyx(180., 90.))),
        ],
        false,
    );

    Clips {
        punch,
        kick,
        lariat,
        stomp,
        kneedrop,
        grab,
        whip,
        taunt,
        hit_high,
        hit_body,
        block,
        fall_back,
        fall_front,
        getup_back,
        getup_front,
        rope,
        pushed,
        slam: PairClip { a: slam_a, v: slam_v, impact: 1.15, dmg: 13.0, a_end: End::Stand, v_end: End::LieUp, name: "BODY SLAM" },
        suplex: PairClip { a: suplex_a, v: suplex_v, impact: 1.35, dmg: 17.0, a_end: End::LieUp, v_end: End::LieUp, name: "SUPLEX" },
        ddt: PairClip { a: ddt_a, v: ddt_v, impact: 1.05, dmg: 32.0, a_end: End::LieUp, v_end: End::LieDown, name: "HEAD OF STATE" },
    }
}

/// Ermittelt aus einer liegenden Endpose die Gierung relativ zur kanonischen Liegepose
pub fn lying_yaw(rq: Quat, face_up: bool) -> f32 {
    let legs = rq * vec3(0., -1., 0.);
    let a = legs.x.atan2(legs.z);
    if face_up {
        a
    } else {
        a - PI
    }
}
