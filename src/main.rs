use std::time::Duration;

use eframe::egui;
use egui::{Color32, Pos2, Stroke, Vec2};

const TENSION: f32 = 500.0;
const TARGET_DISTANCE: f32 = 30.0;
const TARGET_LEEWAY: f32 = 0.8;
const EPSILON: f32 = 0.00001;
const KNOT_RADIUS: f32 = 10.0;
const ROPE_THICKNESS: f32 = 7.0;
const NUM_POSITION_PASSES: usize = 20;

fn main() {
    let window_size = Vec2::new(1280.0, 760.0);

    let options = eframe::NativeOptions {
        initial_window_size: Some(window_size),
        ..eframe::NativeOptions::default()
    };

    let mut netsim = MyApp::new();

    let center = Vec2::new(window_size.x / 2.0, window_size.y / 2.0);
    netsim.add_knot(center + Vec2::new(0.0, 0.0));
    netsim.add_knot(center + Vec2::new(0.0, 30.0));
    netsim.add_knot(center + Vec2::new(0.0, 60.0));
    netsim.add_knot(center + Vec2::new(0.0, 90.0));
    netsim.add_knot(center + Vec2::new(0.0, 120.0));
    netsim.add_knot(center + Vec2::new(0.0, 150.0));
    netsim.add_knot(center + Vec2::new(0.0, 180.0));
    netsim.add_knot(center + Vec2::new(0.0, 210.0));
    netsim.add_knot(center + Vec2::new(0.0, 240.0));
    netsim.add_knot(center + Vec2::new(0.0, 270.0));
    netsim.add_knot(center + Vec2::new(0.0, 300.0));
    netsim.add_knot(center + Vec2::new(0.0, 330.0));

    /*netsim.add_knot(center + Vec2::new(30.0, 0.0));
    netsim.add_knot(center + Vec2::new(30.0, 30.0));
    netsim.add_knot(center + Vec2::new(30.0, 60.0));
    netsim.add_knot(center + Vec2::new(30.0, 90.0));
    netsim.add_knot(center + Vec2::new(30.0, 120.0));
    netsim.add_knot(center + Vec2::new(30.0, 150.0));
    netsim.add_knot(center + Vec2::new(30.0, 180.0));
    netsim.add_knot(center + Vec2::new(30.0, 210.0));
    netsim.add_knot(center + Vec2::new(30.0, 240.0));
    netsim.add_knot(center + Vec2::new(30.0, 270.0));
    netsim.add_knot(center + Vec2::new(30.0, 300.0));
    netsim.add_knot(center + Vec2::new(30.0, 330.0));

    netsim.add_knot(center + Vec2::new(60.0, 0.0));
    netsim.add_knot(center + Vec2::new(60.0, 30.0));
    netsim.add_knot(center + Vec2::new(60.0, 60.0));
    netsim.add_knot(center + Vec2::new(60.0, 90.0));
    netsim.add_knot(center + Vec2::new(60.0, 120.0));
    netsim.add_knot(center + Vec2::new(60.0, 150.0));
    netsim.add_knot(center + Vec2::new(60.0, 180.0));
    netsim.add_knot(center + Vec2::new(60.0, 210.0));
    netsim.add_knot(center + Vec2::new(60.0, 240.0));
    netsim.add_knot(center + Vec2::new(60.0, 270.0));
    netsim.add_knot(center + Vec2::new(60.0, 300.0));
    netsim.add_knot(center + Vec2::new(60.0, 330.0));*/

    let num_knots = netsim.knots.len();
    let num_columns = 1;
    let column_length = num_knots / num_columns;

    for c in 0..num_columns {
        let first_knot = c * column_length;
        netsim.knots[first_knot].add_neighbour(first_knot + 1);
        for i in 1..column_length - 1 {
            let this_knot = c * column_length + i;
            netsim.knots[this_knot].add_neighbour(this_knot - 1);
            netsim.knots[this_knot].add_neighbour(this_knot + 1);
        }
        let last_knot = (c + 1) * column_length - 1;
        netsim.knots[last_knot].add_neighbour(last_knot - 1);
    }

    for c in 0..num_columns - 1 {
        for i in (0..column_length).step_by(4) {
            let this_knot = c * column_length + i;
            netsim.knots[this_knot].add_neighbour(this_knot + column_length);
        }
    }

    for c in 1..num_columns {
        for i in (0..column_length).step_by(4) {
            let this_knot = c * column_length + i;
            netsim.knots[this_knot].add_neighbour(this_knot - column_length);
        }
    }

    eframe::run_native("Net Simulator", options, Box::new(|_cc| Box::new(netsim)));
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
            neighbours: vec![],
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
            let direction = if difference.length() > EPSILON {
                difference.normalized()
            } else {
                Vec2::new(0.0, 0.0)
            };

            let target = neighbour_position + direction * TARGET_DISTANCE;

            if (target - knot_position).length() > TARGET_LEEWAY {
                velocity += target - knot_position;
            }
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

            for _ in 0..NUM_POSITION_PASSES {
                for i in 0..self.knots.len() {
                    let velocity = self.calculate_velocity_for(i);
                    let dt_per_pass = ui.input().stable_dt.min(0.1) / NUM_POSITION_PASSES as f32;
                    self.knots[i].position += velocity * TENSION * dt_per_pass;

                    if let Some(dragged) = self.dragging {
                        if dragged == i {
                            self.knots[i].position = mouse.to_vec2();
                        }
                    }
                }
            }

            for i in 1..self.knots.len() {
                for n in &self.knots[i].neighbours {
                    painter.line_segment(
                        [
                            self.knots[i].position.to_pos2(),
                            self.knots[*n].position.to_pos2(),
                        ],
                        Stroke {
                            width: ROPE_THICKNESS,
                            color: Color32::GRAY,
                        },
                    );
                }
            }

            for i in 0..self.knots.len() {
                painter.circle_filled(
                    self.knots[i].position.to_pos2(),
                    KNOT_RADIUS,
                    Color32::from_rgb(255 / (i + 1) as u8, 127, (i + 1) as u8),
                );
            }
        });

        ctx.request_repaint_after(Duration::from_millis(5));
    }
}
