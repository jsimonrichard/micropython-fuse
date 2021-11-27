mod repl;
mod dev_listener;
mod replfs;

use repl::Repl;
use dev_listener::start_dev_listener;
use replfs::ReplFS;

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{create_dir, remove_dir};



type LockedRepl = Arc<Mutex<Box<Repl>>>;
type ReplHashMap = Arc<RwLock<HashMap<String, LockedRepl>>>;


fn main() {
  /*
  // Start the logger
  pretty_env_logger::init();

  let repls: ReplHashMap = Arc::new(
    RwLock::new(
      HashMap::new()
    )
  );

  start_dev_listener(repls);

  loop {

  }
  */

  let mountpoint = "/home/jsimonrichard/Desktop/replfs";
  create_dir(mountpoint).unwrap();

  let options = ["-o", "ro", "-o", "fsname=replfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
  
  let a;
  unsafe {
    a = fuse::spawn_mount(ReplFS, &mountpoint, &options).unwrap();
  }

  loop {
  }
}
