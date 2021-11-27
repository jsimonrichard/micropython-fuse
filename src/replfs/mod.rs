mod ilistdir_item;
mod filesystem_status;

use ilistdir_item::IListDirItem;
use filesystem_status::FilesystemStatus;
use crate::LockedRepl;

use std::ffi::OsStr;
use time::Timespec;
use libc::ENOENT;
use std::io::{Result, Error, ErrorKind};
use std::convert::TryFrom;
use fuse::{
  FileType,
  FileAttr,
  Filesystem,
  Request,
  ReplyData,
  ReplyEntry,
  ReplyAttr,
  ReplyDirectory
};


const TTL: Timespec = Timespec {sec: 1, nsec: 0};
const UNIX_EPOCH: Timespec = Timespec {sec: 0, nsec: 0};


const ROOT_DIR_ATTR: FileAttr = FileAttr {
  ino: 1, // indexes start at 1 (0 indicates no inode)
  size: 0,
  blocks: 0,
  atime: UNIX_EPOCH,                                  // 1970-01-01 00:00:00
  mtime: UNIX_EPOCH,
  ctime: UNIX_EPOCH,
  crtime: UNIX_EPOCH,
  kind: FileType::Directory,
  perm: 0o755,
  nlink: 2,
  uid: 501,
  gid: 20,
  rdev: 0,
  flags: 0,
};


struct File {
  path: String,
  attr: FileAttr
}


pub struct ReplFS {
  repl: LockedRepl,
  inodes: Vec<File>,
  filesystem_status: Option<FilesystemStatus>
}


impl ReplFS {
  pub fn new(repl: LockedRepl) -> ReplFS {
    ReplFS {
      repl: repl,
      inodes: vec![File {
        path: "/".to_string(),
        attr: ROOT_DIR_ATTR
      }],
      filesystem_status: None
    }
  }

  pub fn start(&mut self) -> Result<()> {
    self.filesystem_status = Some(
      self._get_vfs_info()?
    );

    

    Ok(())
  }

  fn _get_vfs_info(&mut self) -> Result<FilesystemStatus> {
    let result = self.repl.lock().unwrap()
      .run("os.statvfs()".to_string())?;
    
    match FilesystemStatus::try_from(result) {
      Ok(f) => {
        return Ok(f);
      }
      Err(e) => {
        return Err(Error::new(
          ErrorKind::InvalidData,
          e
        ));
      }
    }
  }

  fn build_inodes(&mut self, dir: &str) -> Result<()> {
    let filesystem_status = self.filesystem_status
          .expect("filesystem status must be obtained first");

    let items: Vec<IListDirItem> = parse_list(
      self.repl.lock().unwrap()
      .run(format!("list(os.ilistdir(\"{}\"))", dir))?
    );

    let inode_index: u64 = self.inodes.len() as u64 + 1; // index starts at 1
    for item in items {
      self.inodes.push(
        File {

          path: format!(
            "{}/{}",
            dir.to_string()
              .trim_end_matches("/"),
            item.name
          ),

          attr: FileAttr {
            ino: inode_index,
            size: item.size.unwrap(),
            blocks: (item.size.unwrap()+filesystem_status.bsize)
                            / filesystem_status.bsize,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: item.item_type,
            perm: 0o755,
            nlink: 1,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
          }
          
        }
      );

      // Increment inode index for next file/directory
      inode_index += 1;
    }

    Ok(())
  }
}


impl Filesystem for ReplFS {
  fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
    if parent == 1 && name.to_str() == Some("hello.txt") {
      reply.entry(&TTL, &HELLO_TXT_ATTR, 0);
    } else {
      reply.error(ENOENT);
    }
  }

  fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
    match ino {
      1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
      2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
      _ => reply.error(ENOENT),
    }
  }

  fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, _size: u32, reply: ReplyData) {
    if ino == 2 {
      reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
    } else {
      reply.error(ENOENT);
    }
  }

  fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
    if ino != 1 {
      reply.error(ENOENT);
      return;
    }

    let entries = vec![
      (1, FileType::Directory, "."),
      (1, FileType::Directory, ".."),
      (2, FileType::RegularFile, "hello.txt"),
    ];

    for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
      // i + 1 means the index of the next entry
      reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
    }
    reply.ok();
  }
}


fn parse_list<T>(list_str: String) -> Vec<T> where
  T: TryFrom<String>,
  <T as TryFrom<String>>::Error: std::fmt::Debug
{
  list_str.trim_matches(&['[', ']'] as &[_]).split(", ")
    .map(|item| T::try_from(item.trim_matches(&['"'] as &[_])
                    .to_string()).unwrap())
    .collect()
}