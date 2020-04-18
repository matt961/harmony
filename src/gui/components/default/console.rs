use crate::{
    core::input::Input,
    gui::{
        animation::EasingFunctions,
        components::{AnimationBuilder, Component, Log, PaddingBuilder, WindowBuilder},
        core::{Color, Rectangle},
        renderables::{Renderable, Text},
    },
    AssetManager,
};
use glyph_brush::GlyphCruncher;
use nalgebra_glm::{Vec2, Vec4};
use std::cell::{RefCell, RefMut};
use circular_queue::CircularQueue;

#[derive(Clone)]
pub enum ModuleType {
    Asset,
    GUI,
    Renderer,
}

pub struct Console {
    components: Vec<Box<dyn Component>>,
    open: bool,
    log: Log,
    lines: Vec<(ModuleType, String)>,
    measure_brush: Option<RefCell<glyph_brush::GlyphBrush<'static, ()>>>,
    frame_times: CircularQueue<f32>,
}

impl Console {
    pub fn new() -> Self {
        let log = Log::new();
        Self {
            components: Vec::new(),
            open: false,
            log,
            lines: Vec::new(),
            measure_brush: None,
            frame_times: CircularQueue::with_capacity(100),
        }
    }

    pub(crate) fn add_frame_time(&mut self, frame_time: f32) {
        self.frame_times.push(frame_time);
    }

    fn calculate_average_frame_time(frame_times: &CircularQueue<f32>) -> f32 {
        frame_times.iter().sum::<f32>() / frame_times.len() as f32
    }

    fn build_graph(width: f32, height: f32, frame_times: &CircularQueue<f32>) -> crate::gui::components::Window {
        let mut padding = PaddingBuilder::new(Vec4::new(10.0, 10.0, 10.0, 10.0));
        padding
            .with_child(crate::gui::components::Text {
                text: format!("Average frame time: {}", Self::calculate_average_frame_time(frame_times)),
                size: 18.0,
                position: Vec2::new(0.0, 0.0),
                color: Color::from_rgb8(255, 140, 0),
                font: "fantasque.ttf".to_string(),
            });
        let mut window_builder = WindowBuilder::new();
        window_builder
            .set_border(Some(1), Some(Color::from_rgb(0.8, 0.8, 0.8)), None)
            .set_size(Vec2::new(width, height))
            .hide_title_bar()
            .set_content(
                padding.build()
            );

        window_builder.build()
    }

    pub fn load(&mut self, asset_manager: &AssetManager) {
        let font = asset_manager.get_font("fantasque.ttf".to_string());
        let measure_brush = RefCell::new(
            glyph_brush::GlyphBrushBuilder::using_font_bytes(font.data.clone()).build(),
        );
        self.measure_brush = Some(measure_brush);
    }

    fn get_module_info(module: ModuleType) -> (String, Color) {
        match module {
            ModuleType::Asset => ("[Asset]".to_string(), Color::from_rgb(1.0, 0.529, 0.149)),
            ModuleType::GUI => ("[GUI]".to_string(), Color::from_rgb(0.313, 0.998, 0.705)),
            _ => ("[None]".to_string(), Color::from_rgb(1.0, 1.0, 1.0)),
        }
    }

    pub fn info<T>(&mut self, _module: ModuleType, _message: T)
    where
        T: Into<String>,
    {
        // Think of where to put this.
        //self.lines.push((module, message.into()));
    }

    fn build_line(
        data: (ModuleType, String),
        measure_brush: &mut RefMut<'_, glyph_brush::GlyphBrush<'static, ()>>,
    ) -> Renderable {
        let (module, message) = data;
        let (module_name, color) = Self::get_module_info(module);

        let section = wgpu_glyph::Section {
            text: &module_name.clone(),
            scale: wgpu_glyph::Scale { x: 18.0, y: 18.0 },
            bounds: (100000.0, 100000.0),
            ..Default::default()
        };

        let text_bounds = measure_brush.glyph_bounds(section).unwrap();

        let start_text = Renderable::Text(Text {
            bounds: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 1000.0,
                height: 20.0,
            },
            size: 18.0,
            text: module_name,
            font: "fantasque.ttf".to_string(),
            color: color,
        });
        let end_text = Renderable::Text(Text {
            bounds: Rectangle {
                x: text_bounds.width().ceil() + 5.0,
                y: 0.0,
                width: 1000.0,
                height: 20.0,
            },
            size: 18.0,
            text: message.clone(),
            font: "fantasque.ttf".to_string(),
            color: Color::from_rgb(1.0, 1.0, 1.0),
        });

        Renderable::Group {
            bounds: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 20.0,
            },
            renderables: vec![start_text, end_text],
        }
    }

    pub fn update(&mut self, input: &Input, delta_time: f32) {
        if input.is_key_pressed(winit::event::VirtualKeyCode::Grave) {
            self.components.clear();
            let mut measure_brush = self.measure_brush.as_mut().unwrap().borrow_mut();
            self.log.clear();
            for line in self.lines.iter() {
                let renderable =
                    Self::build_line((line.0.clone(), line.1.clone()), &mut measure_brush);
                self.log.add_line(renderable);
            }

            let mut padding = PaddingBuilder::new(Vec4::new(15.0, 15.0, 20.0, 20.0));
            {
                let graph = Self::build_graph(200.0, 200.0, &self.frame_times);
                padding.with_child(graph);
            }

            let window_darkness = 0.2;
            let mut window1 = WindowBuilder::new();
            window1
                .set_background(crate::gui::core::Background::from(Color::from_rgba(
                    window_darkness,
                    window_darkness,
                    window_darkness,
                    0.98,
                )))
                .set_size(Vec2::new(1024.0, 256.0))
                .set_margin(0.0, 0.0, 0.0, 0.0)
                .set_title("Console")
                .set_content(padding.build());
            self.open = !self.open;
            if self.open {
                let mut builder = AnimationBuilder::new();
                builder
                    .set_easing_function(EasingFunctions::easeInOutQuad)
                    .with_child(window1.build())
                    .with_position(Vec2::new(0.0, -256.0))
                    .with_destination(Vec2::new(0.0, 0.0))
                    .with_duration(5.0);
                let mut animation = builder.build();
                animation.start(0.25, Vec2::new(0.0, 0.0));
                self.components.push(Box::new(animation));
            } else {
                let mut builder = AnimationBuilder::new();
                builder
                    .set_easing_function(EasingFunctions::easeInOutQuad)
                    .with_child(window1.build())
                    .with_position(Vec2::new(0.0, 0.0))
                    .with_destination(Vec2::new(0.0, -256.0))
                    .with_duration(5.0);
                let mut animation = builder.build();
                animation.start(0.25, Vec2::new(0.0, -256.0));
                self.components.push(Box::new(animation));
            }
        }
        for component in self.components.iter_mut() {
            component.update(delta_time);
        }
    }
}

// This let's the gui system pull the components out of your scene.
impl crate::gui::Scene for Console {
    fn get_components(&self) -> &Vec<Box<dyn Component>> {
        &self.components
    }
}
