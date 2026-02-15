mod math;
mod ply_parser;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::rc::Rc;
use std::time::Instant;
use crate::math::algebra_r3::*;
use crate::math::raster_lib::math_lib::Shape;
use eframe::{egui, Frame};
use egui::Context;
use egui::emath::OrderedFloat;
use crate::math::math_lib::Light;
use crate::ply_parser::PlyObject;

struct App {
    virtual_space_plot: VirtualSpacePlot,
    last_frame: Instant,

    focal_length: Rc<RefCell<f32>>,
    zoom: Rc<RefCell<f32>>,
    time: Rc<RefCell<f32>>,
    delta_time: Rc<RefCell<f32>>,

    light: Rc<RefCell<Light>>,
    ambient_light: Rc<RefCell<f32>>,

}

impl App {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let time = Rc::new(RefCell::new(0.));
        let focal_length = Rc::new(RefCell::new(1000.));
        let zoom = Rc::new(RefCell::new(0.8));
        let delta_time = Rc::new(RefCell::new(0.));
        let light = Rc::new(RefCell::new(Light::new(Vector::new(0., 0., 200.), 0.5)));
        let ambient_light = Rc::new(RefCell::new(0.1));

        Self {
            virtual_space_plot: VirtualSpacePlot::new(
                cc,
                focal_length.clone(),
                zoom.clone(),
                time.clone(),
                delta_time.clone(),
                light.clone(),
                ambient_light.clone(),
            ),
            last_frame: Instant::now(),
            focal_length,
            zoom,
            time,
            delta_time,
            light,
            ambient_light,
        }
    }
    pub fn update_state(&mut self, delta_time: f32) {
        *self.time.borrow_mut() += delta_time;
        *self.delta_time.borrow_mut() = delta_time;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.request_repaint();
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.update_state(dt);
        self.virtual_space_plot.update(ctx, frame);


        egui::SidePanel::right("settings")
            .default_width(320.)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Settings");
                    ui.add(egui::Slider::new(&mut *self.zoom.borrow_mut(), 0.05..=2.5).text("Zoom"));
                    ui.add(egui::Slider::new(&mut (*self.light.borrow_mut()).intensity, 0.1..=1.0).text("Light Intensity"));
                    ui.add(egui::Slider::new(&mut *self.ambient_light.borrow_mut(), 0.1..=0.3).text("Ambient Light"));
                    
                    ctx.input(|i| {
                        if i.smooth_scroll_delta.y != 0.0 {
                            let zoom = *self.zoom.borrow();
                            *self.zoom.borrow_mut() = (zoom * (1.0 + i.smooth_scroll_delta.y * 0.005)).clamp(0.05, 2.5);
                        }
                    });


                    egui::Frame::new()
                        .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                        .corner_radius(egui::CornerRadius::same(4))
                        .inner_margin(egui::Margin::same(4))
                        .show(ui, |ui| {
                            ui.set_min_height(120.0);

                            ui.scope(|ui| {
                                ui.style_mut().text_styles.insert(
                                    egui::TextStyle::Body,
                                    egui::FontId::proportional(18.0),
                                );

                                ui.heading("Shape Creator");

                            });


                        });

                });
            });
    }
}


struct VirtualSpacePlot {
    shapes: Vec<Box<dyn Shape>>,

    light: Rc<RefCell<Light>>,
    ambient_light: Rc<RefCell<f32>>,
    focal_length: Rc<RefCell<f32>>,
    zoom: Rc<RefCell<f32>>,
    time: Rc<RefCell<f32>>,
    delta_time: Rc<RefCell<f32>>,
}

impl VirtualSpacePlot {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>,
                   focal_length: Rc<RefCell<f32>>,
                   zoom: Rc<RefCell<f32>>,
                   time: Rc<RefCell<f32>>,
                   delta_time: Rc<RefCell<f32>>,
                   light: Rc<RefCell<Light>>,
                   ambient_light: Rc<RefCell<f32>>,
    ) -> Self {
        Self {
            shapes: vec![
                Box::new(PlyObject::new("src/apple.ply".to_owned())),
            ],
            light,
            ambient_light,
            focal_length,
            zoom,
            time,
            delta_time
        }
    }
    pub fn update_state(&mut self, delta_time: f32) {
        let speed = PI / 2.;
        let t = *self.time.borrow();


        let transformations = vec![
            Transform {
                translation: Vector::new(0., 0., 0.),
                rotation: Quaternion::new_rotation(speed * t, Vector::new(0., 0., 1.)),
                scale: Vector::new(1., 1., 1.)
            }
        ];
        self.shapes
            .iter_mut()
            .zip(transformations.into_iter())
            .for_each(|(shape, transform)| {
                shape.set_transform(transform)
            });
    }
}

impl eframe::App for VirtualSpacePlot {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let dt = *self.delta_time.borrow();

        self.update_state(dt);


        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let viewport_rect = ui.max_rect();
                let _response = ui.allocate_rect(viewport_rect, egui::Sense::click_and_drag());

                ui.painter().rect_filled(
                    _response.rect,
                    0.0,
                    egui::Color32::BLACK,
                );

                let x_offset = _response.rect.width() / 2.;
                let y_offset = _response.rect.height() / 2.;
                let focal_length = *self.focal_length.borrow();
                let zoom = *self.zoom.borrow();

                let mut map: BTreeMap<OrderedFloat<f32>, Vec<egui::Mesh>> = BTreeMap::new();

                self.shapes.iter().for_each(|shape| {
                    shape.produce_mesh()
                        .into_iter()
                        .filter(|tri_face| tri_face.check_face_culling(focal_length))
                        .for_each(|mut tri_face| {
                            tri_face.calculate_colors(&*self.light.borrow(), *self.ambient_light.borrow_mut());
                            let (mesh, depth) = tri_face.project_to_screen_space(focal_length, x_offset, y_offset, zoom);

                            map.entry(OrderedFloat::from(depth)).or_default().push(mesh);
                        })
                });

                map.into_iter().for_each(|(_, meshes)| {
                    meshes.into_iter().for_each(|mesh| {
                        let _ = ui.painter().add(egui::Shape::mesh(mesh));
                    });
                });


            });
    }
}



fn main() {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Projections",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc))))
    ).unwrap();
}