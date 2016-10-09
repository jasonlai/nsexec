extern crate argparse;
extern crate libc;

mod libmount;
mod nsexec;
mod syscall;

use std::fs::create_dir_all;
use std::io::Error;
use std::process::exit;

use libc::getpid;

use libmount::{
    Context,
    Table,
};
use nsexec::{
    FromArguments,
    NSExecutor,
    Volume,
};
use syscall::{
    bind_mount,
    umount,
    unshare_mount,
};

fn main() {
    match NSExecutor::from_cmd_arguments() {
        Ok(executor) => {
            let volumes: &Vec<Volume> = executor.volumes();

            if !volumes.is_empty() {
                let non_existing: Vec<&Volume> =
                    volumes.iter().filter(|&v| !v.source().exists()).collect();

                if !non_existing.is_empty() {
                    panic!("Some paths do not exist: {:?}", non_existing);
                }

                let mount_info_path = format!("/proc/{}/mountinfo", unsafe { getpid() });
                let table = Table::from_file(mount_info_path.as_str()).map_err(|_| Error::last_os_error()).unwrap();

                let volumes_to_bind_mount: Vec<&Volume> =
                    volumes.iter().filter(|&v| table.find_target(v.source().to_str().unwrap()).is_none()).collect();

                unshare_mount().unwrap();

                for volume in &volumes_to_bind_mount {
                    let path = volume.source().to_str().unwrap();
                    println!("Bind mounting {:?}...", path);
                    bind_mount(path, path).unwrap();
                }

                let new_table = Table::from_file(mount_info_path.as_str()).unwrap();

                for volume in &volumes_to_bind_mount {
                    let path = volume.source().to_str().unwrap();
                    println!("Unmounting {:?}...", path);
                    umount(path).unwrap();
                }

                executor.nsenter().unwrap();
                unshare_mount().unwrap();

                for volume in volumes {
                    let mut context = Context::new().unwrap();
                    let fs = new_table.find_target(volume.source().to_str().unwrap()).unwrap();
                    let target = volume.destination();

                    if !target.exists() {
                        println!("Creating {:?}...", target);
                        create_dir_all(target).unwrap();
                    }

                    context.set_fs(&fs);
                    context.set_target(target.to_str().unwrap());
                    context.mount()
                }
            } else {
                executor.nsenter().unwrap();
            }

            panic!(executor.exec());
        },
        Err(status) => {
            exit(status);
        },
    }
}
