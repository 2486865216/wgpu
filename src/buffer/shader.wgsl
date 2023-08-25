

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec3f
}

//顶点着色器
struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f
};

@vertex
fn vs_main (
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4f(model.position, 1.0);
    return out;
}


//接下来是片元着色器。还是在 shader.wgsl 中添加以下代码：
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}