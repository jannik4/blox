#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
}
#import bevy_pbr::mesh_view_bindings::globals;

const SIZE: f32 = 15.0;
const THRESHOLD_A: f32 = 0.015;
const THRESHOLD_B: f32 = 0.075;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Input
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // Grid
    let line = (check(in.uv.x, THRESHOLD_A) && check(in.uv.y, THRESHOLD_B))
        || (check(in.uv.y, THRESHOLD_A) && check(in.uv.x, THRESHOLD_B));
    if line {
        let effect = (sin(1.0 * globals.time + 2.0 * (in.uv.x + in.uv.y)) + 1.0) * 0.5;
        let value = 0.0 + 0.01 * effect;
        pbr_input.material.base_color = vec4(value, value, value, 0.8);
    } else {
        pbr_input.material.base_color = vec4(0.0, 0.0, 0.0, 0.2);
    }

    // Output
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    bevy_core_pipeline::oit::oit_draw(in.position, out.color);
    discard;
}

fn check(value: f32, threshold: f32) -> bool {
    let fract = fract(value * SIZE);
    return fract < threshold || fract > 1.0 - threshold;
}
