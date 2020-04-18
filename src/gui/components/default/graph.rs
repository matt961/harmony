use crate::{
    gui::{
        core::{Background, Color, Rectangle},
        renderables::{Renderable, Text},
    },
};
use nalgebra_glm::Vec2;
use std::any::Any;

pub struct Graph {
    average_frame_time: f32,
    frame_times: Vec<f32>,
    width: f32,
    height: f32,
    position: Vec2,
}

impl Graph {
    pub fn new(position: Vec2, width: f32, height: f32, average_frame_time: f32, frame_times: Vec<f32>) -> Self {
        Self {
            average_frame_time,
            frame_times,
            width,
            height,
            position,
        }
    }
}

impl crate::gui::components::Component for Graph {
    fn update(&mut self, _delta_time: f32) { }

    fn draw(&self, parent_bounds: Rectangle) -> Renderable {
        let bounds = Rectangle {
            x: parent_bounds.x + self.position.x,
            y: parent_bounds.y + self.position.y,
            width: parent_bounds.width,
            height: parent_bounds.height,
        };
        
        Renderable::Group {
            bounds,
            renderables: vec![
                Renderable::Quad {
                    bounds: Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: self.width,
                        height: self.width,
                    },
                    background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0)),
                    border_radius: 0,
                    border_width: 2,
                    border_color: Color::from_rgb(0.1, 0.1, 0.1),
                },
                Renderable::Text(Text {
                    text: format!("Average frame time: {}", self.average_frame_time),
                    size: 18.0,
                    bounds: Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: 200.0,
                        height: 200.0,
                    },
                    color: Color::from_rgb8(255, 140, 0),
                    font: "fantasque.ttf".to_string(),
                })
            ],
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
