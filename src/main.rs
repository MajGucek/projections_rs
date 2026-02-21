mod math;
mod ply_parser;
mod rbr;
mod udp_reader;
mod graph;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use arc_swap::{ArcSwap, ArcSwapAny};
use crate::math::algebra_r3::*;
use crate::math::raster_lib::math_lib::Shape;
use eframe::{egui, emath, Frame};
use eframe::emath::{Pos2};
use eframe::epaint::{Color32, StrokeKind};
use eframe::HardwareAcceleration::Preferred;
use egui::{Context, CornerRadius, Mesh, Rangef, Sense, Stroke, Vec2};
use egui::emath::OrderedFloat;
use egui_gauge::Gauge;
use crate::graph::Graph;
use crate::math::math_lib::{Light, ProjectedTriangle};
use crate::ply_parser::PlyObject;
use crate::rbr::{RbrHeader, Time};
use crate::udp_reader::{udp_start, AppData};

struct App {
    rbr_data_handle: Arc<ArcSwap<RbrHeader>>,
    app_data_handle: Arc<ArcSwap<AppData>>,
    app_data: AppData,

    virtual_space_plot: VirtualSpacePlot,
    pedal_graph: Graph,
    reverse_pedal_graph_direction: bool,

    last_frame: Instant,

    #[allow(unused)]
    // This doesn't get read anywhere in App, but it does get read in VirtualSpacePlot!
    focal_length: Rc<RefCell<f32>>,
    zoom: Rc<RefCell<f32>>,
    time: Rc<RefCell<f32>>,
    delta_time: Rc<RefCell<f32>>,

    light: Rc<RefCell<Light>>,
    ambient_light: Rc<RefCell<f32>>,

}

impl App {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>, rbr_data_handle: Arc<ArcSwapAny<Arc<RbrHeader>>>, app_data_handle: Arc<ArcSwapAny<Arc<AppData>>>) -> Self {
        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));

        let time = Rc::new(RefCell::new(0.));
        let focal_length = Rc::new(RefCell::new(1000.));
        let zoom = Rc::new(RefCell::new(0.8));
        let delta_time = Rc::new(RefCell::new(0.));
        
        let light = Rc::new(RefCell::new(Light::new(Vector::new(0., 0., 200.), 0.5)));
        
        let ambient_light = Rc::new(RefCell::new(0.1));

        let mut pedal_graph= Graph::new(1000);
        pedal_graph.add_function(Color32::GREEN, 1.);
        pedal_graph.add_function(Color32::RED, 1.);
        pedal_graph.add_function(Color32::ORANGE, 1.);
        pedal_graph.add_function(Color32::YELLOW, 1.);
        pedal_graph.add_function(Color32::GRAY, 1.);

        Self {
            rbr_data_handle,
            app_data_handle,
            app_data: AppData::default(),
            virtual_space_plot: VirtualSpacePlot::new(
                cc,
                focal_length.clone(),
                zoom.clone(),
                time.clone(),
                delta_time.clone(),
                light.clone(),
                ambient_light.clone(),
            ),
            pedal_graph,
            reverse_pedal_graph_direction: false,
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

        let rbr_header = self.rbr_data_handle.load();


        egui::SidePanel::right("settings")
            .default_width(400.)
            .width_range(Rangef::new(300., 1000.))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Control");

                    ui.add(egui::Slider::new(&mut *self.zoom.borrow_mut(), 0.05..=2.5).text("Zoom"));
                    ui.add(egui::Slider::new(&mut (*self.light.borrow_mut()).intensity, 0.1..=1.0).text("Light Intensity"));
                    ui.add(egui::Slider::new(&mut *self.ambient_light.borrow_mut(), 0.1..=0.3).text("Ambient Light"));
                    ctx.input(|i| {
                        if i.smooth_scroll_delta.y != 0.0 {
                            let zoom = *self.zoom.borrow();
                            *self.zoom.borrow_mut() = (zoom * (1.0 + i.smooth_scroll_delta.y * 0.005)).clamp(0.05, 2.5);
                        }
                    });
                    ui.add_space(20.);
                    ui.heading("UDP Port").on_hover_text("Check inside RSF Launcher under Advanced -> Telemetry. Make sure that checkmark is ticked!");

                    if ui.text_edit_singleline(&mut self.app_data.port).changed() {
                        self.app_data_handle.store(Arc::new(self.app_data.clone()));
                    }

                    let mut is_valid = false;
                    if rbr_header.error.is_none() {
                        ui.colored_label(Color32::LIGHT_GREEN, "Connected to UDP port and reading!");
                        is_valid = true;
                    } else if !rbr_header.error.clone().unwrap().contains("10035") {
                        ui.colored_label(Color32::RED, rbr_header.error.clone().unwrap());
                        is_valid = false;
                    }

                    let rbr_time = if is_valid {
                        rbr_header.telemetry.get_time()
                    } else {
                        Time::default()
                    };
                    ui.add_space(10.);
                    ui.heading("Time");
                    ui.heading(format!("{:?} : {:?} : {:?}", rbr_time.hours, rbr_time.minutes, rbr_time.seconds));

                    ui.add_space(10.);
                    ui.heading("Pedals");
                    if ui.checkbox(&mut self.reverse_pedal_graph_direction, "Reverse Direction").changed() {
                        self.pedal_graph.reverse_graph_dir();
                    };
                    if is_valid {
                        self.pedal_graph.add_point(vec![
                            rbr_header.telemetry.control.throttle,
                            rbr_header.telemetry.control.brake,
                            rbr_header.telemetry.control.clutch,
                            rbr_header.telemetry.control.handbrake,
                            rbr_header.telemetry.control.gear as f32,
                        ]);
                    }
                    self.pedal_graph.render(ui, None, 120.);


                    ui.add_space(10.);
                    ui.heading("Steering");
                    egui::Frame::dark_canvas(ui.style())
                        .show(ui, |ui| {
                            ui.set_height(200.);
                            ui.set_width(ui.available_width());

                            let mut angle: f32 = 0.;
                            if is_valid {
                                angle = rbr_header.telemetry.control.steering;
                            }

                            let (_resp, painter) = ui.allocate_painter(Vec2::new(
                                ui.available_width(),
                                200.
                            ), Sense::hover());
                            let mut center_pos = _resp.rect.center();
                            center_pos.y -= 25.;

                            let radius = 50.;
                            let stroke_thickness = 10.;
                            painter.circle_stroke(
                                center_pos,
                                radius,
                                Stroke::new(stroke_thickness, Color32::GRAY)
                            );
                            center_pos.x += (radius + stroke_thickness / 2.) * angle.sin();
                            center_pos.y += (radius + stroke_thickness / 2.) * -angle.cos();
                            painter.circle_filled(
                                center_pos,
                                stroke_thickness,
                                Color32::YELLOW,
                            );


                            let margin = 4.;
                            let hor_center = _resp.rect.center().x;
                            let rect_width = _resp.rect.width() / 2. - margin;
                            let rect_height = 20.;
                            let bottom_pos = _resp.rect.max.y;
                            painter.rect_stroke(
                                emath::Rect {
                                    min: Pos2::new(hor_center - rect_width, bottom_pos - margin - rect_height),
                                    max: Pos2::new(hor_center, bottom_pos - margin),
                                },
                                CornerRadius::same(1),
                                Stroke::new(
                                    1.,
                                    Color32::GRAY,
                                ),
                                StrokeKind::Outside,
                            );
                            painter.rect_stroke(
                                emath::Rect {
                                    min: Pos2::new(hor_center, bottom_pos - margin - rect_height),
                                    max: Pos2::new(hor_center + rect_width, bottom_pos - margin),
                                },
                                CornerRadius::same(1),
                                Stroke::new(
                                    1.,
                                    Color32::GRAY,
                                ),
                                StrokeKind::Outside,
                            );


                            let max_angle = 1200.0_f32.to_radians();
                            let normalized = (angle / max_angle).clamp(-1., 1.);
                            let fill_width = rect_width * normalized.abs();
                            if normalized < 0.0 {
                                painter.rect_filled(
                                    egui::Rect::from_min_max(
                                        egui::pos2(
                                            hor_center - fill_width,
                                            bottom_pos - margin - rect_height,
                                        ),
                                        egui::pos2(
                                            hor_center,
                                            bottom_pos - margin,
                                        ),
                                    ),
                                    CornerRadius::same(1),
                                    Color32::GRAY,
                                );
                            } else {
                                painter.rect_filled(
                                    egui::Rect::from_min_max(
                                        egui::pos2(
                                            hor_center,
                                            bottom_pos - margin - rect_height,
                                        ),
                                        egui::pos2(
                                            hor_center + fill_width,
                                            bottom_pos - margin,
                                        ),
                                    ),
                                    CornerRadius::same(1),
                                    Color32::GRAY,
                                );
                            }
                        });
                    });

                /*
                ui.add(Gauge::new(
                    20.,
                    RangeInclusive::new(0., 10_000.),
                    200.,
                    egui::ecolor::Color32::GRAY,
                ));

                 */


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
    pub fn new<'a>(_cc: &'a eframe::CreationContext<'a>,
                   focal_length: Rc<RefCell<f32>>,
                   zoom: Rc<RefCell<f32>>,
                   time: Rc<RefCell<f32>>,
                   delta_time: Rc<RefCell<f32>>,
                   light: Rc<RefCell<Light>>,
                   ambient_light: Rc<RefCell<f32>>,
    ) -> Self {
        Self {
            shapes: vec![
                Box::new(PlyObject::try_load("assets/apple.ply".to_owned()).unwrap()),
            ],
            light,
            ambient_light,
            focal_length,
            zoom,
            time,
            delta_time
        }
    }
    pub fn update_state(&mut self, _delta_time: f32) {
        let speed = PI / 2.;
        let t = *self.time.borrow();


        let transformations = vec![
            Transform {
                translation: Vector::new(t.cos() * speed / 30., 0., 0.),
                rotation: Quaternion::new_rotation(speed * t, Vector::new(0., 0., 1.)),
                scale: Vector::new(1., t.sin().abs().max(0.25), 1.)
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
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let dt = *self.delta_time.borrow();

        self.update_state(dt);


        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let viewport_rect = ui.max_rect();
                let _response = ui.allocate_rect(viewport_rect, Sense::click_and_drag());

                ui.painter().rect_filled(
                    _response.rect,
                    0.0,
                    Color32::BLACK,
                );

                let x_offset = _response.rect.width() / 2.;
                let y_offset = _response.rect.height() / 2.;
                let focal_length = *self.focal_length.borrow();
                let zoom = *self.zoom.borrow();

                let mut draw_vec: Vec<ProjectedTriangle> = Vec::new();

                self.shapes.iter().for_each(|shape| {
                    shape.produce_mesh()
                        .into_iter()
                        .for_each(|mut tri_face| {
                            tri_face.calculate_colors(&*self.light.borrow(), *self.ambient_light.borrow_mut());
                            let proj_tri = tri_face.project_to_screen_space(focal_length, x_offset, y_offset, zoom);
                            draw_vec.push(proj_tri);
                        })
                });

                draw_vec.sort_unstable_by(|a, b| {
                    b.depth
                        .partial_cmp(&a.depth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });


                let mut mesh: Mesh = Mesh::default();
                for tri in draw_vec {
                    let base = mesh.vertices.len() as u32;

                    mesh.vertices.extend(tri.vertices);
                    mesh.indices.extend([base, base + 1, base + 2]);
                }

                ui.painter().add(egui::Shape::mesh(mesh));
            });
    }
}



fn main() {
    let udp = Arc::new(ArcSwap::from_pointee(RbrHeader::default()));
    let udp_producer = udp.clone();
    let ui_to_udp = Arc::new(ArcSwap::from_pointee(AppData::default()));
    let ui_to_udp_reader = ui_to_udp.clone();

    std::thread::Builder::new()
        .name("udp-reader".into())
        .spawn(move || {

            udp_start(udp_producer, ui_to_udp_reader);

        }).unwrap_or_else(|e| {
        rfd::MessageDialog::new()
            .set_title("Thread spawning Failed!")
            .set_description(format!("Couldn't spawn a thread for UDP reading!\n{:?}", e))
            .set_level(rfd::MessageLevel::Error)
            .show();
       std::process::exit(0);
    });

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        depth_buffer: 32,
        hardware_acceleration: Preferred,
        run_and_return: false,
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Projections",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, udp, ui_to_udp))))
    ).unwrap();
}