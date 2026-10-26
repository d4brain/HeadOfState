//! Spiellogik: Kampf, Würfe, Pinfall, Ringrichter, Kamera, Darstellung.
use crate::anim::*;
use crate::arena::*;
use crate::audio::Sfx;
use crate::mesh::*;
use crate::model::*;
use crate::render::*;
use crate::wrestler::*;
use macroquad::models::Mesh;
use macroquad::prelude::*;
use std::f32::consts::PI;

pub struct Msg {
    pub text: String,
    pub t: f32,
    pub color: Color,
    pub big: bool,
}

pub struct Particle {
    pub p: Vec3,
    pub v: Vec3,
    pub life: f32,
    pub size: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RefSt {
    Roam,
    Count { t: f32, n: u32 },
    Raise { t: f32 },
}

pub struct Referee {
    pub meshes: Vec<Mesh>,
    pub off: [Vec3; NJ],
    pub pos: Vec2,
    pub yaw: f32,
    pub vel: Vec2,
    pub st: RefSt,
    pub pose: Pose,
    pub phase: f32,
    pub clock: f32,
}

pub struct Pin {
    pub att: usize,
    pub count: u32,
    pub kick: f32,
    pub kick_at: f32,
    pub t: f32,
    pub counting: bool,
    pub wait: f32,
}

pub struct Cam {
    pub pos: Vec3,
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub zoom: f32,
    pub shake: f32,
}

pub struct Game {
    pub w: Vec<Wrestler>,
    pub referee: Referee,
    pub ropes: Ropes,
    pub msgs: Vec<Msg>,
    pub parts: Vec<Particle>,
    pub pin: Option<Pin>,
    pub time: f32,
    pub excite: f32,
    pub cam: Cam,
    pub over: Option<(usize, f32)>,
    pub bells: Vec<f32>,
    pub drop_mesh: Mesh,
    pub flash_mesh: Mesh,
    pub flashes: Vec<(Vec3, f32)>,
    pub intro: f32,
    pub count_pop: f32,
    pub slowmo: f32,
}

pub fn ring_side(p: Vec2) -> (usize, f32, f32) {
    // nächstgelegene Seite, Eindringtiefe, Position entlang der Seite
    let c = [(p.y, 0usize, p.x), (-p.y, 1, p.x), (p.x, 2, p.y), (-p.x, 3, p.y)];
    let mut best = c[0];
    for k in c {
        if k.0 > best.0 {
            best = k;
        }
    }
    (best.1, best.0 - BOUND, best.2)
}

impl Game {
    pub fn new(ctrl0: Ctrl, ctrl1: Ctrl) -> Game {
        let mut a = Wrestler::new("DER PRÄSIDENT", &look_president(), 1.04, ctrl0, hex(0xd8343c, 1.0));
        let mut b = Wrestler::new("DER KANZLER", &look_chancellor(), 1.0, ctrl1, hex(0x3c6ee0, 1.0));
        a.pos = vec2(1.9, 1.9);
        b.pos = vec2(-1.9, -1.9);
        a.yaw = (-1.0f32).atan2(-1.0);
        b.yaw = (1.0f32).atan2(1.0);
        let mut drop = MB::new();
        drop.ell(Vec3::ZERO, Vec3::splat(1.0), Color::new(0.85, 0.9, 1.0, 0.95));
        let mut fl = MB::new();
        fl.w = -1.0;
        fl.ellipsoid(Mat4::IDENTITY, Vec3::ZERO, Vec3::splat(1.0), 6, 4, &|_| WHITE);
        let referee = Referee {
            meshes: build_body(&look_referee()),
            off: offsets(),
            pos: vec2(0.0, 2.3),
            yaw: PI,
            vel: Vec2::ZERO,
            st: RefSt::Roam,
            pose: ref_stand(0.0),
            phase: 0.0,
            clock: 0.0,
        };
        Game {
            w: vec![a, b],
            referee,
            ropes: Ropes::new(),
            msgs: Vec::new(),
            parts: Vec::new(),
            pin: None,
            time: 0.0,
            excite: 0.3,
            cam: Cam { pos: vec3(0., 6., 12.), target: vec3(0., 2., 0.), yaw: 0.35, pitch: 0.36, dist: 8.5, zoom: 1.0, shake: 0.0 },
            over: None,
            bells: vec![0.2, 0.55, 0.9],
            drop_mesh: drop.build1(),
            flash_mesh: fl.build1(),
            flashes: Vec::new(),
            intro: 2.2,
            count_pop: 0.0,
            slowmo: 0.0,
        }
    }

    pub fn msg(&mut self, text: &str, color: Color, big: bool) {
        self.msgs.retain(|m| m.big != big || m.t < 1.2);
        self.msgs.push(Msg { text: text.to_string(), t: 0.0, color, big });
    }

    fn burst(&mut self, at: Vec3, n: usize, power: f32) {
        for _ in 0..n {
            let v = vec3(rand::gen_range(-1.0, 1.0), rand::gen_range(0.2, 1.4), rand::gen_range(-1.0, 1.0)) * power;
            self.parts.push(Particle { p: at, v, life: rand::gen_range(0.35, 0.8), size: rand::gen_range(0.008, 0.02) });
        }
    }

    fn cam_basis(&self) -> (Vec2, Vec2) {
        let f = vec2(self.cam.target.x - self.cam.pos.x, self.cam.target.z - self.cam.pos.z).normalize_or_zero();
        let r = vec2(-f.y, f.x);
        (f, r)
    }

    pub fn update(&mut self, dt_real: f32, sfx: &Option<Sfx>) {
        let mut dt = dt_real.min(1.0 / 20.0);
        if self.slowmo > 0.0 {
            self.slowmo -= dt_real;
            dt *= 0.35;
        }
        self.time += dt;
        self.intro -= dt;
        for b in self.bells.iter_mut() {
            *b -= dt_real;
            if *b <= 0.0 {
                if let Some(s) = sfx {
                    s.play(&s.bell, 0.7);
                }
            }
        }
        self.bells.retain(|b| *b > 0.0);
        for m in self.msgs.iter_mut() {
            m.t += dt_real;
        }
        self.msgs.retain(|m| m.t < 2.2);
        self.excite = (self.excite - dt * 0.12).max(0.25);
        self.count_pop = (self.count_pop - dt_real * 2.5).max(0.0);

        // Eingaben
        let (cf, cr) = self.cam_basis();
        let mut intents = [Intent::default(); 2];
        for i in 0..2 {
            let (me, o) = if i == 0 {
                let (a, b) = self.w.split_at_mut(1);
                (&mut a[0], &b[0])
            } else {
                let (a, b) = self.w.split_at_mut(1);
                (&mut b[0], &a[0])
            };
            let it = match &mut me.ctrl {
                Ctrl::Human(k) => k.read(cf, cr),
                Ctrl::Cpu(_) => Intent::default(),
            };
            let it = if let Ctrl::Cpu(ai) = &mut me.ctrl {
                // KI braucht beide Wrestler; Ai temporär herausnehmen
                let mut tmp = std::mem::replace(ai, Ai::new(1.0));
                let r = ai_intent(me, o, &mut tmp, dt, self.time);
                if let Ctrl::Cpu(ai2) = &mut me.ctrl {
                    *ai2 = tmp;
                }
                r
            } else {
                it
            };
            intents[i] = it;
        }
        if self.intro > 0.0 || self.over.is_some() {
            intents = [Intent::default(); 2];
        }

        let events = sfx;
        for i in 0..2 {
            self.step(i, intents[i], dt, events);
        }
        self.separate();
        self.update_pin(dt, intents, sfx);
        self.update_ref(dt);
        self.ropes.update(dt);

        for w in self.w.iter_mut() {
            w.clock += dt;
            w.flash = (w.flash - dt * 4.0).max(0.0);
            w.update_pose(dt);
        }

        // Partikel
        for p in self.parts.iter_mut() {
            p.v.y -= 9.0 * dt;
            p.p += p.v * dt;
            p.life -= dt;
            if p.p.y < MAT_H + 0.01 && p.p.x.abs() < APRON && p.p.z.abs() < APRON {
                p.p.y = MAT_H + 0.01;
                p.v *= 0.2;
            }
        }
        self.parts.retain(|p| p.life > 0.0);
        // Blitzlichter im Publikum
        for f in self.flashes.iter_mut() {
            f.1 -= dt_real;
        }
        self.flashes.retain(|f| f.1 > 0.0);

        if let Some((winner, t)) = &mut self.over {
            *t += dt_real;
            let _ = winner;
        }

        if let Some(s) = sfx {
            s.ambience(0.18 + self.excite * 0.35);
        }
    }

    fn sound(&self, sfx: &Option<Sfx>, which: &str, vol: f32) {
        if let Some(s) = sfx {
            let snd = match which {
                "punch" => &s.punch,
                "kick" => &s.kick,
                "slam" => &s.slam,
                "thud" => &s.thud,
                "cheer" => &s.cheer,
                "slap" => &s.slap,
                "whoosh" => &s.whoosh,
                "rope" => &s.rope,
                _ => &s.bell,
            };
            s.play(snd, vol);
        }
    }

    fn pop(&mut self, amount: f32, sfx: &Option<Sfx>) {
        self.excite = (self.excite + amount).min(1.0);
        if amount > 0.25 {
            self.sound(sfx, "cheer", 0.35 + amount * 0.5);
        }
    }

    /// Kollision zwischen den beiden Wrestlern und mit den Seilen
    fn separate(&mut self) {
        let solid = |w: &Wrestler| matches!(w.st, St::Idle | St::Action { .. } | St::React { .. } | St::Run { .. } | St::GetUp { .. });
        if solid(&self.w[0]) && solid(&self.w[1]) {
            let d = self.w[1].pos - self.w[0].pos;
            let l = d.length();
            let min = 0.62;
            if l < min && l > 1e-4 {
                let push = d / l * (min - l) * 0.5;
                self.w[0].pos -= push;
                self.w[1].pos += push;
            }
        }
        for k in 0..2 {
            let w = &mut self.w[k];
            if matches!(w.st, St::Throw { .. } | St::Lockup { .. }) {
                continue;
            }
            let lim = if w.is_down() { BOUND + 0.15 } else { BOUND };
            let (side, pen, along) = ring_side(w.pos);
            if pen > 0.0 && !w.is_down() {
                self.ropes.push(side, (pen * 2.5).min(0.3), along);
            }
            w.pos.x = w.pos.x.clamp(-lim, lim);
            w.pos.y = w.pos.y.clamp(-lim, lim);
        }
    }

    fn start_pin(&mut self, att: usize) {
        let v = 1 - att;
        let vic = &self.w[v];
        let target = vic.pos + rot2(vec2(0.62, -0.30), vic.yaw) * vic.scale;
        let yaw = wrap(vic.yaw - PI / 2.0);
        let health = vic.health;
        let human = vic.is_human();
        self.w[att].cover_target = target;
        self.w[att].yaw = yaw;
        self.w[att].set(St::Cover { t: 0.0 });
        self.w[v].set(St::Pinned { t: 0.0 });
        // KI-Kickout vorbestimmen
        let p_kick = if health > 50.0 { 1.0 } else { ((health - 6.0) / 40.0).clamp(0.0, 0.97) };
        let kick_at = if human {
            f32::MAX
        } else if rand::gen_range(0.0, 1.0) < p_kick {
            rand::gen_range(0.5, 2.15)
        } else {
            f32::MAX
        };
        self.pin = Some(Pin { att, count: 0, kick: 0.0, kick_at, t: 0.0, counting: false, wait: 0.0 });
        self.msg("COVER!", WHITE, false);
    }

    fn update_pin(&mut self, dt: f32, intents: [Intent; 2], sfx: &Option<Sfx>) {
        let Some(pin) = &mut self.pin else { return };
        let att = pin.att;
        let v = 1 - att;
        pin.wait += dt;
        if pin.counting {
            pin.t += dt;
            if self.w[v].is_human() && intents[v].mash {
                pin.kick += 5.0 + self.w[v].health * 0.16;
            }
            pin.kick = (pin.kick - dt * 6.0).max(0.0);
            let kicked = pin.kick >= 100.0 || pin.t >= pin.kick_at;
            if kicked && pin.count < 3 {
                self.pin = None;
                self.w[v].set(St::Down { up: true, t: 0.0, dur: 0.5 });
                self.w[att].set(St::React { r: React::Pushed, t: 0.0 });
                self.referee.st = RefSt::Roam;
                self.msg("KICK-OUT!", Color::new(1.0, 0.85, 0.3, 1.0), true);
                self.pop(0.5, sfx);
                return;
            }
        }
        if let RefSt::Count { n, .. } = self.referee.st {
            if let Some(pin) = &mut self.pin {
                pin.counting = true;
                if n > pin.count {
                    pin.count = n;
                    self.count_pop = 1.0;
                    self.sound(sfx, "slap", 0.9);
                    self.excite = (self.excite + 0.2).min(1.0);
                    if n >= 3 {
                        let winner = att;
                        self.pin = None;
                        self.over = Some((winner, 0.0));
                        self.w[winner].set(St::Win { t: 0.0 });
                        self.w[v].set(St::Lost { t: 0.0 });
                        self.referee.st = RefSt::Raise { t: 0.0 };
                        self.bells = vec![0.15, 0.5, 0.85, 1.2];
                        self.pop(1.0, sfx);
                        let name = self.w[winner].name.clone();
                        self.msg(&format!("SIEGER: {}", name), Color::new(1.0, 0.85, 0.3, 1.0), true);
                    }
                }
            }
        }
    }

    fn update_ref(&mut self, dt: f32) {
        let r = &mut self.referee;
        r.clock += dt;
        let mid = (self.w[0].pos + self.w[1].pos) * 0.5;
        let mut desired = r.pos;
        let mut face = mid;
        let mut speed: f32 = 2.4;
        match (&self.pin, r.st) {
            (Some(pin), RefSt::Roam) => {
                let vic = &self.w[1 - pin.att];
                let mut spot = vic.pos + rot2(vec2(-0.55, -0.95), vic.yaw) * vic.scale;
                spot.x = spot.x.clamp(-BOUND, BOUND);
                spot.y = spot.y.clamp(-BOUND, BOUND);
                desired = spot;
                face = vic.pos + rot2(vec2(0., -0.6), vic.yaw);
                speed = 4.2;
                if (r.pos - spot).length() < 0.3 || pin.wait > 2.0 {
                    r.st = RefSt::Count { t: 0.0, n: 0 };
                }
            }
            (Some(pin), RefSt::Count { t, n }) => {
                let vic = &self.w[1 - pin.att];
                face = vic.pos + rot2(vec2(0., -0.6), vic.yaw);
                let t2 = t + dt;
                let period = 0.92;
                let n2 = ((t2 - 0.45) / period).floor().max(-1.0) as i32 + 1;
                let n2 = n2.clamp(0, 3) as u32;
                r.st = RefSt::Count { t: t2, n: n2.max(n) };
            }
            (None, RefSt::Count { .. }) => {
                r.st = RefSt::Roam;
            }
            (_, RefSt::Raise { .. }) => {
                if let Some((wi, _)) = self.over {
                    let wp = self.w[wi].pos;
                    desired = wp + (r.pos - wp).normalize_or_zero() * 1.2;
                    face = wp;
                }
            }
            _ => {
                // seitlich neben der Kampflinie bleiben
                let d = (self.w[1].pos - self.w[0].pos).normalize_or_zero();
                let side = vec2(-d.y, d.x);
                let s = if (r.pos - mid).dot(side) >= 0.0 { 1.0 } else { -1.0 };
                desired = mid + side * s * 2.1;
                for w in &self.w {
                    let away = r.pos - w.pos;
                    if away.length() < 1.1 {
                        desired += away.normalize_or_zero() * 1.0;
                    }
                }
            }
        }
        desired.x = desired.x.clamp(-BOUND, BOUND);
        desired.y = desired.y.clamp(-BOUND, BOUND);
        let counting = matches!(r.st, RefSt::Count { .. });
        let to = desired - r.pos;
        let want = if to.length() > 0.15 && !counting { to.normalize() * speed.min(to.length() * 3.0) } else { Vec2::ZERO };
        r.vel = r.vel.lerp(want, 1.0 - (-dt * 8.0).exp());
        r.pos += r.vel * dt;
        let fv = face - r.pos;
        if fv.length() > 0.05 {
            let ty = fv.x.atan2(fv.y);
            let d = wrap(ty - r.yaw);
            r.yaw = wrap(r.yaw + d.clamp(-8.0 * dt, 8.0 * dt));
        }
        let sp = r.vel.length();
        r.phase += dt * (sp * 4.2).max(0.0);
        let target = match r.st {
            RefSt::Count { t, .. } => {
                let ph = ((t - 0.45) / 0.92).fract();
                let slap = if t < 0.45 { 0.0 } else if ph < 0.25 { ph / 0.25 } else { 1.0 - (ph - 0.25) / 0.75 };
                ref_count(slap.clamp(0., 1.))
            }
            RefSt::Raise { t } => ref_stand(t).r(UARM_R, -10., 0., -160.).r(FARM_R, 0., 0., -15.),
            RefSt::Roam => {
                let w = (sp / 1.4).clamp(0., 1.);
                Pose::lerp(&ref_stand(r.clock), &walk(r.phase, (sp - 2.0).clamp(0., 1.)), w)
            }
        };
        if let RefSt::Raise { t } = &mut r.st {
            *t += dt;
        }
        let target = if matches!(r.st, RefSt::Count { .. }) { target } else { ground(target, &r.off) };
        r.pose = Pose::lerp(&r.pose, &target, 1.0 - (-dt * 14.0).exp());
    }

    fn pair(&mut self, i: usize) -> (&mut Wrestler, &mut Wrestler) {
        let (a, b) = self.w.split_at_mut(1);
        if i == 0 {
            (&mut a[0], &mut b[0])
        } else {
            (&mut b[0], &mut a[0])
        }
    }

    fn step(&mut self, i: usize, it: Intent, dt: f32, sfx: &Option<Sfx>) {
        let o_idx = 1 - i;
        let st = self.w[i].st;
        match st {
            St::Idle => self.step_idle(i, it, dt, sfx),
            St::Action { a, t, hit } => {
                let clip = Wrestler::act_clip(a);
                let t2 = t + dt;
                let hit_t = match a {
                    Act::Punch => 0.22,
                    Act::Kick => 0.33,
                    Act::Lariat => 0.24,
                    Act::Stomp => 0.36,
                    Act::KneeDrop => 0.66,
                    Act::Grab => 0.16,
                    Act::Whip | Act::Taunt => 99.0,
                };
                let me = &mut self.w[i];
                me.st = St::Action { a, t: t2, hit };
                // Restschwung
                me.vel *= (-dt * 3.5).exp();
                me.pos += me.vel * dt;
                if a == Act::Lariat && t2 < 0.3 {
                    // während des Sprints Richtung Gegner nachführen
                }
                if !hit && t2 >= hit_t {
                    if let St::Action { hit: h, .. } = &mut self.w[i].st {
                        *h = true;
                    }
                    if a == Act::Grab {
                        self.try_grab(i, sfx);
                        if !matches!(self.w[i].st, St::Action { .. }) {
                            return;
                        }
                    } else {
                        self.try_hit(i, a, sfx);
                    }
                }
                if t2 >= clip.dur() {
                    let me = &mut self.w[i];
                    let end = clip.sample(clip.dur());
                    let end = if me.mirror_action { end.mirror() } else { end };
                    me.transfer(end.root, end.rq, End::Stand);
                    if a == Act::Taunt {
                        me.special = (me.special + 18.0).min(100.0);
                    }
                    me.mirror_action = false;
                    me.set_hard(St::Idle);
                    me.vel = Vec2::ZERO;
                }
            }
            St::React { r, t } => {
                let clip = Wrestler::react_clip(r);
                let t2 = t + dt;
                let me = &mut self.w[i];
                me.st = St::React { r, t: t2 };
                me.vel *= (-dt * 4.0).exp();
                me.pos += me.vel * dt;
                if t2 >= clip.dur() {
                    let end = clip.sample(clip.dur());
                    me.transfer(end.root, end.rq, End::Stand);
                    me.set_hard(St::Idle);
                }
            }
            St::Lockup { t, init } => {
                let t2 = t + dt;
                self.w[i].st = St::Lockup { t: t2, init };
                self.w[i].stamina = (self.w[i].stamina - dt * 2.0).max(0.0);
                // nur der Initiator verwaltet den Clinch
                if !init {
                    // Konter-Versuch des Verteidigers
                    if it.strike || it.kick || it.grapple {
                        let th = if it.kick { Throw::Suplex } else if it.grapple { Throw::Slam } else { Throw::Slam };
                        self.resolve_lock(i, th, false, sfx);
                    }
                    return;
                }
                let choice = if it.pin {
                    Some(if self.w[i].special >= 100.0 { Some(Throw::Ddt) } else { Some(Throw::Slam) })
                } else if it.strike {
                    Some(Some(Throw::Slam))
                } else if it.kick {
                    Some(Some(Throw::Suplex))
                } else if it.grapple {
                    Some(None)
                } else {
                    None
                };
                match choice {
                    Some(Some(th)) => self.resolve_lock(i, th, true, sfx),
                    Some(None) => self.resolve_whip(i, sfx),
                    None => {
                        if t2 > 2.8 {
                            // Clinch löst sich
                            for k in [i, o_idx] {
                                let w = &mut self.w[k];
                                w.vel = -w.fwd() * 1.5;
                                w.set(St::React { r: React::Block, t: 0.0 });
                            }
                        }
                    }
                }
            }
            St::Throw { th, t, attacker, impact } => {
                let t2 = t + dt;
                self.w[i].st = St::Throw { th, t: t2, attacker, impact };
                if !attacker {
                    return;
                }
                let pc = Wrestler::pair_clip(th);
                if !impact && t2 >= pc.impact {
                    if let St::Throw { impact: im, .. } = &mut self.w[i].st {
                        *im = true;
                    }
                    let y = MAT_H;
                    let vp = self.w[o_idx].pelvis_world(y);
                    self.w[o_idx].health = (self.w[o_idx].health - pc.dmg * 0.85).max(0.0);
                    self.w[o_idx].flash = 1.0;
                    self.w[o_idx].hits_taken += 1;
                    self.w[i].special = (self.w[i].special + pc.dmg * 1.1).min(100.0);
                    self.w[o_idx].special = (self.w[o_idx].special + pc.dmg * 0.35).min(100.0);
                    self.cam.shake = 0.55;
                    self.sound(sfx, "slam", 1.0);
                    self.burst(vp + vec3(0., 0.1, 0.), 22, 2.2);
                    let col = if th == Throw::Ddt { Color::new(1.0, 0.8, 0.25, 1.0) } else { WHITE };
                    self.msg(pc.name, col, th == Throw::Ddt);
                    self.pop(if th == Throw::Ddt { 1.0 } else { 0.45 }, sfx);
                    // Seile vibrieren mit
                    for s in 0..4 {
                        self.ropes.impulse(s, 0.6, 0.0);
                    }
                }
                if t2 >= pc.dur() {
                    let (me, o) = self.pair(i);
                    let ea = pc.a.sample(pc.dur());
                    let ev = pc.v.sample(pc.dur());
                    me.pos = me.anchor.0;
                    me.yaw = me.anchor.1;
                    o.pos = me.anchor.0;
                    o.yaw = me.anchor.1;
                    me.transfer(ea.root, ea.rq, pc.a_end);
                    o.transfer(ev.root, ev.rq, pc.v_end);
                    me.st = match pc.a_end {
                        End::Stand => St::Idle,
                        End::LieUp => St::Down { up: true, t: 0.0, dur: 0.35 },
                        End::LieDown => St::Down { up: false, t: 0.0, dur: 0.35 },
                    };
                    me.blend = if pc.a_end == End::Stand { 0.0 } else { 1.0 };
                    me.snap = me.pose;
                    let base = 1.2 + (100.0 - o.health) / 100.0 * 2.4 + rand::gen_range(0.0, 0.6);
                    let dur = base + if th == Throw::Ddt { 2.2 } else { 0.0 };
                    o.set_hard(St::Down { up: pc.v_end == End::LieUp, t: 0.0, dur });
                    me.lock_choice = None;
                }
            }
            St::Fall { up, t, thud } => {
                let clip = if up { &clips().fall_back } else { &clips().fall_front };
                let t2 = t + dt;
                self.w[i].st = St::Fall { up, t: t2, thud };
                if !thud && t2 > 0.68 {
                    if let St::Fall { thud: th, .. } = &mut self.w[i].st {
                        *th = true;
                    }
                    self.sound(sfx, "thud", 0.9);
                    self.cam.shake = self.cam.shake.max(0.25);
                    let p = self.w[i].pelvis_world(MAT_H);
                    self.burst(p, 8, 1.4);
                }
                if t2 >= clip.dur() {
                    let me = &mut self.w[i];
                    let end = clip.sample(clip.dur());
                    me.transfer(end.root, end.rq, if up { End::LieUp } else { End::LieDown });
                    let dur = 0.9 + (100.0 - me.health) / 100.0 * 2.2 + rand::gen_range(0.0, 0.6);
                    me.set_hard(St::Down { up, t: 0.0, dur });
                }
            }
            St::Down { up, t, dur } => {
                let mut t2 = t + dt;
                if it.mash {
                    t2 += 0.16;
                }
                let me = &mut self.w[i];
                me.stamina = (me.stamina + dt * 14.0).min(100.0);
                me.st = St::Down { up, t: t2, dur };
                if t2 >= dur && self.over.is_none() {
                    me.set(St::GetUp { up, t: 0.0 });
                }
            }
            St::GetUp { up, t } => {
                let clip = if up { &clips().getup_back } else { &clips().getup_front };
                let t2 = t + dt;
                let me = &mut self.w[i];
                me.st = St::GetUp { up, t: t2 };
                if t2 >= clip.dur() {
                    let end = clip.sample(clip.dur());
                    me.transfer(end.root, end.rq, End::Stand);
                    me.set_hard(St::Idle);
                }
            }
            St::Run { dir, t, n, rebound } => {
                let me = &mut self.w[i];
                let mut dir = dir;
                let mut n = n;
                let mut rebound = rebound;
                if rebound > 0.0 {
                    rebound -= dt;
                    if rebound <= 0.0 {
                        dir = -dir;
                        me.yaw = dir.x.atan2(dir.y);
                        me.snap = me.pose;
                        me.blend = 0.0;
                    }
                    me.st = St::Run { dir, t: t + dt, n, rebound };
                    return;
                }
                me.phase += dt * 11.0;
                me.pos += dir * 5.4 * dt;
                me.speed = 5.4;
                me.vel = dir * 5.4;
                me.turn_to(dir.x.atan2(dir.y), 16.0, dt);
                let ahead = me.pos + dir * 0.1;
                if ahead.x.abs() > BOUND || ahead.y.abs() > BOUND {
                    let (side, _, along) = ring_side(me.pos);
                    if n == 0 {
                        n = 1;
                        rebound = 0.36;
                        me.snap = me.pose;
                        me.blend = 0.0;
                        me.yaw = (-dir.x).atan2(-dir.y);
                        self.ropes.impulse(side, 2.6, along);
                        self.sound(sfx, "rope", 0.9);
                        self.w[i].st = St::Run { dir, t: t + dt, n, rebound };
                        return;
                    } else {
                        self.ropes.impulse(side, 1.2, along);
                        let me = &mut self.w[i];
                        me.vel = Vec2::ZERO;
                        me.set(St::React { r: React::Block, t: 0.0 });
                        return;
                    }
                }
                me.st = St::Run { dir, t: t + dt, n, rebound };
                if it.strike && n >= 1 {
                    me.vel = dir * 4.5;
                    me.mirror_action = false;
                    me.set(St::Action { a: Act::Lariat, t: 0.0, hit: false });
                    self.sound(sfx, "whoosh", 0.5);
                }
            }
            St::Cover { t } => {
                let me = &mut self.w[i];
                me.st = St::Cover { t: t + dt };
                me.pos = me.pos.lerp(me.cover_target, 1.0 - (-dt * 10.0).exp());
            }
            St::Pinned { t } => {
                self.w[i].st = St::Pinned { t: t + dt };
            }
            St::Win { t } => {
                let me = &mut self.w[i];
                me.st = St::Win { t: t + dt };
                me.yaw = wrap(me.yaw + dt * 0.5);
            }
            St::Lost { t } => {
                self.w[i].st = St::Lost { t: t + dt };
            }
        }
    }

    fn step_idle(&mut self, i: usize, it: Intent, dt: f32, sfx: &Option<Sfx>) {
        let o_idx = 1 - i;
        let opos = self.w[o_idx].pos;
        let o_down = matches!(self.w[o_idx].st, St::Down { .. });
        let o_down_any = self.w[o_idx].is_down();
        let o_up = matches!(self.w[o_idx].st, St::Down { up: true, .. });
        let obody = self.w[o_idx].body_points();
        let me = &mut self.w[i];
        me.blocking = it.block;
        let running = it.run && it.mv.length() > 0.1 && me.stamina > 3.0 && !it.block;
        let max_speed = if it.block { 0.9 } else if running { 4.6 } else { 1.9 };
        let want = it.mv * max_speed;
        me.vel = me.vel.lerp(want, 1.0 - (-dt * 9.0).exp());
        me.pos += me.vel * dt;
        me.speed = me.vel.length();
        me.phase += dt * (me.speed * 3.4 + if me.speed > 0.1 { 1.5 } else { 0.0 });
        me.stamina = (me.stamina + dt * if running { -9.0 } else { 9.0 }).clamp(0.0, 100.0);
        me.combo_t -= dt;
        if me.combo_t < 0.0 {
            me.combo = 0;
        }
        let to_o = opos - me.pos;
        if running {
            me.turn_to(it.mv.x.atan2(it.mv.y), 12.0, dt);
        } else if to_o.length() > 0.05 && (to_o.length() < 6.0 || o_down_any) {
            let tgt = if o_down_any { obody[1] - me.pos } else { to_o };
            me.turn_to(tgt.x.atan2(tgt.y), 9.0, dt);
        } else if it.mv.length() > 0.1 {
            me.turn_to(it.mv.x.atan2(it.mv.y), 9.0, dt);
        }
        if it.block {
            return;
        }
        let near_down = obody.iter().map(|p| (*p - me.pos).length()).fold(f32::MAX, f32::min);
        let act = |me: &mut Wrestler, a: Act, cost: f32| {
            me.stamina = (me.stamina - cost).max(0.0);
            me.set(St::Action { a, t: 0.0, hit: false });
        };
        if it.strike {
            if me.speed > 3.0 {
                me.mirror_action = false;
                act(me, Act::Lariat, 8.0);
                self.sound(sfx, "whoosh", 0.5);
            } else if o_down && near_down < 1.5 {
                me.mirror_action = false;
                act(me, Act::Stomp, 4.0);
            } else {
                me.punch_alt = !me.punch_alt;
                me.mirror_action = me.punch_alt;
                act(me, Act::Punch, 4.0);
                self.sound(sfx, "whoosh", 0.25);
            }
        } else if it.kick {
            me.mirror_action = false;
            if o_down && near_down < 1.9 {
                me.vel = Vec2::ZERO;
                act(me, Act::KneeDrop, 8.0);
            } else {
                act(me, Act::Kick, 8.0);
                self.sound(sfx, "whoosh", 0.35);
            }
        } else if it.grapple {
            if !o_down_any {
                me.mirror_action = false;
                act(me, Act::Grab, 5.0);
            }
        } else if it.pin {
            let pd = (obody[0] - me.pos).length().min((obody[1] - me.pos).length());
            if o_up && pd < 1.7 && self.pin.is_none() && self.over.is_none() {
                self.start_pin(i);
            } else if to_o.length() > 1.6 && !o_down_any || (o_down_any && pd > 1.7) {
                let me = &mut self.w[i];
                me.mirror_action = false;
                me.vel = Vec2::ZERO;
                me.set(St::Action { a: Act::Taunt, t: 0.0, hit: false });
                self.pop(0.15, sfx);
            }
        }
    }

    fn try_grab(&mut self, i: usize, sfx: &Option<Sfx>) {
        let o_idx = 1 - i;
        let (me, o) = self.pair(i);
        let to = o.pos - me.pos;
        let ok_state = matches!(o.st, St::Idle | St::React { r: React::High | React::Body, .. } | St::Action { a: Act::Punch | Act::Kick | Act::Taunt | Act::Grab, .. });
        if to.length() < 1.25 && to.normalize_or_zero().dot(me.fwd()) > 0.5 && ok_state {
            let fwd = me.fwd();
            o.pos = me.pos + fwd * LOCK_D;
            o.yaw = wrap(me.yaw + PI);
            o.vel = Vec2::ZERO;
            me.vel = Vec2::ZERO;
            me.set(St::Lockup { t: 0.0, init: true });
            o.set(St::Lockup { t: 0.0, init: false });
            o.mirror_action = false;
            if let Ctrl::Cpu(ai) = &mut me.ctrl {
                ai.lock_delay = rand::gen_range(0.35, 1.1) / ai.diff;
            }
            let human = me.is_human();
            let _ = o_idx;
            self.sound(sfx, "punch", 0.35);
            if human {
                self.msg("CLINCH!  J Slam · K Suplex · L Whip · Leer Finisher", WHITE, false);
            } else {
                self.msg("CLINCH!", WHITE, false);
            }
        }
    }

    fn resolve_lock(&mut self, chooser: usize, th: Throw, initiator: bool, sfx: &Option<Sfx>) {
        let other = 1 - chooser;
        let (a, b) = (&self.w[chooser], &self.w[other]);
        let mut chance = if initiator { 0.64 } else { 0.30 };
        chance += (a.stamina - b.stamina) / 260.0 + (a.health - b.health) / 320.0;
        if th == Throw::Ddt {
            chance += 0.18;
        }
        let chance = chance.clamp(0.12, 0.92);
        self.w[chooser].stamina = (self.w[chooser].stamina - 10.0).max(0.0);
        if rand::gen_range(0.0, 1.0) < chance {
            if th == Throw::Ddt {
                self.w[chooser].special = 0.0;
                self.slowmo = 0.55;
            }
            self.start_throw(chooser, th);
        } else {
            let rev = if rand::gen_range(0.0, 1.0) < 0.5 { Throw::Slam } else { Throw::Suplex };
            self.msg("KONTER!", Color::new(1.0, 0.6, 0.3, 1.0), false);
            self.pop(0.3, sfx);
            self.start_throw(other, rev);
        }
    }

    fn resolve_whip(&mut self, i: usize, sfx: &Option<Sfx>) {
        let (me, o) = self.pair(i);
        let dir = me.fwd();
        me.mirror_action = false;
        me.set(St::Action { a: Act::Whip, t: 0.0, hit: false });
        o.set(St::Run { dir, t: 0.0, n: 0, rebound: 0.0 });
        o.yaw = dir.x.atan2(dir.y);
        self.msg("IRISH WHIP", WHITE, false);
        self.sound(sfx, "whoosh", 0.6);
    }

    fn start_throw(&mut self, att: usize, th: Throw) {
        let (me, o) = self.pair(att);
        // Opfer exakt vor den Angreifer setzen
        let anchor = (me.pos, me.yaw);
        me.anchor = anchor;
        o.anchor = anchor;
        me.set(St::Throw { th, t: 0.0, attacker: true, impact: false });
        o.set(St::Throw { th, t: 0.0, attacker: false, impact: false });
        // Überblendung für das Opfer im Angreiferraum: Schnappschuss umrechnen
        let rel = rot2(o.pos - anchor.0, -anchor.1) / o.scale;
        let dyaw = wrap(o.yaw - anchor.1);
        o.snap.root = vec3(rel.x, 0.0, rel.y) + Quat::from_rotation_y(dyaw) * o.snap.root;
        o.snap.rq = Quat::from_rotation_y(dyaw) * o.snap.rq;
    }

    fn try_hit(&mut self, i: usize, a: Act, sfx: &Option<Sfx>) {
        let o_idx = 1 - i;
        let me = &self.w[i];
        let o = &self.w[o_idx];
        let fwd = me.fwd();
        let (reach, radius, dmg) = match a {
            Act::Punch => (0.78, 0.55, 5.0),
            Act::Kick => (0.95, 0.58, 8.5),
            Act::Lariat => (0.55, 0.75, 11.0),
            Act::Stomp => (0.75, 0.6, 5.5),
            Act::KneeDrop => (0.62, 0.62, 11.0),
            _ => return,
        };
        let root_off = rot2(vec2(me.pose.root.x, me.pose.root.z), me.yaw) * me.scale;
        let hp = me.pos + root_off + fwd * reach;
        let ground_attack = matches!(a, Act::Stomp | Act::KneeDrop);
        let pts = o.body_points();
        let dist = if ground_attack {
            pts.iter().map(|p| (*p - hp).length()).fold(f32::MAX, f32::min)
        } else {
            (o.pos - hp).length()
        };
        let valid = if ground_attack { matches!(o.st, St::Down { .. }) } else { o.standing_hittable() };
        if !valid || dist > radius {
            return;
        }
        let o_running = matches!(o.st, St::Run { rebound, .. } if rebound <= 0.0);
        let to_me = (me.pos - o.pos).normalize_or_zero();
        let blocked = !ground_attack && o.blocking && matches!(o.st, St::Idle) && o.fwd().dot(to_me) > 0.3 && a != Act::Lariat;
        let hit_point = vec3(hp.x, MAT_H + if ground_attack { 0.35 } else { 1.45 }, hp.y);
        let my_speed = me.vel.length();
        let my_human = me.is_human();
        if blocked {
            let o = &mut self.w[o_idx];
            o.health = (o.health - dmg * 0.15).max(0.0);
            o.stamina = (o.stamina - 6.0).max(0.0);
            o.vel = -to_me * 1.4;
            o.set(St::React { r: React::Block, t: 0.0 });
            self.sound(sfx, "punch", 0.4);
            self.burst(hit_point, 3, 0.8);
            if my_human {
                self.msg("GEBLOCKT", Color::new(0.7, 0.8, 1.0, 1.0), false);
            }
            return;
        }
        let me = &mut self.w[i];
        me.special = (me.special + dmg * 1.0).min(100.0);
        if a == Act::Punch {
            me.combo += 1;
            me.combo_t = 1.3;
        }
        let combo = me.combo;
        let o = &mut self.w[o_idx];
        let dmg = 0.8 * dmg * if o_running { 1.4 } else { 1.0 } + if a == Act::Lariat { my_speed * 0.8 } else { 0.0 };
        o.health = (o.health - dmg).max(0.0);
        o.special = (o.special + dmg * 0.4).min(100.0);
        o.flash = 0.7;
        o.hits_taken += 1;
        let low = o.health < 40.0;
        let knock = match a {
            Act::Lariat => true,
            Act::Kick => o_running || rand::gen_range(0.0, 1.0) < if low { 0.55 } else { 0.22 },
            Act::Punch => o_running || (combo >= 3 && rand::gen_range(0.0, 1.0) < 0.55) || (low && rand::gen_range(0.0, 1.0) < 0.12),
            _ => false,
        };
        if ground_attack {
            if let St::Down { up, t, dur } = o.st {
                o.st = St::Down { up, t: t.min(dur - 0.8).max(0.0), dur: dur + 0.25 };
            }
        } else if knock {
            o.vel = Vec2::ZERO;
            o.yaw = to_me.x.atan2(to_me.y);
            o.set(St::Fall { up: true, t: 0.0, thud: false });
        } else {
            o.vel = -to_me * if a == Act::Kick { 1.6 } else { 0.9 };
            let r = if a == Act::Kick && rand::gen_range(0.0, 1.0) < 0.5 { React::Body } else { React::High };
            o.yaw = to_me.x.atan2(to_me.y);
            o.set(St::React { r, t: 0.0 });
        }
        let snd = if matches!(a, Act::Punch) { "punch" } else { "kick" };
        self.sound(sfx, snd, 0.9);
        self.burst(hit_point, if knock { 14 } else { 7 }, if knock { 1.8 } else { 1.1 });
        self.cam.shake = self.cam.shake.max(if knock { 0.3 } else { 0.12 });
        if knock {
            self.pop(0.35, sfx);
            let name = match a {
                Act::Lariat => "CLOTHESLINE!",
                Act::Kick => "BIG BOOT!",
                _ => "NIEDERGESCHLAGEN!",
            };
            self.msg(name, WHITE, false);
        } else {
            self.excite = (self.excite + 0.05).min(1.0);
        }
    }

    // ------------------------------------------------------------- Kamera

    pub fn update_camera(&mut self, dt: f32, menu: bool) {
        let c = &mut self.cam;
        if menu {
            c.yaw += dt * 0.12;
            c.target = c.target.lerp(vec3(0., MAT_H + 0.9, 0.), 1.0 - (-dt * 2.0).exp());
            c.dist = 10.5;
            c.pitch = 0.30;
        } else {
            if is_key_down(KeyCode::Q) {
                c.yaw -= dt * 1.6;
            }
            if is_key_down(KeyCode::E) {
                c.yaw += dt * 1.6;
            }
            if is_mouse_button_down(MouseButton::Right) {
                let d = mouse_delta_position();
                c.yaw += d.x * 2.5;
                c.pitch = (c.pitch - d.y * 1.5).clamp(0.08, 1.1);
            }
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                c.zoom = (c.zoom * if wheel > 0.0 { 0.92 } else { 1.08 }).clamp(0.55, 1.8);
            }
            let p0 = self.w[0].pelvis_world(MAT_H);
            let p1 = self.w[1].pelvis_world(MAT_H);
            let mid = (p0 + p1) * 0.5;
            let sep = (p0 - p1).length();
            let tgt = vec3(mid.x * 0.85, MAT_H + 0.85, mid.z * 0.85);
            c.target = c.target.lerp(tgt, 1.0 - (-dt * 4.0).exp());
            let want = (5.2 + sep * 0.75).clamp(5.5, 10.0) * c.zoom;
            c.dist += (want - c.dist) * (1.0 - (-dt * 2.0).exp());
        }
        let off = vec3(c.yaw.sin() * c.pitch.cos(), c.pitch.sin(), c.yaw.cos() * c.pitch.cos()) * c.dist;
        let mut pos = c.target + off;
        c.shake = (c.shake - dt * 1.6).max(0.0);
        if c.shake > 0.0 {
            let s = c.shake * c.shake * 0.25;
            pos += vec3(rand::gen_range(-s, s), rand::gen_range(-s, s), rand::gen_range(-s, s));
        }
        c.pos = pos;
        if menu {
            c.pitch = 0.30;
        }
        let _ = PI;
    }

    // ------------------------------------------------------------- Darstellung

    pub fn draw_world(&mut self, gfx: &Gfx, arena: &Arena, dt: f32) {
        let c = &self.cam;
        set_camera(&Camera3D {
            position: c.pos,
            target: c.target,
            up: Vec3::Y,
            fovy: 42f32.to_radians(),
            z_near: 0.15,
            z_far: 300.0,
            ..Default::default()
        });
        clear_background(Color::new(0.005, 0.005, 0.01, 1.0));
        gfx.frame_uniforms(c.pos, self.time, self.excite);
        gl_use_material(&gfx.lit);
        for m in &arena.statics {
            draw_mesh(m);
        }
        for m in &arena.crowd {
            draw_mesh(m);
        }
        for m in &arena.textured {
            draw_mesh(m);
        }
        let ropes = self.ropes.mesh();
        draw_mesh(&ropes);

        // Schatten
        gl_use_material(&gfx.shadow);
        let sm = shadow_matrix(vec3(0.12, -1.0, 0.08).normalize(), MAT_H + 0.012);
        for w in &self.w {
            let jw = w.joint_world(MAT_H);
            for k in 0..NJ {
                draw_at(&w.meshes[k], sm * jw[k]);
            }
        }
        {
            let r = &self.referee;
            let cm = Mat4::from_translation(vec3(r.pos.x, MAT_H, r.pos.y)) * Mat4::from_rotation_y(r.yaw);
            let f = fk(&r.pose, &r.off);
            for k in 0..NJ {
                draw_at(&r.meshes[k], sm * cm * Mat4::from_rotation_translation(f[k].0, f[k].1));
            }
        }

        // Figuren
        gl_use_material(&gfx.lit);
        for w in &self.w {
            let jw = w.joint_world(MAT_H);
            gfx.lit.set_uniform("Flash", vec4(1.0, 0.85, 0.75, w.flash * 0.35));
            for k in 0..NJ {
                draw_at(&w.meshes[k], jw[k]);
            }
        }
        gfx.lit.set_uniform("Flash", vec4(0., 0., 0., 0.));
        {
            let r = &self.referee;
            let cm = Mat4::from_translation(vec3(r.pos.x, MAT_H, r.pos.y)) * Mat4::from_rotation_y(r.yaw);
            let f = fk(&r.pose, &r.off);
            for k in 0..NJ {
                draw_at(&r.meshes[k], cm * Mat4::from_rotation_translation(f[k].0, f[k].1));
            }
        }
        for p in &self.parts {
            draw_at(&self.drop_mesh, Mat4::from_scale_rotation_translation(Vec3::splat(p.size), Quat::IDENTITY, p.p));
        }
        // Blitzlichter
        let rate = 3.0 + self.excite * 25.0;
        if !arena.flash_spots.is_empty() && rand::gen_range(0.0, 1.0) < rate * dt {
            let k = rand::gen_range(0, arena.flash_spots.len());
            self.flashes.push((arena.flash_spots[k], 0.07));
        }
        for (p, _) in &self.flashes {
            draw_at(&self.flash_mesh, Mat4::from_scale_rotation_translation(Vec3::splat(0.07), Quat::IDENTITY, *p));
        }
        gl_use_default_material();
    }

    /// Testhilfe: bestimmte Aktion sofort auslösen
    pub fn debug_force(&mut self, what: &str) {
        let act = |w: &mut Wrestler, a: Act| w.set(St::Action { a, t: 0.0, hit: false });
        match what {
            "slam" => self.start_throw(0, Throw::Slam),
            "suplex" => self.start_throw(0, Throw::Suplex),
            "ddt" => self.start_throw(0, Throw::Ddt),
            "fall" => self.w[1].set(St::Fall { up: true, t: 0.0, thud: false }),
            "fallf" => self.w[1].set(St::Fall { up: false, t: 0.0, thud: false }),
            "getup" => self.w[1].set_hard(St::Down { up: true, t: 0.0, dur: 0.2 }),
            "getupf" => self.w[1].set_hard(St::Down { up: false, t: 0.0, dur: 0.2 }),
            "pin" => {
                self.w[1].health = 5.0;
                self.w[1].set_hard(St::Down { up: true, t: 0.0, dur: 50.0 });
                self.start_pin(0);
            }
            "punch" => act(&mut self.w[0], Act::Punch),
            "kick" => act(&mut self.w[0], Act::Kick),
            "lariat" => act(&mut self.w[0], Act::Lariat),
            "taunt" => act(&mut self.w[0], Act::Taunt),
            "whip" => self.resolve_whip(0, &None),
            "stomp" => {
                self.w[1].set_hard(St::Down { up: true, t: 0.0, dur: 50.0 });
                act(&mut self.w[0], Act::Stomp);
            }
            "knee" => {
                self.w[1].set_hard(St::Down { up: true, t: 0.0, dur: 50.0 });
                act(&mut self.w[0], Act::KneeDrop);
            }
            _ => {}
        }
    }
}
