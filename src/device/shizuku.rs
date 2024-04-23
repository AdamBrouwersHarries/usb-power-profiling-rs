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
    last_request_id: Option<i32>,
    explected_replies: Option<i32>,
}

impl ShizukuDevice{
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

impl Display for ShizukuDevice{
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
    fn try_create(device: Device) -> Result<Box<Self>, Device>
    where
        Self: Sized,
    {
        USBDevice::create_if_matching_id(device, &SHIZUKU_PRODUCT_IDS).and_then(|usbdevice| {
            usbdevice.query();
            Ok(Box::new(ShizukuDevice {
                handle: Mutex::new(42),
                device_handle: Mutex::new(usbdevice),
                stop_flag: false.into(),
                last_request_id: None,
                explected_replies: None,
            }))
        })
    }
    fn sample(&self) -> f32 {
        let g = self.handle.lock().unwrap();
        *g as f32
    }

    fn start_sampling(&self) {
        // Lock the device, so only we can use it.
        let mut device_handle = self.device_handle.lock().unwrap();
        // Reset the device.
        let _ = device_handle.reset().unwrap();
        let (endpoint_in, endpoint_out) = device_handle.find_bulk_in_out_end_points().unwrap();

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
