mod repl;
mod dev_listener;

use repl::Repl;
use dev_listener::start_dev_listener;
use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;


type LockedRepl = Arc<Mutex<Box<Repl>>>;
type ReplHashMap = Arc<RwLock<HashMap<String, LockedRepl>>>;


fn main() {
  let repls: ReplHashMap = Arc::new(
    RwLock::new(
      HashMap::new()
    )
  );

  start_dev_listener(repls);

  loop {
    
  }
}
