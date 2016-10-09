extern crate libc;

use std::ffi::CString;
use std::fs::File;
use std::io::{Error, Result};
use std::os::unix::io::AsRawFd;
use std::ptr::null;

use libc::{
    CLONE_NEWNS,
    MS_BIND,
    pid_t,
};
use libc::{
    mount,
    setns,
    unshare,
};

const NAMESPACES: &'static[&'static str] = &["mnt", "uts", "ipc", "pid", "net"];

pub fn bind_mount(source: &str, target: &str) -> Result<()> {
    let src = CString::new(source).unwrap();
    let dest = CString::new(target).unwrap();

    match unsafe { mount(src.as_ptr(), dest.as_ptr(), null(), MS_BIND, null()) } {
        0 => Ok(()),
        _ => Err(Error::last_os_error()),
    }
}

pub fn umount(target: &str) -> Result<()> {
    let dest = CString::new(target).unwrap();

    match unsafe { libc::umount(dest.as_ptr()) } {
        0 => Ok(()),
        _ => Err(Error::last_os_error()),
    }
}

pub fn nsenter(pid: &pid_t) -> Result<()> {
    let ns_files: Vec<File> = NAMESPACES
        .into_iter()
        .map(|ns| File::open(format!("/proc/{}/ns/{}", pid, ns)).unwrap())
        .collect();

    for file in ns_files {
        if unsafe { setns(file.as_raw_fd(), 0) } != 0 {
            return Err(Error::last_os_error())
        }
    }

    Ok(())
}

pub fn unshare_mount() -> Result<()> {
    match unsafe { unshare(CLONE_NEWNS) } {
        0 => Ok(()),
        _ => Err(Error::last_os_error()),
    }
}
