
//顶点着色器
//首先，声明一个 struct 来存储顶点着色器的输出。目前只有一个字段，即 clip_position。
//@builtin(position) 属性标记了此字段将作为顶点在裁剪坐标系中的位置来使用。这类似于 GLSL 的 gl_Position 变量。
//
//形如 vec4 的向量类型是泛型。目前你必须指定向量将包含的值的类型。因此一个使用 32 位浮点数的 3 维向量写做 vec3f。
struct VertexOutput {
    @builtin(position) clip_position: vec4f
};

/*
着色器代码的下一部分是 vs_main 函数。@vertex 属性标记了这个函数是顶点着色器的有效入口。
我们预期有一个 u32 类型的变量 in_vertex_index，它的值来自 @builtin(vertex_index)。

然后使用 VertexOutput 结构体声明一个名为 out 的变量。我们为顶点的裁剪空间坐标创建另外两个 x y 变量。

f32() 和 i32() 表示类型强制转换，将括号里的值转换为此类型。

现在我们可以把 clip_position 保存到 out。然后只需返回 out 就完成了顶点着色器的工作!
*/
@vertex
fn vs_main (
    @builtin(vertex_index) in_vertex_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4f(x, y, 0.0, 1.0);
    return out;
}

/*
//我们也可以不使用 stuct，直接按以下代码来实现：
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32
) -> @builtin(position) vec4f {
    // 顶点着色器 code...
}
*/

//接下来是片元着色器。还是在 shader.wgsl 中添加以下代码：
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(0.3, 0.2, 0.1, 1.0);
}
/*
这将当前片元的颜色设置为棕色。

注意，顶点和片元着色器的入口点分别被命名为 vs_main 和 fs_main。在 wgpu 的早期版本中，这两个函数有相同的名字是可以的，
但较新版本的 WGSL spec 要求这些名字必须不同。因此在整个教程中都使用（从 wgpu demo 中采用）上述命名方案。

@location(0) 属性标记了该函数返回的 vec4 值将存储在第一个颜色附件（Color Attachment）中。
*/