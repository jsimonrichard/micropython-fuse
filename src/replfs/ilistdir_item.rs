use fuse::FileType;
use std::convert::TryFrom;
use std::num::ParseIntError;

pub struct IListDirItem {
  pub name: String,
  pub item_type: FileType,
  pub inode: u64,
  pub size: Option<u64>
}

impl TryFrom<String> for IListDirItem {
  type Error = ParseIntError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    let entries: Vec<String> = s.trim_matches(&['(', ')'] as &[_])
                                .split(", ")
                                .map(|item| item.trim_matches(&['"'] as &[_]).to_string())
                                .collect();
  
    let size: Option<u64> = match entries.get(3) {
      Some(s) => Some(s.parse()?),
      None => None
    };
    
    Ok(Self {
      name: entries[0],
      item_type: if entries[1].parse::<u64>()? == 0x4000 {
        FileType::Directory
      } else {
        FileType::RegularFile
      },

      inode: entries[2].parse()?,
      size: size
    })
  }
}