#![allow(non_camel_case_types)]
use libc::{
    c_char,
    c_int,
    dev_t,
};

pub enum libmnt_context {}
pub enum libmnt_table {}
pub enum libmnt_fs {}

pub const MNT_ITER_FORWARD: c_int = 0;
#[allow(dead_code)]
pub const MNT_ITER_BACKWARD: c_int = 1;

#[allow(dead_code)]
#[link(name = "mount")]
extern "C" {
    pub fn mnt_context_mount(ctx: *mut libmnt_context) -> c_int;
    pub fn mnt_context_set_fs(ctx: *mut libmnt_context, fs: *mut libmnt_fs) -> c_int;
    pub fn mnt_context_set_target(ctx: *mut libmnt_context, target: *const c_char) -> c_int;
    pub fn mnt_free_context(ctx: *mut libmnt_context);
    pub fn mnt_new_context() -> *mut libmnt_context;

    pub fn mnt_free_table(tb: *mut libmnt_table);
    pub fn mnt_new_table() -> *mut libmnt_table;
    pub fn mnt_reset_table(tb: *mut libmnt_table) -> c_int;
    pub fn mnt_table_find_source(tb: *mut libmnt_table, filename: *const c_char, direction: c_int) -> *mut libmnt_fs;
    pub fn mnt_table_find_target(tb: *mut libmnt_table, filename: *const c_char, direction: c_int) -> *mut libmnt_fs;
    pub fn mnt_table_get_nents(tb: *mut libmnt_table) -> c_int;
    pub fn mnt_table_get_root_fs(tb: *mut libmnt_table, root: *mut *mut libmnt_fs) -> c_int;
    pub fn mnt_table_parse_file(tb: *mut libmnt_table, filename: *const c_char) -> c_int;

    pub fn mnt_copy_fs(dest: *mut libmnt_fs, src: *const libmnt_fs) -> *mut libmnt_fs;
    pub fn mnt_free_fs(fs: *mut libmnt_fs);
    pub fn mnt_fs_get_devno(fs: *mut libmnt_fs) -> dev_t;
    pub fn mnt_fs_get_fstype(fs: *mut libmnt_fs) -> *const c_char;
    pub fn mnt_fs_get_id(fs: *mut libmnt_fs) -> c_int;
    pub fn mnt_fs_get_parent_id(fs: *mut libmnt_fs) -> c_int;
    pub fn mnt_fs_get_root(fs: *mut libmnt_fs) -> *const c_char;
    pub fn mnt_fs_get_source(fs: *mut libmnt_fs) -> *const c_char;
    pub fn mnt_fs_get_srcpath(fs: *mut libmnt_fs) -> *const c_char;
    pub fn mnt_fs_get_target(fs: *mut libmnt_fs) -> *const c_char;
    pub fn mnt_new_fs() -> *mut libmnt_fs;
}
