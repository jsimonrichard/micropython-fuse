use crate::repl::Repl;
use crate::{ReplHashMap};

use std::thread;
use std::time::Duration;
use std::convert::TryInto;
use std::collections::HashSet;
use log::{warn, info};

const THREAD_DELAY_NANOS: u32 = 50_000_000;


pub fn start_dev_listener(repls_lock: ReplHashMap) {
  // Start the listener thread
  thread::spawn(move || {
    info!("Device Listener Thread started");

    let delay = Duration::new(0, THREAD_DELAY_NANOS);

    // Blacklist for devices that we failed to connect with
    let mut black_list: HashSet<String> = HashSet::new();

    loop {

      { // Scope for repls lock
        let mut repls = repls_lock.write().unwrap();
        let mut devices: HashSet<String> = HashSet::new();

        // Look for new devices
        for device in serialport::available_ports().unwrap() {
          // Add name to vec (for second part)
          devices.insert(device.port_name.clone());

          // Create a repl has been created for that device
          if !repls.contains_key(&device.port_name) 
                && !black_list.contains(&device.port_name) {

            info!("Mounting device: {}", &device.port_name);

            match Repl::from_path(&device.port_name) {
              Ok(r) => {
                repls.insert(
                  device.port_name.clone(),
                  r.try_into().unwrap()
                );
              }
              Err(e) => {
                // If it failed, add the device to the black list
                warn!("Unable to start REPL for {}\n Error: {}", device.port_name, e);
                black_list.insert(device.port_name);
              }
            }
          }
        }

        // Look for devices that have been removed
        repls.retain(|key, _| {
          if devices.contains(key) { // Retain this repl
            return true;
          } else { // The device has been disconnected
            // Remove from blacklist (if it was there)
            black_list.remove(key);

            info!("Unmounting device: {}", key);

            // Do not retain
            return false;
          }
        });

      } // Release repls lock

      // Sleep (and allow other threads to use repls lock)
      thread::sleep(delay);
    }
  });
}