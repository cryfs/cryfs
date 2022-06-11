// TODO If https://github.com/GuillaumeGomez/sysinfo/issues/446 succeeds, we should use that instead of our manual copy&paste here

use anyhow::{ensure, Result};
use std::path::Path;

/// Returns the available disk space of the file system that contains the given path
pub fn get_available_disk_space(path: &Path) -> Result<u64> {
    get_available_disk_space_impl(path)
}

// Convert a path to a NUL-terminated Vec<u8> suitable for use with C functions
// Copied from https://github.com/GuillaumeGomez/sysinfo/blob/master/src/utils.rs#L56
#[cfg(not(any(target_os = "windows", target_os = "unknown", target_arch = "wasm32")))]
pub fn to_cpath(path: &Path) -> Vec<u8> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path_os: &OsStr = path.as_ref();
    let mut cpath = path_os.as_bytes().to_vec();
    cpath.push(0);
    cpath
}

// Copied from https://github.com/GuillaumeGomez/sysinfo/blob/190b88b6c5c751c4a88b2d2e984357efd6949949/src/linux/disk.rs
#[cfg(any(target_os = "linux", target_os = "android"))]
fn get_available_disk_space_impl(path: &Path) -> Result<u64> {
    use libc::statvfs64;

    let mount_point_cpath = to_cpath(&path);
    let (stat, retval) = unsafe {
        let mut stat: statvfs64 = std::mem::zeroed();
        let retval = statvfs64(mount_point_cpath.as_ptr() as *const _, &mut stat);
        (stat, retval)
    };
    let success = 0 == retval;
    ensure!(success, errno::errno());
    Ok(u64::from(stat.f_bsize) * u64::from(stat.f_bavail))
}

// Copied from https://github.com/GuillaumeGomez/sysinfo/blob/master/src/apple/disk.rs#L54
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn get_available_disk_space_impl(path: &Path) -> Result<u64> {
    // macos doesn't seem to have statvfs64 yet and statvfs is implemented in terms of statfs ,
    // see "IMPLEMENTATION NOTES" at https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/statvfs.3.html).
    // But it has statfs and according to the libc crate, statfs is 64bit on macOS,
    // see https://github.com/rust-lang/libc/blob/c8539f5bc6061c41a56d13451297fb9d5a258c59/src/unix/bsd/apple/mod.rs#L610
    // Possible because it directly links to it: https://github.com/rust-lang/libc/blob/c8539f5bc6061c41a56d13451297fb9d5a258c59/src/unix/bsd/apple/mod.rs#L3580
    // So let's use statfs.
    use libc::statfs;

    let mount_point_cpath = to_cpath(&path);
    let (stat, retval) = unsafe {
        let mut stat: statfs = std::mem::zeroed();
        let retval = statfs(mount_point_cpath.as_ptr() as *const _, &mut stat);
        (stat, retval)
    };
    let success = 0 == retval;
    ensure!(success, errno::errno());
    Ok(u64::from(stat.f_bsize) * u64::from(stat.f_bavail))
}

// TODO Run tests on Windows
// Copied from https://github.com/GuillaumeGomez/sysinfo/blob/master/src/apple/disk.rs#L54
#[cfg(target_os = "windows")]
fn get_available_disk_space_impl(path: &Path) -> Result<u64> {
    use winapi::um::fileapi::GetDiskFreeSpaceExW;
    use winapi::um::winnt::ULARGE_INTEGER;

    let (size, retval) = unsafe {
        let mut size: ULARGE_INTEGER = std::mem::zeroed();
        let retval = GetDiskFreeSpaceExW(
            path.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut size,
        );
        (stat, retval)
    };
    let success = 0 != retval;
    ensure!(success, errno::errno());
    Ok(*size.QuadPart())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn given_existing_path_when_querying_available_disk_space_then_succeeds() {
        let tempdir = TempDir::new("").unwrap();
        let path = tempdir.path();
        let size =
            get_available_disk_space(&path).expect("Expect get_available_disk_space to succeed");
        assert!(size > 0, "Expect size to be larger than zero");
    }

    #[test]
    fn given_nonexisting_path_when_querying_available_disk_space_then_fails() {
        let tempdir = TempDir::new("").unwrap();
        let path = tempdir.path().join("notexisting");
        let error = get_available_disk_space(&path)
            .unwrap_err()
            .downcast::<errno::Errno>()
            .expect("Expect get_available_disk_space to return an errno error");
        const ERRNO_ENOENT: i32 = 2;
        assert_eq!(errno::Errno(ERRNO_ENOENT), error);
    }
}
