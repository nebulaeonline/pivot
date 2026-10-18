use std::{ffi::c_void, io, os::windows::io::AsRawHandle, ptr};
use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, LocalFree},
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        },
        GetTokenInformation, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
    },
    System::{
        Pipes::{GetNamedPipeClientProcessId, GetNamedPipeServerProcessId},
        Threading::{
            GetCurrentProcess, GetCurrentProcessId, OpenProcess, OpenProcessToken,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
};

fn sid_for_process(pid: u32) -> io::Result<String> {
    // SAFETY: all Windows handles and allocated buffers remain live until each
    // call completes. Token data is pointer-aligned; SID text is LocalFree-owned.
    unsafe {
        let process = if pid == GetCurrentProcessId() {
            GetCurrentProcess()
        } else {
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)
        };
        if process.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut token: HANDLE = ptr::null_mut();
        let opened = OpenProcessToken(process, TOKEN_QUERY, &mut token);
        if pid != GetCurrentProcessId() {
            CloseHandle(process);
        }
        if opened == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut size = 0;
        GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut size);
        let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
        let ok = GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            size,
            &mut size,
        );
        CloseHandle(token);
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
        let mut text: *mut u16 = ptr::null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut text) == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut length = 0;
        while *text.add(length) != 0 {
            length += 1;
        }
        let sid = String::from_utf16_lossy(std::slice::from_raw_parts(text, length));
        LocalFree(text.cast());
        Ok(sid)
    }
}
fn current_sid() -> io::Result<String> {
    sid_for_process(unsafe { GetCurrentProcessId() })
}
pub fn default_endpoint() -> io::Result<String> {
    Ok(format!(r"\\.\pipe\pivot-{}", current_sid()?))
}
fn validate_endpoint(endpoint: &str) -> io::Result<()> {
    let name = endpoint.strip_prefix(r"\\.\pipe\").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "only local named pipes are accepted",
        )
    })?;
    if name.is_empty()
        || name.len() > 180
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid pipe name",
        ));
    }
    Ok(())
}
fn create(endpoint: &str, first: bool) -> io::Result<NamedPipeServer> {
    validate_endpoint(endpoint)?;
    let sddl: Vec<_> = format!("D:P(A;;GA;;;{})", current_sid()?)
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: the descriptor is valid until CreateNamedPipe completes; Windows
    // copies it. A protected DACL grants only the current user's SID access.
    unsafe {
        let mut descriptor: *mut c_void = ptr::null_mut();
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            ptr::null_mut(),
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        let mut attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        let result = ServerOptions::new()
            .first_pipe_instance(first)
            .reject_remote_clients(true)
            .create_with_security_attributes_raw(
                endpoint,
                (&mut attributes as *mut SECURITY_ATTRIBUTES).cast(),
            );
        LocalFree(descriptor);
        result
    }
}
pub struct Listener {
    endpoint: String,
    pending: NamedPipeServer,
}
pub type Stream = NamedPipeClient;
impl Listener {
    pub fn bind(endpoint: &str) -> io::Result<Self> {
        Ok(Self {
            endpoint: endpoint.into(),
            pending: create(endpoint, true)?,
        })
    }
    pub async fn accept(&mut self) -> io::Result<NamedPipeServer> {
        if let Err(error) = self.pending.connect().await {
            // An opener can disappear before ConnectNamedPipe completes. Reset
            // that instance so it cannot poison the listener for later peers.
            let _ = self.pending.disconnect();
            return Err(error);
        }
        // Keep an instance alive to prevent endpoint replacement between clients.
        let next = create(&self.endpoint, false)?;
        let connected = std::mem::replace(&mut self.pending, next);
        let mut pid = 0;
        if unsafe { GetNamedPipeClientProcessId(connected.as_raw_handle().cast(), &mut pid) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if sid_for_process(pid)? != current_sid()? {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "peer user mismatch",
            ));
        }
        Ok(connected)
    }
}
pub async fn connect(endpoint: &str) -> io::Result<Stream> {
    validate_endpoint(endpoint)?;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    let stream = loop {
        // Security identification prevents an untrusted same-user server from
        // obtaining a higher impersonation level through this client handle.
        match ClientOptions::new()
            .security_qos_flags(0x00010000)
            .open(endpoint)
        {
            Ok(stream) => break stream,
            Err(e)
                if matches!(e.raw_os_error(), Some(2 | 231))
                    && tokio::time::Instant::now() < deadline =>
            {
                tokio::time::sleep(std::time::Duration::from_millis(25)).await
            }
            Err(e) => return Err(e),
        }
    };
    let mut pid = 0;
    if unsafe { GetNamedPipeServerProcessId(stream.as_raw_handle().cast(), &mut pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if sid_for_process(pid)? != current_sid()? {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "server user mismatch",
        ));
    }
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Security::{
        Authorization::{
            ConvertSecurityDescriptorToStringSecurityDescriptorW, GetSecurityInfo, SE_KERNEL_OBJECT,
        },
        DACL_SECURITY_INFORMATION,
    };
    #[tokio::test]
    async fn actual_pipe_acl_is_current_user_only_and_first_instance_is_exclusive() {
        let endpoint = format!(r"\\.\pipe\pivot-acl-test-{}", std::process::id());
        let listener = Listener::bind(&endpoint).unwrap();
        assert!(Listener::bind(&endpoint).is_err());
        // SAFETY: GetSecurityInfo allocates a self-contained descriptor; both it
        // and the converted string are released with LocalFree after inspection.
        unsafe {
            let mut descriptor = ptr::null_mut();
            assert_eq!(
                GetSecurityInfo(
                    listener.pending.as_raw_handle().cast(),
                    SE_KERNEL_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut descriptor
                ),
                0
            );
            let mut text: *mut u16 = ptr::null_mut();
            assert_ne!(
                ConvertSecurityDescriptorToStringSecurityDescriptorW(
                    descriptor,
                    1,
                    DACL_SECURITY_INFORMATION,
                    &mut text,
                    ptr::null_mut()
                ),
                0
            );
            let mut length = 0;
            while *text.add(length) != 0 {
                length += 1;
            }
            let sddl = String::from_utf16_lossy(std::slice::from_raw_parts(text, length));
            LocalFree(text.cast());
            LocalFree(descriptor);
            assert!(sddl.starts_with("D:P"), "{sddl}");
            assert_eq!(sddl.matches("(A;").count(), 1, "{sddl}");
            assert!(sddl.contains(&current_sid().unwrap()), "{sddl}");
        }
        drop(listener);
        assert!(Listener::bind(&endpoint).is_ok());
        assert!(validate_endpoint(r"\\remote\pipe\pivot").is_err());
    }
}
