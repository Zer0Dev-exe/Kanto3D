// Sombreado cel: la luz de Bevy (con sombras incluidas) se reduce a 3 bandas planas,
// más un brillo de borde (rim light) en la cara iluminada.
import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
};

@if(PREPASS_PIPELINE)
import bevy_pbr::{
    prepass::io::{VertexOutput, FragmentOutput},
    deferred::functions::deferred_output,
};
@else
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};

struct Toon {
    // x: intensidad del rim light, y: brillo general
    params: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> toon: Toon;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

    var out: FragmentOutput;
@if(PREPASS_PIPELINE) {
    out = deferred_output(in, pbr_input);
} @else {
    let base = pbr_input.material.base_color;
    let emissive = pbr_input.material.emissive.rgb;

    // Luz "real" sin emisivo, para saber cuánto ilumina el sol este punto
    var lighting_input = pbr_input;
    lighting_input.material.emissive = vec4<f32>(0.0);
    let lit = apply_pbr_lighting(lighting_input);

    let weights = vec3<f32>(0.2126, 0.7152, 0.0722);
    let ratio = dot(lit.rgb, weights) / max(dot(base.rgb, weights), 0.001);

    // Con la exposición por defecto: a pleno sol ratio ≈ 3, en sombra ≈ 0.5
    var band = 0.62;
    if ratio > 0.95 { band = 0.82; }
    if ratio > 2.0 { band = 1.0; }

    let facing = 1.0 - max(dot(pbr_input.N, pbr_input.V), 0.0);
    let rim = step(0.72, facing) * toon.params.x * step(0.9, band);

    let color = base.rgb * (band + rim) * toon.params.y + emissive;
    out.color = main_pass_post_lighting_processing(pbr_input, vec4<f32>(color, base.a));
}
    return out;
}
