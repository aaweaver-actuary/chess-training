use assert_cmd::Command;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct IssuePayload {
    title: String,
    body: String,
}

fn action_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/actions/run-with-error-logging/run.sh")
}

fn write_curl_stub(bin_dir: &Path) -> Result<(), Box<dyn Error>> {
    let stub_path = bin_dir.join("curl");
    let script = r#"#!/bin/bash
set -euo pipefail

out_dir="${CURL_STUB_OUTPUT_DIR:?}"

printf '%s\n' "$@" >"$out_dir/args.txt"

payload=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -d)
      shift || break
      if [ "$#" -gt 0 ]; then
        payload="$1"
      fi
      ;;
  esac
  shift || break
done

if [ -n "$payload" ]; then
  printf '%s' "$payload" >"$out_dir/payload.json"
fi

printf '%s\n' '{"number": 777}'
"#;
    fs::write(&stub_path, script)?;

    // Use conditional compilation for platform-specific permission setting
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&stub_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&stub_path, perms)?;
    }

    #[cfg(windows)]
    {
        // On Windows, files with .bat or .cmd extensions are executable by default
        // For bash scripts, we assume WSL or Git Bash is being used where the script
        // will be executable. No additional permissions are needed on Windows.
        // If needed, we could rename to curl.bat, but that would require changes
        // to how the script is called.
    }

    Ok(())
}

fn assert_log_contains(log_dir: &Path, needle: &str) -> Result<(), Box<dyn Error>> {
    let mut entries = fs::read_dir(log_dir)?
        .map(|res| res.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    assert_eq!(entries.len(), 1, "expected exactly one log file");
    let content = fs::read_to_string(&entries[0])?;
    assert!(
        content.contains(needle),
        "log file did not contain `{needle}`. contents: {content}"
    );
    Ok(())
}

fn read_payload(payload_path: &Path) -> Result<IssuePayload, Box<dyn Error>> {
    let payload_str = fs::read_to_string(payload_path)?;
    let json: Value = serde_json::from_str(&payload_str)?;
    Ok(serde_json::from_value(json)?)
}

#[test]
fn failing_command_creates_log_and_issue_payload() -> Result<(), Box<dyn Error>> {
    let script_path = action_script_path();
    let script_path_display = script_path.display().to_string();
    assert!(
        script_path.exists(),
        "expected run-with-error-logging script at {script_path_display}"
    );

    let temp = tempdir()?;
    let log_dir = temp.path().join("ci-logs");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&bin_dir)?;
    write_curl_stub(&bin_dir)?;

    let mut cmd = Command::new("bash");
    cmd.arg(&script_path)
        .arg("--command")
        .arg("bash -c 'echo run-output; echo run-error >&2; exit 1'")
        .arg("--label")
        .arg("unit-test")
        .arg("--working-directory")
        .arg(temp.path().to_string_lossy().to_string())
        .env("GPT_ISSUE_KEY", "test-token")
        .env("GITHUB_REPOSITORY", "example/repo")
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin_dir.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env(
            "CURL_STUB_OUTPUT_DIR",
            temp.path().to_string_lossy().to_string(),
        );

    let output = cmd.output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit code 1, got {:?}",
        output.status.code()
    );

    assert_log_contains(&log_dir, "run-output")?;
    assert_log_contains(&log_dir, "run-error")?;

    let payload_path = temp.path().join("payload.json");
    assert!(
        payload_path.exists(),
        "expected curl stub to capture payload at {}",
        payload_path.display()
    );

    let payload = read_payload(&payload_path)?;
    assert!(
        payload.title.contains("unit-test"),
        "unexpected title: {}",
        payload.title
    );
    let body = &payload.body;
    assert!(
        body.contains("CI step `unit-test` failed"),
        "unexpected issue body: {body}"
    );
    assert!(
        body.contains("run-output") && body.contains("run-error"),
        "issue body did not include log output"
    );

    Ok(())
}

#[test]
fn missing_issue_token_skips_issue_creation() -> Result<(), Box<dyn Error>> {
    let script_path = action_script_path();
    assert!(script_path.exists());

    let temp = tempdir()?;
    let _log_dir = temp.path().join("ci-logs");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&bin_dir)?;
    write_curl_stub(&bin_dir)?;

    let mut cmd = Command::new("bash");
    cmd.arg(&script_path)
        .arg("--command")
        .arg("bash -c 'exit 1'")
        .arg("--label")
        .arg("no-token-test")
        .arg("--working-directory")
        .arg(temp.path().to_string_lossy().to_string())
        .env_remove("GPT_ISSUE_KEY")
        .env_remove("GITHUB_TOKEN")
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin_dir.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env(
            "CURL_STUB_OUTPUT_DIR",
            temp.path().to_string_lossy().to_string(),
        );

    let output = cmd.output()?;
    assert_eq!(output.status.code(), Some(1));

    let payload_path = temp.path().join("payload.json");
    assert!(
        !payload_path.exists(),
        "curl should not be invoked without an issue token"
    );

    Ok(())
}
