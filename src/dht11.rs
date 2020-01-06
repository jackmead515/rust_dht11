use std::thread;
use std::time::Duration;
use std::fmt;

use crate::gpio::GPIOPin;

#[derive(Debug)]
pub enum SensorError {
  FailedRead(String),
  FailedInit(String),
  Timeout(String)
}

impl fmt::Display for SensorError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &*self {
      SensorError::FailedRead(s) => write!(f, "Failed to Read Sensor: {}", s),
      SensorError::FailedInit(s) => write!(f, "Failed to Initialize Sensor: {}", s),
      SensorError::Timeout(s) => write!(f, "Sensor Timed Out: {}", s)
    }
  }
}

pub struct DHT11 {
  pin: GPIOPin
}

pub fn create(pin: u8) -> Result<DHT11, SensorError> {
  let pin = match GPIOPin::new(pin) {
    Ok(p) => p,
    Err(_) => return Err(SensorError::FailedInit("Failed to create gpio pin".to_string()))
  };

  return Ok(DHT11 {
    pin: pin
  });
}

impl DHT11 {
  pub fn read_retry(&mut self, retry: usize) -> Result<(f32, f32), SensorError> {
    for _ in 0..retry {
      let result = self.read_sensor();

      if result.is_ok() {
        return Ok(result.unwrap());
      }

      thread::sleep(Duration::from_millis(50));
    }

    return Err(SensorError::FailedRead("dht11 failed".to_string()));
  }

  pub fn read_sensor(&mut self) -> Result<(f32, f32), SensorError> {
    self.pin.set_output();
    self.pin.set_high();
    thread::sleep(Duration::from_millis(500));
    self.pin.set_low();
    thread::sleep(Duration::from_millis(20));
    self.pin.set_high();
    thread::sleep(Duration::from_micros(30));
    self.pin.set_input();

    // Capture the pulses from the sensor.
    const DHT_MAXCOUNT: usize = 32_000;
    const DHT_PULSES: usize = 41;
    let mut pulse_counts: [u32; DHT_PULSES*2] = [0; DHT_PULSES*2];

    for i in (0..DHT_PULSES * 2).step_by(2) {
      while self.pin.is_low() {
        pulse_counts[i] += 1;
        if pulse_counts[i] >= DHT_MAXCOUNT as u32 {
          return Err(SensorError::Timeout("timed out low pulse capture".to_string()));
        }
      }
      while self.pin.is_high() {
        pulse_counts[i + 1] += 1;
        if pulse_counts[i + 1] >= DHT_MAXCOUNT as u32 {
          return Err(SensorError::Timeout("timed out high pulse capture".to_string()));
        }
      }
    }
  
    // Computes the threshold
    let mut threshold = 0;
    for i in (2..DHT_PULSES * 2).step_by(2) {
      threshold += pulse_counts[i];
    }
    threshold /= DHT_PULSES as u32 - 1;
  
    // Collects the data
    let mut data: [u8; 5] = [0; 5];
    for i in (3..DHT_PULSES * 2).step_by(2) {
      let index = (i - 3) / 16;
      data[index] <<= 1;
      if pulse_counts[i] >= threshold {
        data[index] |= 1;
      }
    }
  
    // Ensures that collected data is valid and returns
    if data[4] == data[0] + data[1] + data[2] + data[3] {
      let hint = data[0] as f32; let hdec = data[1] as f32;
      let tint = data[2] as f32; let tdec = data[3] as f32;
      let humid: f32 = hint + (hdec / 10.0);
      let temp: f32 = tint + (tdec / 10.0);
  
      return Ok((temp, humid));
    } else {
      return Err(SensorError::FailedRead("failed checksum validation".to_string()));
    }
  }
}


