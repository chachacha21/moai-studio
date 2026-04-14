#!/usr/bin/env bash
# M1 C-3: 10-min 4-workspace stress + RSS <400MB.
# Requires built MoAI Studio.app; measures RSS every 30s.
#
# @MX:NOTE: [AUTO] opt-in 스크립트 — CI에서 자동 실행되지 않음. 빌드된 .app 필요.
# @MX:NOTE: [AUTO] SKIP 조건: .app 미존재 시 exit 0 처리. 수동 실행 전용.

set -euo pipefail
APP="${1:-/Applications/MoAI Studio.app}"
DURATION_MIN="${2:-10}"
RSS_LIMIT_MB=400

if [ ! -d "$APP" ]; then
  echo "SKIP: app not found at $APP"
  exit 0
fi

echo "Starting stress test: $APP (${DURATION_MIN}min, RSS limit ${RSS_LIMIT_MB}MB)"
open -a "$APP"
sleep 5
PID=$(pgrep -f "MoAI Studio" | head -1)
[ -z "$PID" ] && { echo "FAIL: app not running"; exit 1; }

end=$(( $(date +%s) + DURATION_MIN * 60 ))
peak=0
while [ $(date +%s) -lt $end ]; do
  rss_kb=$(ps -o rss= -p $PID 2>/dev/null | tr -d ' ')
  if [ -z "$rss_kb" ]; then
    echo "FAIL: process $PID 종료됨"
    exit 1
  fi
  rss_mb=$((rss_kb / 1024))
  [ $rss_mb -gt $peak ] && peak=$rss_mb
  remaining=$(( (end - $(date +%s)) / 60 ))
  echo "RSS: ${rss_mb}MB (peak: ${peak}MB, 잔여: ${remaining}분)"
  sleep 30
done

echo "Peak RSS: ${peak}MB (limit ${RSS_LIMIT_MB}MB)"
[ $peak -le $RSS_LIMIT_MB ] && { echo "PASS"; exit 0; } || { echo "FAIL: 피크 RSS ${peak}MB가 제한 ${RSS_LIMIT_MB}MB 초과"; exit 1; }
