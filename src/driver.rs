use std::{ffi::OsStr, future::Future, process::Output};

#[cfg(feature = "async-process-driver")]
use futures_lite::AsyncWriteExt as FuturesAsyncWriteExt;
#[cfg(any(feature = "async-process-driver", feature = "tokio-driver"))]
use std::process::Stdio;
#[cfg(feature = "tokio-driver")]
use tokio::io::AsyncWriteExt as TokioAsyncWriteExt;

/// A process driver internally used by the helper to run the "nft" process asynchronously, write to its stdin and
/// retrieve its output.
pub trait Driver {
    /// Run the provided program with the provided arguments and retrieve its output.
    /// If stdin is [None], stdin should be nulled by the driver and no interaction should occur with the process.
    /// If stdin is [Some], stdin should be piped and the byte payload as well as a subsequent EOF (closure of the pipe)
    /// should be transmitted before the process is waited on.
    fn run_process(
        program: &OsStr,
        args: &[&OsStr],
        stdin: Option<&[u8]>,
    ) -> impl Future<Output = Result<Output, std::io::Error>> + Send;
}

/// A [Driver] implementation using the tokio crate for I/O.
#[cfg(feature = "tokio-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-process")))]
pub struct TokioDriver;

#[cfg(feature = "tokio-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-process")))]
impl Driver for TokioDriver {
    async fn run_process(
        program: &OsStr,
        args: &[&OsStr],
        stdin: Option<&[u8]>,
    ) -> Result<Output, std::io::Error> {
        let mut command = tokio::process::Command::new(program);
        command.args(args);

        match stdin {
            Some(stdin) => {
                let mut child = command.stdin(Stdio::piped()).spawn()?;
                child
                    .stdin
                    .take()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Stdin not redirected successfully by tokio",
                        )
                    })?
                    .write_all(stdin)
                    .await?;

                child.wait_with_output().await
            }
            None => command.stdin(Stdio::null()).output().await,
        }
    }
}

/// A [Driver] implementation using the async-process crate for I/O.
#[cfg(feature = "async-process-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-process")))]
pub struct AsyncProcessDriver;

#[cfg(feature = "async-process-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-process")))]
impl Driver for AsyncProcessDriver {
    async fn run_process(
        program: &OsStr,
        args: &[&OsStr],
        stdin: Option<&[u8]>,
    ) -> Result<Output, std::io::Error> {
        let mut command = async_process::Command::new(program);
        command.args(args);

        match stdin {
            Some(stdin) => {
                let mut child = command.stdin(Stdio::piped()).spawn()?;
                child
                    .stdin
                    .take()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Stdin not redirected successfully by async-process",
                        )
                    })?
                    .write_all(stdin)
                    .await?;

                child.output().await
            }
            None => command.stdin(Stdio::null()).output().await,
        }
    }
}
