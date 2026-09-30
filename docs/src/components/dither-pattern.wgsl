// Renders one texel per dither cell into an R8 target, so the fbm runs once per cell.

struct Uniforms {
  time: f32,
  noiseAmp: f32,
  loadProgress: f32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;

var<private> time: f32;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
  let uv = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn noise(p: vec2f) -> f32 {
  return sin(p.x * 10.0) * sin(p.y * (3.0 + sin(time * 0.090909))) + 0.2;
}

fn rotate(angle: f32) -> mat2x2f {
  let c = cos(angle);
  let s = sin(angle);
  return mat2x2f(c, -s, s, c);
}

fn fbm(p_in: vec2f) -> f32 {
  var p = p_in * 1.1;
  var f = 0.0;
  var amp = 0.5 * u.noiseAmp;

  let modify = rotate(time * 0.02);
  f += amp * noise(p);
  p = modify * p * 2.0;
  amp *= 0.454545;

  f += amp * noise(p);
  p = modify * p * 2.0;
  amp *= 0.454545;

  f += amp * noise(p);

  return f;
}

fn pattern(p: vec2f) -> f32 {
  let rot01 = rotate(0.1 * time);
  let rot1 = rotate(0.1);

  let q = vec2f(fbm(p + 1.0), fbm(rot01 * p + 1.0));
  let r = vec2f(fbm(rot1 * q), fbm(q));
  return fbm(p + r);
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4f) -> @location(0) vec4f {
  time = u.time * 0.333333;
  let s = floor(fragCoord.xy) / vec2f(30.0, 15.0);
  let intensity = pattern(s * 0.1) * 1.3 - 0.03;

  let cellRandom = fract(sin(dot(s, vec2f(12.9898, 78.233))) * 43758.5453);
  let fade = smoothstep(0.0, 1.0, clamp((u.loadProgress - cellRandom * 0.8) / 0.2, 0.0, 1.0));

  return vec4f(clamp(intensity * fade, 0.0, 1.0), 0.0, 0.0, 1.0);
}
