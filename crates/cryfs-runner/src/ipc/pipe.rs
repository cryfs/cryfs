use anyhow::{Result, bail};
use interprocess::os::unix::unnamed_pipe::UnnamedPipeExt;
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::{Read, Write},
    marker::PhantomData,
    os::fd::AsFd,
    time::{Duration, Instant},
};

/// Maximum message size (1 MiB). Protects against DoS from malicious/buggy senders.
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Create a new pipe that can be used across forking for interprocess communication.
///
/// T: The type of the data that will be sent through the pipe.
pub fn pipe<T>() -> Result<(Sender<T>, Receiver<T>)>
where
    T: Serialize + DeserializeOwned,
{
    let (sender, recver) = interprocess::unnamed_pipe::pipe()?;
    Ok((Sender::new(sender), Receiver::new(recver)))
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

    pub fn send(&mut self, data: &T) -> Result<()> {
        let bytes = postcard::to_stdvec(data)?;
        if bytes.len() > MAX_MESSAGE_SIZE {
            bail!(
                "Message size {} exceeds maximum {MAX_MESSAGE_SIZE}",
                bytes.len()
            );
        }
        let len = bytes.len() as u32;
        self.sender.write_all(&len.to_le_bytes())?;
        self.sender.write_all(&bytes)?;
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

    pub fn recv(&mut self) -> Result<T> {
        self.recver.set_nonblocking(false)?;
        let mut len_bytes = [0u8; 4];
        self.recver.read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        if len > MAX_MESSAGE_SIZE {
            bail!("Message size {len} exceeds maximum {MAX_MESSAGE_SIZE}");
        }
        let mut buf = vec![0u8; len];
        self.recver.read_exact(&mut buf)?;
        Ok(postcard::from_bytes(&buf)?)
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
    use std::thread;

    #[test]
    fn dropped_recver() {
        let (mut sender, recver) = pipe::<u32>().unwrap();
        drop(recver);
        assert!(sender.send(&42).is_err());
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
}
