use std::{fmt::Display, time::Duration};

use rusb::{Device, EndpointDescriptor, UsbContext};

pub type VendorID = u16;
pub type ProductID = u16;

// Wrap an rusb device in a struct to simplify common operations.
// *FOR NOW* use `rusb::Context`, to simplify the types.
pub struct USBDevice {
    device: rusb::Device<rusb::Context>,
    timeout: Duration,
}

impl USBDevice {
    pub fn from_device(device: Device<rusb::Context>) -> USBDevice {
        USBDevice {
            device,
            timeout: Duration::from_millis(1000),
        }
    }

    pub fn matches_product_ids(&self, ids: &[ProductID]) -> bool {
        let descriptor = self.device.device_descriptor().unwrap();
        let id = descriptor.product_id();
        ids.contains(&id)
    }

    pub fn set_timeout(&mut self, timeout: Duration) -> () {
        self.timeout = timeout;
    }
}

impl Display for USBDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let descriptor = self.device.device_descriptor().unwrap();

        match self.device.open() {
            Ok(dev_handle) => {
                writeln!(f, "Device: {:?} (", self.device)?;
                let languages = dev_handle.read_languages(self.timeout).unwrap();
                for l in languages {
                    let product_string =
                        dev_handle.read_product_string(l, &descriptor, self.timeout);
                    let manufacturer_string =
                        dev_handle.read_manufacturer_string(l, &descriptor, self.timeout);
                    writeln!(
                        f,
                        "\t({:?}, {:?})",
                        product_string.unwrap(),
                        manufacturer_string.unwrap()
                    )?;
                }
                writeln!(f, ")")?;
                Ok(())
            }
            Err(_) => Err(std::fmt::Error)
        }
    }
}

pub trait DeviceInterface : Display + Send + Sync {
    fn try_create(device: USBDevice) -> Option<Self>
    where
        Self: Sized;
    fn sample(&self) -> f32;
    fn start_sampling(&self);
    fn stop_sampling(&self);
}

pub fn find_bulk_in_out_end_points<'a, T: UsbContext>(
    device: &'a Device<T>,
) -> (EndpointDescriptor<'a>, EndpointDescriptor<'a>) {
    unimplemented!()
}



pub mod dummy;
pub mod shizuku;
