use egui::{examples::ExampleApp, label, Align, Button, Layout, TextStyle};
use kiss3d::light::Light;
use kiss3d::window::Window;
use std::time::Instant;

fn main() {
    let mut window: Window<kiss3d_egui::EguiContext> = Window::new_with_ui("Kiss3d: UI", ());
    window.set_background_color(1.0, 1.0, 1.0);
    let mut c = window.add_cube(0.1, 0.1, 0.1);
    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    let mut example_app: ExampleApp = ExampleApp::default();
    //read_json(app_json_path).unwrap_or_default();
    let mut running = true;
    let mut frame_start = Instant::now();
    // let mut frame_times = egui::MovementTracker::new(1000, 1.0);

    // Render loop.
    while window.render() {
        let egui_start = Instant::now();
        window.ui_mut().frame(|ui| {
            example_app.ui(ui, "");
            let mut ui = ui.centered_column(ui.available().width().min(480.0));
            ui.set_layout(Layout::vertical(Align::Min));
            ui.add(label!("Egui running inside of Glium").text_style(TextStyle::Heading));
            if ui.add(Button::new("Quit")).clicked {
                // running = false;
            }

            ui.add(
                label!(
                    "CPU usage: {:.2} ms (excludes painting)",
                    /*1e3 * frame_times.average().unwrap_or_default()*/ 0.0
                )
                .text_style(TextStyle::Monospace),
            );
            ui.add(
                label!(
                    "FPS: {:.1}",
                    /*1.0 / frame_times.mean_time_interval().unwrap_or_default()*/ 0.0
                )
                .text_style(TextStyle::Monospace),
            );
        });

        // frame_times.add(
        //     raw_input.time,
        //     (Instant::now() - egui_start).as_secs_f64() as f32,
        // );
    }
}
