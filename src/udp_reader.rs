use std::net::UdpSocket;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use arc_swap::ArcSwap;
use crate::rbr::{RbrHeader, Telemetry};
use bincode;

#[derive(Clone)]
pub struct AppData {
    pub ip: String,
    pub port: String,
}
impl AppData {
    pub fn default() -> Self {
        Self {
            ip: "127.0.0.1".into(),
            port: "6776".into(),
        }
    }
    pub fn to_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}


const SIZE_OF_TELEMETRY_PACKET: usize = size_of::<Telemetry>();

pub fn udp_start(state: Arc<ArcSwap<RbrHeader>>, app_data_handle: Arc<ArcSwap<AppData>>) {
    let mut socket_res: Result<UdpSocket, String> = Err("".into());
    let mut buffer = [0; SIZE_OF_TELEMETRY_PACKET];

    loop {
        let app_data = app_data_handle.load();
        let mut errors: Option<String> = None;
        let mut data = RbrHeader::default();

        if socket_res.is_err() {
            errors = socket_res.err();
            socket_res = bind_retry(app_data.to_addr().as_str());
            sleep(Duration::from_millis(100));
        }

        // Attempt to receive
        if errors.is_none() {
            errors = receive_packet(socket_res.as_ref().unwrap(), &mut buffer).err();
        }


        if errors.is_none() {
            // Now we have a valid buf
            match bincode::deserialize::<Telemetry>(&buffer) {
                Ok(telemetry) => {
                    data.telemetry = telemetry;
                    data.telemetry.format();
                }
                Err(e) => {
                    errors = Some(e.to_string());
                }
            }
        }

        data.error = errors;
        state.store(Arc::new(data));
        sleep(Duration::from_millis(1));
    }
}


fn receive_packet(socket: &UdpSocket, buf: &mut [u8; SIZE_OF_TELEMETRY_PACKET]) -> Result<(), String> {
    match socket.recv(buf) {
        Ok(received_size) => {
            if received_size != SIZE_OF_TELEMETRY_PACKET {
                // discard the message
                Err("packet size not the same as the Struct size".into())
            } else {
                Ok(())
            }
        }
        Err(e) => {
            Err(e.to_string())
        }
    }
}



fn bind_retry(addr: &str) -> Result<UdpSocket, String> {
    let socket = UdpSocket::bind(addr).map_err(|e| e.to_string())?;
    
    socket.set_nonblocking(true).map_err(|e| e.to_string())?;
    
    Ok(socket)
}