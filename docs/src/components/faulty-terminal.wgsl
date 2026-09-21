struct Uniforms {
  iResolution: vec3f,
  iTime: f32,
  uGridMul: vec2f,
  uMouse: vec2f,
  uFg: vec3f,
  uScale: f32,
  uDigitSize: f32,
  uScanlineIntensity: f32,
  uGlitchAmount: f32,
  uFlickerAmount: f32,
  uNoiseAmp: f32,
  uChromaticAberration: f32,
  uDither: f32,
  uCurvature: f32,
  uMouseStrength: f32,
  uUseMouse: f32,
  uPageLoadProgress: f32,
  uUsePageLoadAnimation: f32,
  uBrightness: f32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;

var<private> time: f32;

struct VertexOutput {
  @builtin(position) position: vec4f,
  @location(0) vUv: vec2f,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
  let uv = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
  var out: VertexOutput;
  out.vUv = uv;
  out.position = vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
  return out;
}

fn glsl_mod(x: f32, y: f32) -> f32 {
  return x - y * floor(x / y);
}

fn hash21(p_in: vec2f) -> f32 {
  var p = fract(p_in * 234.56);
  p += dot(p, p + 34.56);
  return fract(p.x * p.y);
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
  var amp = 0.5 * u.uNoiseAmp;

  let modify0 = rotate(time * 0.02);
  f += amp * noise(p);
  p = modify0 * p * 2.0;
  amp *= 0.454545;

  let modify1 = rotate(time * 0.02);
  f += amp * noise(p);
  p = modify1 * p * 2.0;
  amp *= 0.454545;

  f += amp * noise(p);

  return f;
}

fn pattern(p: vec2f) -> f32 {
  let offset1 = vec2f(1.0);
  let offset0 = vec2f(0.0);
  let rot01 = rotate(0.1 * time);
  let rot1 = rotate(0.1);

  let q = vec2f(fbm(p + offset1), fbm(rot01 * p + offset1));
  let r = vec2f(fbm(rot1 * q + offset0), fbm(q + offset0));
  return fbm(p + r);
}

fn digit(p_in: vec2f) -> f32 {
  let grid = u.uGridMul * 15.0;
  let s = floor(p_in * grid) / grid;
  var p = p_in * grid;
  var intensity = pattern(s * 0.1) * 1.3 - 0.03;

  if (u.uUseMouse > 0.5) {
    let mouseWorld = u.uMouse * u.uScale;
    let distToMouse = distance(s, mouseWorld);
    let mouseInfluence = exp(-distToMouse * 8.0) * u.uMouseStrength * 10.0;
    intensity += mouseInfluence;

    let ripple = sin(distToMouse * 20.0 - u.iTime * 5.0) * 0.1 * mouseInfluence;
    intensity += ripple;
  }

  if (u.uUsePageLoadAnimation > 0.5) {
    let cellRandom = fract(sin(dot(s, vec2f(12.9898, 78.233))) * 43758.5453);
    let cellDelay = cellRandom * 0.8;
    let cellProgress = clamp((u.uPageLoadProgress - cellDelay) / 0.2, 0.0, 1.0);

    let fadeAlpha = smoothstep(0.0, 1.0, cellProgress);
    intensity *= fadeAlpha;
  }

  p = fract(p);
  p *= u.uDigitSize;

  let px5 = p.x * 5.0;
  let py5 = (1.0 - p.y) * 5.0;
  let x = fract(px5);
  let y = fract(py5);

  let i = floor(py5) - 2.0;
  let j = floor(px5) - 2.0;
  let n = i * i + j * j;
  let f = n * 0.0625;

  let isOn = step(0.1, intensity - f);
  let brightness = isOn * (0.2 + y * 0.8) * (0.75 + x * 0.25);

  return step(0.0, p.x) * step(p.x, 1.0) * step(0.0, p.y) * step(p.y, 1.0) * brightness;
}

fn onOff(a: f32, b: f32, c: f32) -> f32 {
  return step(c, sin(u.iTime + a * cos(u.iTime * b))) * u.uFlickerAmount;
}

fn sq(x: f32) -> f32 {
  return clamp(sin(x) * 10.0, -1.0, 1.0);
}

fn displace(look: vec2f) -> f32 {
  let y = look.y - glsl_mod(u.iTime * 0.25, 1.0);
  let window = 1.0 / (1.0 + 50.0 * y * y);
  return sq(look.y * 20.0 + u.iTime) * 0.0125 * onOff(4.0, 2.0, 0.8)
    * (1.0 + cos(u.iTime * 60.0)) * window;
}

fn getColor(p_in: vec2f) -> vec3f {
  var p = p_in;

  var bar = step(glsl_mod(p.y + time * 20.0, 1.0), 0.2) * 0.4 + 1.0;
  bar *= u.uScanlineIntensity;

  let displacement = displace(p);
  p.x += displacement;

  if (u.uGlitchAmount != 1.0) {
    let extra = displacement * (u.uGlitchAmount - 1.0);
    p.x += extra;
  }

  let middle = digit(p);

  const off = 0.002;
  let sum = digit(p + vec2f(-off, -off)) + digit(p + vec2f(0.0, -off)) + digit(p + vec2f(off, -off))
    + digit(p + vec2f(-off, 0.0)) + digit(p + vec2f(0.0, 0.0)) + digit(p + vec2f(off, 0.0))
    + digit(p + vec2f(-off, off)) + digit(p + vec2f(0.0, off)) + digit(p + vec2f(off, off));

  return vec3f(0.9) * middle + sum * 0.1 * vec3f(1.0) * bar;
}

fn barrel(uv: vec2f) -> vec2f {
  var c = uv * 2.0 - 1.0;
  let r2 = dot(c, c);
  c *= 1.0 + u.uCurvature * r2;
  return c * 0.5 + 0.5;
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4f, @location(0) vUv: vec2f) -> @location(0) vec4f {
  time = u.iTime * 0.333333;
  var uv = vUv;

  if (u.uCurvature != 0.0) {
    uv = barrel(uv);
  }

  let p = uv * u.uScale;
  var col = getColor(p);

  if (u.uChromaticAberration != 0.0) {
    let ca = vec2f(u.uChromaticAberration) / u.iResolution.xy;
    col.r = getColor(p + ca).r;
    col.b = getColor(p - ca).b;
  }

  col *= u.uBrightness;

  if (u.uDither > 0.0) {
    let rnd = hash21(fragCoord.xy);
    col += (rnd - 0.5) * (u.uDither * 0.003922);
  }

  let a = clamp(max(max(col.r, col.g), col.b), 0.0, 1.0);
  return vec4f(u.uFg * a, a);
}
