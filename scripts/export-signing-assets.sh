#!/usr/bin/env bash
# export-signing-assets.sh
# GitHub Secrets 업로드를 위한 .p12 base64 인코딩 헬퍼.
# BUILD_CERTIFICATE_BASE64 및 기타 필요 Secret 값을 출력합니다.
#
# 사용법:
#   ./scripts/export-signing-assets.sh <.p12 경로>
#
# 예시:
#   ./scripts/export-signing-assets.sh ~/moai-studio-signing.p12
set -euo pipefail

# -------------------------------------------------------
# 인자 파싱
# -------------------------------------------------------
P12_PATH="${1:-}"

if [ -z "$P12_PATH" ]; then
    read -r -p ".p12 파일 경로: " P12_PATH
fi

# -------------------------------------------------------
# 입력 검증
# -------------------------------------------------------
if [ ! -f "$P12_PATH" ]; then
    echo "[오류] .p12 파일을 찾을 수 없습니다: $P12_PATH" >&2
    exit 1
fi

# -------------------------------------------------------
# base64 인코딩 (macOS base64 는 개행 없이 출력)
# -------------------------------------------------------
B64_VALUE=$(base64 -i "$P12_PATH")

# -------------------------------------------------------
# 랜덤 KEYCHAIN_PASSWORD 생성
# -------------------------------------------------------
SUGGESTED_KP=$(openssl rand -base64 32)

# -------------------------------------------------------
# 출력
# -------------------------------------------------------
echo
echo "========================================"
echo " MoAI Studio — GitHub Secrets 준비"
echo "========================================"
echo
echo "아래 값을 복사하여 GitHub Secrets 에 등록하세요."
echo "저장소 → Settings → Secrets and variables → Actions"
echo
echo "========================================"
echo " SECRET: BUILD_CERTIFICATE_BASE64"
echo "========================================"
echo "$B64_VALUE"
echo
echo "========================================"
echo " SECRET: KEYCHAIN_PASSWORD (제안값)"
echo " (직접 생성해도 됩니다 — 임시 키체인 암호로만 사용)"
echo "========================================"
echo "$SUGGESTED_KP"
echo
echo "========================================"
echo " 나머지 Secrets (직접 입력)"
echo "========================================"
echo
echo " 2. P12_PASSWORD"
echo "    → .p12 내보내기 시 설정한 암호"
echo
echo " 3. KEYCHAIN_PASSWORD"
echo "    → 위 제안값 또는 직접 생성한 값"
echo
echo " 4. APPLE_TEAM_ID"
echo "    → https://developer.apple.com/account → Membership → Team ID (10자)"
echo
echo "========================================"
echo " 모든 Secrets 등록 후 검증 방법"
echo "========================================"
echo
echo " main 브랜치에 push 또는 PR 생성 후"
echo " GitHub Actions → Swift CI 워크플로우에서"
echo " 'Install signing (if available)' 와"
echo " 'UITest (signed)' 스텝이 초록색인지 확인하세요."
echo
echo "========================================"
echo " 참고: .github/SIGNING.md"
echo "========================================"
echo
