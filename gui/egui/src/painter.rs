use egui::{
    paint::{PaintBatches, Triangles},
    Rect,
};
use kiss3d::{
    context::Context,
    resource::{
        AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform, Texture,
    },
};
use nalgebra::{Point2, Point3, Point4, Vector2};

macro_rules! verify(
    ($e: expr) => {
        {
            let res = $e;
            #[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
            { assert_eq!(kiss3d::context::Context::get().get_error(), 0); }
            res
        }
    }
);

pub struct Painter {
    // program: glium::Program,
    // shader: Effect,
    // texture: texture::texture2d::Texture2d,
    // texture: Texture,
    // current_texture_id: Option<u64>,
    triangle_shader: Effect,
    triangle_window_size: ShaderUniform<Vector2<f32>>,
    triangle_pos: ShaderAttribute<Point2<f32>>,
    triangle_color: ShaderAttribute<Point4<f32>>,
    points: GPUVec<f32>,
    indices: GPUVec<Point3<u16>>,
}

impl Painter {
    pub fn new() -> Painter {
        // let shader = Effect::new_from_str(
        //     "
        //         #version 140
        //         uniform vec4 u_clip_rect; // min_x, min_y, max_x, max_y
        //         uniform vec2 u_screen_size;
        //         uniform vec2 u_tex_size;
        //         in vec2 a_pos;
        //         in vec4 a_color;
        //         in vec2 a_tc;
        //         out vec2 v_pos;
        //         out vec4 v_color;
        //         out vec2 v_tc;
        //         out vec4 v_clip_rect;
        //         void main() {
        //             gl_Position = vec4(
        //                 2.0 * a_pos.x / u_screen_size.x - 1.0,
        //                 1.0 - 2.0 * a_pos.y / u_screen_size.y,
        //                 0.0,
        //                 1.0);
        //             v_pos = a_pos;
        //             v_color = a_color / 255.0;
        //             v_tc = a_tc / u_tex_size;
        //             v_clip_rect = u_clip_rect;
        //         }
        //     ",
        //     "
        //         #version 140
        //         uniform sampler2D u_sampler;
        //         in vec2 v_pos;
        //         in vec4 v_color;
        //         in vec2 v_tc;
        //         in vec4 v_clip_rect;
        //         out vec4 f_color;

        //         // glium expects linear output.
        //         vec3 linear_from_srgb(vec3 srgb) {
        //             bvec3 cutoff = lessThan(srgb, vec3(0.04045));
        //             vec3 higher = pow((srgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
        //             vec3 lower = srgb / vec3(12.92);
        //             return mix(higher, lower, cutoff);
        //         }

        //         void main() {
        //             if (v_pos.x < v_clip_rect.x) { discard; }
        //             if (v_pos.y < v_clip_rect.y) { discard; }
        //             if (v_pos.x > v_clip_rect.z) { discard; }
        //             if (v_pos.y > v_clip_rect.w) { discard; }
        //             f_color = v_color;
        //             f_color.rgb = linear_from_srgb(f_color.rgb);
        //             f_color *= texture(u_sampler, v_tc).r;
        //         }
        //     ",
        // );

        // let pixels = vec![vec![255u8, 0u8], vec![0u8, 255u8]];
        // let format = texture::UncompressedFloatFormat::U8;
        // let mipmaps = texture::MipmapsOption::NoMipmap;
        // let texture =
        //     texture::texture2d::Texture2d::with_format(facade, pixels, format, mipmaps).unwrap();

        let triangle_shader = Effect::new_from_str(TRIANGLES_VERTEX_SRC, TRIANGLES_FRAGMENT_SRC);

        Painter {
            // program,
            // texture,
            // current_texture_id: None,
            // shader,
            triangle_window_size: triangle_shader.get_uniform("window_size").unwrap(),
            triangle_pos: triangle_shader.get_attrib("position").unwrap(),
            triangle_color: triangle_shader.get_attrib("color").unwrap(),
            triangle_shader,
            points: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
            indices: GPUVec::new(
                Vec::new(),
                BufferType::ElementArray,
                AllocationType::StreamDraw,
            ),
        }
    }

    // fn upload_texture(&mut self, facade: &dyn glium::backend::Facade, texture: &egui::Texture) {
    //     if self.current_texture_id == Some(texture.id) {
    //         return; // No change
    //     }

    //     let pixels: Vec<Vec<u8>> = texture
    //         .pixels
    //         .chunks(texture.width as usize)
    //         .map(|row| row.to_vec())
    //         .collect();

    //     let format = texture::UncompressedFloatFormat::U8;
    //     let mipmaps = texture::MipmapsOption::NoMipmap;
    //     self.texture =
    //         texture::texture2d::Texture2d::with_format(facade, pixels, format, mipmaps).unwrap();
    //     self.current_texture_id = Some(texture.id);
    // }

    pub fn paint_batches(
        &mut self,
        // display: &glium::Display,
        width: f32,
        height: f32,
        batches: PaintBatches,
        texture: &egui::Texture,
    ) {
        let ctxt = Context::get();

        verify!(ctxt.disable(Context::CULL_FACE));
        let _ = verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::FILL));
        verify!(ctxt.enable(Context::BLEND));
        verify!(ctxt.blend_func_separate(
            Context::SRC_ALPHA,
            Context::ONE_MINUS_SRC_ALPHA,
            Context::ONE,
            Context::ONE_MINUS_SRC_ALPHA,
        ));
        verify!(ctxt.disable(Context::DEPTH_TEST));
        verify!(ctxt.enable(Context::SCISSOR_TEST));

        // self.upload_texture(display, texture);

        // let mut target = display.draw();
        // target.clear_color(0.0, 0.0, 0.0, 0.0);
        for (clip_rect, triangles) in batches {
            self.paint_batch(
                // &mut target,
                &ctxt, /*display,*/ width, height, clip_rect, &triangles, texture,
            )
        }
        // target.finish().unwrap();

        verify!(ctxt.enable(Context::DEPTH_TEST));
        verify!(ctxt.disable(Context::BLEND));
        ctxt.scissor(0, 0, width as i32, height as i32);
    }

    #[inline(never)] // Easier profiling
    fn paint_batch(
        &mut self,
        // target: &mut Frame,
        ctxt: &Context,
        // display: &glium::Display,
        width: f32,
        height: f32,
        clip_rect: Rect,
        triangles: &Triangles,
        texture: &egui::Texture,
    ) {
        // let vertex_buffer = {
        //     #[derive(Copy, Clone)]
        //     struct Vertex {
        //         a_pos: [f32; 2],
        //         a_color: [u8; 4],
        //         a_tc: [u16; 2],
        //     }
        //     // implement_vertex!(Vertex, a_pos, a_color, a_tc);

        //     let vertices: Vec<Vertex> = triangles
        //         .vertices
        //         .iter()
        //         .map(|v| Vertex {
        //             a_pos: [v.pos.x, v.pos.y],
        //             a_color: [v.color.r, v.color.g, v.color.b, v.color.a],
        //             a_tc: [v.uv.0, v.uv.1],
        //         })
        //         .collect();

        //     glium::VertexBuffer::new(display, &vertices).unwrap()
        // };
        self.points.data_mut().as_mut().unwrap().clear();
        self.indices.data_mut().as_mut().unwrap().clear();

        let vertices = self.points.data_mut().as_mut().unwrap();
        let indices = self.indices.data_mut().as_mut().unwrap();

        vertices.extend(triangles.vertices.iter().flat_map(|v| {
            vec![
                v.pos.x,
                v.pos.y,
                v.color.r as f32 / 255.0,
                v.color.g as f32 / 255.0,
                v.color.b as f32 / 255.0,
                v.color.a as f32 / 255.0,
            ]
        }));
        indices.extend(
            triangles
                .indices
                .chunks_exact(3)
                .map(|x| Point3::new(x[0] as u16, x[1] as u16, x[2] as u16)),
        );

        // let indices: Vec<u32> = triangles.indices.iter().map(|idx| *idx as u32).collect();

        // let index_buffer =
        //     glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap();

        // let pixels_per_point = display.gl_window().get_hidpi_factor() as f32;
        // let (width_pixels, height_pixels) = display.get_framebuffer_dimensions();
        // let width_points = width_pixels as f32 / pixels_per_point;
        // let height_points = height_pixels as f32 / pixels_per_point;

        // let uniforms = uniform! {
        //     u_clip_rect: [clip_rect.min.x, clip_rect.min.y, clip_rect.max.x, clip_rect.max.y],
        //     u_screen_size: [width_points, height_points],
        //     u_tex_size: [texture.width as f32, texture.height as f32],
        //     u_sampler: &self.texture,
        // };

        // // Emilib outputs colors with premultiplied alpha:
        // let blend_func = glium::BlendingFunction::Addition {
        //     source: glium::LinearBlendingFactor::One,
        //     destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
        // };
        // let blend = glium::Blend {
        //     color: blend_func,
        //     alpha: blend_func,
        //     ..Default::default()
        // };

        // let params = glium::DrawParameters {
        //     blend,
        //     ..Default::default()
        // };

        // target
        //     .draw(
        //         &vertex_buffer,
        //         &index_buffer,
        //         &self.program,
        //         &uniforms,
        //         &params,
        //     )
        //     .unwrap();

        self.triangle_shader.use_program();
        self.triangle_pos.enable();
        self.triangle_color.enable();

        self.triangle_window_size
            .upload(&Vector2::new(width, height));
        unsafe {
            self.triangle_color
                .bind_sub_buffer_generic(&mut self.points, 5, 2)
        };
        unsafe {
            self.triangle_pos
                .bind_sub_buffer_generic(&mut self.points, 5, 0)
        };
        self.indices.bind();

        verify!(ctxt.draw_elements(
            Context::TRIANGLES,
            self.indices.len() as i32 * 3,
            Context::UNSIGNED_SHORT,
            0
        ));

        self.triangle_pos.disable();
        self.triangle_color.disable();
    }
}

static TRIANGLES_VERTEX_SRC: &'static str = "#version 100
attribute vec2 position;
attribute vec4 color;

uniform vec2 window_size;

varying vec4 v_color;

void main(){
    gl_Position = vec4(position / window_size * 2.0, 0.0, 1.0);
    v_color = color;
}";

static TRIANGLES_FRAGMENT_SRC: &'static str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

varying vec4 v_color;

void main() {
  gl_FragColor = v_color;
}";
