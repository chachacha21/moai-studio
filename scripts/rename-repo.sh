#!/usr/bin/env bash
# rename-repo.sh — MoAI Studio 리포지토리 리네임 스크립트
# moai-cli → moai-studio (DESIGN.v4 §14 O6)
#
# 사용법: bash scripts/rename-repo.sh
#
# 주의: 이 스크립트는 디렉터리 이동 후에도 계속 실행되어야 하므로
#       실행 전에 반드시 ~/moai/moai-cli 내에서 실행하세요.

set -euo pipefail

# ─────────────────────────────────────────────
# 색상 출력 헬퍼
# ─────────────────────────────────────────────
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
warn()    { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
success() { echo -e "${GREEN}[OK]${RESET}    $*"; }
error()   { echo -e "${RED}[ERROR]${RESET} $*" >&2; }
header()  { echo -e "\n${BOLD}${CYAN}══════════════════════════════════════${RESET}"; echo -e "${BOLD}${CYAN}  $*${RESET}"; echo -e "${BOLD}${CYAN}══════════════════════════════════════${RESET}"; }

confirm() {
    local prompt="$1"
    local default="${2:-n}"
    local yn
    if [[ "$default" == "y" ]]; then
        echo -ne "${YELLOW}${prompt} [Y/n] ${RESET}"
    else
        echo -ne "${YELLOW}${prompt} [y/N] ${RESET}"
    fi
    read -r yn
    yn="${yn:-$default}"
    [[ "$yn" =~ ^[Yy]$ ]]
}

# ─────────────────────────────────────────────
# 상수
# ─────────────────────────────────────────────
OLD_NAME="moai-cli"
NEW_NAME="moai-studio"
PARENT_DIR="$HOME/moai"
OLD_DIR="$PARENT_DIR/$OLD_NAME"
NEW_DIR="$PARENT_DIR/$NEW_NAME"
GITHUB_ORG="modu-ai"
OLD_REMOTE="git@github.com:${GITHUB_ORG}/${OLD_NAME}.git"
NEW_REMOTE="git@github.com:${GITHUB_ORG}/${NEW_NAME}.git"

DOCS_TO_UPDATE=(
    "NEXT-STEPS.md"
    "REFERENCES.md"
)

# ─────────────────────────────────────────────
# PHASE 0: Pre-flight checks
# ─────────────────────────────────────────────
header "Phase 0: Pre-flight Checks"

# 0-1. DESIGN.v4.md 존재 확인 (올바른 리포인지 검증)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [[ ! -f "$REPO_ROOT/DESIGN.v4.md" ]]; then
    error "DESIGN.v4.md 를 찾을 수 없습니다. 올바른 리포지토리가 아닙니다."
    error "현재 경로: $REPO_ROOT"
    exit 1
fi
success "DESIGN.v4.md 확인됨 — 올바른 리포지토리"

# 0-2. 현재 디렉터리 위치 확인
CURRENT_DIR="$(pwd)"
info "현재 경로: $CURRENT_DIR"
info "스크립트 루트: $REPO_ROOT"

# 0-3. git 상태 확인 (uncommitted 변경사항 없어야 함)
cd "$REPO_ROOT"
if ! git diff --quiet || ! git diff --cached --quiet; then
    error "커밋되지 않은 변경사항이 있습니다. 모두 커밋 후 실행하세요."
    git status --short
    exit 1
fi
success "git 상태 클린 — 커밋되지 않은 변경사항 없음"

# 0-4. 대상 디렉터리가 이미 존재하는지 확인 (idempotent 지원)
DISK_RENAME_NEEDED=true
if [[ -d "$NEW_DIR" ]] && [[ ! -d "$OLD_DIR" ]]; then
    warn "디렉터리가 이미 $NEW_DIR 로 이름 변경되어 있습니다."
    DISK_RENAME_NEEDED=false
elif [[ -d "$NEW_DIR" ]] && [[ -d "$OLD_DIR" ]]; then
    error "$OLD_DIR 와 $NEW_DIR 가 동시에 존재합니다. 수동으로 정리 후 재실행하세요."
    exit 1
elif [[ ! -d "$OLD_DIR" ]]; then
    warn "$OLD_DIR 를 찾을 수 없습니다. (이미 이름 변경되었거나 다른 경로일 수 있음)"
    DISK_RENAME_NEEDED=false
fi

# 0-5. .references/ 심볼릭 링크 경고
if [[ -d "$REPO_ROOT/.references" ]]; then
    warn ".references/ 디렉터리가 있습니다. 리네임 후 일부 심볼릭 링크가 깨질 수 있습니다."
    BROKEN_LINKS=()
    while IFS= read -r -d '' link; do
        target="$(readlink "$link" 2>/dev/null || true)"
        if [[ "$target" == *"$OLD_NAME"* ]]; then
            BROKEN_LINKS+=("$link → $target")
        fi
    done < <(find "$REPO_ROOT/.references" -type l -print0 2>/dev/null)
    if [[ ${#BROKEN_LINKS[@]} -gt 0 ]]; then
        warn "영향받는 심볼릭 링크 (${#BROKEN_LINKS[@]}개):"
        for l in "${BROKEN_LINKS[@]}"; do
            warn "  $l"
        done
    fi
fi

echo ""
info "Pre-flight 검사 완료. 다음 작업을 수행합니다:"
echo ""
[[ "$DISK_RENAME_NEEDED" == "true" ]] && echo "  1. 디스크 디렉터리 리네임: $OLD_DIR → $NEW_DIR"
echo "  2. 문서 경로 업데이트: NEXT-STEPS.md, REFERENCES.md"
echo "  3. README.md '리네임 예정' 문구 제거"
echo "  4. .references/ 심볼릭 링크 재생성 (해당되는 경우)"
echo "  5. GitHub 리포지토리 리네임 (선택)"
echo ""

if ! confirm "계속 진행하시겠습니까?"; then
    info "취소되었습니다."
    exit 0
fi

# ─────────────────────────────────────────────
# PHASE 1: 디스크 디렉터리 리네임
# ─────────────────────────────────────────────
header "Phase 1: 디스크 디렉터리 리네임"

if [[ "$DISK_RENAME_NEEDED" == "true" ]]; then
    warn "디렉터리 이동: $OLD_DIR → $NEW_DIR"
    warn "이 작업 후 현재 쉘의 working directory가 잠시 변경될 수 있습니다."
    if confirm "디렉터리 이름을 변경하시겠습니까?"; then
        cd "$PARENT_DIR"
        mv "$OLD_NAME" "$NEW_NAME"
        cd "$NEW_NAME"
        REPO_ROOT="$NEW_DIR"
        success "디렉터리 리네임 완료: $NEW_DIR"
    else
        warn "디렉터리 리네임 건너뜀."
    fi
else
    info "디렉터리 리네임 불필요 (이미 완료되었거나 경로가 다름)"
    # REPO_ROOT를 NEW_DIR로 업데이트
    if [[ -d "$NEW_DIR" ]]; then
        REPO_ROOT="$NEW_DIR"
        cd "$REPO_ROOT"
    fi
fi

# ─────────────────────────────────────────────
# PHASE 2: 문서 경로 업데이트
# ─────────────────────────────────────────────
header "Phase 2: 문서 경로 업데이트"

OLD_PATH_PATTERN="~/moai/${OLD_NAME}"
NEW_PATH_VALUE="~/moai/${NEW_NAME}"

for doc in "${DOCS_TO_UPDATE[@]}"; do
    DOC_PATH="$REPO_ROOT/$doc"
    if [[ ! -f "$DOC_PATH" ]]; then
        warn "$doc 파일을 찾을 수 없습니다. 건너뜁니다."
        continue
    fi

    if grep -q "$OLD_PATH_PATTERN" "$DOC_PATH" 2>/dev/null; then
        info "$doc 에서 경로 업데이트 중..."
        # macOS/BSD sed와 GNU sed 모두 지원
        if sed --version 2>/dev/null | grep -q GNU; then
            sed -i "s|${OLD_PATH_PATTERN}|${NEW_PATH_VALUE}|g" "$DOC_PATH"
        else
            sed -i '' "s|${OLD_PATH_PATTERN}|${NEW_PATH_VALUE}|g" "$DOC_PATH"
        fi
        success "$doc 업데이트 완료"
    else
        info "$doc: 변경할 경로 없음 (이미 업데이트되었거나 해당 없음)"
    fi
done

# README.md '리네임 예정' 문구 제거
README_PATH="$REPO_ROOT/README.md"
if [[ -f "$README_PATH" ]]; then
    # '리네임 예정' 패턴 감지 (다양한 형태 지원)
    if grep -q "리네임 예정" "$README_PATH" 2>/dev/null; then
        info "README.md 에서 '리네임 예정' 문구 제거 중..."
        # 라인 단위로 '리네임 예정'이 포함된 줄 제거
        if sed --version 2>/dev/null | grep -q GNU; then
            sed -i '/리네임 예정/d' "$README_PATH"
        else
            sed -i '' '/리네임 예정/d' "$README_PATH"
        fi
        success "README.md '리네임 예정' 문구 제거 완료"
    else
        info "README.md: '리네임 예정' 문구 없음 (이미 제거됨)"
    fi
else
    warn "README.md 를 찾을 수 없습니다."
fi

# ─────────────────────────────────────────────
# PHASE 3: .references/ 심볼릭 링크 재생성
# ─────────────────────────────────────────────
header "Phase 3: .references/ 심볼릭 링크 재생성"

REFS_DIR="$REPO_ROOT/.references"
if [[ -d "$REFS_DIR" ]]; then
    FIXED=0
    while IFS= read -r -d '' link; do
        target="$(readlink "$link" 2>/dev/null || true)"
        if [[ "$target" == *"$OLD_NAME"* ]]; then
            new_target="${target//$OLD_NAME/$NEW_NAME}"
            info "심볼릭 링크 재생성: $(basename "$link")"
            info "  이전: $target"
            info "  이후: $new_target"
            rm "$link"
            ln -s "$new_target" "$link"
            FIXED=$((FIXED + 1))
        fi
    done < <(find "$REFS_DIR" -type l -print0 2>/dev/null)

    if [[ $FIXED -gt 0 ]]; then
        success ".references/ 심볼릭 링크 ${FIXED}개 재생성 완료"
    else
        info ".references/ 재생성이 필요한 심볼릭 링크 없음"
    fi
else
    info ".references/ 디렉터리 없음 — 건너뜀"
fi

# ─────────────────────────────────────────────
# PHASE 4: GitHub 리포지토리 리네임 (선택)
# ─────────────────────────────────────────────
header "Phase 4: GitHub 리포지토리 리네임 (선택)"

warn "이 작업은 GitHub 리포지토리 이름을 변경하고 remote URL을 업데이트합니다."
warn "기존 clone이나 fork를 사용하는 다른 사람들에게 영향을 줄 수 있습니다."

if confirm "GitHub 리포지토리를 '${NEW_NAME}'으로 리네임하시겠습니까?"; then
    # gh CLI 설치 확인
    if ! command -v gh &>/dev/null; then
        error "gh CLI가 설치되어 있지 않습니다. 수동으로 GitHub에서 리네임하세요."
        info "  GitHub 웹 → Settings → Repository name → 변경"
        info "  그 후 수동으로: git remote set-url origin ${NEW_REMOTE}"
    else
        # 현재 remote 확인
        CURRENT_REMOTE="$(git -C "$REPO_ROOT" remote get-url origin 2>/dev/null || echo 'unknown')"
        info "현재 remote: $CURRENT_REMOTE"

        # GitHub 리네임 실행
        info "gh repo rename 실행 중..."
        if gh repo rename "$NEW_NAME" --repo "${GITHUB_ORG}/${OLD_NAME}" --yes 2>/dev/null; then
            success "GitHub 리포지토리 리네임 완료: ${GITHUB_ORG}/${NEW_NAME}"
        else
            # 이미 리네임된 경우 또는 권한 없는 경우
            warn "gh repo rename 실패 (이미 리네임되었거나 권한 부족)"
            info "수동으로 GitHub 웹에서 리네임하거나, 이미 완료된 경우 계속 진행합니다."
        fi

        # remote URL 업데이트
        CURRENT_REMOTE_CHECK="$(git -C "$REPO_ROOT" remote get-url origin 2>/dev/null || echo '')"
        if [[ "$CURRENT_REMOTE_CHECK" != "$NEW_REMOTE" ]]; then
            info "git remote URL 업데이트: $NEW_REMOTE"
            git -C "$REPO_ROOT" remote set-url origin "$NEW_REMOTE"
            success "git remote URL 업데이트 완료"
        else
            info "git remote URL 이미 최신 상태: $NEW_REMOTE"
        fi
    fi
else
    info "GitHub 리네임 건너뜀."
    info "나중에 수동으로 실행하려면:"
    info "  gh repo rename moai-studio --repo ${GITHUB_ORG}/${OLD_NAME}"
    info "  git remote set-url origin ${NEW_REMOTE}"
fi

# ─────────────────────────────────────────────
# PHASE 5: Post-rename 검증
# ─────────────────────────────────────────────
header "Phase 5: Post-rename 검증"

VERIFICATION_FAILED=false

# 5-1. git status 확인
info "git status 확인..."
GIT_STATUS="$(git -C "$REPO_ROOT" status --porcelain 2>/dev/null || echo 'ERROR')"
if [[ "$GIT_STATUS" == "ERROR" ]]; then
    warn "git status 실행 실패"
elif [[ -z "$GIT_STATUS" ]]; then
    success "git status: 클린 (변경 없음)"
else
    warn "git status: 스테이지되지 않은 변경사항 있음 (문서 업데이트 후 커밋 필요)"
    echo "$GIT_STATUS"
fi

# 5-2. Cargo 검사 (Rust 프로젝트)
if [[ -f "$REPO_ROOT/Cargo.toml" ]]; then
    info "cargo check --workspace 실행 중..."
    if cargo check --workspace --manifest-path "$REPO_ROOT/Cargo.toml" 2>&1; then
        success "cargo check: 통과"
    else
        error "cargo check: 실패"
        VERIFICATION_FAILED=true
    fi
else
    info "Cargo.toml 없음 — cargo check 건너뜀"
fi

# 5-3. Swift 빌드 (app/ 디렉터리)
APP_DIR="$REPO_ROOT/app"
if [[ -d "$APP_DIR" ]] && [[ -f "$APP_DIR/Package.swift" ]]; then
    info "swift build 실행 중 (app/)..."
    if (cd "$APP_DIR" && swift build 2>&1); then
        success "swift build: 통과"
    else
        error "swift build: 실패"
        VERIFICATION_FAILED=true
    fi
else
    info "app/Package.swift 없음 — swift build 건너뜀"
fi

# ─────────────────────────────────────────────
# 결과 요약
# ─────────────────────────────────────────────
header "완료 요약"

if [[ "$VERIFICATION_FAILED" == "true" ]]; then
    error "일부 검증 단계가 실패했습니다. 위의 오류 메시지를 확인하세요."
    echo ""
    echo -e "${YELLOW}다음 단계:${RESET}"
    echo "  1. 실패한 빌드 오류 수정"
    echo "  2. 변경사항 커밋: git add -A && git commit -m 'chore: repo renamed moai-cli → moai-studio'"
    exit 1
else
    success "리포지토리 리네임이 성공적으로 완료되었습니다!"
    echo ""
    echo -e "${GREEN}다음 단계:${RESET}"
    if [[ -n "${GIT_STATUS:-}" ]]; then
        echo "  1. 변경사항 커밋:"
        echo "       cd $REPO_ROOT"
        echo "       git add NEXT-STEPS.md REFERENCES.md README.md"
        echo "       git commit -m 'chore: repo renamed moai-cli → moai-studio'"
    fi
    echo ""
    echo "  리포지토리 경로: $REPO_ROOT"
    CURRENT_REMOTE_FINAL="$(git -C "$REPO_ROOT" remote get-url origin 2>/dev/null || echo '(확인 불가)')"
    echo "  GitHub remote:  $CURRENT_REMOTE_FINAL"
fi
