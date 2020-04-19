use crate::{
    gui::{
        core::{Background, Color, Rectangle},
        renderables::{Renderable, Text},
    },
};
use nalgebra_glm::Vec2;
use std::any::Any;

#[derive(Clone)]
pub struct Graph {
    average_frame_time: f32,
    pub frame_times: Vec<f32>,
    width: f32,
    height: f32,
    position: Vec2,
    quads: Vec<Renderable>,
}

impl Graph {
    pub fn new(position: Vec2, width: f32, height: f32, average_frame_time: f32, frame_times: Vec<f32>) -> Self {
        Self {
            average_frame_time,
            frame_times,
            width,
            height,
            position,
            quads: Vec::new(),
        }
    }
}

impl Graph {
    fn get_quads(&mut self) -> Vec<Renderable> {
        let mut quads = Vec::new();
        self.frame_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let max: f32 = *self.frame_times.last().unwrap_or(&0.0);
        let mut x = 0.0;
        for frame_time in self.frame_times.iter() {
            let perc = frame_time / max;
            quads.push(Renderable::Quad {
                /// The bounds of the quad
                bounds: Rectangle {
                    x: x,
                    y: (self.height * perc),
                    width: 1.0,
                    height: self.height,
                },
                /// The background of the quad
                background: Background::Color(Color::from_rgb(0.0, 1.0, 0.0)),
                /// The border radius of the quad
                border_radius: 0,
                /// The border width of the quad
                border_width: 0,
                /// The border color of the quad
                border_color: Color::default(),
            });
            x += 1.0;
        }
        quads
    }
}

impl crate::gui::components::Component for Graph {
    fn update(&mut self, _delta_time: f32) {
        self.quads = self.get_quads();
    }

    fn draw(&self, parent_bounds: Rectangle) -> Renderable {
        let bounds = Rectangle {
            x: parent_bounds.x + self.position.x,
            y: parent_bounds.x + self.position.y,
            width: self.width,
            height: self.height,
        };
        
        Renderable::Group {
            bounds,
            renderables: self.quads.clone(),
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
