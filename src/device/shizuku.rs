use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};

use super::*;

pub const SHIZUKU_PRODUCT_IDS: [ProductID; 3] = [0xFFFF, 0xFFFE, 0x374B];

const BEGIN_DATA: u8 = 0xA5;
const END_DATA: u8 = 0x5A;
const CMD_STOP: u8 = 0x7;
const CMD_START_SAMPLING: u8 = 0x9;

pub struct ShizukuDevice {
    handle: Mutex<i32>,
    device_handle: Mutex<USBDevice>,
    stop_flag: AtomicBool,
    endpoint_in: Option<i32>,
    endpoint_out: Option<i32>,
    last_request_id: Option<i32>,
    explected_replies: Option<i32>,
}

impl ShizukuDevice {
    // pub fn init() -> ShizukuDevice {
    //     ShizukuDevice {
    //         handle: Mutex::new(42),
    //         stop_flag: false.into(),
    //         endpoint_in: None,
    //         endpoint_out: None,
    //         last_request_id: None,
    //         explected_replies : None,
    //     }
    // }
}

impl Display for ShizukuDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // For now, let's just print the underlying device:
        let guard = self.device_handle.lock();
        match guard {
            Ok(g) => g.fmt(f),
            _ => Err(std::fmt::Error),
        }
    }
}

impl DeviceInterface for ShizukuDevice {
    fn try_create(device: USBDevice) -> Option<Self>
    where
        Self: Sized,
    {
        if device.matches_product_ids(&SHIZUKU_PRODUCT_IDS) {
            Some(ShizukuDevice {
                handle: Mutex::new(42),
                device_handle: Mutex::new(device),
                stop_flag: false.into(),
                endpoint_in: None,
                endpoint_out: None,
                last_request_id: None,
                explected_replies: None,
            })
        } else {
            None
        }
    }
    fn sample(&self) -> f32 {
        let g = self.handle.lock().unwrap();
        *g as f32
    }

    fn start_sampling(&self) {
        let interval = Duration::from_millis(500);
        loop {
            // unlock the handle and modify
            {
                let mut guard = self.handle.lock().unwrap();
                *guard += 1;
            }

            if self.stop_flag.load(Ordering::SeqCst) {
                break;
            } else {
                thread::sleep(interval);
            }
        }
    }

    fn stop_sampling(&self) {
        self.stop_flag.store(false, Ordering::SeqCst);
    }

    // fn is_valid_id(id: ProductID) -> bool {
    //     SHIZUKU_PRODUCT_IDS.contains(&id)
    // }
}
