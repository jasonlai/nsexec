mod ffi;

use std::ffi::{
    CString,
    IntoStringError,
};
use std::ptr::null_mut;

use libc::{
    c_char,
    c_int,
    dev_t,
};

use self::ffi::{
    MNT_ITER_FORWARD,
    libmnt_context,
    libmnt_fs,
    libmnt_table,
};
use self::ffi::{
    mnt_context_mount,
    mnt_context_set_fs,
    mnt_context_set_target,
    mnt_free_context,
    mnt_new_context,

    mnt_free_table,
    mnt_new_table,
    mnt_reset_table,
    mnt_table_find_source,
    mnt_table_find_target,
    mnt_table_get_nents,
    mnt_table_get_root_fs,
    mnt_table_parse_file,

    mnt_copy_fs,
    mnt_free_fs,
    mnt_fs_get_devno,
    mnt_fs_get_fstype,
    mnt_fs_get_id,
    mnt_fs_get_parent_id,
    mnt_fs_get_root,
    mnt_fs_get_source,
    mnt_fs_get_srcpath,
    mnt_fs_get_target,
    mnt_new_fs,
};

#[derive(Debug)]
pub struct Context {
    ctx: *mut libmnt_context,
}

pub struct Table {
    tb: *mut libmnt_table,
}

#[derive(Debug)]
pub struct FS {
    fs: *mut libmnt_fs,
    is_ref: bool,
}

impl Context {
    pub fn new() -> Option<Self> {
        match unsafe { mnt_new_context().as_mut() } {
            Some(ctx) => Some(Context { ctx: ctx }),
            None => None,
        }
    }

    pub fn mount(&mut self) {
        if unsafe { mnt_context_mount(self.ctx) } != 0 {
            panic!("Error mounting FS: {:?}", self);
        }
    }

    pub fn set_fs(&mut self, fs: &FS) {
        if unsafe { mnt_context_set_fs(self.ctx, fs.fs) } != 0 {
            panic!("Error setting FS: {:?}", fs);
        }
    }

    pub fn set_target(&mut self, target: &str) {
        let path = CString::new(target).unwrap();
        if unsafe { mnt_context_set_target(self.ctx, path.as_ptr()) } != 0 {
            panic!("Error setting target: {:?}", path);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { mnt_free_context(self.ctx) };
    }
}

#[allow(dead_code)]
impl Table {
    pub fn new() -> Option<Self> {
        match unsafe { mnt_new_table().as_mut() } {
            Some(tb) => Some(Table { tb: tb }),
            None => None,
        }
    }

    pub fn from_file(filename: &str) -> Result<Self, c_int> {
        let path = CString::new(filename).unwrap();
        match Table::new() {
            Some(table) => match unsafe { mnt_table_parse_file(table.tb, path.as_ptr()) } {
                0 => Ok(table),
                e => Err(e),
            },
            None => Err(0),
        }
    }

    pub fn clear(&mut self) {
        unsafe { mnt_reset_table(self.tb) };
    }

    pub fn is_empty(&self) -> bool {
        unsafe { mnt_table_get_nents(self.tb) > 0 }
    }

    pub fn len(&self) -> usize {
        unsafe { mnt_table_get_nents(self.tb) as usize }
    }

    pub fn get_root_fs(&self) -> Option<FS> {
        let mut root_fs: *mut libmnt_fs = null_mut();
        match unsafe { mnt_table_get_root_fs(self.tb, &mut root_fs) == 0 } && !root_fs.is_null() {
            true => Some(FS { fs: root_fs, is_ref: true, }),
            false => None,
        }
    }

    pub fn find_source(&self, path: &str) -> Option<FS> {
        let source_path = CString::new(path).unwrap();

        match unsafe { mnt_table_find_source(self.tb, source_path.as_ptr(), MNT_ITER_FORWARD).as_mut() } {
            Some(fs) => Some(FS { fs: fs, is_ref: true, }),
            None => None,
        }
    }

    pub fn find_target(&self, path: &str) -> Option<FS> {
        let target_path = CString::new(path).unwrap();

        match unsafe { mnt_table_find_target(self.tb, target_path.as_ptr(), MNT_ITER_FORWARD).as_mut() } {
            Some(fs) => Some(FS { fs: fs, is_ref: true, }),
            None => None,
        }
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        unsafe { mnt_free_table(self.tb) };
    }
}

#[allow(dead_code)]
impl FS {
    pub fn new() -> Option<Self> {
        match unsafe { mnt_new_fs().as_mut() } {
            Some(fs) => Some(FS {
                fs: fs,
                is_ref: false,
            }),
            None => None,
        }
    }

    pub fn id(&self) -> i32 {
        unsafe { mnt_fs_get_id(self.fs) as i32 }
    }

    pub fn parent_id(&self) -> i32 {
        unsafe { mnt_fs_get_parent_id(self.fs) as i32 }
    }

    pub fn devno(&self) -> dev_t {
        unsafe { mnt_fs_get_devno(self.fs) }
    }

    pub fn root(&self) -> Result<String, IntoStringError> {
        unsafe { CString::from_raw(mnt_fs_get_root(self.fs) as *mut c_char).into_string() }
    }

    pub fn source(&self) -> Result<String, IntoStringError> {
        unsafe { CString::from_raw(mnt_fs_get_source(self.fs) as *mut c_char).into_string() }
    }

    pub fn srcpath(&self) -> Result<String, IntoStringError> {
        unsafe { CString::from_raw(mnt_fs_get_srcpath(self.fs) as *mut c_char).into_string() }
    }

    pub fn target(&self) -> Result<String, IntoStringError> {
        unsafe { CString::from_raw(mnt_fs_get_target(self.fs) as *mut c_char).into_string() }
    }

    pub fn fstype(&self) -> Result<String, IntoStringError> {
        unsafe { CString::from_raw(mnt_fs_get_fstype(self.fs) as *mut c_char).into_string() }
    }
}

impl Clone for FS {
    fn clone(&self) -> Self {
        match Self::new() {
            Some(new_obj) => {
                if unsafe { mnt_copy_fs(new_obj.fs, self.fs as *const libmnt_fs) } != new_obj.fs {
                    panic!("Unable to clone to new FS object");
                } 
                new_obj
            },
            None => { panic!("Unable to allocate new FS object") }
        }
    }
}

impl Drop for FS {
    fn drop(&mut self) {
        if !self.is_ref {
            unsafe { mnt_free_fs(self.fs) };
        }
    }
}