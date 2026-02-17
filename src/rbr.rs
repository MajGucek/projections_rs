#![allow(unused)]
use serde::Deserialize;

// Telemetry data is ported from the NGP rbr.telemetry.data.TelemetryData.h header
// that is part of the NGP plugin.

#[derive(Deserialize, Default, Copy, Clone)]
pub struct TireSegment {
    pub temperature: f32,
    pub wear: f32,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Tire {
    pub pressure: f32,
    pub temperature: f32,
    pub carcass_temperature: f32,
    pub tread_temperature: f32,
    pub current_segment: u32,
    pub segment1: TireSegment,
    pub segment2: TireSegment,
    pub segment3: TireSegment,
    pub segment4: TireSegment,
    pub segment5: TireSegment,
    pub segment6: TireSegment,
    pub segment7: TireSegment,
    pub segment8: TireSegment,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct BrakeDisk {
    pub layer_temperature: f32,
    pub temperature: f32,
    pub wear: f32,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Wheel {
    pub brake_disk: BrakeDisk,
    pub tire: Tire,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Damper {
    pub damage: f32,
    pub piston_velocity: f32,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Suspension {
    pub spring_deflection: f32,
    pub rollbar_force: f32, // [N], -1.52779
    pub spring_force: f32, // [N], 3684.53
    pub damper_force: f32, // [N], 0.00844441
    pub strut_force: f32, // [N], -3682.99
    pub helper_spring_is_active: i32, // [""], bool?
    pub damper: Damper,
    pub wheel: Wheel,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Engine {
    pub rpm: f32,
    pub radiator_coolant_temperature: f32,
    pub engine_coolant_temperature: f32,
    pub engine_temperature: f32,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Motion {
    pub surge: f32,
    pub sway: f32,
    pub heave: f32,
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Car {
    pub index: i32,
    pub speed: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocities: Motion,
    pub accelerations: Motion,
    pub engine: Engine,
    pub suspension_lf: Suspension,
    pub suspension_rf: Suspension,
    pub suspension_lb: Suspension,
    pub suspension_rb: Suspension,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Control {
    pub steering: f32,
    pub throttle: f32,
    pub brake: f32,
    pub handbrake: f32,
    pub clutch: f32,
    pub gear: i32,
    pub footbrake_pressure: f32,
    pub handbrake_pressure: f32,
}

#[derive(Deserialize, Default, Copy, Clone)]
pub struct Stage {
    pub index: i32,
    pub progress: f32, // meters
    pub race_time: f32, // seconds
    pub drive_line_location: f32, // no units?
    pub distance_to_end: f32, // meters
}

#[repr(C)]
#[derive(Deserialize, Default, Copy, Clone)]
pub struct Telemetry {
    pub total_steps: u32, // meters
    pub stage: Stage,
    pub control: Control,
    pub car: Car,
}

pub struct RbrHeader {
    pub telemetry: Telemetry,
    pub error: Option<String>,
    pub udp_port_number: String,
}
impl RbrHeader {
    pub fn default() -> Self {
        Self {
            telemetry: Telemetry::default(),
            error: Some("No data present!".into()),
            udp_port_number: "".into(),
        }
    }
}



const KELVIN_TO_C: f32 = 273.15;

impl Telemetry {
    pub fn default() -> Self {
        Self {
            total_steps: 0,
            stage: Default::default(),
            control: Default::default(),
            car: Default::default(),
        }
    }

    pub fn format(&mut self) {
        //self.stage.race_time += 0.7;
        self.control.brake *= 100.0;
        self.control.throttle *= 100.0;
        self.control.clutch *= 100.0;
        self.control.gear -= 1;
        self.car.suspension_lf.wheel.brake_disk.temperature -= KELVIN_TO_C;
        self.car.suspension_rf.wheel.brake_disk.temperature -= KELVIN_TO_C;
        self.car.suspension_lb.wheel.brake_disk.temperature -= KELVIN_TO_C;
        self.car.suspension_rb.wheel.brake_disk.temperature -= KELVIN_TO_C;
        self.car.suspension_lf.wheel.tire.temperature -= KELVIN_TO_C;
        self.car.suspension_rf.wheel.tire.temperature -= KELVIN_TO_C;
        self.car.suspension_lb.wheel.tire.temperature -= KELVIN_TO_C;
        self.car.suspension_rb.wheel.tire.temperature -= KELVIN_TO_C;
    }

    pub fn get_time(&self) -> Time {
        // https://www.inchcalculator.com/seconds-to-time-calculator/
        let race_time: &f32 = &self.stage.race_time;
        let mut time: Time = Default::default();
        let hr = race_time / 3600.0;
        time.hours = hr.floor();
        let min = hr.fract() * 60.0;
        time.minutes = min.floor();
        time.seconds = f32::trunc((min.fract() * 60.0) * 100.0) / 100.0;
        time
    }
}


pub struct Time {
    pub seconds: f32,
    pub minutes: f32,
    pub hours: f32,
}
impl Default for Time {
    fn default() -> Self {
        Time {
            seconds: 0.0,
            minutes: 0.0,
            hours: 0.0,
        }
    }
}