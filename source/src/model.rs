//! Prozedurale Körpermodelle (Wrestler + Ringrichter)
use crate::anim::*;
use crate::mesh::*;
use macroquad::models::Mesh;
use macroquad::prelude::*;

#[derive(Clone)]
pub struct Look {
    pub skin: Color,
    pub trunks: Color,
    pub trim: Color,
    pub boots: Color,
    pub pads: Color,
    pub hair: Option<Color>,
    pub beard: Option<Color>,
    pub bulk: f32,
    pub belly: f32,
    pub referee: bool,
    pub hair_style: u8, // 0 kurz, 1 Glatze, 2 Pferdeschwanz
}

pub fn shade(c: Color, f: f32) -> Color {
    Color::new((c.r * f).min(1.), (c.g * f).min(1.), (c.b * f).min(1.), c.a)
}

pub fn build_body(l: &Look) -> Vec<Mesh> {
    let b = l.bulk;
    let skin = l.skin;
    let id = Mat4::IDENTITY;
    let black = hex(0x101012, 0.25);
    let white = hex(0xf2f2ee, 0.3);
    let tape = hex(0xe8e6de, 0.15);
    let rf = l.referee;
    // Schiri-Hemd: vertikale Streifen
    let shirt = move |p: Vec3| -> Color {
        let a = p.x.atan2(p.z);
        if ((a / std::f32::consts::PI * 7.0).floor() as i32).rem_euclid(2) == 0 {
            hex(0x111111, 0.12)
        } else {
            hex(0xeeeeee, 0.12)
        }
    };
    let torso_col = move |p: Vec3| if rf { shirt(p) } else { skin };
    let mut parts: Vec<MB> = (0..NJ).map(|_| MB::new()).collect();

    // ---------------- Becken / Hose
    {
        let m = &mut parts[PELVIS];
        let trunks = l.trunks;
        let trim = l.trim;
        let tc = move |p: Vec3| {
            if rf {
                hex(0x15151a, 0.12)
            } else if p.y > 0.065 {
                trim
            } else if p.y < -0.10 && p.x.abs() > 0.10 {
                trim
            } else {
                trunks
            }
        };
        m.smooth_limb(
            id,
            &[(0.105, 0.16 * b, 0.115 + l.belly * 0.5), (0.02, 0.172 * b, 0.122 + l.belly * 0.25), (-0.06, 0.178 * b, 0.125), (-0.13, 0.13 * b, 0.10)],
            3,
            16,
            &tc,
        );
        for s in [-1.0f32, 1.0] {
            m.ellipsoid(id, vec3(s * 0.07 * b, -0.065, -0.05), vec3(0.085 * b, 0.10, 0.085), 14, 10, &tc);
        }
        m.ellipsoid(id, vec3(0., -0.12, 0.015), vec3(0.09, 0.06, 0.08), 12, 8, &tc);
    }

    // ---------------- Bauch
    {
        let m = &mut parts[SPINE];
        m.smooth_limb(
            id,
            &[(0.0, 0.158 * b, 0.115 + l.belly), (0.10, 0.152 * b, 0.11 + l.belly * 1.25), (0.21, 0.168 * b, 0.115 + l.belly * 0.6)],
            3,
            16,
            &torso_col,
        );
        if l.belly < 0.03 && !rf {
            for row in 0..3 {
                for s in [-1.0f32, 1.0] {
                    m.ellipsoid(
                        id,
                        vec3(s * 0.034, 0.035 + row as f32 * 0.058, 0.088 + l.belly),
                        vec3(0.031, 0.027, 0.024) * b,
                        10,
                        6,
                        &|_| skin,
                    );
                }
            }
        }
        for s in [-1.0f32, 1.0] {
            m.ellipsoid(id, vec3(s * 0.12 * b, 0.07, 0.025), vec3(0.05 * b, 0.085, 0.07), 12, 8, &torso_col);
        }
    }

    // ---------------- Brustkorb
    {
        let m = &mut parts[CHEST];
        m.smooth_limb(
            id,
            &[(0.0, 0.168 * b, 0.115), (0.12, 0.198 * b, 0.128), (0.20, 0.204 * b, 0.13), (0.27, 0.155 * b, 0.105)],
            3,
            16,
            &torso_col,
        );
        for s in [-1.0f32, 1.0] {
            m.ellipsoid(id, vec3(s * 0.072 * b, 0.165, 0.082), vec3(0.086 * b, 0.066 * b, 0.052 * b), 14, 10, &torso_col);
            m.ellipsoid(id, vec3(s * 0.14 * b, 0.10, -0.03), vec3(0.062 * b, 0.12, 0.08), 12, 8, &torso_col);
            if !rf {
                m.ellipsoid(id, vec3(s * 0.078 * b, 0.145, 0.082 + 0.05 * b), vec3(0.009, 0.009, 0.005), 8, 6, &|_| shade(skin, 0.7));
            }
        }
        m.ellipsoid(id, vec3(0., 0.25, -0.04), vec3(0.13 * b, 0.07, 0.072), 14, 8, &torso_col);
        m.ellipsoid(id, vec3(0., 0.15, -0.07), vec3(0.15 * b, 0.10, 0.07), 14, 8, &torso_col);
        if rf {
            // Kragen
            m.ellipsoid(id, vec3(0., 0.275, 0.0), vec3(0.085, 0.03, 0.085), 14, 6, &|_| hex(0x111111, 0.1));
        }
    }

    // ---------------- Hals
    parts[NECK].smooth_limb(id, &[(-0.02, 0.068 * b, 0.072), (0.06, 0.064 * b, 0.07), (0.13, 0.06 * b, 0.066)], 2, 12, &|_| skin);

    // ---------------- Kopf
    {
        let m = &mut parts[HEAD];
        let hair = l.hair;
        let beard = l.beard;
        let style = l.hair_style;
        let skull_c = move |p: Vec3| -> Color {
            if let Some(h) = hair {
                let top = p.y > 0.168 - (p.z.min(0.0) * 0.6);
                let back = p.z < -0.025 && p.y > 0.055;
                let side = p.x.abs() > 0.07 && p.y > 0.115 && p.z < 0.035;
                let is_hair = match style {
                    1 => p.y > 0.06 && p.z < -0.02 && p.y < 0.17 && p.x.abs() > 0.03, // Haarkranz
                    _ => top || back || side,
                };
                if is_hair {
                    return h;
                }
            }
            if let Some(bc) = beard {
                if p.y < 0.075 && p.z > -0.03 {
                    return bc;
                }
            }
            skin
        };
        m.ellipsoid(id, vec3(0., 0.12, 0.0), vec3(0.086, 0.113, 0.103), 18, 14, &skull_c);
        m.ellipsoid(id, vec3(0., 0.045, 0.032), vec3(0.068, 0.056, 0.07), 14, 10, &skull_c);
        m.ellipsoid(id, vec3(0., 0.142, 0.078), vec3(0.071, 0.017, 0.03), 12, 6, &|_| skin);
        m.ell_rot(vec3(0., 0.104, 0.098), vec3(0.015, 0.031, 0.022), qx(-12.), shade(skin, 0.98));
        for s in [-1.0f32, 1.0] {
            m.ell(vec3(s * 0.045, 0.098, 0.072), vec3(0.03, 0.022, 0.03), skin);
            m.ell(vec3(s * 0.087, 0.11, -0.005), vec3(0.012, 0.03, 0.02), shade(skin, 0.95));
            m.ellipsoid(id, vec3(s * 0.032, 0.123, 0.083), vec3(0.0125, 0.0115, 0.011), 10, 8, &|_| hex(0xeeeae2, 0.9));
            m.ellipsoid(id, vec3(s * 0.032, 0.123, 0.0935), vec3(0.0062, 0.0062, 0.0025), 8, 6, &|_| hex(0x2a1a10, 0.9));
            // Augenbrauen
            let brow = hair.unwrap_or(shade(skin, 0.5));
            m.ell_rot(vec3(s * 0.033, 0.142, 0.099), vec3(0.018, 0.004, 0.005), Quat::from_rotation_z(s * 0.15), brow);
        }
        let lip = if beard.is_some() { beard.unwrap() } else { mixc(skin, hex(0xa04848, 0.4), 0.45) };
        m.ell(vec3(0., 0.062, 0.093), vec3(0.024, 0.007, 0.011), lip);
        if let Some(bc) = beard {
            m.ell(vec3(0., 0.074, 0.098), vec3(0.03, 0.008, 0.01), bc);
        }
        if let (Some(h), 2) = (hair, style) {
            m.ell_rot(vec3(0., 0.09, -0.12), vec3(0.025, 0.08, 0.025), qx(25.), h);
        }
    }

    // ---------------- Schultern, Arme, Hände
    for (s, clav, ua, fa, ha) in [(1.0f32, CLAV_L, UARM_L, FARM_L, HAND_L), (-1.0f32, CLAV_R, UARM_R, FARM_R, HAND_R)] {
        parts[clav].ellipsoid(id, vec3(s * 0.09, 0.0, -0.01), vec3(0.10 * b, 0.048, 0.062), 12, 8, &torso_col);

        let sleeve = move |p: Vec3| if rf && p.y > -0.11 { shirt(vec3(p.x, p.y, p.z)) } else { skin };
        let m = &mut parts[ua];
        m.smooth_limb(id, &[(0.02, 0.07 * b, 0.07 * b), (-0.12, 0.066 * b, 0.07 * b), (-0.28, 0.05 * b, 0.052 * b)], 3, 14, &sleeve);
        m.ellipsoid(id, vec3(s * 0.02, -0.035, 0.0), vec3(0.083 * b, 0.105, 0.09 * b), 14, 10, &sleeve);
        if !rf {
            m.ellipsoid(id, vec3(0., -0.14, 0.03), vec3(0.05 * b, 0.085, 0.047 * b), 12, 8, &|_| skin);
            m.ellipsoid(id, vec3(0., -0.12, -0.035), vec3(0.052 * b, 0.10, 0.045 * b), 12, 8, &|_| skin);
        }

        let wr = !rf;
        let fac = move |p: Vec3| if wr && p.y < -0.195 { tape } else { skin };
        let m = &mut parts[fa];
        m.smooth_limb(id, &[(0.0, 0.05 * b, 0.05 * b), (-0.07, 0.054 * b, 0.051 * b), (-0.25, 0.036, 0.031)], 3, 12, &fac);
        m.ellipsoid(id, vec3(s * 0.01, -0.075, 0.01), vec3(0.048 * b, 0.085, 0.046 * b), 12, 8, &fac);

        let m = &mut parts[ha];
        m.ell(vec3(0., -0.055, 0.0), vec3(0.042, 0.055, 0.023), skin);
        m.ell(vec3(0., -0.122, 0.006), vec3(0.04, 0.045, 0.019), skin);
        m.ell_rot(vec3(s * 0.032, -0.05, 0.022), vec3(0.014, 0.034, 0.014), Quat::from_rotation_z(s * 0.4), skin);
    }

    // ---------------- Beine
    for (s, th, sh, ft) in [(1.0f32, THIGH_L, SHIN_L, FOOT_L), (-1.0f32, THIGH_R, SHIN_R, FOOT_R)] {
        let trunks = l.trunks;
        let trim = l.trim;
        let thc = move |p: Vec3| {
            if rf {
                hex(0x15151a, 0.12)
            } else if p.y > -0.06 {
                if p.y < -0.035 { trim } else { trunks }
            } else {
                skin
            }
        };
        let m = &mut parts[th];
        m.smooth_limb(id, &[(0.04, 0.105 * b, 0.10 * b), (-0.15, 0.09 * b, 0.093 * b), (-0.43, 0.06, 0.063)], 3, 16, &thc);
        m.ellipsoid(id, vec3(s * 0.005, -0.19, 0.035), vec3(0.07 * b, 0.15, 0.056 * b), 12, 10, &thc);
        m.ellipsoid(id, vec3(0., -0.21, -0.04), vec3(0.06 * b, 0.14, 0.05 * b), 12, 8, &thc);
        if !rf {
            m.ellipsoid(id, vec3(0., -0.45, 0.022), vec3(0.066, 0.078, 0.064), 14, 10, &|_| l.pads);
        }

        let boots = l.boots;
        let lace = shade(boots, 1.6);
        let shc = move |p: Vec3| {
            if rf {
                hex(0x15151a, 0.12)
            } else if p.y < -0.17 {
                if p.z > 0.035 && ((p.y * 55.0).fract().abs() < 0.35) {
                    lace
                } else {
                    boots
                }
            } else {
                skin
            }
        };
        let m = &mut parts[sh];
        m.smooth_limb(id, &[(0.0, 0.06, 0.062), (-0.10, 0.058, 0.07), (-0.38, 0.045, 0.048)], 3, 14, &shc);
        m.ellipsoid(id, vec3(0., -0.12, -0.035), vec3(0.054 * b, 0.11, 0.05 * b), 12, 8, &shc);

        let shoe = if rf { hex(0x0c0c0c, 0.5) } else { boots };
        let sole = hex(0x181818, 0.2);
        let m = &mut parts[ft];
        m.ellipsoid(id, vec3(0., -0.035, 0.055), vec3(0.05, 0.045, 0.13), 14, 10, &move |p: Vec3| if p.y < -0.064 { sole } else { shoe });
        m.ellipsoid(id, vec3(0., 0.0, -0.005), vec3(0.047, 0.05, 0.06), 12, 8, &move |_| shoe);
    }
    let _ = (black, white);
    parts.iter().map(|p| p.build1()).collect()
}

pub fn look_president() -> Look {
    Look {
        skin: hex(0xb07a55, 0.55),
        trunks: hex(0x9c0f18, 0.35),
        trim: hex(0xd8a630, 0.7),
        boots: hex(0x9c0f18, 0.55),
        pads: hex(0x1a1a1a, 0.3),
        hair: Some(hex(0x1c1410, 0.2)),
        beard: Some(hex(0x1f1712, 0.15)),
        bulk: 1.14,
        belly: 0.025,
        referee: false,
        hair_style: 1,
    }
}

pub fn look_chancellor() -> Look {
    Look {
        skin: hex(0xe3b593, 0.5),
        trunks: hex(0x123a8c, 0.35),
        trim: hex(0xc8ccd4, 0.8),
        boots: hex(0x0d1a3a, 0.6),
        pads: hex(0xc8ccd4, 0.35),
        hair: Some(hex(0xc9a35a, 0.25)),
        beard: None,
        bulk: 1.0,
        belly: 0.0,
        referee: false,
        hair_style: 2,
    }
}

pub fn look_referee() -> Look {
    Look {
        skin: hex(0xd6a687, 0.4),
        trunks: hex(0x15151a, 0.1),
        trim: hex(0x15151a, 0.1),
        boots: hex(0x0c0c0c, 0.4),
        pads: hex(0x0c0c0c, 0.2),
        hair: Some(hex(0x5a5a5a, 0.2)),
        beard: Some(hex(0x6a6a6a, 0.1)),
        bulk: 0.86,
        belly: 0.03,
        referee: true,
        hair_style: 0,
    }
}
