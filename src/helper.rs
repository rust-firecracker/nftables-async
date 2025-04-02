use std::{ffi::OsStr, future::Future};

use nftables::{
    helper::{NftablesError, DEFAULT_ARGS, DEFAULT_NFT},
    schema::Nftables,
};

use crate::{driver::Driver, util::MapFuture};

pub trait Helper {
    fn apply_ruleset(
        nftables: &Nftables,
    ) -> impl Future<Output = Result<(), NftablesError>> + Send {
        Self::apply_ruleset_with_args(nftables, DEFAULT_NFT, DEFAULT_ARGS)
    }

    fn apply_ruleset_with_args<
        'a,
        P: AsRef<OsStr> + Sync + ?Sized,
        A: AsRef<OsStr> + Sync + ?Sized + 'a,
        I: IntoIterator<Item = &'a A> + Send,
    >(
        nftables: &Nftables,
        program: Option<&P>,
        args: I,
    ) -> impl Future<Output = Result<(), NftablesError>> + Send {
        let payload =
            serde_json::to_string(nftables).expect("Failed to serialize Nftables struct to JSON");
        Self::apply_ruleset_raw(payload, program, args)
    }

    fn apply_ruleset_raw<
        'a,
        P: AsRef<OsStr> + Sync + ?Sized,
        A: AsRef<OsStr> + Sync + ?Sized + 'a,
        I: IntoIterator<Item = &'a A> + Send,
    >(
        payload: String,
        program: Option<&P>,
        args: I,
    ) -> impl Future<Output = Result<(), NftablesError>> + Send;

    fn get_current_ruleset() -> impl Future<Output = Result<Nftables<'static>, NftablesError>> + Send
    {
        Self::get_current_ruleset_with_args(DEFAULT_NFT, DEFAULT_ARGS)
    }

    fn get_current_ruleset_with_args<
        'a,
        P: AsRef<OsStr> + Sync + ?Sized,
        A: AsRef<OsStr> + Sync + ?Sized + 'a,
        I: IntoIterator<Item = &'a A> + Send,
    >(
        program: Option<&P>,
        args: I,
    ) -> impl Future<Output = Result<Nftables<'static>, NftablesError>> + Send {
        MapFuture::new(
            Self::get_current_ruleset_raw(program, args),
            |result: Result<String, NftablesError>| {
                result.and_then(|output| {
                    serde_json::from_str(&output).map_err(NftablesError::NftInvalidJson)
                })
            },
        )
    }

    fn get_current_ruleset_raw<
        'a,
        P: AsRef<OsStr> + Sync + ?Sized,
        A: AsRef<OsStr> + Sync + ?Sized + 'a,
        I: IntoIterator<Item = &'a A> + Send,
    >(
        program: Option<&P>,
        args: I,
    ) -> impl Future<Output = Result<String, NftablesError>> + Send;
}

impl<D: Driver> Helper for D {
    async fn apply_ruleset_raw<
        'a,
        P: AsRef<OsStr> + Sync + ?Sized,
        A: AsRef<OsStr> + Sync + ?Sized + 'a,
        I: IntoIterator<Item = &'a A> + Send,
    >(
        payload: String,
        program: Option<&P>,
        args: I,
    ) -> Result<(), NftablesError> {
        let program = program.map(|v| v.as_ref()).unwrap_or(OsStr::new("nft"));
        let mut all_args = vec![OsStr::new("-j"), OsStr::new("-f"), OsStr::new("-")];

        all_args.extend(args.into_iter().map(|v| v.as_ref()));

        let mut driver = D::spawn(&program, all_args.as_slice(), true).map_err(|err| {
            NftablesError::NftExecution {
                program: program.to_owned().into(),
                inner: err,
            }
        })?;

        driver
            .fill_stdin(payload.as_bytes())
            .await
            .map_err(|err| NftablesError::NftExecution {
                program: program.to_owned().into(),
                inner: err,
            })?;

        match driver.wait_with_output().await {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => {
                let stdout = read(&program, output.stdout)?;
                let stderr = read(&program, output.stderr)?;

                Err(NftablesError::NftFailed {
                    program: program.to_owned().into(),
                    hint: "applying ruleset".to_string(),
                    stdout,
                    stderr,
                })
            }
            Err(err) => Err(NftablesError::NftExecution {
                program: program.to_owned().into(),
                inner: err,
            }),
        }
    }

    async fn get_current_ruleset_raw<
        'a,
        P: AsRef<OsStr> + Sync + ?Sized,
        A: AsRef<OsStr> + Sync + ?Sized + 'a,
        I: IntoIterator<Item = &'a A> + Send,
    >(
        program: Option<&P>,
        args: I,
    ) -> Result<String, NftablesError> {
        let program = program.map(|v| v.as_ref()).unwrap_or(OsStr::new("nft"));
        let mut all_args = vec![OsStr::new("-j"), OsStr::new("list"), OsStr::new("ruleset")];

        all_args.extend(args.into_iter().map(|v| v.as_ref()));

        let output = D::output(program, all_args.as_slice())
            .await
            .map_err(|err| NftablesError::NftExecution {
                program: program.to_owned().into(),
                inner: err,
            })?;

        let stdout = read(&program, output.stdout)?;

        if !output.status.success() {
            let stderr = read(&program, output.stderr)?;

            return Err(NftablesError::NftFailed {
                program: program.to_owned().into(),
                hint: "getting the current ruleset".to_string(),
                stdout,
                stderr,
            });
        }

        Ok(stdout)
    }
}

#[inline]
fn read(program: &OsStr, stream: Vec<u8>) -> Result<String, NftablesError> {
    String::from_utf8(stream).map_err(|err| NftablesError::NftOutputEncoding {
        program: program.to_owned().into(),
        inner: err,
    })
}
