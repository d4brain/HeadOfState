//! Shader, Materialien und Zeichen-Helfer.
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Comparison, CullFace, Equation, PipelineParams, UniformType};
use macroquad::models::Mesh;
use macroquad::prelude::*;

const LIT_VS: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;
uniform mat4 Model;
uniform mat4 Projection;
uniform float Time;
uniform float Excite;
varying vec4 vcol;
varying vec3 vn;
varying vec3 vp;
varying vec2 vuv;
varying float vemit;
void main() {
    vec3 pos = position;
    vemit = 0.0;
    if (normal.w > 0.5) {
        float ph = normal.w;
        float b = max(0.0, sin(Time * (5.0 + mod(ph * 13.0, 3.0)) + ph * 7.0));
        pos.y += b * (0.015 + 0.13 * Excite);
    } else if (normal.w < -0.5) {
        vemit = 1.0;
    }
    vec4 wp = Model * vec4(pos, 1.0);
    vp = wp.xyz;
    vn = (Model * vec4(normal.xyz, 0.0)).xyz;
    vcol = color0 / 255.0;
    vuv = texcoord;
    gl_Position = Projection * wp;
}
"#;

const LIT_FS: &str = r#"#version 100
precision highp float;
varying vec4 vcol;
varying vec3 vn;
varying vec3 vp;
varying vec2 vuv;
varying float vemit;
uniform sampler2D Texture;
uniform vec3 CamPos;
uniform vec4 Flash;
uniform float Excite;
uniform float Time;

void spot(vec3 lp, vec3 dir, float cosO, vec3 lc, vec3 p, vec3 n, vec3 V, float shin, float ks,
          inout vec3 dif, inout vec3 spc) {
    vec3 L = lp - p;
    float d2 = dot(L, L);
    L *= inversesqrt(d2);
    float cone = smoothstep(cosO, cosO + 0.10, dot(-L, dir));
    float att = cone / (1.0 + 0.010 * d2);
    float nl = dot(n, L);
    float d = max(nl * 0.75 + 0.25, 0.0);
    dif += lc * att * d;
    vec3 H = normalize(L + V);
    spc += lc * att * pow(max(dot(n, H), 0.0), shin) * ks * step(0.0, nl) * (shin + 8.0) / 40.0;
}

#ifdef SHADOW
uniform sampler2D MatTex;
#endif
void main() {
#ifdef SHADOW
    vec2 muv = (vp.xz + vec2(3.65)) / 7.3;
    muv.y = 1.0 - muv.y;
    vec4 tex = texture2D(MatTex, muv);
    vec3 base = tex.rgb;
    float ks = 0.18;
#else
    vec4 tex = texture2D(Texture, vuv);
    vec3 base = vcol.rgb * tex.rgb;
    float ks = vcol.a;
    if (vemit > 0.5) {
        gl_FragColor = vec4(base, tex.a);
        return;
    }
#endif
    vec3 alb = pow(base, vec3(2.2));
#ifdef SHADOW
    vec3 n = vec3(0.0, 1.0, 0.0);
#else
    vec3 n = normalize(vn);
#endif
    vec3 V = normalize(CamPos - vp);
    if (dot(n, V) < 0.0) n = -n;
    float shin = 6.0 + ks * ks * 90.0;
    vec3 dif = vec3(0.0);
    vec3 spc = vec3(0.0);
    vec3 tgt = vec3(0.0, 1.0, 0.0);
    vec3 lc = vec3(1.0, 0.94, 0.86) * 1.65;
    vec3 l1 = vec3( 5.0, 9.5,  5.0);
    vec3 l2 = vec3(-5.0, 9.5,  5.0);
    vec3 l3 = vec3( 5.0, 9.5, -5.0);
    vec3 l4 = vec3(-5.0, 9.5, -5.0);
    spot(l1, normalize(tgt - l1), 0.86, lc, vp, n, V, shin, ks, dif, spc);
    spot(l2, normalize(tgt - l2), 0.86, lc, vp, n, V, shin, ks, dif, spc);
    spot(l3, normalize(tgt - l3), 0.86, lc, vp, n, V, shin, ks, dif, spc);
    spot(l4, normalize(tgt - l4), 0.86, lc * vec3(0.85, 0.9, 1.1), vp, n, V, shin, ks, dif, spc);
    spot(vec3(0.0, 11.0, 0.0), vec3(0.0, -1.0, 0.0), 0.80, vec3(1.0, 0.97, 0.92) * 1.4, vp, n, V, shin, ks, dif, spc);
    spot(vec3(0.0, 9.0, -20.0), normalize(vec3(0.0, 3.0, -26.0) - vec3(0.0, 9.0, -20.0)), 0.7, vec3(0.5, 0.35, 1.0) * 1.4, vp, n, V, shin, ks, dif, spc);
    float hemi = n.y * 0.5 + 0.5;
    vec3 amb = mix(vec3(0.020, 0.018, 0.024), vec3(0.075, 0.075, 0.10), hemi) * (0.85 + 0.5 * Excite);
    float far = clamp(length(vp.xz) / 25.0, 0.0, 1.0);
    amb += vec3(0.05, 0.045, 0.06) * (1.0 - far) * hemi;
#ifdef SHADOW
    dif *= 0.22;
    spc *= 0.05;
    amb *= 0.8;
#endif
    vec3 c = (dif + amb) * alb + spc;
    float rim = pow(1.0 - max(dot(n, V), 0.0), 3.0);
    c += rim * vec3(0.30, 0.42, 0.75) * 0.18 * (0.4 + ks) * (1.0 - far);
#ifndef SHADOW
    c += Flash.rgb * Flash.a;
#endif
    float fd = length(vp - CamPos);
    c = mix(c, vec3(0.008, 0.008, 0.014), clamp((fd - 16.0) / 50.0, 0.0, 0.75));
    c = (c * (2.51 * c + 0.03)) / (c * (2.43 * c + 0.59) + 0.14);
    c = pow(clamp(c, 0.0, 1.0), vec3(1.0 / 2.2));
    gl_FragColor = vec4(c, tex.a);
}
"#;

pub struct Gfx {
    pub lit: Material,
    pub lit_blend: Material,
    pub shadow: Material,
}

fn alpha_blend() -> Option<BlendState> {
    Some(BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    ))
}

pub fn load_gfx() -> Gfx {
    let uniforms = || {
        vec![
            UniformDesc::new("CamPos", UniformType::Float3),
            UniformDesc::new("Flash", UniformType::Float4),
            UniformDesc::new("Excite", UniformType::Float1),
            UniformDesc::new("Time", UniformType::Float1),
        ]
    };
    let lit = load_material(
        ShaderSource::Glsl { vertex: LIT_VS, fragment: LIT_FS },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Nothing,
                ..Default::default()
            },
            uniforms: uniforms(),
            textures: vec![],
        },
    )
    .expect("Shader lit");
    let lit_blend = load_material(
        ShaderSource::Glsl { vertex: LIT_VS, fragment: LIT_FS },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: false,
                cull_face: CullFace::Nothing,
                color_blend: alpha_blend(),
                ..Default::default()
            },
            uniforms: uniforms(),
            textures: vec![],
        },
    )
    .expect("Shader lit_blend");
    let sh_fs: &'static str = Box::leak(LIT_FS.replacen("precision highp float;", "precision highp float;\n#define SHADOW 1", 1).into_boxed_str());
    let shadow = load_material(
        ShaderSource::Glsl { vertex: LIT_VS, fragment: sh_fs },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Nothing,
                ..Default::default()
            },
            uniforms: uniforms(),
            textures: vec!["MatTex".to_string()],
        },
    )
    .expect("Shader shadow");
    Gfx { lit, lit_blend, shadow }
}

impl Gfx {
    pub fn frame_uniforms(&self, cam: Vec3, time: f32, excite: f32) {
        for m in [&self.lit, &self.lit_blend, &self.shadow] {
            m.set_uniform("CamPos", cam);
            m.set_uniform("Time", time);
            m.set_uniform("Excite", excite);
            m.set_uniform("Flash", vec4(0., 0., 0., 0.));
        }
    }
}

pub fn draw_at(mesh: &Mesh, model: Mat4) {
    unsafe {
        get_internal_gl().quad_gl.push_model_matrix(model);
    }
    draw_mesh(mesh);
    unsafe {
        get_internal_gl().quad_gl.pop_model_matrix();
    }
}

/// Projiziert Geometrie entlang `d` flach auf die Ebene y = h
pub fn shadow_matrix(d: Vec3, h: f32) -> Mat4 {
    let ax = d.x / d.y;
    let az = d.z / d.y;
    Mat4::from_cols(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(-ax, 0.0, -az, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(ax * h, h, az * h, 1.0),
    )
}
