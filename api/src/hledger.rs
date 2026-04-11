use crate::error::AppError;
use tokio::process::Command;
use tracing::info;

/// Run an hledger subcommand and return its JSON output.
///
/// `args` is everything after "hledger", e.g. `["balance", "-f", "/data/all.journal"]`.
/// `--output-format=json` is appended automatically.
pub async fn run(args: &[&str]) -> Result<serde_json::Value, AppError> {
    info!(cmd = %format!("hledger {}", args.join(" ")), "running hledger");

    let output = Command::new("hledger")
        .args(args)
        .arg("--output-format=json")
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(AppError::HledgerCommand {
            exit_code: output.status.code().unwrap_or(-1),
            stderr,
        });
    }

    let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    Ok(value)
}

/// Run an hledger subcommand and return raw stdout as a string.
/// Used for commands where JSON output is not desired (e.g., CSV conversion).
pub async fn run_raw(args: &[&str]) -> Result<String, AppError> {
    info!(cmd = %format!("hledger {}", args.join(" ")), "running hledger");

    let output = Command::new("hledger").args(args).output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(AppError::HledgerCommand {
            exit_code: output.status.code().unwrap_or(-1),
            stderr,
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_output_is_parse_error() {
        let result: Result<serde_json::Value, _> = serde_json::from_slice(b"");
        assert!(result.is_err());
        // Confirm it converts to AppError::HledgerParse
        let err: AppError = result.unwrap_err().into();
        assert!(matches!(err, AppError::HledgerParse(_)));
    }
}
