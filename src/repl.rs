use serialport::{SerialPort};
use std::io::{Error, ErrorKind, Result};
use std::time::Duration;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use log::{trace, debug, info};

use crate::LockedRepl;


pub struct Repl {
  port: Box<dyn SerialPort>
}


impl Repl {
  pub fn new(port: Box<dyn SerialPort>) -> Box<Repl> {
    Box::new(Repl {
      port: port
    })
  }

  pub fn from_path(port_path: &str) -> Result<Box<Repl>> {
    info!("Connecting to serial port {}", port_path);

    let builder = serialport::new(port_path, 115_200);
    let port = builder
                .timeout(Duration::from_millis(10))
                .open()?;

    info!("Starting REPL for {}", port_path);

    let mut repl = Repl::new(port);
    repl.start()?;

    Ok(repl)
  }

  pub fn start(&mut self) -> Result<()> {
    debug!("Preparing client...");

    // Stop anything that's running
    trace!("Stopping any currently running programs");
    self.port.write(&[b'\x03', b'\x03'])?;

    // Go to raw repl mode
    trace!("Entering Raw REPL mode");
    self.port.write(&[b'\x01'])?;

    // flush input
    trace!("Flushing input");
    self.flush_input()?;

    Ok(())
  }

  pub fn run(&mut self, code: String) -> Result<String> {
    debug!("--- Running command ---\n{}\n --- end of command ---", code);

    trace!("Sending code to client");
    self.port.write(code.as_bytes())?;
    self.port.write(&[b'\x04'])?;

    let mut result = String::new();

    self.read_all_to_string(&mut result)?;

    debug!("--- Raw result ---\n{}\n--- end of result ---", result);

    if result.starts_with("OK") {
      return Ok(result[2..result.len()-4].to_string())
    } else {
      return Err(
        Error::new(ErrorKind::InvalidData, result)
      );
    }
  }

  fn read_all_to_string(&mut self, buf: &mut String) -> Result<()> {
    trace!("Reading result to string...");
    match self.port.read_to_string(buf) {
      Ok(_) => {},
      Err(ref e) if e.kind() == ErrorKind::TimedOut => {},
      Err(e) => {
        return Err(e);
      }
    }

    Ok(())
  }

  fn flush_input(&mut self) -> Result<()> {
    trace!("Flushing input");
    self.read_all_to_string(&mut String::new())?;
    Ok(())
  }
}


impl TryInto<LockedRepl> for Box<Repl> {
  type Error = Error;

  fn try_into(self: Self) -> std::result::Result<LockedRepl, Error> {
    Ok(Arc::new(
      Mutex::new(self)
    ))
  }
}