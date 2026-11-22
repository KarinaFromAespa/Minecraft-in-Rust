// Improved transparency's cloud stages (`clouds.fsh` under OIT_*), appended to
// entity.wgsl after oit.wgsl. The clouds' vertex shader and draw are the
// classic ones; only the fragment colour goes through the three stages.

// The cloud's colour; it blends towards the fog colour everywhere but in
// the depth bounds stage, which sees the unfogged colour.
fn oit_cloud_color(in: Out, front: bool, fade: bool) -> vec4<f32> {
    let color = textureSample(atlas, atlas_sampler, in.uv) * select(in.back_color, in.color, front);
    let f = max(linear_fog(in.fog_distance.x, globals.fog.x, globals.fog.y),
                linear_fog(in.fog_distance.y, globals.fog.z, globals.fog.w)) * select(0.0, 1.0, fade);
    return vec4(mix(color.rgb, globals.fog_color.rgb, f * globals.fog_color.a), color.a);
}
@fragment fn fs_oit_cloud_bounds(in: Out, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let color = oit_cloud_color(in, front, false);
    if color.a < 0.01 { discard; }
    let linear = oit_linear_depth(in.clip.z);
    let opaque = select(0.0, in.clip.z, color.a > OIT_FULLY_OPAQUE_ALPHA);
    return vec4(-linear, linear, in.clip.z, opaque);
}
@fragment fn fs_oit_cloud_transmittance(in: Out, @builtin(front_facing) front: bool) -> OitCoefficients {
    let color = oit_cloud_color(in, front, true);
    return oit_add_transmittance(1.0 - color.a, in.clip.z, in.clip.xy);
}
@fragment fn fs_oit_cloud_accumulate(in: Out, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    var color = oit_cloud_color(in, front, true);
    if HDR_WRITE_LIGHT && globals.hdr_light.w != 0.0 {
        color = vec4(hdr_written(color.rgb, hdr_scene_alpha(globals.hdr_light.y),
            globals.hdr_settings, globals.hdr_shares), color.a);
    }
    let seen = oit_sample_transmittance(vec2<i32>(in.clip.xy),
        oit_normalize_depth(in.clip.z, in.clip.xy), 1.0 - color.a);
    return vec4(color.rgb * color.a, color.a) * seen;
}
