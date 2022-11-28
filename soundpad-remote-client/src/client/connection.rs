use super::Command;
use derivative::Derivative;
use tokio::{io, net::windows::named_pipe::NamedPipeClient, sync::mpsc, time};
use tracing::{info, instrument};

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Connection {
    #[derivative(Debug = "ignore")]
    pub(crate) rx: mpsc::Receiver<Command>,
    pub(crate) pipe: NamedPipeClient,
    pub(crate) debounce: core::time::Duration,
}

#[instrument]
pub(crate) async fn run_actor(mut connection: Connection) -> io::Result<()> {
    while let Some(command) = connection.rx.recv().await {
        info!("Starting to work on command");
        let _ = command.do_work(&mut connection).await;
    }
    Ok(())
}

impl Connection {
    pub(crate) async fn send(&self, command: impl AsRef<[u8]>) -> io::Result<()> {
        let bytes = command.as_ref();
        let mut written = 0;

        while written < bytes.len() {
            self.pipe.writable().await?;
            match self.pipe.try_write(&bytes[written..]) {
                Ok(amount) => written += amount,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn receive(&self) -> io::Result<String> {
        let mut response = String::new();
        let mut buf = [0u8; 512];

        loop {
            // Wait for the pipe to be readable
            self.pipe.readable().await?;

            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match self.pipe.try_read(&mut buf) {
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if n < buf.len() {
                        return Ok(response);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
