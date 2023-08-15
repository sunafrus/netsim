use std::time::Duration;

use egui::{Color32, Vec2, Stroke, Pos2};
use eframe::egui;

const TARGET_DISTANCE: f32 = 100.0;
const EPSILON: f32 = 0.00001;
const KNOT_RADIUS: f32 = 10.0;
const ROPE_THICKNESS: f32 = 7.0;

fn main() {
    let window_size = Vec2::new(1280.0, 760.0);

    let options = eframe::NativeOptions {
        initial_window_size: Some(window_size),
        ..eframe::NativeOptions::default()
    };

    let mut my_app = MyApp::new();

    let center = Vec2::new(window_size.x / 2.0, window_size.y / 2.0);
    my_app.add_knot(center);
    my_app.add_knot(center + Vec2::new(0.0, 50.0));
    my_app.add_knot(center + Vec2::new(0.0, 100.0));
    my_app.add_knot(center + Vec2::new(0.0, 150.0));
    my_app.add_knot(center + Vec2::new(0.0, 200.0));
    my_app.add_knot(center + Vec2::new(0.0, 250.0));
    my_app.add_knot(center + Vec2::new(0.0, 300.0));

    let num_knots = my_app.knots.len();
    
    my_app.knots[0].add_neighbour(1);
    for i in 1..(num_knots-1) {
        my_app.knots[i].add_neighbour(i-1);
        my_app.knots[i].add_neighbour(i+1);
    }
    my_app.knots[num_knots-1].add_neighbour(num_knots-2);

    eframe::run_native(
        "Net Simulator",
        options,
        Box::new(|_cc| Box::new(my_app)),
    );
}

#[derive(Debug)]
struct Knot {
    position: Vec2,
    neighbours: Vec<usize>,
}

impl Knot {
    fn add_neighbour(&mut self, other: usize) {
        self.neighbours.push(other);
    }
}

impl From<Vec2> for Knot {
    fn from(value: Vec2) -> Self {
        Knot {
            position: value,
            neighbours: vec![]
        }
    }
}

struct MyApp {
    knots: Vec<Knot>,
    dragging: Option<usize>,
}

impl MyApp {
    fn new() -> Self {
        MyApp {
            knots: vec![],
            dragging: None,
        }
    }

    fn add_knot(&mut self, other: Vec2) {
        self.knots.push(Knot::try_from(other).unwrap());
    }

    fn calculate_velocity_for(&self, index: usize) -> Vec2 {
        let mut velocity = Vec2::new(0.0, 0.0);

        for neighbour in &self.knots[index].neighbours {
            let knot_position = self.knots[index].position;
            let neighbour_position = self.knots[*neighbour].position;

            let difference = knot_position - neighbour_position;
            let direction = if difference.length() > EPSILON { difference.normalized() } else { Vec2::new(0.0, 0.0) };

            let target = neighbour_position + direction * TARGET_DISTANCE;

            velocity += target - knot_position;
        }

        velocity
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            let mut mouse = Pos2 { x: 0.0, y: 0.0 };

            if let Some(mouse_position) = { ctx.input().pointer.hover_pos() } {
                mouse = mouse_position;
                if ctx.input().pointer.any_pressed() {
                    for i in 0..self.knots.len() {
                        let distance = (self.knots[i].position - mouse_position.to_vec2()).length();
                        if distance <= KNOT_RADIUS {
                            self.dragging = Some(i);
                            break;
                        }
                    }
                } else if ctx.input().pointer.any_released() {
                    self.dragging = None;
                }
            }

            for i in 0..self.knots.len() {
                let velocity = self.calculate_velocity_for(i);
                self.knots[i].position += velocity * ui.input().stable_dt;

                if let Some(dragged) = self.dragging {
                    if dragged == i {
                        self.knots[i].position = mouse.to_vec2();
                    }
                }
            }

            for i in 1..self.knots.len() {
                painter.line_segment([self.knots[i].position.to_pos2(), self.knots[i-1].position.to_pos2()], Stroke { width: ROPE_THICKNESS, color: Color32::GRAY });
            }

            for i in 0..self.knots.len() {
                painter.circle_filled(self.knots[i].position.to_pos2(), KNOT_RADIUS, Color32::from_rgb(255/(i+1) as u8, 127, (i+1) as u8));
            }
        });

        ctx.request_repaint_after(Duration::from_millis(5));
    }
}