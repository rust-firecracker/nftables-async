use std::{ffi::OsStr, future::Future, process::Output};

#[cfg(feature = "async-process-driver")]
use futures_lite::AsyncWriteExt as FuturesAsyncWriteExt;
#[cfg(any(feature = "async-process-driver", feature = "tokio-driver"))]
use std::process::Stdio;
#[cfg(feature = "tokio-driver")]
use tokio::io::AsyncWriteExt as TokioAsyncWriteExt;

/// A process driver to use for asynchronous I/O, supporting only the functionality needed by
/// the nftables-async crate.
pub trait Driver: Send + Sized {
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
