mod dht11;
mod gpio;

fn main() {
  let pin = 4;
  let retry = 3;

  let mut sensor = dht11::create(pin).unwrap();
  let (temp, humid) = sensor.read_retry(retry).unwrap();

  println!("Temperature: {}C, Humidity: {}%", temp, humid);
}
