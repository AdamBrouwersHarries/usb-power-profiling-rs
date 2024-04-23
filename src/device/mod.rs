use std::{fmt::Display, time::Duration};

use aes::cipher::consts::True;
use rusb::{ConfigDescriptor, Device as RUsbDevice, DeviceHandle, Direction, EndpointDescriptor, InterfaceDescriptor, TransferType, UsbContext};

pub type VendorID = u16;
pub type ProductID = u16;

// *FOR NOW* use `rusb::Context`, to simplify the types.
pub type Device = RUsbDevice<rusb::Context>;

// Wrap an rusb device in a struct to simplify common operations.
pub struct USBDevice {
    handle: DeviceHandle<rusb::Context>,
    configd: ConfigDescriptor,
    timeout: Duration,
}

impl USBDevice {
    pub fn create_if_matching_id(device: Device, ids: &[ProductID]) -> Result<USBDevice, Device> {
        // TODO: Better error handling for device descriptor and config descriptor
        let descriptor = device.device_descriptor().unwrap();
        let configd = device.active_config_descriptor().unwrap();
        let id = descriptor.product_id();
        match ids.contains(&id) {
            true => {
                // We have a matching device!
                // Open/claim it, and create the USBDevice.
                // TODO: better error handling here.
                // It implies that we can re-use the device if we fail to open it...
                match device.open() {
                    Ok(handle) => Ok(USBDevice {
                        handle,
                        configd,
                        timeout: Duration::from_millis(1000),
                    }),
                    Err(_) => Err(device),
                }
            }
            false => Err(device),
        }
    }

    pub fn query(&self) -> () {
        match self.handle.device().active_config_descriptor() {
            Ok(c) => {
                println!("We have an active config: {:?}", c);
                for interface in c.interfaces() {
                    println!("Interface num: {:?}", interface.number());
                    for descriptor in interface.descriptors() {
                        println!("Descriptor: {:?}", descriptor);
                        for endpoint in descriptor.endpoint_descriptors() {
                            println!(
                                "Endpoint: {:?}\n Directi9on: {:?}, type: {:?}",
                                endpoint,
                                endpoint.direction(),
                                endpoint.transfer_type()
                            );
                        }
                    }
                }
            }
            _ => {
                println!("No active descriptor!");
            }
        }
    }

    pub fn find_bulk_in_out_end_points<'a>(
        &'a mut self,
    ) -> Option<(EndpointDescriptor<'a>, EndpointDescriptor<'a>)> {
        // More or less a direct port of the JS version. There's probably a more "rusty" way to do it.
        let mut endpoint_in: Option<EndpointDescriptor<'a>> = None;
        let mut endpoint_out: Option<EndpointDescriptor<'a>> = None;

        let confd = &self.configd;

        for interface in confd.interfaces() {
            // track whether or not we've already claimed this interface
            let mut claimed: bool = false;
            for descriptor in interface.descriptors() {
                for endpoint in descriptor.endpoint_descriptors() {
                    // Ignore all endpoints that aren't bulk transfer
                    if endpoint.transfer_type() != TransferType::Bulk {
                        continue;
                    }

                    if endpoint.direction() == Direction::In && endpoint_in.is_none() {
                        endpoint_in = Some(endpoint);
                        if !claimed {
                            claimed = true;
                            let _ = self.handle.claim_interface(descriptor.interface_number()).unwrap();
                        }
                    } else if endpoint.direction() == Direction::Out && endpoint_in.is_none() {
                        endpoint_out = Some(endpoint);
                        // Detach the kernel driver on Linux if it's already active.
                        if let Ok(true) = &self.handle.kernel_driver_active(descriptor.interface_number()) {
                            // Throws an error on non-linux platforms. Ignroe.
                            let _ =  self.handle.detach_kernel_driver(descriptor.interface_number()).unwrap();
                        }
                        if !claimed {
                            claimed = true;
                            let _ =  self.handle.claim_interface(descriptor.interface_number()).unwrap();
                        }
                    }
                }
            }
        }

        match (endpoint_in, endpoint_out) {
            (Some(ei), Some(eo)) => Some((ei,eo)),
            _ => None
        }
    }

    pub fn reset(&mut self) -> Result<(), rusb::Error> {
        self.handle.reset()
    }

    pub fn set_timeout(&mut self, timeout: Duration) -> () {
        self.timeout = timeout;
    }
}

impl Display for USBDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let descriptor = self.handle.device().device_descriptor().unwrap();

        writeln!(f, "Device: {:?} (", self.handle.device())?;
        let languages = self.handle.read_languages(self.timeout).unwrap();
        for l in languages {
            let product_string = self
                .handle
                .read_product_string(l, &descriptor, self.timeout);
            let manufacturer_string =
                self.handle
                    .read_manufacturer_string(l, &descriptor, self.timeout);
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
}

pub trait DeviceInterface: Display + Send + Sync {
    fn try_create(device: Device) -> Result<Box<Self>, Device>
    where
        Self: Sized;
    fn sample(&self) -> f32;
    fn start_sampling(&self);
    fn stop_sampling(&self);
}

pub mod dummy;
pub mod shizuku;
