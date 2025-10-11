#!/usr/bin/env bash
set -uo pipefail

command=""
label=""
working_directory=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --command)
      shift || break
      command="${1:-}"
      ;;
    --label)
      shift || break
      label="${1:-}"
      ;;
    --working-directory)
      shift || break
      working_directory="${1:-}"
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
  shift || break
done

if [[ -z "$command" ]]; then
  echo "Missing required --command argument" >&2
  exit 2
fi

repo_root="$(pwd)"
if [[ -n "$working_directory" ]]; then
  cd "$working_directory"
fi

log_label="$label"
if [[ -z "$log_label" ]]; then
  log_label="step"
fi

tmp_log="$(mktemp)"
tmp_script="$(mktemp)"

printf '%s\n' "$command" >"$tmp_script"

bash "$tmp_script" 2>&1 | tee "$tmp_log"
status=${PIPESTATUS[0]}

if [[ "$status" -ne 0 ]]; then
  log_root="${CI_ERROR_LOG_DIR:-$repo_root/ci-errors}"
  mkdir -p "$log_root"

  safe_label=$(printf '%s' "$log_label" | tr '[:space:]/' '__')
  safe_label=$(printf '%s' "$safe_label" | tr -cd '[:alnum:]_.-')
  if [[ -z "$safe_label" ]]; then
    safe_label="step"
  fi

  timestamp=$(date -u +"%Y%m%dT%H%M%SZ")
  dest="$log_root/${timestamp}_${safe_label}.log"
  cp "$tmp_log" "$dest"
  echo "Saved CI error log to $dest"

  issue_token="${GPT_ISSUE_KEY:-${GITHUB_TOKEN:-}}"

  if [[ -n "${issue_token:-}" && -n "${GITHUB_REPOSITORY:-}" ]]; then
    issue_title="CI failure: $log_label ($timestamp)"
    issue_api_url="https://api.github.com/repos/$GITHUB_REPOSITORY/issues"

    export ISSUE_TITLE="$issue_title"
    export ISSUE_LABEL="$log_label"
    export ISSUE_TIMESTAMP="$timestamp"
    export ISSUE_LOG_PATH="$tmp_log"

    issue_payload=$(python3 <<'PY'
import json
import os
import sys

title = os.environ.get("ISSUE_TITLE", "CI failure")
label = os.environ.get("ISSUE_LABEL", "step")
timestamp = os.environ.get("ISSUE_TIMESTAMP", "")
log_path = os.environ.get("ISSUE_LOG_PATH")

if not log_path:
    print("::warning::Missing CI log path; cannot create GitHub issue.", file=sys.stderr)
    sys.exit(0)

try:
    max_log_bytes = 65536  # 64 KB
    with open(log_path, "rb") as handle:
        handle.seek(0, os.SEEK_END)
        file_size = handle.tell()
        if file_size > max_log_bytes:
            handle.seek(-max_log_bytes, os.SEEK_END)
            log_bytes = handle.read(max_log_bytes)
            log_content = log_bytes.decode("utf-8", errors="replace")
            log_content = "[...truncated, showing last 64 KB...]\n" + log_content
        else:
            handle.seek(0)
            log_bytes = handle.read()
            log_content = log_bytes.decode("utf-8", errors="replace")
except OSError as exc:
    print(f"::warning::Failed to read CI log: {exc}", file=sys.stderr)
    sys.exit(0)

body = f"CI step `{label}` failed at {timestamp}.\n\n```\n{log_content}\n```"

print(json.dumps({"title": title, "body": body}))
PY
)

    if [[ -n "$issue_payload" ]]; then
      curl_stderr="$(mktemp)"
      response=$(curl -sS -X POST \
        -H "Authorization: Bearer $issue_token" \
        -H "Accept: application/vnd.github+json" \
        -H "Content-Type: application/json" \
        "$issue_api_url" \
        -d "$issue_payload" 2>"$curl_stderr")

      curl_status=$?
      if [[ "$curl_status" -ne 0 ]]; then
        if [[ -s "$curl_stderr" ]]; then
          error_details=$(cat "$curl_stderr")
        else
          error_details="No error details available"
        fi
        echo "::warning::Failed to contact GitHub API to create CI failure issue. Exit code: $curl_status. Details: $error_details"
      else
        issue_number=$(python3 <<'PY'
import json
import sys

try:
    data = json.load(sys.stdin)
except json.JSONDecodeError:
    sys.exit(1)

number = data.get("number")
if number is not None:
    print(number)
PY
 <<<"$response")

        if [[ -n "$issue_number" ]]; then
          echo "Created GitHub issue #$issue_number for CI failure logs."
        else
          echo "::warning::Received unexpected response while creating GitHub issue."
        fi
      fi

      rm -f "$curl_stderr"
    else
      echo "::warning::Failed to build GitHub issue payload for CI failure."
    fi
  else
    echo "::warning::Skipping GitHub issue creation; missing GPT_ISSUE_KEY/GITHUB_TOKEN or GITHUB_REPOSITORY."
  fi
fi

rm -f "$tmp_log" "$tmp_script"
exit "$status"
