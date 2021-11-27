use std::convert::TryFrom;
use std::num::ParseIntError;
pub struct FilesystemStatus {
  pub bsize: u64, // Block size
  pub frsize: u64, // Fragment size
  pub blocks: u64, // Size of file system in f_frsize,
  pub bfree: u64, // Number of free blocks
  pub bavail: u64, // Blocks free of unprivileged users
  pub files: u64, // Number of inodes
  pub ffree: u64, // Number of free inodes
  pub favail: u64, // Number of free inodes for unprivileged users
  pub flag: u64, // Mount flags
  pub namemax: u64 // Max filename length
}

impl TryFrom<String> for FilesystemStatus {
  type Error = ParseIntError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    let entries: Vec<u64> = s.trim_matches(&['(', ')'] as &[_])
                              .split(", ")
                              .map(|item| item.parse())
                              .collect::<Result<Vec<u64>, Self::Error>>()?;
    
    Ok(FilesystemStatus {
      bsize: entries[0],
      frsize: entries[1],
      blocks: entries[2],
      bfree: entries[3],
      bavail: entries[4],
      files: entries[5],
      ffree: entries[6],
      favail: entries[7],
      flag: entries[8],
      namemax: entries[9]
    })
  }
}