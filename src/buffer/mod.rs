/*
什么是缓冲区?
缓冲区（Buffer）一个可用于 GPU 操作的内存块。缓冲区数据是以线性布局存储的，这意味着分配的每个字节都可以通过其从缓冲区开始的偏移量来寻址，但要根据操作的不同而有对齐限制。

缓冲区常用于存储结构体或数组等简单的数据，但也可以存储更复杂的数据，如树等图结构（只要所有节点都存储在一起，且不引用缓冲区以外的任何数据）。
我们会经常用到缓冲区，所以让我们从最重要的两个开始：顶点缓冲区（Vertex Buffer）和索引缓冲区（Index Buffer）。
*/

/*
顶点缓冲区
之前我们是直接在顶点着色器中存储的顶点数据。这在学习的起始阶段很有效，但这不是长远之计，因为需要绘制的对象的类型会有不同的大小，
且每当需要更新模型时就得重新编译着色器，这会大大减慢我们的程序。我们将改为使用顶点缓冲区来存储想要绘制的顶点数据。

创建一个新的结构体来描述顶点
每个顶点都会有一个位置（position）和颜色（color）字段。
位置代表顶点在三维空间中的 x、y 和 z 坐标。
颜色是顶点的红、绿、蓝三通道色值。
我们需要令 Vertex 支持 Copy trait，这样就可以用它创建一个缓冲区。

修改 VERTICES 常量
对于 Vertex 的定义有几处需要修改。到目前为止，我们一直在使用 color 字段来设置网格颜色。
现在我们要用 tex_coords 代替 color，这些坐标会被传递给采样器以获取纹素（Texel）的颜色。

由于 tex_coords 是二维的，需要修改这个字段的类型为两个浮点数的数组。
*/
use bytemuck::Zeroable;
use wgpu::{BufferAddress, vertex_attr_array, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    // pub color: [f32; 3]
    pub tex_coords: [f32; 2]
}

/*
当结构体里包含了没有实现 Pod 和 Zeroable 的类型时，就需要手动实现这些 trait。这些 trait 不需要我们实现任何函数
*/
// unsafe impl bytemuck::Pod for Vertex {}
// unsafe impl bytemuck::Zeroable for Vertex {}
impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    //创建顶点缓冲区布局
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        /*
        我们需要告诉 render_pipeline 在绘制时使用这个缓冲区，但首先需要告诉它如何读取此缓冲区。
        顶点缓冲区布局（VertexBufferLayout）对象和 vertex_buffers 字段可以用来完成这件事，我保证在创建 render_pipeline 时会详细讨论这个问题。

        顶点缓冲区布局对象定义了缓冲区在内存中的表示方式，render_pipeline 需要它来在着色器中映射缓冲区。下面是填充了顶点的一个缓冲区的布局：

        array_stride 定义了一个顶点所占的字节数。当着色器读取下一个顶点时，它将跳过 array_stride 的字节数。在我们的例子中，array_stride 是 24 个字节。(f32四个字节，顶点加颜色一共6个f32)
        step_mode 告诉管线此缓冲区中的数组数据中的每个元素代表的是每个顶点还是每个实例的数据，如果只想在开始绘制一个新实例时改变顶点，就可以设置为 wgpu::VertexStepMode::Instance。在后面的教程里我们会讲解实例化绘制。
        attributes 描述顶点的各个属性（Attribute）的布局。一般来说，这与结构体的字段是 1:1 映射的，在我们的案例中也是如此。
        */
        /*VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                /*
                offset 定义了属性在一个顶点元素中的字节偏移量。对于第一个属性，偏移量通常为零。其后属性的偏移量应为在其之前各属性的 size_of 之和。
                shader_location 告诉着色器要在什么位置存储这个属性。例如 @location(0) x: vec3f 在顶点着色器中对应于 Vertex 结构体的 position 字段，
                        而 @location(1) x: vec3f 对应 color 字段。
                format 告诉着色器该属性的数据格式。Float32x3对应于着色器代码中的 vec3f。我们可以在一个属性中存储的最大值是Float32x4（Uint32x4 和 Sint32x4 也可以）。
                        当我们需要存储比 Float32x4 更大的东西时请记住这一点。
                */
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3
                }
            ],
        }*/

        /*
        像上边那样指定属性是非常冗长的。我们可以使用 wgpu 提供的 vertex_attr_array 宏来清理一下。现在 VertexBufferLayout 变成了这样：
        */
        /*VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x3]
        }*/

        /*
        这无疑很棒，但 Rust 认为 vertex_attr_array 的结果是一个临时值，所以需要进行调整才能从一个函数中返回。
        我们可以将wgpu::VertexBufferLayout 的生命周期改为 'static，或者使其成为 const
        */
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS
        }
    }
}

/*
它总共有 5 个顶点和 3 个三角形。现在，如果我们想只用顶点来显示这样的东西，我们就需要以下顶点数据：

rust
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E

    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E

    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
];
你会注意到有些顶点被使用了不止一次。C 和 B 顶点被使用了两次，而 E 顶点被重复使用了 3 次。
假设每个浮点数是 4 个字节，那么这意味着在我们用于 VERTICES 的 216 个字节中，有 96 个字节是重复的数据。
如果能只把这些顶点列出来一次不是很好吗？我们可以做到这一点!

这，就是索引缓冲区发挥作用的地方。

大体上来说，我们在 VERTICES 中存储所有唯一的顶点，我们创建另一个缓冲区，将索引存储在 VERTICES 中的元素以创建三角形。下面还是以五边形为例：
*/