use std::{
    fs::{File, Metadata, OpenOptions},
    io,
    path::Path,
};

#[cfg(unix)]
pub(super) fn private_new(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}
#[cfg(unix)]
pub(super) fn open_read(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
}
#[cfg(unix)]
pub(super) fn open_lock(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
}
#[cfg(unix)]
pub(super) fn identity(file: &File) -> io::Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let m = file.metadata()?;
    if !m.is_file() || m.nlink() != 1 || m.uid() != unsafe { libc::geteuid() } {
        return Err(io::Error::other(
            "requires a current-user-owned regular file with one link",
        ));
    }
    Ok((m.dev(), m.ino()))
}
#[cfg(unix)]
pub(super) fn mode(m: &Metadata) -> u32 {
    use std::os::unix::fs::MetadataExt;
    m.mode() & 0o777
}
#[cfg(unix)]
pub(super) fn set_mode(file: &File, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(std::fs::Permissions::from_mode(mode))
}
#[cfg(unix)]
pub(super) fn sync_parent(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

#[cfg(windows)]
pub(super) fn private_new(path: &Path) -> io::Result<File> {
    use std::{
        os::windows::{ffi::OsStrExt, io::FromRawHandle},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE, LocalFree},
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            },
            SECURITY_ATTRIBUTES,
        },
        Storage::FileSystem::{
            CREATE_NEW, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_DELETE, FILE_SHARE_READ,
            FILE_SHARE_WRITE,
        },
    };
    let path: Vec<_> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let sddl: Vec<_> = "D:P(A;;FA;;;OW)".encode_utf16().chain(Some(0)).collect();
    let mut descriptor = ptr::null_mut();
    // Create with an owner-rights-only protected DACL, not inherited public access.
    // Descriptor and path remain alive through CreateFileW; File takes ownership.
    unsafe {
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            ptr::null_mut(),
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        let attrs = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        let handle = CreateFileW(
            path.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            &attrs,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        );
        let error = io::Error::last_os_error();
        LocalFree(descriptor);
        if handle == INVALID_HANDLE_VALUE {
            return Err(error);
        }
        Ok(File::from_raw_handle(handle))
    }
}
#[cfg(windows)]
pub(super) fn open_read(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).open(path)
}
#[cfg(windows)]
pub(super) fn open_lock(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}
#[cfg(windows)]
pub(super) fn identity(file: &File) -> io::Result<(u64, u64)> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
    };
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if !file.metadata()?.is_file() || info.nNumberOfLinks != 1 {
        return Err(io::Error::other("requires a regular file with one link"));
    }
    Ok((
        info.dwVolumeSerialNumber as u64,
        ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64,
    ))
}
#[cfg(windows)]
pub(super) fn mode(m: &Metadata) -> u32 {
    u32::from(m.permissions().readonly())
}
#[cfg(windows)]
pub(super) fn set_mode(file: &File, mode: u32) -> io::Result<()> {
    let mut p = file.metadata()?.permissions();
    p.set_readonly(mode != 0);
    file.set_permissions(p)
}
#[cfg(windows)]
pub(super) fn sync_parent(_path: &Path) -> io::Result<()> {
    // Staged bytes were FlushFileBuffers'd by sync_all. No portable Windows
    // directory-fsync claim; process-crash proof only, no power-loss guarantee.
    Ok(())
}
