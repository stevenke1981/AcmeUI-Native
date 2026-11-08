struct In {
  @location(0) rect: vec4<f32>,
  @location(1) color: vec4<f32>,
  @location(2) border_color: vec4<f32>,
  @location(3) extras: vec4<f32>,
  @location(4) clip: vec4<f32>,
};

struct Out {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) border_color: vec4<f32>,
  @location(2) @interpolate(flat) quad_bounds: vec4<f32>,
  @location(3) @interpolate(flat) clip_bounds: vec4<f32>,
  @location(4) @interpolate(flat) radius_border: vec2<f32>,
};

@vertex fn vs_main(input: In, @builtin(vertex_index) i: u32) -> Out {
  var corners = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
    vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0)
  );
  let p = corners[i];
  let quad_min = input.rect.xy;
  let quad_max = input.rect.xy + max(input.rect.zw, vec2(0.0));
  let vis_min = max(max(quad_min, input.clip.xy), vec2(0.0));
  let vis_max = min(min(quad_max, input.clip.zw), input.extras.zw);
  var raster_min = vec2(0.0);
  var raster_max = vec2(0.0);
  if vis_max.x > vis_min.x && vis_max.y > vis_min.y {
    raster_min = floor(vis_min);
    raster_max = ceil(vis_max);
  }
  let pixel = raster_min + p * (raster_max - raster_min);
  var out: Out;
  out.position = vec4(
    pixel.x / input.extras.z * 2.0 - 1.0,
    1.0 - pixel.y / input.extras.w * 2.0,
    0.0,
    1.0
  );
  out.color = input.color;
  out.border_color = input.border_color;
  out.quad_bounds = vec4(quad_min, quad_max);
  out.clip_bounds = input.clip;
  out.radius_border = input.extras.xy;
  return out;
}

fn overlap(lo: f32, hi: f32, center: f32) -> f32 {
  return clamp(min(center + 0.5, hi) - max(center - 0.5, lo), 0.0, 1.0);
}

fn rect_coverage(bounds: vec4<f32>, center: vec2<f32>) -> f32 {
  if !(bounds.z - bounds.x > 0.0 && bounds.w - bounds.y > 0.0) {
    return 0.0;
  }
  return overlap(bounds.x, bounds.z, center.x) * overlap(bounds.y, bounds.w, center.y);
}

fn disc_rectangle_area(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> f32 {
  let area = (x1 - x0) * (y1 - y0);
  let r2 = r * r;
  if x1 * x1 + y1 * y1 <= r2 {
    return area;
  }
  if x0 * x0 + y0 * y0 >= r2 {
    return 0.0;
  }
  let full_end = clamp(sqrt(max((r - y1) * (r + y1), 0.0)), x0, x1);
  let end = clamp(sqrt(max((r - y0) * (r + y0), 0.0)), full_end, x1);
  let start_y = sqrt(max((r - full_end) * (r + full_end), 0.0));
  let end_y = sqrt(max((r - end) * (r + end), 0.0));
  let width = end - full_end;
  let dy = start_y - end_y;
  let chord = sqrt(width * width + dy * dy);
  let angle = 2.0 * asin(clamp(chord / (2.0 * r), 0.0, 1.0));
  var segment: f32;
  if angle < 0.1 {
    let a2 = angle * angle;
    segment = angle * a2 * (1.0 / 6.0 - a2 * (1.0 / 120.0 - a2 * (1.0 / 5040.0)));
  } else {
    segment = angle - sin(angle);
  }
  let covered = (full_end - x0) * (y1 - y0)
    + width * ((start_y - y0) + (end_y - y0)) * 0.5
    + 0.5 * r2 * segment;
  return clamp(covered, 0.0, area);
}

fn corner_removed_area(
  arc_x: f32,
  arc_y: f32,
  sx: f32,
  sy: f32,
  kcx: f32,
  kcy: f32,
  hx: f32,
  hy: f32,
  r: f32,
) -> f32 {
  let x0 = max((kcx - arc_x) * sx - hx, 0.0);
  let x1 = min((kcx - arc_x) * sx + hx, r);
  let y0 = max((kcy - arc_y) * sy - hy, 0.0);
  let y1 = min((kcy - arc_y) * sy + hy, r);
  if x0 >= x1 || y0 >= y1 {
    return 0.0;
  }
  return (x1 - x0) * (y1 - y0) - disc_rectangle_area(x0, y0, x1, y1, r);
}

fn clipped_quad_coverage(
  bounds: vec4<f32>,
  radius: f32,
  clip: vec4<f32>,
  center: vec2<f32>,
) -> f32 {
  let half_x = (bounds.z - bounds.x) * 0.5;
  let half_y = (bounds.w - bounds.y) * 0.5;
  if !(half_x > 0.0 && half_y > 0.0) {
    return 0.0;
  }
  if !(clip.z - clip.x > 0.0 && clip.w - clip.y > 0.0) {
    return 0.0;
  }
  let clipped_bounds = vec4(max(bounds.xy, clip.xy), min(bounds.zw, clip.zw));
  let base = rect_coverage(clipped_bounds, center);
  if base <= 0.0 {
    return 0.0;
  }
  let r = min(max(radius, 0.0), min(half_x, half_y));
  if r <= 0.0 {
    return base;
  }
  let kx0 = max(center.x - 0.5, clip.x);
  let kx1 = min(center.x + 0.5, clip.z);
  let ky0 = max(center.y - 0.5, clip.y);
  let ky1 = min(center.y + 0.5, clip.w);
  let kcx = 0.5 * (kx0 + kx1);
  let kcy = 0.5 * (ky0 + ky1);
  let hx = 0.5 * (kx1 - kx0);
  let hy = 0.5 * (ky1 - ky0);
  let removed =
    corner_removed_area(bounds.x + r, bounds.y + r, -1.0, -1.0, kcx, kcy, hx, hy, r)
    + corner_removed_area(bounds.z - r, bounds.y + r, 1.0, -1.0, kcx, kcy, hx, hy, r)
    + corner_removed_area(bounds.x + r, bounds.w - r, -1.0, 1.0, kcx, kcy, hx, hy, r)
    + corner_removed_area(bounds.z - r, bounds.w - r, 1.0, 1.0, kcx, kcy, hx, hy, r);
  return clamp(base - removed, 0.0, base);
}

@fragment fn fs_main(input: Out) -> @location(0) vec4<f32> {
  let center = input.position.xy;
  let radius = input.radius_border.x;
  let border = input.radius_border.y;
  let coverage = clipped_quad_coverage(input.quad_bounds, radius, input.clip_bounds, center);
  if coverage <= 0.0 {
    return vec4(0.0);
  }
  if border > 0.0 {
    let inner_bounds = vec4(
      input.quad_bounds.xy + vec2(border),
      input.quad_bounds.zw - vec2(border)
    );
    let inner_radius = max(radius - border, 0.0);
    let inside = clipped_quad_coverage(inner_bounds, inner_radius, input.clip_bounds, center);
    let stroke = max(coverage - inside, 0.0);
    let fill_a = input.color.a * inside;
    let stroke_a = input.border_color.a * stroke;
    let out_a = stroke_a + fill_a * (1.0 - stroke_a);
    if out_a <= 0.0 {
      return vec4(0.0);
    }
    let out_rgb = (
      input.border_color.rgb * stroke_a
      + input.color.rgb * fill_a * (1.0 - stroke_a)
    ) / out_a;
    return vec4(out_rgb, out_a);
  }
  return vec4(input.color.rgb, input.color.a * coverage);
}
