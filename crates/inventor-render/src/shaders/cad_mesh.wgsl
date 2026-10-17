struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
};

struct LightUniform {
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = in.position;
    out.world_normal = in.normal;
    out.color = in.color;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var N = normalize(in.world_normal);
    let V = normalize(camera.eye_pos.xyz - in.world_pos);
    
    // Double-sided lighting: flip normal if viewing backface
    if (dot(N, V) < 0.0) {
        N = -N;
    }

    let L1 = normalize(light.light_dir.xyz);
    let diff1 = max(dot(N, L1), 0.0);

    // Secondary fill light for subtle ambient bounce
    let L2 = normalize(vec3<f32>(-0.4, -0.6, -0.5));
    let diff2 = max(dot(N, -L2), 0.0) * 0.25;

    // Specular highlight (Blinn-Phong)
    let H = normalize(L1 + V);
    let spec = pow(max(dot(N, H), 0.0), 32.0) * 0.35;

    let ambient = light.ambient.rgb;
    let diffuse = (diff1 + diff2) * light.light_color.rgb;
    let lighting = ambient + diffuse;

    let final_rgb = in.color.rgb * lighting + vec3<f32>(spec);
    return vec4<f32>(final_rgb, in.color.a);
}
