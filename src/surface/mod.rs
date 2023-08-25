//将所有字段封装在一个结构体内，并在其上添加一些函数

use std::default::Default;
use std::fmt::format;
use rand::Rng;
use web_sys::console::count;
use winit::{window::Window, dpi::PhysicalSize};
use winit::event::WindowEvent;

use wgpu::{Surface, SurfaceConfiguration, SurfaceError, Device, DeviceDescriptor, Features, Limits, Queue, Instance, InstanceDescriptor, Backends, RequestAdapterOptions, TextureUsages, TextureViewDescriptor, PresentMode, CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, Color, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, include_wgsl, PipelineLayout, PipelineLayoutDescriptor, VertexState, FragmentState, ColorTargetState, BlendState, ColorWrites, PrimitiveState, PrimitiveTopology, FrontFace, Face, PolygonMode, MultisampleState, Buffer, BufferUsages, VertexFormat, vertex_attr_array, BindGroup};
use wgpu::util::DeviceExt;
use wgpu::VertexBufferLayout;
use wgpu::BufferAddress;
use wgpu::VertexStepMode;
use wgpu::VertexAttribute;

pub struct State {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    //使用着色器
    render_pipeline: RenderPipeline,

    //现在有了顶点数据，需要将其存储在一个缓冲区中
    vertex_buffer: Buffer,

    num_vertices: u32,

    //索缓冲区
    index_buffer: Buffer,
    num_indices: u32,

    //绑定组
    diffuse_bind_group: BindGroup,
}

//三角形实际顶点数据
use crate::buffer::Vertex;

/*
按逆时针顺序排列顶点：上、左下、右下。这样做的部分理由是出于惯例，
但主要是因为我们在 render_pipeline 的 primitive 中指定了三角形的 front_face 是 Ccw（counter-clockwise），
这样就可以做背面剔除。这意味着任何面向我们的三角形的顶点都应该是按逆时针顺序排列。
*/
/*const VERTEXS: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];*/

//索引缓冲区
/*
现在这种设置下，VERTICES 占用了 120 个字节，而 INDICES 只有 18 个字节，
因为 u16 类型是 2 个字节长。在这种情况下，wgpu 会自动增加 2 个字节的填充，以确保缓冲区被对齐到 4 个字节，但它仍然只有 20 个字节。
五边形总共是 140 字节，这意味着我们节省了 76 个字节! 这可能看起来不多，但当处理数十万的三角形时，索引可以节省大量的内存。

为了使用索引，有几处我们需要修改。首先需要创建一个缓冲区来存储索引。在 State 的 new() 函数中，创建了 vertex_buffer 之后创建 index_buffer。
同时将 num_vertices 改为num_indices，令其值等于 INDICES.len()。
*/
/*const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
];*/

// Changed
const VERTICES: &[Vertex] = &[
    // 修改后的
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4
];

impl State {
    //创建某些wgpu类型需要使用异步
    pub async fn new(window: &Window) -> Self {
        //获取窗口大小
        let size = window.inner_size();

        //instance变量是GPU实例
        //GPU 实例（Instance）是使用 wgpu 时所需创建的第一个对象，其主要用途是创建适配器（Adapter）和展示平面（Surface）。
        //Backends::all 对应Vulkan, Metal, DX12, WebGL等所有后端图形驱动
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        //展示平面（Surface）是我们绘制到窗口的部分，需要它来将绘制结果展示（或者说，呈现）到屏幕上。
        // 窗口程序需要实现 raw-surface-handle 包的 HasRawWindowHandle trait 来创建展示平面。
        // 所幸 winit 的 Window 符合这个要求。我们还需要展示平面来请求适配器。
        let surface = unsafe {
            instance.create_surface(window).unwrap()
        };

        //适配器（Adapter）是指向 WebGPU API 实现的实例，一个系统上往往存在多个 WebGPU API 实现实例。
        // 也就是说，适配器是固定在特定图形后端的。假如你使用的是 Windows 且有 2 个显卡（集成显卡 + 独立显卡），
        // 则至少有 4 个适配器可供使用，分别有 2 个固定在 Vulkan 和 DirectX 后端。
        // 我们可以用它获取关联显卡的信息，例如显卡名称与其所适配到的后端图形驱动等。稍后我们会用它来创建逻辑设备和命令队列。
        // 现在先讨论一下 RequestAdapterOptions 所涉及的字段。

        //power_preference 枚举有两个可选项：LowPower 和 HighPerformance。
        //      LowPower 对应偏向于高电池续航的适配器（如集成显卡上的 WebGPU 实现实例），
        //      HighPerformance 对应高功耗高性能的适配器（如独立显卡上的 WebGPU 实现实例）。
        //      一旦不存在符合 HighPerformance 选项的适配器，wgpu 就会选择 LowPower。
        // compatible_surface 字段告诉 wgpu 找到与所传入的展示平面兼容的适配器。
        // force_fallback_adapter 强制 wgpu 选择一个能在所有系统上工作的适配器，
        //      这通常意味着渲染后端将使用一个软渲染系统，而非 GPU 这样的硬件。
        //      需要注意的是：WebGPU 标准并没有要求所有系统上都必须实现 fallback adapter 。
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.unwrap();

        //此处传递给 request_adapter 的参数不能保证对所有设备都有效，但是应该对大多数设备都有效。
        // 当 wgpu 找不到符合要求的适配器，request_adapter 将返回 None。
        // 如果你想获取某个特定图形后端的所有适配器，可以使用 enumerate_adapters 函数，它会返回一个迭代器，你可以遍历检查其中是否有满足需求的适配器
        /*let adapter_all = instance.enumerate_adapters(Backends::all())
            .filter(|temp_adapter| {
                // 检查该适配器是否兼容我们的展示平面
                temp_adapter.is_surface_supported(&surface)
            }).next().unwrap();*/

        //使用适配器来创建逻辑设备 (Device) 和命令队列 (Queue)。
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                //DeviceDescriptor上的 features 字段允许我们指定想要的扩展功能。对于这个简单的例子，我决定不使用任何额外的功能。
                //
                // 显卡会限制可用的扩展功能，所以如果想使用某些功能，你可能需要限制支持的设备或提供变通函数。
                //
                // 可以使用 adapter.features() 或 device.features() 获取设备支持的扩展功能列表。
                features: Features::empty(),
                // WebGL 后端并不支持 wgpu 的所有功能，
                // 所以如果要以 web 为构建目标，就必须禁用一些功能。
                //limits 字段描述了创建某些类型的资源的限制。我们在本教程中使用默认值，所以可以支持大多数设备。
                limits: if cfg!(target_arch="wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
                label: None,
            },
            None, //追踪API调用路径
        ).await.unwrap();

        let caps = surface.get_capabilities(&adapter);

        // usage 字段描述了 SurfaceTexture 如何被使用。RENDER_ATTACHMENT 指定将被用来渲染到屏幕的纹理（我们将在后面讨论更多的 TextureUsages 枚举值）。
        //
        // format 定义了 SurfaceTexture 在 GPU 内存上如何被存储。不同的显示设备偏好不同的纹理格式。
        // 我们使用surface.get_capabilities(&adapter).formats 来获取当前显示设备的最佳格式。
        //
        // width 和 height 指定 SurfaceTexture 的宽度和高度（物理像素，等于逻辑像素乘以屏幕缩放因子）。这通常就是窗口的宽和高。
        //
        // 需要确保 SurfaceTexture 的宽高不能为 0，这会导致你的应用程序崩溃。
        //
        // present_mode 指定的 wgpu::PresentMode 枚举值决定了展示平面如何与显示设备同步。我们选择的PresentMode::Fifo 指定了显示设备的刷新率做为渲染的帧速率，
        // 这本质上就是垂直同步（VSync），所有平台都得支持这种呈现模式（PresentMode）。你可以在文档中查看所有的模式。
        //当你想让用户来选择他们使用的呈现模式时，可以使用 surface.get_capabilities(&adapter) 获取展示平面支持的所有呈现模式的列表:
        //
        // let modes = surface.get_capabilities(&adapter).present_modes;
        //
        // PresentMode::Fifo 模式无论如何都是被支持的，PresentMode::AutoVsync 和 PresentMode::AutoNoVsync 支持回退，因此也能工作在所有平台上。
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        //纹理
        /*
        此处代码从图像文件中读取字节，并将其加载到 image 对象中，然后转换为 rgba 动态数组。我们还保存了图像的尺寸信息以便在创建实际纹理时使用。
        */
        let diffuse_bytes = include_bytes!("../texture.jpeg");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        //创建纹理
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // 所有纹理都是以 3D 形式存储的，我们通过设置深度 1 来表示 2D 纹理
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // 大多数图像都是使用 sRGB 来存储的，我们需要在这里指定。
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING 表示我们要在着色器中使用这个纹理。
                // COPY_DST 表示我们能将数据复制到这个纹理上。
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                view_formats: &[],
            }
        );

        /*
        填充数据到纹理中
        Texture 结构体没有函数可以直接与数据交互。但我们可以使用之前创建的命令队列上的 write_texture 命令来填充纹理数据。下边是具体代码：
        */
        queue.write_texture(
            // 告诉 wgpu 从何处复制像素数据
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            //实际像素数据
            &diffuse_rgba,
            //纹理的内存布局
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        /*
        填充纹理数据的经典方式是将像素数据先复制到一个缓冲区，然后再从缓冲区复制到纹理中。
        使用 write_texture 更有效率，因为它少用了一个缓冲区 -- 不过这里还是介绍一下，以防读者有需要：
        */
        /*let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Temp Buffer"),
                contents: &diffuse_rgba,
                usage: wgpu::BufferUsages::COPY_SRC,
            }
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });

        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * dimensions.0,
                rows_per_image: dimensions.1,
            },
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            size,
        );

        queue.submit(std::iter::once(encoder.finish()));*/

        /*
        纹理视图与采样器
        现在纹理中已经有了数据，我们需要一种方法来使用它。这，就是纹理视图（TextureView）和采样器（Sampler）的用处。

        纹理视图描述纹理及其关联的元数据。采样器控制纹理如何被 采样。采样工作类似于 GIMP/Photoshop 中的滴管工具。
        我们的程序在纹理上提供一个坐标（被称为 纹理坐标 ），然后采样器根据纹理和一些内部参数返回相应的颜色。

        现在我们来定义 diffuse_texture_view 和 diffuse_sampler：
        */
        let diffuse_texture_view = diffuse_texture.create_view(&TextureViewDescriptor::default());
        /*
        address_mode_* 参数指定了如果采样器得到的纹理坐标超出了纹理边界时该如何处理。我们有几个选项可供选择：

        ClampToEdge：任何在纹理外的纹理坐标将返回离纹理边缘最近的像素的颜色。
        Repeat。当纹理坐标超过纹理的尺寸时，纹理将重复。
        MirrorRepeat。类似于Repeat，但图像在越过边界时将翻转。

        mag_filter 与 min_filter 字段描述了当采样足迹小于或大于一个纹素（Texel）时该如何处理。当场景中的贴图远离或靠近 camera 时，这两个字段的设置通常会有效果。

        有 2 个选项:

        Linear：在每个维度中选择两个纹素，并在它们的值之间返回线性插值。
        Nearest：返回离纹理坐标最近的纹素的值。这创造了一个从远处看比较清晰但近处有像素的图像。然而，如果你的纹理被设计成像素化的，比如像素艺术游戏，或者像 Minecraft 这样的体素游戏，这可能是符合预期的。
        Mipmaps 是一个复杂的话题，需要在未来单独写一个章节。现在，我们可以说 mipmap_filter 的功能有点类似于 (mag/min)_filter，因为它告诉采样器如何在 mipmaps 之间混合。

        其他字段使用了默认值。如果想了解字段详情，请查看 wgpu 文档。

        现在，我们需要用到 BindGroup 和 PipelineLayout 来将所有这些不同的资源都接入。
        */
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        /*
        绑定组
        绑定组（BindGroup）描述了一组资源以及如何通过着色器访问它们。我们先来创建一个绑定组布局（BindGroupLayout）：
        */
        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            }
        );
        /*
        texture_bind_group_layout 有两个条目：一个是绑定到 0 资源槽的纹理，另一个是绑定到 1 资源槽的采样器。
        这两个绑定只对由 visibility 字段指定的片元着色器可见。这个字段的可选值是 NONE、VERTEX、FRAGMENT 或 COMPUTE 的任意按位或（|）组合。

        现在使用绑定组布局（texture_bind_group_layout）来创建绑定组：
        看着这个，你可能会有一点似曾相识的感觉! 这是因为绑定组是绑定组布局的一个更具体的声明。
        它们分开的原因是，只要是共享同一个绑定组布局的绑定组，就能在运行时实时切换。创建的每个纹理和采样器都需要添加到一个绑定组中。
        为了达成目的，我们将为每个纹理创建一个新的绑定组。
        */
        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        //加载shader
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("../texture/shader.wgsl").into()),
        });

        //也可以使用 include_wgsl! 宏作为创建 ShaderModuleDescriptor 的快捷方式。
        // let shader = device.create_shader_module(include_wgsl!("../pipeline/shader.wgsl"));

        /*
        管线布局
        还记得在管线章节创建的管线布局（PipelineLayout）吗？现在我们终于可以使用它了! 管线布局包含一个管线可以使用的绑定组布局的列表。
        修改 render_pipeline_layout 以使用
        */

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(" Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        /*
        可以在这里指定着色器中的哪个函数应该是入口点（ entry_point）。那是我们用 @vertex 和 @fragment 标记的函数。
        buffers 字段告诉 wgpu 要把什么类型的顶点数据传递给顶点着色器。我们会在顶点着色器中指定顶点，所以这里先留空。下一个教程中会在此加入一些数据。
        fragment 字段是 Option 类型，所以必须用 Some() 来包装 FragmentState 实例。如果想把颜色数据存储到 surface 就需要用到它 。
        targets 字段告诉 wgpu 应该设置哪些颜色输出目标。目前只需设置一个输出目标。格式指定为使用 surface 的格式，并且指定混合模式为仅用新的像素数据替换旧的。
        我们还告诉 wgpu 可写入全部 4 个颜色通道：红、蓝、绿和透明度。
        */
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),

            /*vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },*/
            //使用缓存区顶点数据
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },

            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            /*
            图元（primitive）字段描述了将如何解释顶点来转换为三角形。

            PrimitiveTopology::TriangleList 意味着每三个顶点组成一个三角形。
            front_face 字段告诉 wgpu 如何确定三角形的朝向。FrontFace::Ccw 指定顶点的帧缓冲区坐标（framebuffer coordinates）
                按逆时针顺序给出的三角形为朝前（面向屏幕外）。
            cull_mode 字段告诉 wgpu 如何做三角形剔除。CullMode::Back 指定朝后（面向屏幕内）的三角形会被剔除（不被渲染）。
                我们会在讨论缓冲区（Buffer）时详细介绍剔除问题。
            */
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                // 将此设置为 Fill 以外的任何值都要需要开启 Feature::NON_FILL_POLYGON_MODE
                polygon_mode: PolygonMode::Fill,
                // 需要开启 Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // 需要开启 Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            /*
            我们目前没有使用深度/模板缓冲区，因此将 depth_stencil 保留为 None。以后会用到。
            count 确定管线将使用多少个采样。多重采样是一个复杂的主题，因此不会在这里展开讨论。
            mask 指定哪些采样应处于活动状态。目前我们使用全部采样。
            alpha_to_coverage_enabled 与抗锯齿有关。在这里不介绍抗锯齿，因此将其保留为 false。
            multiview 表示渲染附件可以有多少数组层。我们不会渲染到数组纹理，因此将其设置为 None。
            */
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        //创建顶点缓冲区
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: BufferUsages::VERTEX,
            }
        );

        //创建索引缓冲区
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: BufferUsages::INDEX,
            }
        );
        //我们不需要为索引实现 Pod 和 Zeroable，因为 bytemuck 已经为 u16 等基本类型实现了它们。只需将 index_buffer 和 num_indices 添加到 State 结构体中。
        let num_indices = INDICES.len() as u32;

        let num_vertices = VERTICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,

            render_pipeline,

            vertex_buffer,

            num_vertices,

            index_buffer,

            num_indices,

            diffuse_bind_group
        }
    }

    //调整宽高
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { .. } => {
                // self.render().expect("failed to render!");
            }
            _ => {}
        }
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        //首先，我们需要获取一个帧（Frame）对象以供渲染
        let output = self.surface.get_current_texture().unwrap();

        //这一行创建了一个默认设置的纹理视图（TextureView），渲染代码需要利用纹理视图来与纹理交互。
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        //我们还需要创建一个命令编码器（CommandEncoder）来记录实际的命令发送给 GPU。
        // 大多数现代图形框架希望命令在被发送到 GPU 之前存储在一个命令缓冲区中。命令编码器创建了一个命令缓冲区，然后我们可以将其发送给 GPU。
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        //现在可以开始执行期盼已久的清屏（用统一的颜色填充指定渲染区域）了。我们需要使用 encoder 来创建渲染通道（RenderPass）。渲染通道编码所有实际绘制的命令。
        {
            //首先，我们来谈谈 encoder.begin_render_pass(...) 周围用 {} 开辟出来的块空间。begin_render_pass() 以可变方式借用了encoder（又称 &mut self），
            // 在释放这个可变借用之前，我们不能调用 encoder.finish()。这个块空间告诉 rust，当代码离开这个范围时，丢弃其中的任何变量，
            // 从而释放 encoder 上的可变借用，并允许我们 finish() 它。如果你不喜欢 {}，也可以使用 drop(render_pass) 来达到同样的效果。

            //代码的最后几行告诉 wgpu 完成命令缓冲区，并将其提交给 gpu 的渲染队列。


            //渲染通道描述符（RenderPassDescriptor）只有三个字段: label, color_attachments 和 depth_stencil_attachment。
            //color_attachments 描述了要将颜色绘制到哪里。我们使用之前创建的纹理视图来确保渲染到屏幕上。
            //color_attachments 字段是一个稀疏数组。这允许你使用有多个渲染目标的管线，并且最终只绘制到你所关心的某个渲染目标。
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    //RenderPassColorAttachment 有一个 view 字段，用于通知 wgpu 将颜色保存到什么纹理。
                    //这里我们指定使用 surface.get_current_texture() 创建的 view，这意味着向此附件（Attachment）上绘制的任何颜色都会被绘制到屏幕上。
                    view: &view,
                    //resolve_target 是接收多重采样解析输出的纹理。除非启用了多重采样, 否则不需要设置它，保留为 None 即可。
                    resolve_target: None,
                    //告诉 wgpu 如何处理屏幕上的颜色（由 view 指定）
                    ops: Operations {
                        //load 字段告诉 wgpu 如何处理存储在前一帧的颜色。目前，我们正在用蓝色清屏。
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        //store 字段告诉 wgpu 是否要将渲染的结果存储到纹理视图后面的纹理上（在这个例子中是 SurfaceTexture ）。
                        // 我们希望存储渲染结果，所以设置为 true。
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            //使用管线
            // 把 _render_pass 声明为可变变量并重命名为 render_pass。
            // 在 render_pass 上设置刚刚创建的管线。
            // 告诉 wgpu 用 3 个顶点和 1 个实例（实例的索引就是 @builtin(vertex_index) 的由来）来进行绘制。
            render_pass.set_pipeline(&self.render_pipeline);

            //设置绑定组
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);

            //设置顶点缓冲区
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            /*
            set_vertex_buffer 函数接收两个参数，第一个参数是顶点缓冲区要使用的缓冲槽索引。你可以连续设置多个顶点缓冲区。

            第二个参数是要使用的缓冲区的数据片断。你可以在硬件允许的情况下在一个缓冲区中存储尽可能多的对象，所以 slice 允许我们指定使用缓冲区的哪一部分。
            我们用 .. 来指定整个缓冲区。

            在继续之前，我们需要修改 render_pass.draw() 的调用来使用 VERTICES 所指定的顶点数量。
            在 State 中添加一个num_vertices，令其值等于 VERTICES.len()：
            */

            //设置索引缓冲区
            /*
            命令名称是 set_index_buffer 而不是 set_index_buffers, 一次绘制（draw_XXX()）只能设置一个索引缓冲区。
                但是，你可以在一个渲染通道内调用多次绘制，每次都设置不同的索引缓冲区。
            当使用索引缓冲区时，需使用 draw_indexed 来绘制，draw 命令会忽略索引缓冲区。
                还需确保你使用的是索引数（num_indices）而非顶点数，否则你的模型要么画错，要么因为没有足够的索引数而导致程序恐慌（panic）。
            */
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // render_pass.draw(0..self.num_vertices, 0..1);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            //在上面的修改生效之前，还需要更新着色器，以便从顶点缓冲区中获取数据。
        }

        // submit 命令能接受任何实现了 IntoIter trait 的参数
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}