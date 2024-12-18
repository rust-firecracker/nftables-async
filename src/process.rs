use std::future::Future;
#[cfg(any(feature = "tokio-process", feature = "async-process"))]
use std::process::Stdio;

use futures_util::AsyncWrite;
#[cfg(feature = "tokio-process")]
use tokio_util::compat::TokioAsyncWriteCompatExt;

/// A process backend to use for asynchronous I/O, supporting only the functionality needed by
/// the nftables-async crate.
pub trait Process: Send + Sized {
    type Stdin: AsyncWrite + Send + Unpin;

    fn spawn(program: &str, args: Vec<&str>, pipe_output: bool) -> Result<Self, std::io::Error>;

    fn output(
        program: &str,
        args: Vec<&str>,
    ) -> impl Future<Output = Result<std::process::Output, std::io::Error>> + Send;

    fn take_stdin(&mut self) -> Option<Self::Stdin>;

    fn wait_with_output(
        self,
    ) -> impl Future<Output = Result<std::process::Output, std::io::Error>> + Send;
}

/// A [Process] implementation using the tokio crate for I/O.
#[cfg(feature = "tokio-process")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-process")))]
pub struct TokioProcess(tokio::process::Child);

#[cfg(feature = "tokio-process")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-process")))]
impl Process for TokioProcess {
    type Stdin = tokio_util::compat::Compat<tokio::process::ChildStdin>;

    fn spawn(program: &str, args: Vec<&str>, pipe_output: bool) -> Result<Self, std::io::Error> {
        let mut command = tokio::process::Command::new(program);

        command.args(args);

        if pipe_output {
            command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped());
        }

        command.spawn().map(Self)
    }

    fn output(
        program: &str,
        args: Vec<&str>,
    ) -> impl Future<Output = Result<std::process::Output, std::io::Error>> + Send {
        let mut command = tokio::process::Command::new(program);
        command.args(args).output()
    }

    fn take_stdin(&mut self) -> Option<Self::Stdin> {
        self.0.stdin.take().map(|stdin| stdin.compat_write())
    }

    fn wait_with_output(
        self,
    ) -> impl Future<Output = Result<std::process::Output, std::io::Error>> + Send {
        self.0.wait_with_output()
    }
}

/// A [Process] implementation using the async-process crate for I/O.
#[cfg(feature = "async-process")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-process")))]
pub struct AsyncProcess(async_process::Child);

#[cfg(feature = "async-process")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-process")))]
impl Process for AsyncProcess {
    type Stdin = async_process::ChildStdin;

    fn spawn(program: &str, args: Vec<&str>, pipe_output: bool) -> Result<Self, std::io::Error> {
        let mut command = async_process::Command::new(program);
        command.args(args);

        if pipe_output {
            command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped());
        }

        command.spawn().map(Self)
    }

    fn output(
        program: &str,
        args: Vec<&str>,
    ) -> impl Future<Output = Result<std::process::Output, std::io::Error>> + Send {
        let mut command = async_process::Command::new(program);
        command.args(args).output()
    }

    fn take_stdin(&mut self) -> Option<Self::Stdin> {
        self.0.stdin.take()
    }

    fn wait_with_output(
        self,
    ) -> impl Future<Output = Result<std::process::Output, std::io::Error>> + Send {
        self.0.output()
    }
}
