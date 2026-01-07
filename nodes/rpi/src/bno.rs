// rpi/src/bno.rs
// BNO055センサーのラッパー
// DEBUGモードの場合、センサーを使用しない

use bno055::{BNO055OperationMode, Bno055, Error as BnoError};
use eyre::Context;
use linux_embedded_hal::{Delay, I2CError, I2cdev};
use std::{thread, time::Instant};

pub struct Bno {
    inner: Option<Bno055<I2cdev>>,
    start_time: Instant,
}

impl Bno {
    pub fn new(debug: bool) -> eyre::Result<Self> {
        if debug {
            Ok(Bno {
                inner: None,
                start_time: Instant::now(),
            })
        } else {
            let mut delay = Delay;
            let i2c = I2cdev::new("/dev/i2c-1").context("Failed to open I2C device")?;
            let mut bno055 = Bno055::new(i2c);
            while let Err(_e) = bno055.init(&mut delay) {
                thread::sleep(std::time::Duration::from_secs(1));
            }
            bno055
                .set_mode(BNO055OperationMode::NDOF, &mut delay)
                .unwrap();
            Ok(Bno {
                inner: Some(bno055),
                start_time: Instant::now(),
            })
        }
    }

    pub fn euler_angles(
        &mut self,
    ) -> Result<bno055::mint::EulerAngles<f32, ()>, BnoError<I2CError>> {
        if self.inner.is_none() {
            // Generate a deterministic time-varying dummy orientation when running in debug mode.
            let now = self.start_time.elapsed().as_secs_f32();

            let yaw = (now * 2.0).sin() * 30.0; // degrees-ish swing
            let pitch = (now * 1.1).cos() * 20.0;
            let roll = (now * 0.7).sin() * 10.0;
            let euler = bno055::mint::EulerAngles::from([yaw, pitch, roll]);
            Ok(euler)
        } else {
            let euler = self.inner.as_mut().unwrap().euler_angles()?;
            Ok(euler)
        }
    }
}
