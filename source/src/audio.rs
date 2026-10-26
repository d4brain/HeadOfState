//! Prozedural synthetisierte Sounds (keine externen Dateien nötig).
//! Erzeugt direkt 44.1 kHz Stereo (keine Nachabtastung im Mixer) und lässt
//! genug Headroom, damit sich überlagernde Sounds nicht übersteuern.
use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, PlaySoundParams, Sound};
use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

const RATE: u32 = 44100;
const SR: f32 = RATE as f32;
/// Gesamtlautstärke aller Effekte (Mixer addiert ohne Begrenzer)
const MASTER: f32 = 0.5;

struct Rng(u32);
impl Rng {
    fn n(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
    fn u(&mut self) -> f32 {
        self.n() * 0.5 + 0.5
    }
}

/// Biquad-Filter (RBJ)
#[derive(Clone, Copy)]
struct Bq {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Bq {
    fn new(b: [f32; 3], a: [f32; 3]) -> Bq {
        Bq { b0: b[0] / a[0], b1: b[1] / a[0], b2: b[2] / a[0], a1: a[1] / a[0], a2: a[2] / a[0], x1: 0., x2: 0., y1: 0., y2: 0. }
    }
    fn lp(f: f32, q: f32) -> Bq {
        let w = TAU * f / SR;
        let al = w.sin() / (2. * q);
        let c = w.cos();
        Bq::new([(1. - c) / 2., 1. - c, (1. - c) / 2.], [1. + al, -2. * c, 1. - al])
    }
    fn hp(f: f32, q: f32) -> Bq {
        let w = TAU * f / SR;
        let al = w.sin() / (2. * q);
        let c = w.cos();
        Bq::new([(1. + c) / 2., -(1. + c), (1. + c) / 2.], [1. + al, -2. * c, 1. - al])
    }
    fn bp(f: f32, q: f32) -> Bq {
        let w = TAU * f / SR;
        let al = w.sin() / (2. * q);
        let c = w.cos();
        Bq::new([al, 0., -al], [1. + al, -2. * c, 1. - al])
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// WAV 16 Bit Stereo; l/r getrennt (für leichte Stereobreite)
fn wav(l: &[f32], r: &[f32], peak: f32) -> Vec<u8> {
    let n = l.len().min(r.len());
    let mut o = Vec::with_capacity(44 + n * 4);
    let data_len = (n * 4) as u32;
    o.extend_from_slice(b"RIFF");
    o.extend_from_slice(&(36 + data_len).to_le_bytes());
    o.extend_from_slice(b"WAVEfmt ");
    o.extend_from_slice(&16u32.to_le_bytes());
    o.extend_from_slice(&1u16.to_le_bytes());
    o.extend_from_slice(&2u16.to_le_bytes());
    o.extend_from_slice(&RATE.to_le_bytes());
    o.extend_from_slice(&(RATE * 4).to_le_bytes());
    o.extend_from_slice(&4u16.to_le_bytes());
    o.extend_from_slice(&16u16.to_le_bytes());
    o.extend_from_slice(b"data");
    o.extend_from_slice(&data_len.to_le_bytes());
    let m = l.iter().chain(r.iter()).fold(0.0f32, |m, v| m.max(v.abs())).max(1e-5);
    let g = peak / m;
    for i in 0..n {
        for v in [l[i], r[i]] {
            // weiche Sättigung statt hartem Clipping
            let x = (v * g).tanh();
            let s = (x * 32000.0) as i16;
            o.extend_from_slice(&s.to_le_bytes());
        }
    }
    o
}

fn buf(sec: f32) -> Vec<f32> {
    vec![0.0; (sec * SR) as usize]
}

/// kurze Ein-/Ausblendung gegen Knackser
fn fade(s: &mut [f32], fin: f32, fout: f32) {
    let n = s.len();
    let a = ((fin * SR) as usize).min(n);
    let b = ((fout * SR) as usize).min(n);
    for i in 0..a {
        s[i] *= i as f32 / a as f32;
    }
    for i in 0..b {
        s[n - 1 - i] *= i as f32 / b as f32;
    }
}

/// Körpertreffer: dumpfer Schlag + kurzes gefiltertes Rauschen
fn hit(seed: u32, f0: f32, len: f32, bright: f32) -> Vec<f32> {
    let mut r = Rng(seed);
    let mut s = buf(len);
    let mut ph = 0.0f32;
    let mut lp = Bq::lp(600.0 + bright * 3000.0, 0.7);
    let mut lp2 = Bq::lp(600.0 + bright * 3000.0, 0.7);
    for (i, v) in s.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let f = f0 * (1.0 + 1.2 * (-t * 35.0).exp());
        ph += f / SR;
        let body = (ph * TAU).sin() * (-t * 20.0).exp();
        let nz = lp2.run(lp.run(r.n())) * (-t * 38.0).exp();
        *v = body * 0.9 + nz * 1.6;
    }
    fade(&mut s, 0.001, 0.02);
    s
}

fn slam() -> Vec<f32> {
    let mut r = Rng(99);
    let mut s = buf(1.2);
    let mut ph = 0.0f32;
    let mut lp = Bq::lp(900.0, 0.7);
    let mut lp2 = Bq::lp(900.0, 0.7);
    let mut res = [Bq::bp(173.0, 18.0), Bq::bp(231.0, 18.0), Bq::bp(389.0, 22.0), Bq::bp(517.0, 25.0)];
    for (i, v) in s.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let f = 46.0 + 60.0 * (-t * 14.0).exp();
        ph += f / SR;
        let boom = (ph * TAU).sin() * (-t * 5.5).exp();
        let nz = lp2.run(lp.run(r.n()));
        let exc = nz * (-t * 10.0).exp();
        let mut ring = 0.0;
        for b in res.iter_mut() {
            ring += b.run(exc * 0.6);
        }
        *v = boom * 1.0 + exc * 1.3 + ring * 0.5;
    }
    fade(&mut s, 0.001, 0.15);
    s
}

fn bell() -> Vec<f32> {
    let mut s = buf(2.4);
    for (i, v) in s.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let f = 840.0;
        let parts = [(1.0, 1.0, 1.4), (2.41, 0.45, 2.2), (3.93, 0.25, 3.5), (5.43, 0.12, 5.0), (0.5, 0.2, 1.0)];
        let mut x = 0.0;
        for (ratio, amp, dec) in parts {
            x += (t * f * ratio * TAU).sin() * amp * (-t * dec).exp();
        }
        *v = x * (1.0 - (-t * 600.0).exp());
    }
    fade(&mut s, 0.0, 0.2);
    s
}

/// Publikum: viele Stimmformanten mit langsamer, zufälliger Lautstärke
fn crowd_layer(seed: u32, sec: f32, cheer: bool) -> Vec<f32> {
    let mut r = Rng(seed);
    let mut s = buf(sec);
    let centers: [f32; 7] = [220.0, 380.0, 560.0, 820.0, 1150.0, 1700.0, 2500.0];
    let mut bands: Vec<Bq> = centers.iter().map(|c| Bq::bp(*c, 2.2)).collect();
    // jede Band hat eigene langsame Modulation (Rauschen tiefpassgefiltert)
    let mut mods: Vec<Bq> = centers.iter().map(|_| Bq::lp(2.5, 0.7)).collect();
    let weights: [f32; 7] = if cheer { [0.7, 1.0, 1.0, 0.9, 0.7, 0.45, 0.25] } else { [1.0, 1.0, 0.8, 0.5, 0.3, 0.15, 0.06] };
    let mut tone = Bq::lp(if cheer { 2600.0 } else { 1300.0 }, 0.7);
    let mut tone2 = Bq::lp(if cheer { 2600.0 } else { 1300.0 }, 0.7);
    let mut hp = Bq::hp(120.0, 0.7);
    let n = s.len();
    for (i, v) in s.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let x = r.n();
        let mut acc = 0.0;
        for k in 0..centers.len() {
            let m = (mods[k].run(r.n()) * 30.0 + 0.6).clamp(0.05, 1.6);
            acc += bands[k].run(x) * weights[k] * m;
        }
        let env = if cheer {
            let up = (t / 0.4).min(1.0);
            up * up * (-(t - 0.4).max(0.0) * 0.8).exp()
        } else {
            1.0
        };
        *v = hp.run(tone2.run(tone.run(acc))) * env;
    }
    if cheer {
        // ein paar Pfiffe
        for k in 0..4 {
            let st = ((0.25 + k as f32 * 0.45 + r.u() * 0.2) * SR) as usize;
            let f0 = 1500.0 + r.u() * 700.0;
            let len = (0.4 * SR) as usize;
            let mut ph = 0.0f32;
            for j in 0..len {
                let idx = st + j;
                if idx >= n {
                    break;
                }
                let u = j as f32 / len as f32;
                let f = f0 * (1.0 + 0.35 * u);
                ph += f / SR;
                let env = (u * 12.0).min(1.0) * (1.0 - u);
                s[idx] += (ph * TAU).sin() * 0.06 * env;
            }
        }
        fade(&mut s, 0.01, 0.3);
    }
    s
}

/// nahtlose Schleife durch Überblenden des Endes in den Anfang
fn make_loop(mut s: Vec<f32>, fade_sec: f32) -> Vec<f32> {
    let n = s.len();
    let f = (fade_sec * SR) as usize;
    for j in 0..f {
        let w = j as f32 / f as f32;
        let w_in = (w * PI * 0.5).sin();
        let w_out = (w * PI * 0.5).cos();
        let tail = s[n - f + j];
        s[j] = s[j] * w_in + tail * w_out;
    }
    s.truncate(n - f);
    s
}

fn whoosh() -> Vec<f32> {
    let mut r = Rng(5);
    let mut s = buf(0.25);
    let len = s.len() as f32;
    let mut f = Bq::bp(500.0, 1.5);
    for (i, v) in s.iter_mut().enumerate() {
        let u = i as f32 / len;
        if i % 64 == 0 {
            let st = (f.x1, f.x2, f.y1, f.y2);
            f = Bq::bp(400.0 + 1800.0 * u, 1.4);
            (f.x1, f.x2, f.y1, f.y2) = st;
        }
        *v = f.run(r.n()) * (u * PI).sin().powf(1.5);
    }
    fade(&mut s, 0.005, 0.02);
    s
}

fn rope() -> Vec<f32> {
    let mut s = buf(0.6);
    for (i, v) in s.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let f = 92.0 * (1.0 + 0.05 * (-t * 10.0).exp());
        *v = ((t * f * TAU).sin() + 0.4 * (t * f * 2.02 * TAU).sin() + 0.2 * (t * f * 3.1 * TAU).sin()) * (-t * 7.0).exp();
    }
    let h = hit(17, 70.0, 0.2, 0.15);
    for (k, v) in h.iter().enumerate() {
        if k < s.len() {
            s[k] += *v * 0.5;
        }
    }
    fade(&mut s, 0.002, 0.05);
    s
}

/// leicht verzögerte Kopie als zweiter Kanal => räumlicher Eindruck ohne Phasenprobleme
fn stereo(s: &[f32], delay_ms: f32) -> (Vec<f32>, Vec<f32>) {
    let d = (delay_ms / 1000.0 * SR) as usize;
    let mut r = vec![0.0; s.len()];
    for i in d..s.len() {
        r[i] = s[i] * 0.85 + s[i - d] * 0.15;
    }
    for i in 0..d.min(s.len()) {
        r[i] = s[i] * 0.85;
    }
    (s.to_vec(), r)
}

pub struct Sfx {
    pub punch: Sound,
    pub kick: Sound,
    pub slam: Sound,
    pub thud: Sound,
    pub bell: Sound,
    pub crowd: Sound,
    pub cheer: Sound,
    pub slap: Sound,
    pub whoosh: Sound,
    pub rope: Sound,
    last: RefCell<HashMap<usize, f64>>,
}

impl Sfx {
    pub async fn load() -> Option<Sfx> {
        async fn mono(v: Vec<f32>, peak: f32) -> Option<Sound> {
            let (l, r) = stereo(&v, 0.0);
            load_sound_from_bytes(&wav(&l, &r, peak)).await.ok()
        }
        let crowd_l = make_loop(crowd_layer(3, 9.0, false), 1.0);
        let crowd_r = make_loop(crowd_layer(7, 9.0, false), 1.0);
        let cheer_l = crowd_layer(41, 3.4, true);
        let cheer_r = crowd_layer(43, 3.4, true);
        let mut slap = hit(31, 160.0, 0.14, 0.7);
        fade(&mut slap, 0.0, 0.03);
        Some(Sfx {
            punch: mono(hit(7, 110.0, 0.22, 0.35), 0.8).await?,
            kick: mono(hit(11, 75.0, 0.3, 0.25), 0.85).await?,
            slam: mono(slam(), 0.9).await?,
            thud: mono(hit(23, 55.0, 0.45, 0.12), 0.8).await?,
            bell: mono(bell(), 0.7).await?,
            crowd: load_sound_from_bytes(&wav(&crowd_l, &crowd_r, 0.7)).await.ok()?,
            cheer: load_sound_from_bytes(&wav(&cheer_l, &cheer_r, 0.7)).await.ok()?,
            slap: mono(slap, 0.7).await?,
            whoosh: mono(whoosh(), 0.6).await?,
            rope: mono(rope(), 0.7).await?,
            last: RefCell::new(HashMap::new()),
        })
    }

    /// Effekt abspielen; derselbe Effekt wird kurz nacheinander nicht gestapelt
    pub fn play(&self, s: &Sound, vol: f32) {
        let key = s as *const Sound as usize;
        let now = macroquad::time::get_time();
        let min_gap = if std::ptr::eq(s, &self.cheer) { 1.6 } else { 0.06 };
        {
            let mut last = self.last.borrow_mut();
            if let Some(t) = last.get(&key) {
                if now - t < min_gap {
                    return;
                }
            }
            last.insert(key, now);
        }
        play_sound(s, PlaySoundParams { looped: false, volume: (vol * MASTER).min(0.6) });
    }

    pub fn start_ambience(&self) {
        play_sound(&self.crowd, PlaySoundParams { looped: true, volume: 0.1 });
    }

    pub fn ambience(&self, v: f32) {
        set_sound_volume(&self.crowd, (v * 0.45).min(0.3));
    }
}

/// Testhilfe: alle Sounds als WAV-Dateien schreiben
pub fn dump(dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    let w = |name: &str, l: &[f32], r: &[f32], p: f32| {
        let _ = std::fs::write(format!("{}/{}.wav", dir, name), wav(l, r, p));
    };
    let m = |name: &str, v: Vec<f32>, p: f32| {
        let (l, r) = stereo(&v, 0.0);
        w(name, &l, &r, p);
    };
    m("punch", hit(7, 110.0, 0.22, 0.35), 0.8);
    m("kick", hit(11, 75.0, 0.3, 0.25), 0.85);
    m("slam", slam(), 0.9);
    m("bell", bell(), 0.7);
    m("whoosh", whoosh(), 0.6);
    m("rope", rope(), 0.7);
    w("crowd", &make_loop(crowd_layer(3, 9.0, false), 1.0), &make_loop(crowd_layer(7, 9.0, false), 1.0), 0.7);
    w("cheer", &crowd_layer(41, 3.4, true), &crowd_layer(43, 3.4, true), 0.7);
}
