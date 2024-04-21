// use std::{
//     sync::{
//         atomic::{AtomicBool, Ordering},
//         Mutex,
//     },
//     thread,
//     time::Duration,
// };

// use super::DeviceInterface;

// pub struct DummyDevice {
//     handle: Mutex<i32>,
//     stop_flag: AtomicBool,
// }

// impl DummyDevice {
//     pub fn init() -> DummyDevice {
//         DummyDevice {
//             handle: Mutex::new(42),
//             stop_flag: false.into(),
//         }
//     }
// }

// impl DeviceInterface for DummyDevice {
//     fn sample(&self) -> f32 {
//         let g = self.handle.lock().unwrap();
//         *g as f32
//     }

//     fn start_sampling(&self) {
//         let interval = Duration::from_millis(500);
//         loop {
//             // unlock the handle and modify
//             {
//                 let mut guard = self.handle.lock().unwrap();
//                 *guard += 1;
//             }

//             if self.stop_flag.load(Ordering::SeqCst) {
//                 break;
//             } else {
//                 thread::sleep(interval);
//             }
//         }
//     }

//     fn stop_sampling(&self) {
//         self.stop_flag.store(false, Ordering::SeqCst);
//     }

//     fn is_valid_id(_id: super::ProductID) -> bool {
//         false
//     }
// }
