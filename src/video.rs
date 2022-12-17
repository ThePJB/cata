use glow::*;
use crate::kapp::*;
use crate::renderers::ct_renderer::*;
use crate::renderers::simple_renderer::*;
use crate::renderers::texture_renderer::*;
use crate::renderers::font_rendering::*;

pub struct Video {
    pub gl: glow::Context,
    pub window: glutin::WindowedContext<glutin::PossiblyCurrent>,
    pub xres: f32,
    pub yres: f32,

    pub simple_renderer: SimpleRenderer,
    pub texture_renderer: TextureRenderer,
    pub ct_renderer: CTRenderer
}

impl Video {
    pub fn new(title: &str, xres: f32, yres: f32, event_loop: &glutin::event_loop::EventLoop<()>) -> Video {
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(glutin::dpi::PhysicalSize::new(xres, yres));

        let window = unsafe {
            glutin::ContextBuilder::new()
                // .with_depth_buffer(0)
                // .with_srgb(true)
                // .with_stencil_buffer(0)
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap()
        };

        let gl = unsafe {
            let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            gl.enable(DEPTH_TEST);
            // gl.enable(CULL_FACE);
            gl.blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
            gl.enable(BLEND);
            gl.debug_message_callback(|a, b, c, d, msg| {
                println!("{} {} {} {} msg: {}", a, b, c, d, msg);
            });
            gl
        };

        let simple_renderer = SimpleRenderer::new(&gl);
        let texture_renderer = TextureRenderer::new(&gl);
        let ct_renderer = CTRenderer::new(&gl, "font.png");

        Video {
            gl,
            window,
            xres,
            yres,
            simple_renderer,
            texture_renderer,
            ct_renderer,
        }
    }

    pub fn render(&mut self, outputs: &FrameOutputs, a: f32) {
        unsafe {
            for (buf, idx) in &outputs.set_texture {
                self.texture_renderer.update(&self.gl, buf, *idx);
            }

            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT); 
            
            self.simple_renderer.render(&self.gl, &outputs.canvas);

            for (r, idx, depth) in &outputs.draw_texture {
                self.texture_renderer.render(&self.gl, *r, a, *idx, *depth);
            }

            // self.gl.clear(glow::DEPTH_BUFFER_BIT); 
            

            let font_ct_canvas = glyph_buffer_to_canvas(&outputs.glyphs, a);
            self.ct_renderer.render(&self.gl, &font_ct_canvas);

            self.window.swap_buffers().unwrap();
        }
    }
}

