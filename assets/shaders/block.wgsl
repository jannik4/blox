#import bevy_pbr::{
    mesh_functions,
    pbr_types,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
}
#import bevy_pbr::mesh_view_bindings::globals;

@group(2) @binding(100) var blocks_texture: texture_2d_array<f32>;
@group(2) @binding(101) var blocks_texture_sampler: sampler;

const SIZE: f32 = 15.0;
const THRESHOLD_A: f32 = 0.015;
const THRESHOLD_B: f32 = 0.075;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Get tag
    let tag = mesh_functions::get_tag(in.instance_index);

    // Discard faces
    if (tag & (1u << 8u)) != 0u && in.world_normal.x < 0.0
        || (tag & (1u << 9u)) != 0u && in.world_normal.x > 0.0
        || (tag & (1u << 10u)) != 0u && in.world_normal.y < 0.0
        || (tag & (1u << 11u)) != 0u && in.world_normal.y > 0.0
        || (tag & (1u << 12u)) != 0u && in.world_normal.z < 0.0
        || (tag & (1u << 13u)) != 0u && in.world_normal.z > 0.0
     {
        discard;
    }

    // Get block type and selected
    let block = tag & 0xFF; // Lower 8 bits for block type
    let selected = (tag & (1u << 14u)) != 0u; // 14th bit for selected

    // Get texture layer based on block type and normal
    var layer = 0;
    switch block {
        case 1u: { layer = 0; }
        case 2u: { layer = 1; }
        case 3u: { layer = 2; }
        case 4u: {
            if in.world_normal.y > 0.0 {
                layer = 4;
            } else if in.world_normal.y < 0.0 {
                layer = 0;
            } else {
                layer = 3;
            }
        }
        case 5u: { layer = 5; }
        case 6u: { layer = 6; }
        case 7u: { layer = 7; }
        default: {}
    }

    // Color (TODO: Use selected to add effect)
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = textureSample(blocks_texture, blocks_texture_sampler, in.uv, layer);

    // Hack for more water transparency
    if block == 7u {
        pbr_input.material.base_color.a *= 0.5;
    }

    // Output
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    let alpha_mode = pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode != pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE
        && alpha_mode != pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK {
        bevy_core_pipeline::oit::oit_draw(in.position, out.color);
        discard;
    }

    return out;
}

fn check(value: f32, threshold: f32) -> bool {
    let fract = fract(value * SIZE);
    return fract < threshold || fract > 1.0 - threshold;
}
