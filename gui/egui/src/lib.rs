use egui::{Output, PaintBatches};
use kiss3d::{
    event::{Action, MouseButton, WindowEvent},
    window::UiContext,
};
use nalgebra::Vector2;
use std::{sync::Arc, time::Instant};

mod painter;

pub struct EguiContext {
    ctx: Arc<egui::Context>,
    raw_input: egui::RawInput,
    start_time: Instant,
    painter: painter::Painter,
    ui_frame_output: Option<(Output, PaintBatches)>,
}

impl EguiContext {
    pub fn frame(&mut self, mut closure: impl FnMut(&mut egui::Ui)) {
        let mut ui = self.ctx.begin_frame(self.raw_input.clone()); // TODO: avoid clone
        closure(&mut ui);
        let (output, paint_batches) = self.ctx.end_frame();
        self.ui_frame_output = Some((output, paint_batches));
    }
}

impl UiContext for EguiContext {
    type Init = ();

    fn new(width: u32, height: u32, _ui_init: Self::Init) -> Self {
        let pixels_per_point = 1.0; // FIXME
        let mut ctx = egui::Context::new();
        let mut raw_input = egui::RawInput {
            screen_size: {
                // let (width, height) = display.get_framebuffer_dimensions();
                egui::vec2(width as f32, height as f32) / pixels_per_point
            },
            pixels_per_point: Some(pixels_per_point),
            ..Default::default()
        };
        let start_time = Instant::now();
        Self {
            ctx,
            raw_input,
            start_time,
            painter: painter::Painter::new(),
            ui_frame_output: None,
        }
    }

    fn handle_event(&mut self, event: &WindowEvent, size: Vector2<u32>, hidpi: f64) -> bool {
        window_event_to_egui_input(*event, size, hidpi, &mut self.raw_input)
    }

    fn render(&mut self, width: u32, height: u32, hidpi_factor: f64) {
        self.raw_input.screen_size = egui::vec2(width as f32, height as f32);
        // let (output, paint_batches) = self.ctx.end_frame();
        if let Some((output, paint_batches)) = self.ui_frame_output.take() {
            self.painter.paint_batches(
                width as f32,
                height as f32,
                paint_batches,
                self.ctx.texture(),
            );
        }

        // FIXME: ???
        let raw_input = &mut self.raw_input;
        raw_input.time = self.start_time.elapsed().as_nanos() as f64 * 1e-9;
        raw_input.seconds_since_midnight = Some(local_time_of_day());
        raw_input.scroll_delta = egui::vec2(0.0, 0.0);
        raw_input.events.clear();
    }
}

fn local_time_of_day() -> f64 {
    use chrono::Timelike;
    let time = chrono::Local::now().time();
    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
}

fn window_event_to_egui_input(
    event: WindowEvent,
    size: Vector2<u32>,
    hidpi: f64,
    raw_input: &mut egui::RawInput,
) -> bool {
    match event {
        WindowEvent::MouseButton(MouseButton::Button1, action, mods) => {
            raw_input.mouse_down = action == Action::Press;
            true
        }
        // WindowEvent::CursorEnter(_) => {}
        WindowEvent::CursorPos(x, y, mods) => {
            raw_input.mouse_pos = Some(egui::pos2(x as f32, y as f32));
            true
        }
        // WindowEvent::FramebufferSize(w, h) => {
        //     raw_input.screen_size
        // }
        // WindowEvent::Pos(_, _) => {}
        // WindowEvent::Size(_, _) => {}
        // WindowEvent::Close => {}
        // WindowEvent::Refresh => {}
        // WindowEvent::Focus(_) => {}
        // WindowEvent::Iconify(_) => {}
        // WindowEvent::Scroll(_, _, _) => {}
        // WindowEvent::Key(_, _, _) => {}
        // WindowEvent::Char(_) => {}
        // WindowEvent::CharModifiers(_, _) => {}
        // WindowEvent::Touch(_, _, _, _, _) => {}
        _ => false,
    }
}
