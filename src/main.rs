use std::thread;
use std::time::Duration;

mod dht11;
mod gpio;

fn main() {
  let pin = 4;

  let mut sensor = dht11::create(pin).unwrap();
	
  loop {
    match sensor.read_sensor() {
      Ok(reading) => {
        let (temp, humid) = reading;	  
        println!("Temperature: {}C, Humidity: {}%", temp, humid);
      },
      Err(e) => println!("Failed Read!")
    };
    thread::sleep(Duration::from_millis(1000));
  }
}
