//! Wrestler: Zustand, Animationsauswahl, Eingaben, KI.
use crate::anim::*;
use crate::model::*;
use macroquad::models::Mesh;
use macroquad::prelude::*;
use std::f32::consts::PI;

pub const BOUND: f32 = 2.70;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    Punch,
    Kick,
    Lariat,
    Stomp,
    KneeDrop,
    Grab,
    Whip,
    Taunt,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Throw {
    Slam,
    Suplex,
    Ddt,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum React {
    High,
    Body,
    Block,
    Pushed,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum St {
    Idle,
    Action { a: Act, t: f32, hit: bool },
    React { r: React, t: f32 },
    Lockup { t: f32, init: bool },
    Throw { th: Throw, t: f32, attacker: bool, impact: bool },
    Fall { up: bool, t: f32, thud: bool },
    Down { up: bool, t: f32, dur: f32 },
    GetUp { up: bool, t: f32 },
    Run { dir: Vec2, t: f32, n: u8, rebound: f32 },
    Cover { t: f32 },
    Pinned { t: f32 },
    Win { t: f32 },
    Lost { t: f32 },
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Intent {
    pub mv: Vec2,
    pub run: bool,
    pub strike: bool,
    pub kick: bool,
    pub grapple: bool,
    pub pin: bool,
    pub block: bool,
    pub mash: bool,
}

pub struct Keys {
    pub up: Vec<KeyCode>,
    pub down: Vec<KeyCode>,
    pub left: Vec<KeyCode>,
    pub right: Vec<KeyCode>,
    pub run: Vec<KeyCode>,
    pub strike: Vec<KeyCode>,
    pub kick: Vec<KeyCode>,
    pub grapple: Vec<KeyCode>,
    pub pin: Vec<KeyCode>,
    pub block: Vec<KeyCode>,
}

impl Keys {
    pub fn p1() -> Keys {
        Keys {
            up: vec![KeyCode::W],
            down: vec![KeyCode::S],
            left: vec![KeyCode::A],
            right: vec![KeyCode::D],
            run: vec![KeyCode::LeftShift],
            strike: vec![KeyCode::J],
            kick: vec![KeyCode::K],
            grapple: vec![KeyCode::L],
            pin: vec![KeyCode::Space],
            block: vec![KeyCode::I, KeyCode::LeftControl],
        }
    }
    pub fn p2() -> Keys {
        Keys {
            up: vec![KeyCode::Up],
            down: vec![KeyCode::Down],
            left: vec![KeyCode::Left],
            right: vec![KeyCode::Right],
            run: vec![KeyCode::RightShift],
            strike: vec![KeyCode::Kp1, KeyCode::Comma],
            kick: vec![KeyCode::Kp2, KeyCode::Period],
            grapple: vec![KeyCode::Kp3, KeyCode::Slash],
            pin: vec![KeyCode::Kp0, KeyCode::RightControl],
            block: vec![KeyCode::Kp5, KeyCode::M],
        }
    }
    pub fn read(&self, cam_fwd: Vec2, cam_right: Vec2) -> Intent {
        let down = |ks: &Vec<KeyCode>| ks.iter().any(|k| is_key_down(*k));
        let pressed = |ks: &Vec<KeyCode>| ks.iter().any(|k| is_key_pressed(*k));
        let mut v = Vec2::ZERO;
        if down(&self.up) {
            v += cam_fwd;
        }
        if down(&self.down) {
            v -= cam_fwd;
        }
        if down(&self.right) {
            v += cam_right;
        }
        if down(&self.left) {
            v -= cam_right;
        }
        let strike = pressed(&self.strike);
        let kick = pressed(&self.kick);
        let grapple = pressed(&self.grapple);
        let pin = pressed(&self.pin);
        Intent {
            mv: v.normalize_or_zero(),
            run: down(&self.run),
            strike,
            kick,
            grapple,
            pin,
            block: down(&self.block),
            mash: strike || kick || grapple || pin,
        }
    }
}

pub struct Ai {
    pub diff: f32,
    pub cd: f32,
    pub lock_delay: f32,
    pub block_t: f32,
    pub pin_roll: f32,
    pub down_seen: bool,
    pub strafe: f32,
    pub counter_done: bool,
}

impl Ai {
    pub fn new(diff: f32) -> Ai {
        Ai { diff, cd: 1.0, lock_delay: 0.6, block_t: 0.0, pin_roll: 0.5, down_seen: false, strafe: rand::gen_range(0.0, 6.0), counter_done: false }
    }
}

pub enum Ctrl {
    Human(Keys),
    Cpu(Ai),
}

pub struct Wrestler {
    pub name: String,
    pub meshes: Vec<Mesh>,
    pub off: [Vec3; NJ],
    pub scale: f32,
    pub pos: Vec2,
    pub yaw: f32,
    pub vel: Vec2,
    pub st: St,
    pub health: f32,
    pub stamina: f32,
    pub special: f32,
    pub pose: Pose,
    pub snap: Pose,
    pub blend: f32,
    pub phase: f32,
    pub clock: f32,
    pub speed: f32,
    pub flash: f32,
    pub anchor: (Vec2, f32),
    pub blocking: bool,
    pub ctrl: Ctrl,
    pub punch_alt: bool,
    pub combo: u8,
    pub combo_t: f32,
    pub lock_choice: Option<Throw>,
    pub whip_choice: bool,
    pub color: Color,
    pub mirror_action: bool,
    pub cover_target: Vec2,
    pub hits_taken: u32,
}

pub fn wrap(a: f32) -> f32 {
    let mut a = a % (2.0 * PI);
    if a > PI {
        a -= 2.0 * PI;
    }
    if a < -PI {
        a += 2.0 * PI;
    }
    a
}

pub fn rot2(v: Vec2, yaw: f32) -> Vec2 {
    let (s, c) = yaw.sin_cos();
    vec2(v.x * c + v.y * s, -v.x * s + v.y * c)
}

impl Wrestler {
    pub fn new(name: &str, look: &Look, scale: f32, ctrl: Ctrl, color: Color) -> Wrestler {
        let p = stance(0.0);
        Wrestler {
            name: name.to_string(),
            meshes: build_body(look),
            off: offsets(),
            scale,
            pos: Vec2::ZERO,
            yaw: 0.0,
            vel: Vec2::ZERO,
            st: St::Idle,
            health: 100.0,
            stamina: 100.0,
            special: 0.0,
            pose: p,
            snap: p,
            blend: 1.0,
            phase: 0.0,
            clock: rand::gen_range(0.0, 10.0),
            speed: 0.0,
            flash: 0.0,
            anchor: (Vec2::ZERO, 0.0),
            blocking: false,
            ctrl,
            punch_alt: false,
            combo: 0,
            combo_t: 0.0,
            lock_choice: None,
            whip_choice: false,
            color,
            mirror_action: false,
            cover_target: Vec2::ZERO,
            hits_taken: 0,
        }
    }

    pub fn fwd(&self) -> Vec2 {
        vec2(self.yaw.sin(), self.yaw.cos())
    }

    pub fn set(&mut self, st: St) {
        self.snap = self.pose;
        self.blend = 0.0;
        self.st = st;
    }

    /// Zustandswechsel ohne Überblendung (für nahtlose Übergänge mit Wurzelbewegung)
    pub fn set_hard(&mut self, st: St) {
        self.st = st;
        self.blend = 1.0;
    }

    pub fn is_down(&self) -> bool {
        matches!(self.st, St::Down { .. } | St::Fall { .. } | St::Pinned { .. } | St::Lost { .. })
    }

    pub fn standing_hittable(&self) -> bool {
        matches!(self.st, St::Idle | St::Action { .. } | St::React { .. } | St::Run { .. })
    }

    pub fn is_human(&self) -> bool {
        matches!(self.ctrl, Ctrl::Human(_))
    }

    /// Weltposition von Becken/Brust (für Liegende)
    pub fn body_points(&self) -> [Vec2; 3] {
        match self.st {
            St::Down { up, .. } | St::Fall { up, .. } => {
                let s = if up { -1.0 } else { 1.0 };
                let head = rot2(vec2(0., s), self.yaw);
                [self.pos, self.pos + head * 0.4 * self.scale, self.pos - head * 0.45 * self.scale]
            }
            St::Pinned { .. } | St::Lost { .. } => {
                let head = rot2(vec2(0., -1.), self.yaw);
                [self.pos, self.pos + head * 0.4 * self.scale, self.pos - head * 0.45 * self.scale]
            }
            _ => [self.pos; 3],
        }
    }

    pub fn transfer(&mut self, root: Vec3, rq: Quat, end: End) {
        self.pos += rot2(vec2(root.x, root.z), self.yaw) * self.scale;
        match end {
            End::Stand => {}
            End::LieUp => self.yaw = wrap(self.yaw + lying_yaw(rq, true)),
            End::LieDown => self.yaw = wrap(self.yaw + lying_yaw(rq, false)),
        }
    }

    pub fn act_clip(a: Act) -> &'static Clip {
        let c = clips();
        match a {
            Act::Punch => &c.punch,
            Act::Kick => &c.kick,
            Act::Lariat => &c.lariat,
            Act::Stomp => &c.stomp,
            Act::KneeDrop => &c.kneedrop,
            Act::Grab => &c.grab,
            Act::Whip => &c.whip,
            Act::Taunt => &c.taunt,
        }
    }

    pub fn react_clip(r: React) -> &'static Clip {
        let c = clips();
        match r {
            React::High => &c.hit_high,
            React::Body => &c.hit_body,
            React::Block => &c.block,
            React::Pushed => &c.pushed,
        }
    }

    pub fn pair_clip(th: Throw) -> &'static PairClip {
        let c = clips();
        match th {
            Throw::Slam => &c.slam,
            Throw::Suplex => &c.suplex,
            Throw::Ddt => &c.ddt,
        }
    }

    fn sample(&self, clip: &Clip, t: f32) -> Pose {
        let p = clip.sample(t);
        if clip.ground {
            ground(p, &self.off)
        } else {
            p
        }
    }

    /// Ziel-Pose für den aktuellen Zustand
    pub fn target_pose(&self) -> Pose {
        let c = clips();
        let t = self.clock;
        match self.st {
            St::Idle => {
                let walk_amt = (self.speed / 1.6).clamp(0.0, 1.0);
                let run_amt = ((self.speed - 2.4) / 1.8).clamp(0.0, 1.0);
                let mut base = if self.blocking {
                    stance(t).arms(-70., 10., -140.).r(HEAD, 12., 0., 0.)
                } else {
                    stance(t)
                };
                if walk_amt > 0.01 {
                    // Laufen in Blickrichtung oder seitlich tänzeln
                    let fwd_dot = if self.vel.length() > 0.1 { self.vel.normalize().dot(self.fwd()) } else { 1.0 };
                    if run_amt > 0.05 || fwd_dot > 0.6 {
                        let w = walk(self.phase, run_amt);
                        base = Pose::lerp(&base, &w, walk_amt.max(run_amt));
                    } else {
                        base = shuffle(self.phase, walk_amt, t);
                        if self.blocking {
                            base = base.arms(-70., 10., -140.);
                        }
                    }
                }
                ground(base, &self.off)
            }
            St::Action { a, t, .. } => {
                let p = self.sample(Self::act_clip(a), t);
                if self.mirror_action {
                    p.mirror()
                } else {
                    p
                }
            }
            St::React { r, t } => self.sample(Self::react_clip(r), t),
            St::Lockup { t, init } => {
                let p = lockup(t + if init { 0.0 } else { 1.3 });
                ground(p, &self.off)
            }
            St::Throw { th, t, attacker, .. } => {
                let pc = Self::pair_clip(th);
                if attacker {
                    self.sample(&pc.a, t)
                } else {
                    pc.v.sample(t)
                }
            }
            St::Fall { up, t, .. } => {
                if up {
                    c.fall_back.sample(t)
                } else {
                    c.fall_front.sample(t)
                }
            }
            St::Down { up, .. } => {
                let b = (self.clock * 2.6).sin();
                if up {
                    lie_up().add(CHEST, b * 1.5, 0., 0.).add(HEAD, b * 2.0, 0., 0.)
                } else {
                    lie_down().add(CHEST, b * 1.2, 0., 0.)
                }
            }
            St::GetUp { up, t } => {
                if up {
                    c.getup_back.sample(t)
                } else {
                    c.getup_front.sample(t)
                }
            }
            St::Run { rebound, .. } => {
                if rebound > 0.0 {
                    self.sample(&c.rope, 0.36 - rebound)
                } else {
                    ground(walk(self.phase, 1.0), &self.off)
                }
            }
            St::Cover { .. } => kneel_cover().add(SPINE, (t * 3.0).sin() * 2.0, 0., 0.),
            St::Pinned { t: pt } => {
                let s = (pt * 9.0).sin();
                let s2 = (pt * 7.0).cos();
                lie_up()
                    .r(THIGH_L, -22. + s * 14., 0., 10.)
                    .r(SHIN_L, 40. - s * 10., 0., 0.)
                    .r(THIGH_R, -18. - s * 10., 0., -10.)
                    .r(SHIN_R, 35. + s * 8., 0., 0.)
                    .r(UARM_L, -20. + s2 * 15., 0., 45.)
                    .r(UARM_R, -10., 0., -40. + s2 * 10.)
                    .r(HEAD, 20., 30. * s2, 0.)
            }
            St::Win { t } => celebrate(t),
            St::Lost { .. } => lie_up(),
        }
    }

    pub fn update_pose(&mut self, dt: f32) {
        let target = self.target_pose();
        if self.blend < 1.0 {
            self.blend = (self.blend + dt / 0.13).min(1.0);
            let u = self.blend * self.blend * (3. - 2. * self.blend);
            self.pose = Pose::lerp(&self.snap, &target, u);
        } else {
            self.pose = target;
        }
    }

    pub fn char_matrix(&self, y: f32) -> Mat4 {
        let (p, yaw) = match self.st {
            St::Throw { .. } => (self.anchor.0, self.anchor.1),
            _ => (self.pos, self.yaw),
        };
        Mat4::from_translation(vec3(p.x, y, p.y)) * Mat4::from_rotation_y(yaw) * Mat4::from_scale(Vec3::splat(self.scale))
    }

    pub fn joint_world(&self, y: f32) -> [Mat4; NJ] {
        let cm = self.char_matrix(y);
        let f = fk(&self.pose, &self.off);
        let mut out = [Mat4::IDENTITY; NJ];
        for k in 0..NJ {
            out[k] = cm * Mat4::from_rotation_translation(f[k].0, f[k].1);
        }
        out
    }

    pub fn head_world(&self, y: f32) -> Vec3 {
        let m = self.joint_world(y);
        m[HEAD].transform_point3(vec3(0., 0.1, 0.))
    }

    pub fn pelvis_world(&self, y: f32) -> Vec3 {
        let m = self.joint_world(y);
        m[PELVIS].transform_point3(Vec3::ZERO)
    }

    pub fn turn_to(&mut self, target_yaw: f32, rate: f32, dt: f32) {
        let d = wrap(target_yaw - self.yaw);
        let step = rate * dt;
        self.yaw = wrap(self.yaw + d.clamp(-step, step));
    }
}

/// KI-Entscheidungen
pub fn ai_intent(me: &Wrestler, o: &Wrestler, ai: &mut Ai, dt: f32, time: f32) -> Intent {
    let mut it = Intent::default();
    let to = o.pos - me.pos;
    let d = to.length();
    let dir = to.normalize_or_zero();
    let diff = ai.diff;
    ai.cd -= dt;
    ai.block_t -= dt;
    match me.st {
        St::Lockup { init, .. } => {
            ai.lock_delay -= dt;
            if init {
                if ai.lock_delay <= 0.0 {
                    if me.special >= 100.0 {
                        it.pin = true;
                    } else {
                        let r = rand::gen_range(0.0, 1.0);
                        if r < 0.38 {
                            it.strike = true;
                        } else if r < 0.72 {
                            it.kick = true;
                        } else {
                            it.grapple = true;
                        }
                    }
                }
            } else if rand::gen_range(0.0, 1.0) < 0.55 * dt * diff {
                if rand::gen_range(0.0, 1.0) < 0.5 {
                    it.strike = true;
                } else {
                    it.kick = true;
                }
            }
            return it;
        }
        St::Down { .. } | St::Pinned { .. } => {
            if rand::gen_range(0.0, 1.0) < 3.5 * dt * diff {
                it.mash = true;
            }
            return it;
        }
        St::Run { dir: rd, rebound, .. } => {
            if rebound <= 0.0 && !ai.counter_done && rd.dot(dir) > 0.7 && d < 1.9 && o.standing_hittable() {
                ai.counter_done = true;
                if rand::gen_range(0.0, 1.0) < 0.35 * diff {
                    it.strike = true;
                }
            }
            return it;
        }
        St::Idle => {}
        _ => {
            ai.counter_done = false;
            return it;
        }
    }
    ai.counter_done = false;
    if !o.is_down() {
        ai.down_seen = false;
    }

    // Blocken, wenn der Gegner zuschlägt
    if let St::Action { a, t, hit: false } = o.st {
        if matches!(a, Act::Punch | Act::Kick) && d < 1.6 && t < 0.15 && ai.block_t < -0.5 {
            if rand::gen_range(0.0, 1.0) < 0.35 * diff {
                ai.block_t = 0.45;
            } else {
                ai.block_t = -0.0;
            }
        }
    }
    if ai.block_t > 0.0 {
        it.block = true;
        return it;
    }

    match o.st {
        St::Down { up, .. } | St::Fall { up, .. } => {
            if !ai.down_seen {
                ai.down_seen = true;
                ai.pin_roll = rand::gen_range(0.0, 1.0);
                ai.cd = ai.cd.max(rand::gen_range(0.2, 0.7) / diff);
            }
            let bp = o.body_points();
            let target = bp[1];
            let dd = (target - me.pos).length();
            let want_pin = up && (o.health < 35.0 + 25.0 * ai.pin_roll || (o.health < 60.0 && ai.pin_roll > 0.8));
            if want_pin && matches!(o.st, St::Down { .. }) {
                let pd = (bp[0] - me.pos).length();
                if pd > 1.2 {
                    it.mv = (bp[0] - me.pos).normalize_or_zero();
                } else if ai.cd <= 0.0 {
                    it.pin = true;
                    ai.cd = 1.0;
                }
            } else if dd > 0.95 {
                it.mv = (target - me.pos).normalize_or_zero();
            } else if dd < 0.55 {
                it.mv = -(target - me.pos).normalize_or_zero();
            } else if ai.cd <= 0.0 && matches!(o.st, St::Down { .. }) {
                let r = rand::gen_range(0.0, 1.0);
                if r < 0.55 {
                    it.strike = true;
                } else if r < 0.8 {
                    it.kick = true;
                } else if me.special < 100.0 {
                    it.pin = d > 2.0;
                }
                ai.cd = rand::gen_range(0.6, 1.3) / diff;
            }
        }
        St::Run { dir: rd, .. } => {
            if rd.dot(-dir) > 0.6 && d < 1.5 && ai.cd <= 0.0 {
                if rand::gen_range(0.0, 1.0) < 0.6 {
                    it.strike = true;
                } else {
                    it.kick = true;
                }
                ai.cd = 0.6;
            }
        }
        St::Lockup { .. } | St::Throw { .. } => {}
        St::GetUp { .. } => {
            if d > 1.6 {
                it.mv = dir;
            }
        }
        _ => {
            if d > 1.15 {
                it.mv = dir;
                it.run = d > 4.0 && me.stamina > 40.0;
                if d < 2.2 {
                    let side = vec2(-dir.y, dir.x);
                    it.mv = (dir + side * (time * 0.7 + ai.strafe).sin() * 0.6).normalize();
                }
            } else if ai.cd <= 0.0 {
                let r = rand::gen_range(0.0, 1.0);
                if r < 0.38 {
                    it.strike = true;
                } else if r < 0.58 {
                    it.kick = true;
                } else if r < 0.88 && me.stamina > 15.0 {
                    it.grapple = true;
                } else {
                    it.mv = -dir;
                }
                ai.cd = rand::gen_range(0.35, 1.0) / diff;
            } else {
                let side = vec2(-dir.y, dir.x);
                it.mv = side * (time * 0.9 + ai.strafe).sin() * 0.5;
                if d < 0.85 {
                    it.mv -= dir * 0.5;
                }
            }
        }
    }
    it
}
