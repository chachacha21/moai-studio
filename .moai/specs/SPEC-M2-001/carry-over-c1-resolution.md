# C-1 Carry-over 해소 기록

---
spec_id: SPEC-M2-001
carry_over: C-1
status: 인프라 준비 완료 (Secrets 업로드 1회 시 자동화 활성)
resolved_date: 2026-04-14
---

## 문제 정의 (M2 완료 보고서에서)

M2 MS-7 완료 시점에 C-1 "UITest CI 서명" 은 **부분 해소** 상태였습니다.

- **구현**: ad-hoc 서명 (`CODE_SIGN_IDENTITY="-"`) 으로 best-effort UITest 실행
- **한계**: macOS 권한 다이얼로그 / Accessibility API 제한으로 CI에서 실제 UITest 통과 불가
- **차단 요인**: Apple Developer Program 가입 + 유효한 Developer ID Application 인증서 필요

## 해결 방안

Apple Developer Program 가입(1회 수동 작업) 후 CI가 완전 자동화되도록 인프라 전체를 구성했습니다.

### 핵심 설계

1. **HAS_SIGNING 조건부 분기**: `secrets.BUILD_CERTIFICATE_BASE64 != ''` 로 서명 가용 여부를 판별합니다.
   - 서명 있음: composite action 으로 임시 키체인 구성 → signed UITest 실행
   - 서명 없음: skip 메시지 출력 (fork PR 등 안전한 폴백)

2. **임시 키체인 격리 (TN3125)**: 로그인 키체인을 수정하지 않고 `$RUNNER_TEMP/moai-signing.keychain-db` 를 job 전용으로 생성합니다. post 단계에서 자동 삭제됩니다.

3. **xcodebuild 커맨드라인 오버라이드**: `CODE_SIGN_STYLE`, `CODE_SIGN_IDENTITY`, `DEVELOPMENT_TEAM`, `OTHER_CODE_SIGN_FLAGS` 를 명령줄에서 직접 전달하므로 `.xcodeproj` 수정이 불필요합니다.

## 신규 생성 파일

| 파일 | 용도 |
|------|------|
| `.github/actions/install-signing/action.yml` | 재사용 가능한 composite action (TN3125 기반) |
| `.github/SIGNING.md` | Apple 가입 → Secrets 업로드 체크리스트 |
| `scripts/setup-signing-local.sh` | 로컬 개발자용 .p12 임포트 + Signing.local.xcconfig 생성 |
| `scripts/export-signing-assets.sh` | Secrets 값 생성 헬퍼 (base64 + 랜덤 keychain pw) |
| `app/Config/Signing.xcconfig` | 서명 기본 설정 + 로컬 오버라이드 include |

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `.github/workflows/ci-swift.yml` | HAS_SIGNING 조건부 분기 추가, signed UITest 경로 + fallback skip |
| `.gitignore` | `app/Config/Signing.local.xcconfig` 제외 항목 추가 |
| `.github/WORKFLOWS.md` | UITest 서명 섹션 업데이트 + SIGNING.md 링크 |
| `.moai/specs/SPEC-M2-001/progress.md` | C-1 해소 섹션 추가 |
| `.moai/specs/SPEC-M2-001/m2-completion-report.md` | C-1 상태 업데이트 |

## 인간 수동 작업 (1회)

아래 작업은 **GOOS행님이 직접 1회 수행**해야 합니다:

1. Apple Developer Program 가입 ($99/year): <https://developer.apple.com/programs/>
2. Developer ID Application 인증서 생성 및 .p12 내보내기
3. GitHub Secrets 에 4개 값 등록:
   - `BUILD_CERTIFICATE_BASE64`
   - `P12_PASSWORD`
   - `KEYCHAIN_PASSWORD`
   - `APPLE_TEAM_ID`

상세 절차: [`.github/SIGNING.md`](../../.github/SIGNING.md)

## 검증 절차

Secrets 등록 후:

1. `main` 브랜치에 변경사항 push
2. GitHub Actions → `Swift CI` 워크플로우 실행
3. 다음 스텝 확인:
   - `Install signing (if available)` → 성공
   - `UITest (signed)` → 성공, `MoAIStudioUITests` 결과 출력

## 상태 전이

```
M2 완료: C-1 "부분 해소" (ad-hoc best-effort)
    ↓ 인프라 구성 (이 커밋)
post-M2: C-1 "인프라 준비 완료" (Secrets 업로드 1회 시 자동화 활성)
    ↓ GOOS행님 Secrets 업로드
완전 해소: MoAIStudioUITests CI 완전 자동화
```

## 관련 링크

- 설정 가이드: [`.github/SIGNING.md`](../../.github/SIGNING.md)
- CI 워크플로우: [`.github/workflows/ci-swift.yml`](../../.github/workflows/ci-swift.yml)
- Composite Action: [`.github/actions/install-signing/action.yml`](../../.github/actions/install-signing/action.yml)
- Apple TN3125: <https://developer.apple.com/documentation/technotes/tn3125-inside-code-signing-provisioning-profiles>
