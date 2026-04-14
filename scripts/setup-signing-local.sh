#!/usr/bin/env bash
# setup-signing-local.sh
# 로컬 개발자용 Apple .p12 인증서 설치 스크립트.
# 로그인 키체인에 인증서를 임포트하고, Xcode 가 사용하는
# Signing.local.xcconfig 를 생성합니다.
#
# 사용법:
#   ./scripts/setup-signing-local.sh <.p12 경로> <p12 암호> <Team ID>
#
# 예시:
#   ./scripts/setup-signing-local.sh ~/moai-studio-signing.p12 MyPassw0rd ABC1234567
set -euo pipefail

# -------------------------------------------------------
# 인자 파싱
# -------------------------------------------------------
P12_PATH="${1:-}"
P12_PASSWORD="${2:-}"
TEAM_ID="${3:-}"

# 인자가 없으면 대화형으로 입력 받기
if [ -z "$P12_PATH" ]; then
    read -r -p ".p12 파일 경로: " P12_PATH
fi
if [ -z "$P12_PASSWORD" ]; then
    read -r -s -p ".p12 암호: " P12_PASSWORD
    echo
fi
if [ -z "$TEAM_ID" ]; then
    read -r -p "Apple Team ID (10자): " TEAM_ID
fi

# -------------------------------------------------------
# 입력 검증
# -------------------------------------------------------
if [ ! -f "$P12_PATH" ]; then
    echo "[오류] .p12 파일을 찾을 수 없습니다: $P12_PATH" >&2
    exit 1
fi

if [ ${#TEAM_ID} -ne 10 ]; then
    echo "[오류] Team ID 는 정확히 10자여야 합니다 (현재: ${#TEAM_ID}자)" >&2
    exit 1
fi

# -------------------------------------------------------
# 로그인 키체인 임포트 동의 확인
# -------------------------------------------------------
echo
echo "========================================"
echo " MoAI Studio — 로컬 서명 설정"
echo "========================================"
echo " .p12 파일: $P12_PATH"
echo " Team ID  : $TEAM_ID"
echo
echo "[주의] 이 스크립트는 로그인 키체인에 인증서를 임포트합니다."
echo "       Keychain Access 동의 다이얼로그가 나타날 수 있습니다."
read -r -p "계속하시겠습니까? (y/N): " CONFIRM

if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    echo "취소되었습니다."
    exit 0
fi

# -------------------------------------------------------
# 로그인 키체인에 .p12 임포트
# -------------------------------------------------------
echo
echo "[1/3] 인증서를 로그인 키체인에 임포트 중..."

security import "$P12_PATH" \
    -k ~/Library/Keychains/login.keychain-db \
    -P "$P12_PASSWORD" \
    -T /usr/bin/codesign \
    -T /usr/bin/xcodebuild \
    -A

echo "      완료."

# -------------------------------------------------------
# 설치 확인
# -------------------------------------------------------
echo
echo "[2/3] 코드 서명 ID 확인..."
IDENTITY=$(security find-identity -p codesigning -v | grep "Developer ID Application" | head -1 || true)

if [ -z "$IDENTITY" ]; then
    echo "[경고] 코드 서명 ID 목록에서 'Developer ID Application' 을 찾지 못했습니다."
    echo "       Keychain Access 에서 인증서 설치 상태를 직접 확인하세요."
else
    echo "      발견: $IDENTITY"
fi

# -------------------------------------------------------
# Signing.local.xcconfig 생성
# -------------------------------------------------------
echo
echo "[3/3] app/Config/Signing.local.xcconfig 생성 중..."

# 프로젝트 루트 기준 경로 계산
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_DIR="$PROJECT_ROOT/app/Config"
LOCAL_XCCONFIG="$CONFIG_DIR/Signing.local.xcconfig"

# Config 디렉토리 생성 (없는 경우)
mkdir -p "$CONFIG_DIR"

cat > "$LOCAL_XCCONFIG" << EOF
// Signing.local.xcconfig — 로컬 개발자 서명 오버라이드.
// 이 파일은 .gitignore 됩니다. 절대 커밋하지 마세요.
// 생성일: $(date '+%Y-%m-%d %H:%M:%S')
CODE_SIGN_STYLE = Manual
CODE_SIGN_IDENTITY = Developer ID Application
DEVELOPMENT_TEAM = $TEAM_ID
EOF

echo "      생성 완료: $LOCAL_XCCONFIG"

# -------------------------------------------------------
# 완료 메시지
# -------------------------------------------------------
echo
echo "========================================"
echo " 설정 완료!"
echo "========================================"
echo
echo " Xcode 에서 MoAIStudio 프로젝트를 열면"
echo " Signing.local.xcconfig 가 자동으로 로드되어"
echo " Developer ID Application 서명이 적용됩니다."
echo
echo " 되돌리려면: rm $LOCAL_XCCONFIG"
echo
