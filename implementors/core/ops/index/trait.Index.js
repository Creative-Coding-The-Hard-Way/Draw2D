(function() {var implementors = {};
implementors["image"] = [{"text":"impl&lt;Buffer&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.u8.html\">u8</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.u32.html\">u32</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.u32.html\">u32</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"image/flat/struct.FlatSamples.html\" title=\"struct image::flat::FlatSamples\">FlatSamples</a>&lt;Buffer&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Buffer: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt;,&nbsp;</span>","synthetic":false,"types":["image::flat::FlatSamples"]},{"text":"impl&lt;P, Container&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.u32.html\">u32</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.u32.html\">u32</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"image/struct.ImageBuffer.html\" title=\"struct image::ImageBuffer\">ImageBuffer</a>&lt;P, Container&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;P: <a class=\"trait\" href=\"image/trait.Pixel.html\" title=\"trait image::Pixel\">Pixel</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;P::<a class=\"type\" href=\"image/trait.Pixel.html#associatedtype.Subpixel\" title=\"type image::Pixel::Subpixel\">Subpixel</a>: 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;Container: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a>&lt;Target = <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.slice.html\">[</a>P::<a class=\"type\" href=\"image/trait.Pixel.html#associatedtype.Subpixel\" title=\"type image::Pixel::Subpixel\">Subpixel</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.slice.html\">]</a>&gt;,&nbsp;</span>","synthetic":false,"types":["image::buffer_::ImageBuffer"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.Rgb.html\" title=\"struct image::Rgb\">Rgb</a>&lt;T&gt;","synthetic":false,"types":["image::color::Rgb"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.Bgr.html\" title=\"struct image::Bgr\">Bgr</a>&lt;T&gt;","synthetic":false,"types":["image::color::Bgr"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.Luma.html\" title=\"struct image::Luma\">Luma</a>&lt;T&gt;","synthetic":false,"types":["image::color::Luma"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.Rgba.html\" title=\"struct image::Rgba\">Rgba</a>&lt;T&gt;","synthetic":false,"types":["image::color::Rgba"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.Bgra.html\" title=\"struct image::Bgra\">Bgra</a>&lt;T&gt;","synthetic":false,"types":["image::color::Bgra"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"image/trait.Primitive.html\" title=\"trait image::Primitive\">Primitive</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"image/struct.LumaA.html\" title=\"struct image::LumaA\">LumaA</a>&lt;T&gt;","synthetic":false,"types":["image::color::LumaA"]}];
implementors["nalgebra"] = [{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a>, R:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.Dim.html\" title=\"trait nalgebra::base::dimension::Dim\">Dim</a>, C:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.Dim.html\" title=\"trait nalgebra::base::dimension::Dim\">Dim</a>, S:&nbsp;<a class=\"trait\" href=\"nalgebra/base/storage/trait.Storage.html\" title=\"trait nalgebra::base::storage::Storage\">Storage</a>&lt;N, R, C&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"nalgebra/base/struct.Matrix.html\" title=\"struct nalgebra::base::Matrix\">Matrix</a>&lt;N, R, C, S&gt;","synthetic":false,"types":["nalgebra::base::matrix::Matrix"]},{"text":"impl&lt;N, R:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.Dim.html\" title=\"trait nalgebra::base::dimension::Dim\">Dim</a>, C:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.Dim.html\" title=\"trait nalgebra::base::dimension::Dim\">Dim</a>, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"nalgebra/base/struct.Matrix.html\" title=\"struct nalgebra::base::Matrix\">Matrix</a>&lt;N, R, C, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N: <a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"nalgebra/base/storage/trait.Storage.html\" title=\"trait nalgebra::base::storage::Storage\">Storage</a>&lt;N, R, C&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::base::matrix::Matrix"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a>, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.Point.html\" title=\"struct nalgebra::geometry::Point\">Point</a>&lt;N, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, D&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::point::Point"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a>, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.Rotation.html\" title=\"struct nalgebra::geometry::Rotation\">Rotation</a>&lt;N, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, D, D&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::rotation::Rotation"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.Quaternion.html\" title=\"struct nalgebra::geometry::Quaternion\">Quaternion</a>&lt;N&gt;","synthetic":false,"types":["nalgebra::geometry::quaternion::Quaternion"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.DualQuaternion.html\" title=\"struct nalgebra::geometry::DualQuaternion\">DualQuaternion</a>&lt;N&gt;","synthetic":false,"types":["nalgebra::geometry::dual_quaternion::DualQuaternion"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.RealField.html\" title=\"trait nalgebra::RealField\">RealField</a>, D, C:&nbsp;<a class=\"trait\" href=\"nalgebra/geometry/trait.TCategory.html\" title=\"trait nalgebra::geometry::TCategory\">TCategory</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">(</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.tuple.html\">)</a>&gt; for <a class=\"struct\" href=\"nalgebra/geometry/struct.Transform.html\" title=\"struct nalgebra::geometry::Transform\">Transform</a>&lt;N, D, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;D: <a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a> + <a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimNameAdd.html\" title=\"trait nalgebra::base::dimension::DimNameAdd\">DimNameAdd</a>&lt;<a class=\"struct\" href=\"nalgebra/base/dimension/struct.U1.html\" title=\"struct nalgebra::base::dimension::U1\">U1</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, <a class=\"type\" href=\"nalgebra/base/dimension/type.DimNameSum.html\" title=\"type nalgebra::base::dimension::DimNameSum\">DimNameSum</a>&lt;D, <a class=\"struct\" href=\"nalgebra/base/dimension/struct.U1.html\" title=\"struct nalgebra::base::dimension::U1\">U1</a>&gt;, <a class=\"type\" href=\"nalgebra/base/dimension/type.DimNameSum.html\" title=\"type nalgebra::base::dimension::DimNameSum\">DimNameSum</a>&lt;D, <a class=\"struct\" href=\"nalgebra/base/dimension/struct.U1.html\" title=\"struct nalgebra::base::dimension::U1\">U1</a>&gt;&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::transform::Transform"]}];
implementors["syn"] = [{"text":"impl&lt;T, P&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.55.0/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"syn/punctuated/struct.Punctuated.html\" title=\"struct syn::punctuated::Punctuated\">Punctuated</a>&lt;T, P&gt;","synthetic":false,"types":["syn::punctuated::Punctuated"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()