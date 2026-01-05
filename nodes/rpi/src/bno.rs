// rpi/src/bno.rs
// BNO055センサーのラッパー
// DEBUGモードの場合、センサーを使用しない

use bno055::{BNO055OperationMode, Bno055, Error as BnoError};
use eyre::Context;
use linux_embedded_hal::{Delay, I2CError, I2cdev};
use std::thread;

pub struct Bno {
    inner: Option<Bno055<I2cdev>>,
}

impl Bno {
    pub fn new(debug: bool) -> eyre::Result<Self> {
        if debug {
            Ok(Bno { inner: None })
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
            })
        }
    }

    pub fn euler_angles(
        &mut self,
    ) -> Result<bno055::mint::EulerAngles<f32, ()>, BnoError<I2CError>> {
        if self.inner.is_none() {
            let euler = bno055::mint::EulerAngles::from([0.0, 0.0, 0.0]);
            Ok(euler)
        } else {
            let euler = self.inner.as_mut().unwrap().euler_angles()?;
            Ok(euler)
        }
    }
}
