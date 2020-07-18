use glow::HasContext;
use iced_native::{mouse, window, Debug, Event, Program};
use kiss3d::{context::Context, event::WindowEvent, window::UiContext};
use nalgebra::Vector2;

mod backend;
mod program;
mod quad;
mod text;
mod triangle;

pub mod settings;
pub mod widget;

pub use backend::Backend;
pub use iced_graphics::Viewport;
pub use settings::Settings;

pub(crate) use iced_graphics::{Point, Size, Transformation};

pub type Renderer = iced_graphics::Renderer<Backend>;

pub struct IcedContext<P>
where
    P: Program<Renderer = Renderer> + 'static,
{
    // pending_program: Option<P>,
    state: Option<iced_native::program::State<P>>,
    debug: Debug,
    renderer: Renderer,
    viewport: Viewport,
    cursor_position: (f64, f64),
    // TODO
}

impl<P> IcedContext<P>
where
    P: Program<Renderer = Renderer> + 'static,
{
    pub fn set_program(&mut self, program: P) {
        // self.pending_program = Some(program);
        self.state = Some(iced_native::program::State::new(
            program,
            self.viewport.logical_size(),
            // conversion::cursor_position(cursor_position, viewport.scale_factor()),
            Point::new(self.cursor_position.0 as f32, self.cursor_position.1 as f32),
            &mut self.renderer,
            &mut self.debug,
        ))
    }

    pub fn program(&self) -> Option<&P> {
        self.state.as_ref().map(|s| s.program())
    }
}

impl<P> UiContext for IcedContext<P>
where
    P: Program<Renderer = Renderer> + 'static,
{
    fn new(width: u32, height: u32) -> Self {
        Self {
            // pending_program: None,
            state: None,
            debug: Debug::new(),
            renderer: Renderer::new(Backend::new(Settings::default())),
            viewport: Viewport::with_physical_size(Size::new(width, height), 1.0),
            cursor_position: (0.0, 0.0),
        }
    }

    fn handle_event(&mut self, event: &WindowEvent, size: Vector2<u32>, hidpi_factor: f64) -> bool {
        // if let Some(viewport) = &self.cached_viewport {
        if self.viewport.physical_width() != size.x
            || self.viewport.physical_height() != size.y
            || self.viewport.scale_factor() != hidpi_factor
        {
            self.viewport = Viewport::with_physical_size(Size::new(size.x, size.y), hidpi_factor)
        }
        // }
        // let viewport = self
        //     .cached_viewport
        //     .get_or_insert_with(|| Viewport::with_physical_size(Size::new(size.x, size.y), hidpi));

        // if let Some(program) = self.pending_program.take() {
        //     self.state = Some(program::State::new(
        //         program,
        //         self.viewport.logical_size(),
        //         // conversion::cursor_position(cursor_position, viewport.scale_factor()),
        //         Point::new(-1.0, -1.0), // TODO
        //         &mut self.renderer,
        //         &mut self.debug,
        //     ))
        // }

        match event {
            WindowEvent::CursorPos(x, y, mods) => {
                self.cursor_position = (x / hidpi_factor, y / hidpi_factor);
            }
            _ => {}
        }

        let state = match &mut self.state {
            Some(x) => x,
            None => return false,
        };
        if let Some(event) = window_event_to_iced_event(*event, size, hidpi_factor) {
            state.queue_event(event);
        }
        // TODO
        // todo!()
        false
    }

    fn render(&mut self, width: u32, height: u32, hidpi_factor: f64) {
        // let viewport = match &self.cached_viewport {
        //     Some(x) => x,
        //     None => return,
        // };
        if self.viewport.physical_width() != width
            || self.viewport.physical_height() != height
            || self.viewport.scale_factor() != hidpi_factor
        {
            self.viewport = Viewport::with_physical_size(Size::new(width, height), hidpi_factor)
        }
        let state = match &mut self.state {
            Some(x) => x,
            None => return,
        };

        // We update iced
        let _ = state.update(
            self.viewport.logical_size(),
            // conversion::cursor_position(cursor_position, viewport.scale_factor()),
            Point::new(self.cursor_position.0 as f32, self.cursor_position.1 as f32),
            None,
            &mut self.renderer,
            &mut self.debug,
        );

        // Then draw iced on top
        let ctxt = Context::get();
        let gl = ctxt.get_glow();

        // Enable auto-conversion from/to sRGB
        unsafe { gl.enable(glow::FRAMEBUFFER_SRGB) };

        // Enable alpha blending
        unsafe { gl.enable(glow::BLEND) };
        unsafe { gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA) };

        // Disable multisampling by default
        unsafe { gl.disable(glow::MULTISAMPLE) };

        let mouse_interaction = self.renderer.backend_mut().draw(
            gl,
            &self.viewport,
            state.primitive(),
            &self.debug.overlay(),
        );

        unsafe { gl.enable(glow::MULTISAMPLE) };
        unsafe { gl.disable(glow::BLEND) };
        unsafe { gl.disable(glow::FRAMEBUFFER_SRGB) };

        // // And update the mouse cursor
        // window.set_cursor_icon(
        //     iced_winit::conversion::mouse_interaction(
        //         mouse_interaction,
        //     ),
        // );
    }
}

fn window_event_to_iced_event(event: WindowEvent, size: Vector2<u32>, hidpi: f64) -> Option<Event> {
    match event {
        WindowEvent::FramebufferSize(w, h) => Some(Event::Window(window::Event::Resized {
            width: (w as f64 / hidpi) as u32,
            height: (h as f64 / hidpi) as u32,
        })),
        // WindowEvent::Size(w, h) => {}
        // WindowEvent::CursorEnter(_) => {}
        WindowEvent::CursorPos(x, y, mods) => Some(Event::Mouse(mouse::Event::CursorMoved {
            x: (x / hidpi) as f32,
            y: (y / hidpi) as f32,
        })),
        WindowEvent::MouseButton(btn, act, mods) => {
            let button = match btn {
                kiss3d::event::MouseButton::Button1 => mouse::Button::Left,
                kiss3d::event::MouseButton::Button2 => mouse::Button::Right,
                kiss3d::event::MouseButton::Button3 => mouse::Button::Middle,
                kiss3d::event::MouseButton::Button4 => mouse::Button::Other(4),
                kiss3d::event::MouseButton::Button5 => mouse::Button::Other(5),
                kiss3d::event::MouseButton::Button6 => mouse::Button::Other(6),
                kiss3d::event::MouseButton::Button7 => mouse::Button::Other(7),
                kiss3d::event::MouseButton::Button8 => mouse::Button::Other(8),
            };
            Some(Event::Mouse(match act {
                kiss3d::event::Action::Press => mouse::Event::ButtonPressed(button),
                kiss3d::event::Action::Release => mouse::Event::ButtonReleased(button),
            }))
        }
        // WindowEvent::Scroll(_, _, _) => {}
        // WindowEvent::Pos(_, _) => {}
        // WindowEvent::Close => {}
        // WindowEvent::Refresh => {}
        // WindowEvent::Focus(_) => {}
        // WindowEvent::Iconify(_) => {}
        // WindowEvent::Key(_, _, _) => {}
        // WindowEvent::Char(_) => {}
        // WindowEvent::CharModifiers(_, _) => {}
        // WindowEvent::Touch(_, _, _, _, _) => {}
        _ => None,
    }
}
