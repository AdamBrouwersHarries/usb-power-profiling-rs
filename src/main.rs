use actix_web::get;
use actix_web::web::Data;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder};
use device::shizuku::ShizukuDevice;
use rusb::UsbContext;
use core::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

mod device;
use device::{DeviceInterface, USBDevice};
// use device::dummy::DummyDevice;

// Provide an object-safe wrapper around a device interface, and also provide
// a safe way of accessing it at the same time.
struct PowerMeter {
    // underlying_device: &'a dyn DeviceInterface,
    underlying_device: Arc<Box<dyn DeviceInterface>>,
}

impl PowerMeter {
    fn from(di: Box<dyn DeviceInterface>) -> PowerMeter {
        PowerMeter {
            underlying_device: Arc::new(di),
        }
    }

    fn start(&self) {
        thread::spawn({
            let udcr: Arc<Box<dyn DeviceInterface>> = self.underlying_device.clone();
            move || {
                udcr.start_sampling();
            }
        });
    }

    fn stop(&self) {
        unimplemented!()
    }

    fn sample(&self) -> f32 {
        let g = self.underlying_device.clone();
        g.sample()
    }
}

unsafe impl Sync for PowerMeter {}
unsafe impl Send for PowerMeter {}

impl fmt::Display for PowerMeter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.underlying_device.fmt(f)
    }
}

struct PowerMeterIterator<'a> {
    index: usize,
    meters: &'a Vec<PowerMeter>,
}

impl<'a> PowerMeterIterator<'a> {
    fn from(meters: &'a Vec<PowerMeter>) -> PowerMeterIterator<'a> {
        PowerMeterIterator { index: 0, meters }
    }
}

impl<'a> Iterator for PowerMeterIterator<'a> {
    type Item = &'a PowerMeter;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.meters.len() {
            let oindex = self.index;
            self.index += 1;
            self.meters.get(oindex)
        } else {
            None
        }
    }
}

struct PowerSampler {
    usb_context: rusb::Context,
    meters: Vec<PowerMeter>,
}

impl PowerSampler {
    pub fn init() -> rusb::Result<PowerSampler> {
        let usb_context = rusb::Context::new()?;
        Ok(PowerSampler {
            usb_context,
            meters: vec![],
        })
    }

    pub fn find_meters(&mut self) -> rusb::Result<()> {
        let devices = self.usb_context.devices()?;
        for device in devices.iter() {
            if let Some(sd) = ShizukuDevice::try_create(USBDevice::from_device(device)) {
                self.meters.push(PowerMeter::from(Box::new(sd)));
            }
        }
        Ok(())
    }

    pub fn add_device(&mut self, d: PowerMeter) {
        self.meters.push(d)
    }

    pub fn list_meters(&self) -> PowerMeterIterator<'_> {
        PowerMeterIterator::from(&self.meters)
    }

    pub fn start(&mut self) {
        for meter in &self.meters {
            meter.start();
        }
    }

    pub fn stop(&mut self) {
        for meter in &self.meters {
            meter.stop();
        }
    }
}

#[get("/")]
async fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/power")]
async fn power(req: HttpRequest) -> impl Responder {
    let data = req.app_data::<Data<Mutex<PowerSampler>>>().unwrap();
    let g = data.lock().unwrap();
    let mut body: String = "Power!".into();
    let devices = g.list_meters();
    for d in devices {
        let s = d.sample();
        let s = format!("\nSample: {:?}", s);
        body = body + &s;
    }
    HttpResponse::Ok().body(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Ensure devtools.performance.recording.power.external-url is set to http://localhost:2121/power in 'about:config'.");

    // create a dummy device
    // let dm: Box<DummyDevice> = Box::new(DummyDevice::init());
    // let dev = Device::from(dm);

    // Create the power sampler
    let mut sampler = PowerSampler::init().unwrap();
    let _ = sampler.find_meters();

    for meters in sampler.list_meters() {
        println!("{}", meters);
    }

    // sampler.add_device(dev);
    sampler.start();

    // Start a thread for the sampler? Do it async?
    let data = Data::new(Mutex::new(sampler));

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(power)
            .app_data(Data::clone(&data))
    })
    .bind(("127.0.0.1", 2121))?
    .run()
    .await
}
