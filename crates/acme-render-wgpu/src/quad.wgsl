struct In {
  @location(0) rect: vec4<f32>,
  @location(1) color: vec4<f32>,
  @location(2) border_color: vec4<f32>,
  @location(3) extras: vec4<f32>,
};
struct Out {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) local: vec2<f32>,
  @location(2) size_radius: vec3<f32>,
  @location(3) border: vec4<f32>,
};
@vertex fn vs_main(input: In, @builtin(vertex_index) i: u32) -> Out {
  var corners = array<vec2<f32>, 6>(vec2(0.,0.),vec2(1.,0.),vec2(0.,1.),vec2(0.,1.),vec2(1.,0.),vec2(1.,1.));
  let p = corners[i]; let pixel = input.rect.xy + p * input.rect.zw;
  var out: Out;
  out.position = vec4(pixel.x / input.extras.z * 2. - 1., 1. - pixel.y / input.extras.w * 2., 0., 1.);
  out.color=input.color; out.local=p*input.rect.zw; out.size_radius=vec3(input.rect.zw,input.extras.x); out.border=vec4(input.border_color.rgb,input.extras.y); return out;
}
@fragment fn fs_main(input: Out) -> @location(0) vec4<f32> {
  let half_size=input.size_radius.xy*.5; let p=abs(input.local-half_size)-half_size+vec2(input.size_radius.z);
  let dist=length(max(p,vec2(0.)))+min(max(p.x,p.y),0.)-input.size_radius.z;
  let coverage=1.-smoothstep(-.75,.75,dist);
  let border_mix=select(0.,1.,input.border.a>0. && dist > -input.border.a);
  return vec4(mix(input.color.rgb,input.border.rgb,border_mix),input.color.a*coverage);
}
