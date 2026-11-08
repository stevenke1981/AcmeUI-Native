struct In {
  @location(0) rect: vec4<f32>,
  @location(1) uv: vec4<f32>,
  @location(2) color: vec4<f32>,
  @location(3) viewport_mode: vec4<f32>,
  @location(4) clip: vec4<f32>,
};

struct Out {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) @interpolate(flat) color_glyph: f32,
  @location(3) @interpolate(flat) clip_bounds: vec4<f32>,
};

@group(0) @binding(0) var atlas: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@vertex fn vs_main(input: In, @builtin(vertex_index) i: u32) -> Out {
  var corners = array<vec2<f32>, 6>(
    vec2(0., 0.), vec2(1., 0.), vec2(0., 1.),
    vec2(0., 1.), vec2(1., 0.), vec2(1., 1.)
  );
  let corner = corners[i];
  let pixel = input.rect.xy + corner * input.rect.zw;
  var out: Out;
  out.position = vec4(
    pixel.x / input.viewport_mode.x * 2. - 1.,
    1. - pixel.y / input.viewport_mode.y * 2.,
    0., 1.
  );
  out.uv = input.uv.xy + corner * input.uv.zw;
  out.color = input.color;
  out.color_glyph = input.viewport_mode.z;
  out.clip_bounds = input.clip;
  return out;
}

fn overlap(lo: f32, hi: f32, center: f32) -> f32 {
  return clamp(min(center + 0.5, hi) - max(center - 0.5, lo), 0.0, 1.0);
}

fn clip_coverage(clip: vec4<f32>, center: vec2<f32>) -> f32 {
  if !(clip.z - clip.x > 0.0 && clip.w - clip.y > 0.0) {
    return 0.0;
  }
  return overlap(clip.x, clip.z, center.x) * overlap(clip.y, clip.w, center.y);
}

@fragment fn fs_main(input: Out) -> @location(0) vec4<f32> {
  let sample = textureSample(atlas, atlas_sampler, input.uv);
  let mask = clip_coverage(input.clip_bounds, input.position.xy);
  if input.color_glyph > 0.5 {
    return vec4(sample.rgb * input.color.rgb, sample.a * input.color.a * mask);
  }
  return vec4(input.color.rgb, input.color.a * sample.r * mask);
}
