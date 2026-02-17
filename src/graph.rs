use std::collections::VecDeque;
use eframe::emath::{pos2, Vec2};
use eframe::epaint::Color32;
use egui::{Sense, Ui};

pub struct Graph {
    points: Vec<Function>,
    max_points: usize,
}

struct Function {
    pub point_color: Color32,
    pub point_size: f32,
    pub points: VecDeque<f32>,
    reverse_direction: bool,
}


impl Graph {
    pub fn render(&self, ui: &mut Ui, width: Option<f32>, height: f32, ) {
        egui::Frame::dark_canvas(ui.style())
            .corner_radius(egui::CornerRadius::same(4))
            .show(ui, |ui| {
                let width = width.unwrap_or(ui.available_width());
                let (response, painter) = ui.allocate_painter(Vec2::new(
                    width,
                    height
                ), Sense::hover());

                let graph_start_absolute_position_y = response.rect.min.y;
                let graph_start_absolute_position_x = response.rect.min.x;

                self.points.iter().for_each(|function| {
                    function.points.iter().enumerate().for_each(|(index, &p)| {
                        let x_index = if function.reverse_direction {
                            self.max_points - 1 - index
                        } else {
                            index
                        };

                        painter.circle_filled(
                            pos2(
                                graph_start_absolute_position_x + (x_index as f32 * (width / self.max_points as f32)),
                                graph_start_absolute_position_y + p,
                            ),
                            function.point_size,
                            function.point_color,
                        );

                    });
                });
            });
    }

    pub fn add_point(&mut self, points: Vec<f32>) {
        assert_eq!(points.len(), self.points.len());
        points.iter().enumerate().for_each(|(index, point)| {
            self.points[index].points.push_back(*point);
            if self.points[index].points.len() > self.max_points {
                self.points[index].points.pop_front();
            }
        });
    }

    pub fn reverse_graph_dir(&mut self) {
        self.points.iter_mut().for_each(|f| {
           f.reverse_direction = !f.reverse_direction;
        });
    }

    pub fn add_function(&mut self, color: Color32, point_size: f32, reverse_direction: bool) {
        self.points.push(Function {
            point_color: color,
            point_size,
            points: VecDeque::with_capacity(self.max_points),
            reverse_direction,
        });
    }

    pub fn new(max_points: usize) -> Self {
        Self {
            points: Vec::new(),
            max_points
        }
    }
}