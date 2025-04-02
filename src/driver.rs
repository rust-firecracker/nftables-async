#[cfg(any(feature = "tokio-driver", feature = "async-process-driver"))]
use std::process::Stdio;
use std::{ffi::OsStr, future::Future, process::Output};

#[cfg(feature = "async-process-driver")]
use futures_lite::AsyncWriteExt as FuturesAsyncWriteExt;
#[cfg(feature = "tokio-driver")]
use tokio::io::AsyncWriteExt as TokioAsyncWriteExt;

/// A process driver to use for asynchronous I/O, supporting only the functionality needed by
/// the nftables-async crate.
pub trait Driver: Send + Sized {
    fn run_interactive(program: &OsStr, args: &[&OsStr]) -> Result<Self, std::io::Error>;

    fn run(
        program: &OsStr,
        args: &[&OsStr],
    ) -> impl Future<Output = Result<Output, std::io::Error>> + Send;

    fn fill_stdin(
        &mut self,
        payload: &[u8],
    ) -> impl Future<Output = Result<(), std::io::Error>> + Send;

    fn wait(self) -> impl Future<Output = Result<Output, std::io::Error>> + Send;
}

/// A [Driver] implementation using the tokio crate for I/O.
#[cfg(feature = "tokio-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-process")))]
pub struct TokioDriver(tokio::process::Child);

#[cfg(feature = "tokio-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-process")))]
impl Driver for TokioDriver {
    fn run_interactive(program: &OsStr, args: &[&OsStr]) -> Result<Self, std::io::Error> {
        tokio::process::Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
            .map(Self)
    }

    fn run(
        program: &OsStr,
        args: &[&OsStr],
    ) -> impl Future<Output = Result<Output, std::io::Error>> + Send {
        let mut command = tokio::process::Command::new(program);
        command.args(args).output()
    }

    async fn fill_stdin(&mut self, payload: &[u8]) -> Result<(), std::io::Error> {
        self.0
            .stdin
            .take()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Stdin not redirected successfully",
                )
            })?
            .write_all(payload)
            .await
    }

    fn wait(self) -> impl Future<Output = Result<Output, std::io::Error>> + Send {
        self.0.wait_with_output()
    }
}

/// A [Driver] implementation using the async-process crate for I/O.
#[cfg(feature = "async-process-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-process")))]
pub struct AsyncProcessDriver(async_process::Child);

#[cfg(feature = "async-process-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-process")))]
impl Driver for AsyncProcessDriver {
    fn run_interactive(program: &OsStr, args: &[&OsStr]) -> Result<Self, std::io::Error> {
        async_process::Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
            .map(Self)
    }

    fn run(
        program: &OsStr,
        args: &[&OsStr],
    ) -> impl Future<Output = Result<Output, std::io::Error>> + Send {
        let mut command = async_process::Command::new(program);
        command.args(args).output()
    }

    async fn fill_stdin(&mut self, payload: &[u8]) -> Result<(), std::io::Error> {
        self.0
            .stdin
            .take()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Stdin not redirected successfully",
                )
            })?
            .write_all(payload)
            .await
    }

    fn wait(self) -> impl Future<Output = Result<Output, std::io::Error>> + Send {
        self.0.output()
    }
}
