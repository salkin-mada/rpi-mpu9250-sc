extern crate linux_embedded_hal as hal;
extern crate mpu9250_i2c;
extern crate rosc;

mod print;

use hal::{Delay, I2cdev};
use mpu9250_i2c::{calibration::Calibration, vector::Vector, Mpu9250};
use print::Print;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
//use std::time::Duration;
use std::{env, f32, thread};

fn get_addr_from_arg(arg: &str) -> SocketAddrV4 {
    SocketAddrV4::from_str(arg).unwrap()
}

pub fn to_ms(duration: Duration) -> u64 {
  duration.as_secs() * 1_000 + (duration.subsec_millis() as u64)
}

fn main() {

    // i2c
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    
  let cal = Calibration {
    mag_offset: Vector {
      x: 9.5,
      y: 20.78125,
      z: -28.04101,
    },
    mag_scale: Vector {
      x: 1.49696,
      y: 1.44312,
      z: 1.56484,
    },

    // Gryoscope
    gyro_bias_offset: Vector {
      x: 0.57159,
      y: -0.5399,
      z: 0.10633,
    },

    // Accelerometer
    accel_offset: Vector {
      x: 0.00913,
      y: 0.02747,
      z: -0.10344,
    },
    accel_scale_lo: Vector {
      x: 0.99700,
      y: 0.97594,
      z: 0.94592,
    },
    accel_scale_hi: Vector {
      x: -1.0045,
      y: -0.9867,
      z: -1.0648,
    },
  };

  // Let's start
  let mut mpu9250 = Mpu9250::new(dev, Delay, cal).unwrap();
  mpu9250.init().unwrap();

  // Print out some useful information
  Print::mpu9250_settings(&mut mpu9250);
  Print::ak8963_settings(&mut mpu9250);

  // Set up some stuff
  let mut last_mag_read = Instant::now();
  let mut last_read;
  let rate = mpu9250.get_accel_gyro_rate_ms();
  let mag_rate = 10; // 10 milliseconds per read

    // osc
    let args: Vec<String> = env::args().collect();
    let usage = format!(
        "Usage: {} HOST_IP:HOST_PORT CLIENT_IP:CLIENT_PORT",
        &args[0]
    );
    if args.len() < 3 {
        panic!(usage);
    }
    let host_addr = get_addr_from_arg(&args[1]);
    let to_addr = get_addr_from_arg(&args[2]);
    let sock = UdpSocket::bind(host_addr).unwrap();

    // switch view
    let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
        addr: "/3".to_string(),
        args: vec![],
    }))
    .unwrap();

    sock.send_to(&msg_buf, to_addr).unwrap();

  loop {
    // Get the accelerometer and gyro data
    let (va, vg) = mpu9250.get_accel_gyro().unwrap();
    last_read = Instant::now();
    print!(
      "\nAccel=>  {{ x: {:.5}, y: {:.5}, z: {:.5} }}",
      va.x, va.y, va.z
    );
    print!(
      "\nGyro=> {{ x: {:.5}, y: {:.5}, z: {:.5} }}",
      vg.x, vg.y, vg.z
    );

    // The mag refresh rate is around 10 milliseconds
    let mag_elapsed = to_ms(last_mag_read.elapsed());
    if mag_elapsed >= mag_rate {
      let vm = mpu9250.get_mag().unwrap();
      print!(
        "\nmag=> {{ x: {:.5}, y: {:.5}, z: {:.5} }}, Temp=> {:0.5}",
        vm.x,
        vm.y,
        vm.z,
        mpu9250.get_temperature_celsius().unwrap()
      );
      last_mag_read = Instant::now();
    }
    println!();

        // OSC loop
        // accel
        let mut msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/pipe/accel".to_string(),
            args: vec![OscType::Float(va.x), OscType::Float(va.y), OscType::Float(va.z)],
        }))
        .unwrap();
        sock.send_to(&msg_buf, to_addr).unwrap();
        
        // gyro
        msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/pipe/gyro".to_string(),
            args: vec![OscType::Float(vg.x), OscType::Float(vg.y), OscType::Float(vg.z)],
        }))
        .unwrap();
        sock.send_to(&msg_buf, to_addr).unwrap();
        
        // mag
        /*msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/pipe/mag".to_string(),
            args: vec![OscType::Float(vm.x), OscType::Float(vm.y), OscType::Float(vm.z)],
        }))
        .unwrap();
        sock.send_to(&msg_buf, to_addr).unwrap();*/
  


    // Wait until the next read - normally around 4 ms refresh rate
    let elapsed = to_ms(last_read.elapsed());
    if elapsed < rate {
        sleep(Duration::from_millis(rate - elapsed));
    }

    // general relax
    sleep(Duration::from_millis(50));
  }
}
