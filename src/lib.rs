use std::process::Stdio;

use nftables::{helper::NftablesError, schema::Nftables};
use tokio::{io::AsyncWriteExt, process::Command};

const NFT_DEFAULT_PROGRAM: &str = "nft";

pub async fn apply_ruleset(
    nftables: &Nftables,
    program: Option<&str>,
    args: Option<Vec<&str>>,
) -> Result<(), NftablesError> {
    let payload = serde_json::to_string(nftables).map_err(NftablesError::NftInvalidJson)?;
    apply_ruleset_raw(payload, program, args).await
}

pub async fn apply_ruleset_raw(
    payload: String,
    program: Option<&str>,
    args: Option<Vec<&str>>,
) -> Result<(), NftablesError> {
    let program = program.unwrap_or(NFT_DEFAULT_PROGRAM);
    let mut command = Command::new(program);
    command.arg("-j").arg("-f").arg("-");

    if let Some(args) = args {
        command.args(args);
    }

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|err| NftablesError::NftExecution {
        program: program.to_owned(),
        inner: err,
    })?;

    let mut stdin = child
        .stdin
        .take()
        .expect("Stdin was piped to Tokio but could not be retrieved");
    stdin
        .write_all(payload.as_bytes())
        .await
        .map_err(|err| NftablesError::NftExecution {
            program: program.to_owned(),
            inner: err,
        })?;
    drop(stdin);

    match child.wait_with_output().await {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stdout = read(program, output.stdout)?;
            let stderr = read(program, output.stderr)?;

            Err(NftablesError::NftFailed {
                program: program.to_owned(),
                hint: "applying ruleset".to_string(),
                stdout,
                stderr,
            })
        }
        Err(err) => Err(NftablesError::NftExecution {
            program: program.to_owned(),
            inner: err,
        }),
    }
}

pub async fn get_current_ruleset(
    program: Option<&str>,
    args: Option<Vec<&str>>,
) -> Result<Nftables, NftablesError> {
    let output = get_current_ruleset_raw(program, args).await?;
    serde_json::from_str(&output).map_err(NftablesError::NftInvalidJson)
}

pub async fn get_current_ruleset_raw(
    program: Option<&str>,
    args: Option<Vec<&str>>,
) -> Result<String, NftablesError> {
    let program = program.unwrap_or(NFT_DEFAULT_PROGRAM);
    let mut command = Command::new(program);
    command.arg("-j").arg("list").arg("ruleset");
    if let Some(args) = args {
        command.args(args);
    }

    let output = command
        .output()
        .await
        .map_err(|err| NftablesError::NftExecution {
            program: program.to_owned(),
            inner: err,
        })?;

    let stdout = read(program, output.stdout)?;

    if !output.status.success() {
        let stderr = read(program, output.stderr)?;

        return Err(NftablesError::NftFailed {
            program: program.to_owned(),
            hint: "getting the current ruleset".to_string(),
            stdout,
            stderr,
        });
    }

    Ok(stdout)
}

#[inline]
fn read(program: &str, stream: Vec<u8>) -> Result<String, NftablesError> {
    String::from_utf8(stream).map_err(|err| NftablesError::NftOutputEncoding {
        program: program.to_owned(),
        inner: err,
    })
}
