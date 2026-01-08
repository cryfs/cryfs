use anyhow::{Result, bail};
use interprocess::os::unix::unnamed_pipe::UnnamedPipeExt;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::{Read, Write},
    marker::PhantomData,
    thread,
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

fn read_exact_with_timeout(
    reader: &mut impl Read,
    buf: &mut [u8],
    timeout_at: Instant,
) -> Result<()> {
    let mut bytes_read = 0;
    while bytes_read < buf.len() {
        match reader.read(&mut buf[bytes_read..]) {
            Ok(0) => bail!("Sender closed the pipe"),
            Ok(n) => bytes_read += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= timeout_at {
                    bail!("Timeout in ipc::Receiver::recv_timeout");
                }
                thread::sleep(Duration::from_millis(1));
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
    }
}
