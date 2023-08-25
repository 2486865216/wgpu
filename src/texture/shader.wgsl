
/*
有了新的 Vertex 结构体，现在是时候更新着色器了。首先需要将 tex_coords 传递给顶点着色器，然后将它们用于片元着色器，以便从采样器获得最终的颜色。让我们从顶点着色器开始：
*/
struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f
}

//顶点着色器
struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f
};

@vertex
fn vs_main (
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4f(model.position, 1.0);
    return out;
}


//接下来是片元着色器。还是在 shader.wgsl 中添加以下代码：
//现在顶点着色器输出了 tex_coords，我们需要改变片元着色器来接收它们。有了这些坐标，就可以使用采样器从纹理中获取纹素的颜色了:

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

//变量 t_diffuse 和 s_diffuse 就是所谓的 uniforms。我们将在 相机部分 中进一步讨论 uniforms。
//现在，我们需要知道的是，@group(x) 对应于 set_bind_group() 中的第一个参数，@binding(x) 与我们创建绑定组布局和绑定组时指定的 binding 值对应。

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}