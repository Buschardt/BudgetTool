//! Integration tests for hledger subprocess integration.
//!
//! These tests require the `hledger` binary on PATH.
//! Run with: cargo test -- --ignored

use budgettool_api::core::error::AppError;
use budgettool_api::core::hledger;
use std::io::Write;

#[tokio::test]
#[ignore]
async fn balance_returns_json_for_valid_journal() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        tmp,
        "2024-01-01 Opening\n    assets:checking  $1000\n    equity:opening  $-1000\n"
    )
    .unwrap();

    let path = tmp.path().to_str().unwrap().to_owned();
    let result = hledger::run(&["balance", "-f", &path]).await;

    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    // hledger balance --output-format=json returns an array
    assert!(result.unwrap().is_array());
}

#[tokio::test]
#[ignore]
async fn nonexistent_file_returns_hledger_command_error() {
    let result = hledger::run(&["balance", "-f", "/nonexistent/path.journal"]).await;
    assert!(matches!(result, Err(AppError::HledgerCommand { .. })));
}
