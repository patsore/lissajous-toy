use std::f64::consts::PI;
use std::ops::Div;
use notan::draw::*;
use notan::egui;
use notan::egui::{EguiConfig, EguiPluginSugar};
use notan::math::{Mat4, Vec2, vec2, vec3};
use notan::prelude::*;


#[notan_main]
fn main() -> Result<(), String> {
    let win = WindowConfig::new()
        .set_vsync(true)
        .set_maximized(true)
        // .set_lazy_loop(true)
        .set_high_dpi(true);

    notan::init_with(State::default)
        .add_config(win)
        .add_config(DrawConfig)
        .add_config(EguiConfig)
        .update(update)
        .draw(draw)
        .build()
}

#[derive(AppState)]
struct State {
    amplitude_x: f32,
    amplitude_y: f32,
    frequency_x: f32,
    frequency_y: f32,
    phase_shift_x: f32,
    phase_shift_y: f32,
    time: f32,
    details: i32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            amplitude_x: 610f32,
            amplitude_y: 580f32,
            frequency_x: 52f32,
            frequency_y: 51f32,
            phase_shift_x: 0.0,
            phase_shift_y: 0.0,
            time: 0.0,
            details: 500,
        }
    }
}

const WORK_SIZE: Vec2 = vec2(1920.0, 1080.0);

fn update(app: &mut App, state: &mut State) {
    state.time = app.timer.elapsed_f32();
}

fn draw(gfx: &mut Graphics, plugins: &mut Plugins, state: &mut State) {
    let time = state.time.clone();
    let mut output = plugins.egui(|ctx| {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {

            ui.heading("Define the width and height of the curve, respectively.");
            ui.add(egui::Slider::new(&mut state.amplitude_x, 0.0..=1000.0).text("X-axis amplitude"));
            ui.add(egui::Slider::new(&mut state.amplitude_y, 0.0..=1000.0).text("Y-axis amplitude"));

            ui.separator();
            ui.heading("Define the shape of the curve");
            ui.add(egui::Slider::new(&mut state.frequency_x, 0.0..=100.0).text("X-axis frequency"));
            ui.add(egui::Slider::new(&mut state.frequency_y, 0.0..=100.0).text("Y-axis frequency"));

            ui.separator();

            ui.heading("Defines how many points to use for drawing the curve, or the amount of detail");
            ui.label("I don't recommend going all the way up, it doesn't add particularly much, just makes it lag.");
            ui.add(egui::Slider::new(&mut state.details, 0..=5000).text("Detalization level"));

            ui.separator();

            ui.label("Mostly inconsequential, just here for the sake of completeness");
            ui.add(egui::Slider::new(&mut state.phase_shift_x, 0.0..=100.0).text("X-axis phase shift"));
            ui.add(egui::Slider::new(&mut state.phase_shift_y, 0.0..=100.0).text("Y-axis phase shift"));
        });
    });

    let center = WORK_SIZE.div(2.0);
    //prepare everything for drawing
    let mut draw = gfx.create_draw();
    // get the projection that will fit and center our content in the screen
    let (width, height) = gfx.size();
    let win_size = vec2(width as f32, height as f32);
    let (projection, _) = calc_projection(win_size, WORK_SIZE);
    draw.set_projection(Some(projection));
    draw.clear(Color::BLACK);

    let amplitude_x = state.amplitude_x.clone();
    let amplitude_y = state.amplitude_y.clone();
    let frequency_x = state.frequency_x.clone();
    let frequency_y = state.frequency_y.clone();
    let phase_shift_x = state.phase_shift_x.clone() + time;
    let phase_shift_y = state.phase_shift_y.clone() + time;
    {
        let mut path = draw.path();

        let radians = |i: f32| i.to_radians();

        let points = (0..state.details)
            .map(|i| {
                let t = radians(i as f32);
                let next_t = radians((i + 1) as f32);

                let x = amplitude_x * f32::sin(frequency_x * t + phase_shift_x) + center.x;
                let y = amplitude_y * f32::sin(frequency_y * t + phase_shift_y) + center.y;
                let next_x = amplitude_x * f32::sin(frequency_x * next_t + phase_shift_x) + center.x;
                let next_y = amplitude_y * f32::sin(frequency_y * next_t + phase_shift_y) + center.y;

                (x, y, next_x, next_y)
            });

        for (x, y, next_x, next_y) in points.into_iter(){
            let control_point2 = (
                (x + 2.0 * next_x) / 3.0,
                (y + 2.0 * next_y) / 3.0,
            );

            path.quadratic_bezier_to((x, y), control_point2);
        }

        path.round_cap().round_join().stroke(3.0).stroke_color(Color::MAGENTA);
    }
    gfx.render(&draw);
    gfx.render(&output);
}

fn calc_projection(win_size: Vec2, work_size: Vec2) -> (Mat4, f32) {
    let ratio = (win_size.x / work_size.x).min(win_size.y / work_size.y);

    let projection = Mat4::orthographic_rh_gl(0.0, win_size.x, win_size.y, 0.0, -1.0, 1.0);
    let scale = Mat4::from_scale(vec3(ratio, ratio, 1.0));
    let position = vec3(
        (win_size.x - work_size.x * ratio) * 0.5,
        (win_size.y - work_size.y * ratio) * 0.5,
        1.0,
    );
    let translation = Mat4::from_translation(position);
    (projection * translation * scale, ratio)
}