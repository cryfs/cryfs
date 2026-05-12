use anyhow::{Result, bail};
use interprocess::os::unix::unnamed_pipe::UnnamedPipeExt;
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::{Read, Write},
    marker::PhantomData,
    os::fd::{AsFd, AsRawFd},
    time::{Duration, Instant},
};
use tokio::runtime::Handle;

/// Maximum message size (1 MiB). Protects against DoS from malicious/buggy senders.
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Create a new pipe that can be used across forking for interprocess communication.
///
/// Both ends are set CLOEXEC so they're closed by the kernel during `execve`.
/// The fork+exec daemon spawn relies on this: only fds explicitly remapped via
/// `posix_spawn_file_actions` / `pre_exec` (which clear CLOEXEC as a side
/// effect of `dup2`) survive into the child. The underlying `interprocess`
/// crate doesn't set CLOEXEC, so we have to.
///
/// # The CLOEXEC race and why we panic if tokio is running
///
/// The CLOEXEC set is *not* atomic with pipe creation: there's a brief window
/// between `interprocess::pipe()` returning and the `fcntl(F_SETFD)` call
/// below where the fds exist but are still inheritable. If a concurrent thread
/// `fork()`s (directly, or indirectly via `Command::spawn`) inside that
/// window, the resulting child inherits the still-non-CLOEXEC pipe fds.
/// Symptoms range from leaked fds in unrelated children to EOF never being
/// delivered on the pipe (the rightful owner can't detect the other end being
/// dropped because the unrelated child holds a duplicate).
///
/// On Linux/FreeBSD/OpenBSD/NetBSD≥6, `pipe2(O_CLOEXEC)` creates the pipe
/// atomically with CLOEXEC set and the race doesn't exist. **macOS has no
/// `pipe2`, no `SOCK_CLOEXEC` for `socketpair`, and no equivalent atomic
/// primitive.** The standard workaround on macOS would be a process-wide fork
/// lock that every fork site honors (CPython's `subprocess` does this with
/// `_posixsubprocess._fork_lock`), but Rust's `std::process::Command` doesn't
/// expose or honor any such lock, so we can't enforce it across our
/// dependencies.
///
/// We therefore rely on a usage-level invariant rather than atomicity: pipe
/// creation must be single-threaded. We enforce this by panicking if a tokio
/// runtime is already initialized, since tokio's worker threads are the
/// realistic source of "another thread that might fork" in this codebase. The
/// daemon spawn path naturally satisfies this — pipes are created at startup,
/// before tokio is constructed.
///
/// If we ever want to move pipe creation after tokio init, we have to either
/// reach a process-wide fork-lock arrangement, or accept the race on macOS.
/// Switching to `pipe2(O_CLOEXEC)` would close the race on Linux but leaves
/// macOS unchanged.
///
/// T: The type of the data that will be sent through the pipe.
pub fn pipe<T>() -> Result<(Sender<T>, Receiver<T>)>
where
    T: Serialize + DeserializeOwned,
{
    if Handle::try_current().is_ok() {
        panic!(
            "Cannot create an IPC pipe while a tokio runtime is running. \
             Pipe creation must be single-threaded because the CLOEXEC flag \
             is not set atomically with pipe creation (and macOS has no \
             portable atomic alternative); a concurrent fork would inherit \
             the not-yet-CLOEXEC fds. Create the pipe before initializing \
             tokio."
        );
    }
    let (sender, recver) = interprocess::unnamed_pipe::pipe()?;
    set_cloexec(sender.as_raw_fd())?;
    set_cloexec(recver.as_raw_fd())?;
    Ok((Sender::new(sender), Receiver::new(recver)))
}

fn set_cloexec(fd: std::os::fd::RawFd) -> Result<()> {
    // SAFETY: `fd` is an open file descriptor returned by `pipe()` and still
    // owned by the `interprocess` wrapper, so it's valid for the duration of
    // both fcntl calls. F_GETFD has no side effects beyond returning flags.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        bail!("fcntl(F_GETFD) failed: {}", std::io::Error::last_os_error());
    }
    // SAFETY: Same as above. F_SETFD only modifies the descriptor's flags
    // (we OR in FD_CLOEXEC, preserving any others); it doesn't affect the
    // underlying file or pipe state.
    if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        bail!("fcntl(F_SETFD) failed: {}", std::io::Error::last_os_error());
    }
    Ok(())
}

pub struct Sender<T>
where
    T: Serialize + DeserializeOwned,
{
    sender: interprocess::unnamed_pipe::Sender,
    _p: PhantomData<T>,
}

impl<T> Sender<T>
where
    T: Serialize + DeserializeOwned,
{
    fn new(sender: interprocess::unnamed_pipe::Sender) -> Self {
        Self {
            sender,
            _p: PhantomData,
        }
    }

    /// Construct a typed `Sender` from a raw owned file descriptor that the
    /// caller has verified is the write end of a pipe inherited across `execve`.
    /// Used by the fork+exec daemon child to rebuild its `RpcServer`.
    pub fn from_owned_fd(fd: std::os::fd::OwnedFd) -> Self {
        Self::new(interprocess::unnamed_pipe::Sender::from(fd))
    }

    /// Surrender the typed wrapper and recover the underlying owned file
    /// descriptor. Used to `dup2` the fd onto a fixed slot in a child process.
    pub fn into_owned_fd(self) -> std::os::fd::OwnedFd {
        std::os::fd::OwnedFd::from(self.sender)
    }

    pub fn send(&mut self, data: &T) -> Result<()> {
        let bytes = postcard::to_stdvec(data)?;
        self.write_length_prefixed(&bytes)
    }

    /// Send a length-prefixed raw byte payload without postcard encoding.
    /// Used for the build-id handshake before typed RPC begins: encoding the
    /// handshake via postcard would defeat its purpose of validating that
    /// parent and child agree on the postcard schema.
    pub(crate) fn send_raw(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_length_prefixed(bytes)
    }

    fn write_length_prefixed(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > MAX_MESSAGE_SIZE {
            bail!(
                "Message size {} exceeds maximum {MAX_MESSAGE_SIZE}",
                bytes.len()
            );
        }
        let len = bytes.len() as u32;
        self.sender.write_all(&len.to_le_bytes())?;
        self.sender.write_all(bytes)?;
        Ok(())
    }
}

pub struct Receiver<T>
where
    T: Serialize + DeserializeOwned,
{
    recver: interprocess::unnamed_pipe::Recver,
    _p: PhantomData<T>,
}

impl<T> Receiver<T>
where
    T: Serialize + DeserializeOwned,
{
    fn new(recver: interprocess::unnamed_pipe::Recver) -> Self {
        Self {
            recver,
            _p: PhantomData,
        }
    }

    /// Construct a typed `Receiver` from a raw owned file descriptor that the
    /// caller has verified is the read end of a pipe inherited across `execve`.
    /// Used by the fork+exec daemon child to rebuild its `RpcServer`.
    pub fn from_owned_fd(fd: std::os::fd::OwnedFd) -> Self {
        Self::new(interprocess::unnamed_pipe::Recver::from(fd))
    }

    /// Surrender the typed wrapper and recover the underlying owned file
    /// descriptor. Used to `dup2` the fd onto a fixed slot in a child process.
    pub fn into_owned_fd(self) -> std::os::fd::OwnedFd {
        std::os::fd::OwnedFd::from(self.recver)
    }

    pub fn recv(&mut self) -> Result<T> {
        let buf = self.read_length_prefixed()?;
        Ok(postcard::from_bytes(&buf)?)
    }

    /// Receive a length-prefixed raw byte payload without postcard decoding.
    /// Used for the build-id handshake before typed RPC begins.
    pub(crate) fn recv_raw(&mut self) -> Result<Vec<u8>> {
        self.read_length_prefixed()
    }

    fn read_length_prefixed(&mut self) -> Result<Vec<u8>> {
        self.recver.set_nonblocking(false)?;
        let mut len_bytes = [0u8; 4];
        self.recver.read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        if len > MAX_MESSAGE_SIZE {
            bail!("Message size {len} exceeds maximum {MAX_MESSAGE_SIZE}");
        }
        let mut buf = vec![0u8; len];
        self.recver.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn recv_timeout(&mut self, timeout: Duration) -> Result<T>
    where
        T: Send,
    {
        self.recver.set_nonblocking(true)?;
        let timeout_at = Instant::now() + timeout;

        let mut len_bytes = [0u8; 4];
        read_exact_with_timeout(&mut self.recver, &mut len_bytes, timeout_at)?;

        let len = u32::from_le_bytes(len_bytes) as usize;
        if len > MAX_MESSAGE_SIZE {
            bail!("Message size {len} exceeds maximum {MAX_MESSAGE_SIZE}");
        }
        let mut buf = vec![0u8; len];
        read_exact_with_timeout(&mut self.recver, &mut buf, timeout_at)?;

        Ok(postcard::from_bytes(&buf)?)
    }
}

fn read_exact_with_timeout<R: Read + AsFd>(
    reader: &mut R,
    buf: &mut [u8],
    timeout_at: Instant,
) -> Result<()> {
    let mut bytes_read = 0;
    while bytes_read < buf.len() {
        match reader.read(&mut buf[bytes_read..]) {
            Ok(0) => bail!("Sender closed the pipe"),
            Ok(n) => bytes_read += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Wait for data using poll() instead of busy-waiting
                loop {
                    let remaining = timeout_at.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        bail!("Timeout in ipc::Receiver::recv_timeout");
                    }

                    let poll_fd = PollFd::new(reader.as_fd(), PollFlags::POLLIN);
                    let timeout_ms: u16 = remaining.as_millis().try_into().unwrap_or(u16::MAX);
                    match poll(&mut [poll_fd], PollTimeout::from(timeout_ms)) {
                        Ok(0) => bail!("Timeout in ipc::Receiver::recv_timeout"),
                        Ok(_) => break, // Data available, retry read
                        Err(nix::errno::Errno::EINTR) => continue, // Interrupted, retry poll
                        Err(e) => bail!("poll error: {e}"),
                    }
                }
            }
            Err(e) => bail!(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::os::fd::AsRawFd;
    use std::thread;

    #[test]
    fn dropped_recver() {
        let (mut sender, recver) = pipe::<u32>().unwrap();
        drop(recver);
        assert!(sender.send(&42).is_err());
    }

    #[test]
    fn pipe_ends_have_cloexec_set() {
        // Both ends of pipes created by our `pipe()` wrapper must have
        // FD_CLOEXEC set, so they're closed automatically by the kernel when
        // the cryfs daemon child execs the new binary. The underlying
        // `interprocess` crate does not set CLOEXEC, so we set it ourselves
        // right after pipe creation.
        let (sender, recver) = pipe::<u32>().unwrap();
        for (label, raw_fd) in [
            ("sender", sender.sender.as_raw_fd()),
            ("recver", recver.recver.as_raw_fd()),
        ] {
            let flags = unsafe { libc::fcntl(raw_fd, libc::F_GETFD) };
            assert!(flags >= 0, "fcntl(F_GETFD) failed for {label}");
            assert!(
                flags & libc::FD_CLOEXEC != 0,
                "{label} end of pipe is missing FD_CLOEXEC (flags={flags:#x})",
            );
        }
    }

    mod recv {
        use super::*;

        #[test]
        fn primitive_u32() {
            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            sender.send(&42).unwrap();
            assert_eq!(recver.recv().unwrap(), 42);
        }

        #[test]
        fn string() {
            let (mut sender, mut recver) = pipe::<String>().unwrap();
            sender.send(&"Hello, World!".to_string()).unwrap();
            assert_eq!(recver.recv().unwrap(), "Hello, World!");
        }

        #[test]
        fn custom_struct() {
            #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
            struct MyStruct {
                a: u32,
                b: String,
            }

            let (mut sender, mut recver) = pipe::<MyStruct>().unwrap();
            sender
                .send(&MyStruct {
                    a: 42,
                    b: "Hello, World!".to_string(),
                })
                .unwrap();
            assert_eq!(
                recver.recv().unwrap(),
                MyStruct {
                    a: 42,
                    b: "Hello, World!".to_string()
                }
            );
        }

        #[test]
        fn dropped_sender() {
            let (sender, mut recver) = pipe::<u32>().unwrap();
            drop(sender);
            assert!(recver.recv().is_err());
        }

        #[test]
        fn blocks_until_it_gets_data() {
            // TODO Can we make this test deterministic?

            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            let recv_thread = thread::spawn(move || {
                thread::sleep(Duration::from_secs(1));
                sender.send(&42).unwrap();
            });
            assert_eq!(recver.recv().unwrap(), 42);
            recv_thread.join().unwrap();
        }
    }

    mod recv_timeout {
        // TODO Make these tests deterministic by mocking the clock (but do it without affecting global state or time for other tests)

        use super::*;

        #[test]
        fn primitive_u32() {
            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            sender.send(&42).unwrap();
            assert_eq!(recver.recv_timeout(Duration::from_secs(1)).unwrap(), 42);
        }

        #[test]
        fn string() {
            let (mut sender, mut recver) = pipe::<String>().unwrap();
            sender.send(&"Hello, World!".to_string()).unwrap();
            assert_eq!(
                recver.recv_timeout(Duration::from_secs(1)).unwrap(),
                "Hello, World!"
            );
        }

        #[test]
        fn custom_struct() {
            #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
            struct MyStruct {
                a: u32,
                b: String,
            }

            let (mut sender, mut recver) = pipe::<MyStruct>().unwrap();
            sender
                .send(&MyStruct {
                    a: 42,
                    b: "Hello, World!".to_string(),
                })
                .unwrap();
            assert_eq!(
                recver.recv_timeout(Duration::from_secs(1)).unwrap(),
                MyStruct {
                    a: 42,
                    b: "Hello, World!".to_string()
                }
            );
        }

        #[test]
        fn dropped_sender() {
            let (sender, mut recver) = pipe::<u32>().unwrap();
            drop(sender);
            let error = recver.recv_timeout(Duration::from_secs(1)).unwrap_err();
            assert!(
                error.to_string().contains("Sender closed the pipe"),
                "Unexpected error: {:?}",
                error,
            );
        }

        #[test]
        fn blocks_until_it_gets_data_if_within_timeout() {
            // TODO Can we make this test deterministic?

            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            let recv_thread = thread::spawn(move || {
                thread::sleep(Duration::from_secs(1));
                sender.send(&42).unwrap();
            });
            assert_eq!(recver.recv_timeout(Duration::from_secs(10)).unwrap(), 42);
            recv_thread.join().unwrap();
        }

        #[test]
        fn timeout() {
            let (_sender, mut recver) = pipe::<u32>().unwrap();
            let response = recver.recv_timeout(Duration::from_secs(1));
            let error = response.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("Timeout in ipc::Receiver::recv_timeout"),
                "Unexpected error: {:?}",
                error,
            );
        }

        #[test]
        fn zero_timeout_with_data_ready() {
            // Data already in pipe, zero timeout should still succeed
            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            sender.send(&42).unwrap();
            assert_eq!(recver.recv_timeout(Duration::ZERO).unwrap(), 42);
        }

        #[test]
        fn zero_timeout_without_data() {
            // No data, zero timeout should fail immediately
            let (_sender, mut recver) = pipe::<u32>().unwrap();
            let error = recver.recv_timeout(Duration::ZERO).unwrap_err();
            assert!(
                error.to_string().contains("Timeout"),
                "Unexpected error: {:?}",
                error,
            );
        }

        #[test]
        fn very_short_timeout_without_data() {
            // Very short timeout (1ms) without data
            let (_sender, mut recver) = pipe::<u32>().unwrap();
            let start = Instant::now();
            let error = recver.recv_timeout(Duration::from_millis(1)).unwrap_err();
            let elapsed = start.elapsed();
            assert!(
                error.to_string().contains("Timeout"),
                "Unexpected error: {:?}",
                error,
            );
            // Should complete quickly, not hang
            assert!(elapsed < Duration::from_secs(1));
        }

        #[test]
        fn large_message() {
            // Large message that may require multiple read chunks
            // Note: pipe buffers are typically 64KB, so we need to send/recv concurrently
            let (mut sender, mut recver) = pipe::<Vec<u8>>().unwrap();
            let large_data: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
            let expected = large_data.clone();

            // Send in a separate thread to avoid blocking on full pipe buffer
            let send_thread = thread::spawn(move || {
                sender.send(&large_data).unwrap();
            });

            let received = recver.recv_timeout(Duration::from_secs(5)).unwrap();
            send_thread.join().unwrap();
            assert_eq!(received, expected);
        }

        #[test]
        fn multiple_sequential_messages() {
            // Multiple messages in sequence
            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            for i in 0..10 {
                sender.send(&i).unwrap();
            }
            for i in 0..10 {
                assert_eq!(recver.recv_timeout(Duration::from_secs(1)).unwrap(), i);
            }
        }

        #[test]
        fn timeout_waiting_for_length_bytes() {
            // Sender sends nothing, timeout waiting for length prefix
            // This is essentially the same as the `timeout` test but with explicit timing check
            let (_sender, mut recver) = pipe::<u32>().unwrap();
            let start = Instant::now();
            let error = recver.recv_timeout(Duration::from_millis(50)).unwrap_err();
            let elapsed = start.elapsed();
            assert!(
                error.to_string().contains("Timeout"),
                "Unexpected error: {:?}",
                error,
            );
            // Verify timeout was respected (within reasonable margin)
            // Use >= 40ms to account for timing jitter
            assert!(
                elapsed >= Duration::from_millis(40),
                "Timeout returned too quickly: {:?}",
                elapsed
            );
            assert!(
                elapsed < Duration::from_millis(500),
                "Timeout took too long: {:?}",
                elapsed
            );
        }

        #[test]
        fn timeout_waiting_for_payload() {
            // Sender sends length but not payload - tests timeout during payload read
            use interprocess::unnamed_pipe::pipe as raw_pipe;
            use std::io::Write;

            let (mut raw_sender, raw_recver) = raw_pipe().unwrap();
            let mut recver: Receiver<u32> = Receiver::new(raw_recver);

            // Send only the length prefix (4 bytes), not the payload
            let fake_len: u32 = 100;
            raw_sender.write_all(&fake_len.to_le_bytes()).unwrap();

            // Keep sender alive to prevent EOF
            let _keep_sender = raw_sender;

            let start = Instant::now();
            let error = recver.recv_timeout(Duration::from_millis(50)).unwrap_err();
            let elapsed = start.elapsed();
            assert!(
                error.to_string().contains("Timeout"),
                "Unexpected error: {:?}",
                error,
            );
            // Use >= 40ms to account for timing jitter
            assert!(
                elapsed >= Duration::from_millis(40),
                "Timeout returned too quickly: {:?}",
                elapsed
            );
        }

        #[test]
        fn sender_closes_after_partial_length() {
            // Sender sends partial length then closes
            use interprocess::unnamed_pipe::pipe as raw_pipe;
            use std::io::Write;

            let (mut raw_sender, raw_recver) = raw_pipe().unwrap();
            let mut recver: Receiver<u32> = Receiver::new(raw_recver);

            // Send only 2 of 4 length bytes, then close
            raw_sender.write_all(&[1, 2]).unwrap();
            drop(raw_sender);

            let error = recver.recv_timeout(Duration::from_secs(1)).unwrap_err();
            assert!(
                error.to_string().contains("Sender closed the pipe"),
                "Unexpected error: {:?}",
                error,
            );
        }

        #[test]
        fn sender_closes_after_partial_payload() {
            // Sender sends length + partial payload then closes
            use interprocess::unnamed_pipe::pipe as raw_pipe;
            use std::io::Write;

            let (mut raw_sender, raw_recver) = raw_pipe().unwrap();
            let mut recver: Receiver<Vec<u8>> = Receiver::new(raw_recver);

            // Send length indicating 100 bytes, but only send 10
            let len: u32 = 100;
            raw_sender.write_all(&len.to_le_bytes()).unwrap();
            raw_sender.write_all(&[0u8; 10]).unwrap();
            drop(raw_sender);

            let error = recver.recv_timeout(Duration::from_secs(1)).unwrap_err();
            assert!(
                error.to_string().contains("Sender closed the pipe"),
                "Unexpected error: {:?}",
                error,
            );
        }
    }

    mod raw {
        use super::*;

        #[test]
        fn roundtrip_short_payload() {
            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            sender.send_raw(b"hello").unwrap();
            assert_eq!(recver.recv_raw().unwrap(), b"hello");
        }

        #[test]
        fn roundtrip_empty_payload() {
            // Zero-length payload still goes over the wire as
            // [4-byte length=0] [0 bytes payload]. Receiver must complete.
            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            sender.send_raw(b"").unwrap();
            assert_eq!(recver.recv_raw().unwrap(), b"");
        }

        #[test]
        fn roundtrip_near_max_payload() {
            // A payload just under MAX_MESSAGE_SIZE must round-trip cleanly.
            // Send/recv concurrently so we don't deadlock against the pipe's
            // OS-level buffer (~64 KiB on Linux).
            let payload: Vec<u8> = (0..MAX_MESSAGE_SIZE - 4)
                .map(|i| (i % 251) as u8)
                .collect();
            let expected = payload.clone();
            let (mut sender, mut recver) = pipe::<u32>().unwrap();
            let send_thread = thread::spawn(move || {
                sender.send_raw(&payload).unwrap();
            });
            let received = recver.recv_raw().unwrap();
            send_thread.join().unwrap();
            assert_eq!(received, expected);
        }

        #[test]
        fn payload_over_max_size_rejected_on_send() {
            // We can't actually allocate the over-sized buffer cheaply, but
            // we can verify that `send_raw` enforces the same limit as
            // `send`: it bails before touching the underlying fd, so a
            // dummy peer-less sender suffices.
            let (sender, _recver) = interprocess::unnamed_pipe::pipe().unwrap();
            let mut sender: Sender<u32> = Sender::new(sender);
            let oversized = vec![0u8; MAX_MESSAGE_SIZE + 1];
            let err = sender.send_raw(&oversized).unwrap_err();
            assert!(
                err.to_string().contains("exceeds maximum"),
                "Unexpected error: {err:?}",
            );
        }

        #[test]
        fn dropped_sender_gives_eof_to_recv_raw() {
            let (sender, mut recver) = pipe::<u32>().unwrap();
            drop(sender);
            assert!(recver.recv_raw().is_err());
        }

        #[test]
        fn length_prefix_is_four_bytes_little_endian() {
            // Pin the wire format: a `send_raw` of N bytes writes exactly
            // 4+N bytes total, with the leading 4 bytes being the
            // little-endian u32 length. The fork+exec daemon child relies on
            // this format being stable across build_id mismatches (otherwise
            // the handshake check itself can't be validated).
            let (sender, mut raw_recver) = interprocess::unnamed_pipe::pipe().unwrap();
            let mut typed_sender: Sender<u32> = Sender::new(sender);
            typed_sender.send_raw(b"abc").unwrap();
            drop(typed_sender);
            let mut on_wire = Vec::new();
            raw_recver.read_to_end(&mut on_wire).unwrap();
            assert_eq!(on_wire, b"\x03\x00\x00\x00abc");
        }

        #[test]
        fn send_typed_then_recv_raw_observes_postcard_bytes() {
            // Encoding asymmetry: `send` postcard-encodes; `recv_raw`
            // returns the raw bytes that were sent. The receiver of a
            // build-id handshake therefore sees exactly what the sender
            // wrote, not postcard-decoded. Pin this with a value that
            // postcard would encode non-trivially.
            #[derive(Debug, Serialize, Deserialize)]
            struct Msg {
                a: u32,
                b: String,
            }
            let (mut sender, mut recver) = pipe::<Msg>().unwrap();
            sender
                .send(&Msg {
                    a: 0x42,
                    b: "hi".into(),
                })
                .unwrap();
            let raw = recver.recv_raw().unwrap();
            // postcard varint-encodes integers and length-prefixes the
            // string. We don't depend on the exact bytes here, just that
            // we got something non-empty back.
            assert!(!raw.is_empty());
        }
    }
}
