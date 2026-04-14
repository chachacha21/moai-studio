#!/usr/bin/env bash
# Claude CLI E2E validation (M1 C-2 carry-over).
# Prerequisites: `claude` binary in PATH, $ANTHROPIC_API_KEY set.
#
# @MX:NOTE: [AUTO] opt-in 스크립트 — CI에서 자동 실행되지 않음. 수동 또는 선택적 파이프라인에서 실행.

set -euo pipefail

if ! command -v claude &> /dev/null; then
  echo "SKIP: claude CLI not installed"
  exit 0
fi

if [ -z "${ANTHROPIC_API_KEY:-}" ]; then
  echo "SKIP: ANTHROPIC_API_KEY not set"
  exit 0
fi

echo "Running Claude CLI E2E validation..."
# Send a trivial prompt via --bare -p
response=$(echo '{"type":"user","message":{"role":"user","content":"Reply with exactly: PONG"}}' \
  | claude --bare -p "" --output-format stream-json --permission-mode acceptEdits 2>&1 \
  | grep -E '"type":"assistant"' \
  | head -1)

if echo "$response" | grep -q "PONG\|assistant"; then
  echo "PASS: Claude CLI responded"
  exit 0
else
  echo "FAIL: Claude CLI did not respond as expected"
  echo "Output: $response"
  exit 1
fi
