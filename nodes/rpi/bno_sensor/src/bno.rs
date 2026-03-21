// rpi/src/bno.rs
// BNO055センサーのラッパー
// DEBUGモードの場合、センサーを使用しない

use bno055::{BNO055OperationMode, Bno055, Error as BnoError};
use eyre::Context;
use linux_embedded_hal::{Delay, I2CError, I2cdev};
use std::thread;
use types::imu::ImuData;

pub struct Bno {
    inner: Option<Bno055<I2cdev>>,
    phase: f32,
}

impl Bno {
    pub fn new(debug: bool) -> eyre::Result<Self> {
        if debug {
            Ok(Bno {
                inner: None,
                phase: 0.0,
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
                phase: 0.0,
            })
        }
    }

    pub fn euler_angles(&mut self) -> Result<ImuData, BnoError<I2CError>> {
        if let Some(inner) = self.inner.as_mut() {
            let euler = inner.euler_angles()?;
            Ok(ImuData {
                roll: euler.a,
                pitch: euler.b,
                yaw: euler.c,
            })
        } else {
            // 位相を進める
            self.phase += 0.05;
            let noise = (self.phase * 10.0).sin() * 2.0;
            let roll = (self.phase).sin() * 30.0 + noise;
            let pitch = (self.phase * 0.5).cos() * 20.0; // -20〜20度
            let yaw = (self.phase * 0.2).sin() * 45.0; // -45〜45度

            Ok(ImuData { roll, pitch, yaw })
        }
    }
}
