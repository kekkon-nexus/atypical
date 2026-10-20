// Renders one texel per dither cell into an R8 target: the luminance of an image fitted to cover the grid, as in
// paper-design's ImageDithering. fragCoord follows GL: origin bottom-left, y up.

struct Uniforms {
  gridSize: vec2f,
  imageAspect: f32,
  loadProgress: f32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var picture: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
  let uv = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4f) -> @location(0) vec4f {
  let gridAspect = u.gridSize.x / u.gridSize.y;
  var uv = fragCoord.xy / u.gridSize - 0.5;
  if (gridAspect > u.imageAspect) {
    uv.y *= u.imageAspect / gridAspect;
  } else {
    uv.x *= gridAspect / u.imageAspect;
  }
  uv += 0.5;

  // Image rows run top to bottom, the grid's bottom to top.
  let size = vec2f(textureDimensions(picture));
  let texel = vec2i(clamp(vec2f(uv.x, 1.0 - uv.y) * size, vec2f(0.0), size - 1.0));
  let pixel = textureLoad(picture, texel, 0);
  let lum = dot(vec3f(0.2126, 0.7152, 0.0722), pixel.rgb) * pixel.a;

  let cellRandom = fract(sin(dot(floor(fragCoord.xy), vec2f(12.9898, 78.233))) * 43758.5453);
  let fade = smoothstep(0.0, 1.0, clamp((u.loadProgress - cellRandom * 0.8) / 0.2, 0.0, 1.0));

  return vec4f(lum * fade, 0.0, 0.0, 1.0);
}
