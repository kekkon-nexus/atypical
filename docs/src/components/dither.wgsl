// The loader disables naga's coordinate-space adjustment, so fragCoord follows GL: origin bottom-left, y up.

struct Uniforms {
  color: vec4f,
  resolution: vec2f,
  cellSize: f32,
  mode: u32,
  colorSteps: f32,
  inverted: u32,
  subdiv: f32,
  gain: f32,
  threshold: f32,
  shading: f32,
  shapeScale: f32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var grid: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
  let uv = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn hash21(p_in: vec2f) -> f32 {
  var p = fract(p_in * vec2f(0.3183099, 0.3678794)) + 0.1;
  p += dot(p, p + 19.19);
  return fract(p.x * p.y);
}

// Bit-interleaved Bayer index, levels 1/2/3 give the 2x2/4x4/8x8 matrices.
fn bayer(cell: vec2i, levels: u32) -> f32 {
  let mask = (1i << levels) - 1i;
  var x = u32(cell.x & mask);
  var y = u32(cell.y & mask);
  var v = 0u;
  for (var i = 0u; i < levels; i++) {
    v = v * 4u + (((x ^ y) & 1u) << 1u) + (y & 1u);
    x >>= 1u;
    y >>= 1u;
  }
  return f32(v) / f32(1u << (2u * levels));
}

fn threshold(cell: vec2f, lum: f32) -> f32 {
  switch u.mode {
    case 1u: {
      return step(hash21((cell + 0.5) * u.cellSize), lum);
    }
    case 2u: {
      return bayer(vec2i(cell), 1u);
    }
    case 3u: {
      return bayer(vec2i(cell), 2u);
    }
    default: {
      return bayer(vec2i(cell), 3u);
    }
  }
}

fn quantize(cell: vec2f, lum: f32) -> f32 {
  let steps = max(floor(u.colorSteps), 1.0);
  let brightness = clamp(lum + (threshold(cell, lum) - 0.5) / steps, 0.0, 1.0);
  return floor(brightness * steps + 0.5) / steps;
}

fn halftone(local: vec2f, intensity: f32) -> f32 {
  let p = local * u.shapeScale;
  if (p.x > 1.0 || p.y > 1.0) {
    return 0.0;
  }

  let q = vec2f(p.x, 1.0 - p.y) * u.subdiv;
  let mid = floor(u.subdiv / 2.0);
  let k = floor(q) - mid;
  let lit = step(u.threshold, intensity - dot(k, k) / (4.0 * max(mid * mid, 1.0)));
  let bevel = (0.2 + fract(q.y) * 0.8) * (0.75 + fract(q.x) * 0.25);
  return lit * mix(1.0, bevel, u.shading);
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4f) -> @location(0) vec4f {
  let uv = (fragCoord.xy - 0.5 * u.resolution) / u.cellSize;
  let cell = floor(uv);
  let texel = vec2i(cell) + vec2i(textureDimensions(grid)) / 2;
  var lum = textureLoad(grid, texel, 0).r;
  if (u.inverted != 0u) {
    lum = 1.0 - lum;
  }

  var coverage: f32;
  if (u.mode == 5u) {
    coverage = halftone(fract(uv), lum * u.gain);
  } else {
    coverage = quantize(cell, lum);
  }

  return vec4f(u.color.rgb * u.color.a, u.color.a) * coverage;
}
