use std::time::Duration;

use eframe::egui;
use egui::{Color32, Pos2, Stroke, Vec2, Painter};

const TARGET_DISTANCE: f32 = 10.0;
const EPSILON: f32 = 0.00001;
const NUM_KNOTS: usize = 64;
const KNOT_RADIUS: f32 = 3.0;
const ROPE_THICKNESS: f32 = 6.0;

fn main() {
    let window_size = Vec2::new(1280.0, 760.0);

    let options = eframe::NativeOptions {
        initial_window_size: Some(window_size),
        ..eframe::NativeOptions::default()
    };

    let mut netsim = MyApp::new();

    let center = Vec2::new(window_size.x / 2.0, window_size.y / 2.0);

    for _ in 0..NUM_KNOTS {
        netsim.add_knot(Knot { position: center });
    }

    netsim.add_collider(Collidable { position: center, radius: 20.0 });
    netsim.add_collider(Collidable { position: center + Vec2::new(200.0, 0.0), radius: 20.0 });
    netsim.add_collider(Collidable { position: center + Vec2::new(-200.0, 0.0), radius: 20.0 });
    netsim.add_collider(Collidable { position: center + Vec2::new(0.0, 200.0), radius: 20.0 });

    eframe::run_native("Net Simulator", options, Box::new(|_cc| Box::new(netsim)));
}

#[derive(Clone, Copy)]
struct Knot {
    position: Vec2,
}

struct Collidable {
    position: Vec2,
    radius: f32,
}

struct MyApp {
    knots: Vec<Knot>,
    colliders: Vec<Collidable>,
}

impl MyApp {
    fn new() -> Self {
        MyApp {
            knots: vec![],
            colliders: vec![],
        }
    }

    fn add_knot(&mut self, knot: Knot) {
        self.knots.push(knot);
    }

    fn add_collider(&mut self, collider: Collidable) {
        self.colliders.push(collider);
    }

    fn satisfy_constraints(&mut self) {
        for i in 1..self.knots.len() {
            let mut current_knot = self.knots[i];
            let prev_knot = self.knots[i-1];

            let difference = prev_knot.position - current_knot.position;
            let current_distance = difference.length();

            if (TARGET_DISTANCE - current_distance).abs() > EPSILON {
                let adjustment = difference.normalized() * (TARGET_DISTANCE - current_distance);

                current_knot.position -= adjustment;

                self.knots[i] = current_knot;
            }
        }

        for knot in self.knots.iter_mut() {
            for collider in self.colliders.iter() {
                let difference = knot.position - collider.position;
                let distance = difference.length();
                if distance < collider.radius {
                    knot.position = collider.position + difference.normalized() * collider.radius;
                }
            }
        }
    }

    fn paint_colliders(&self, painter: &Painter) {
        for collider in &self.colliders {
            painter.circle_filled(collider.position.to_pos2(), collider.radius, Color32::RED);
        }
    }

    fn paint_knots(&self, painter: &Painter) {
        for knot in &self.knots {
            painter.circle_filled(knot.position.to_pos2(),
            KNOT_RADIUS,
            Color32::WHITE,
            );
        }
    }

    fn paint_segments(&self, painter: &Painter) {
        for i in 0..self.knots.len()-1 {
            painter.line_segment(
                [
                    self.knots[i].position.to_pos2(),
                    self.knots[i+1].position.to_pos2(),
                ],
                Stroke {
                    width: ROPE_THICKNESS,
                    color: Color32::WHITE,
                },
            );
        }
    }

    fn paint_rope(&self, painter: &Painter) {
        self.paint_segments(painter);
        self.paint_knots(painter);
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            let mouse = ctx.input().pointer.hover_pos().unwrap_or(Pos2::default());

            self.knots[0].position = mouse.to_vec2();

            self.satisfy_constraints();

            self.knots[0].position = mouse.to_vec2();

            self.paint_colliders(painter);
            self.paint_rope(painter);
        });

        ctx.request_repaint_after(Duration::from_millis(5));
    }
}
