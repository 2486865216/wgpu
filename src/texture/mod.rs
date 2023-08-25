/*
纹理（Textures）是叠加在三角形网格（Mesh）上的图像，使其看起来有丰富的细节。
有多种类型的纹理，如法线贴图（Normal Maps，也就是法线纹理）、凹凸贴图（Bump Maps）、镜面贴图和漫反射贴图。下边将讨论漫反射贴图，简单来说也就是颜色纹理。

加载图像文件
要把一个图像映射到对象网格上，首先是需要有一个图像文件。就使用下边这棵快乐的小树吧：

一棵快乐的树

我们将使用 image 包 来加载这棵树。先把它添加到依赖项中：

[dependencies.image]
version = "0.24"
default-features = false
features = ["png", "jpeg"]
image 包含的 jpeg 解码器使用 rayon 来加速线程的解码速度。WASM 目前不支持线程，所以我们需要禁用这一特性，这样代码在尝试加载网络上的 jpeg 时就不会崩溃。

在 WASM 中解码 jpeg 性能不高。如果你想在 WASM 中加快图像加载速度，可以选择使用浏览器的内置解码器来替换 wasm-bindgen 构建时使用 的 image。
这涉及到在 Rust 中创建一个 <img> 标记来获取图像，然后创建一个 <canvas> 来获取像素数据，我把这留作读者的练习。
*/