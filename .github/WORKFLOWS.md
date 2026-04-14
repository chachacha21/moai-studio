# CI/CD Workflows (SPEC-M2-001 MS-7)

## 개요

| 워크플로 | 파일 | 트리거 |
|---------|------|--------|
| Rust CI | `workflows/ci-rust.yml` | `core/**` push/PR |
| Swift CI | `workflows/ci-swift.yml` | `app/**` push/PR |

---

## Rust CI (`ci-rust.yml`)

실행 순서: `cargo fmt` → `cargo clippy` → `cargo test` → `cargo check`

- Runner: `macos-14`
- Rust Cache: `Swatinem/rust-cache@v2` (`core -> target`)
- 실패 조건: fmt 불일치, clippy 경고(`-D warnings`), 테스트 실패

---

## Swift CI (`ci-swift.yml`)

실행 순서: xcframework 캐시 → `build-for-testing` → `test-without-building` → UITest ad-hoc

### xcframework 캐싱

| 아티팩트 | 캐시 키 | 빌드 스크립트 |
|---------|---------|-------------|
| `GhosttyKit.xcframework` | `ghostty-<os>-<hash(vendor/ghostty)>` | `scripts/build-ghostty-xcframework.sh` |
| `MoaiCore.xcframework` | `rust-xc-<os>-<hash(core/**/*.rs)>` | `scripts/build-rust-xcframework.sh` |

GhosttyKit 빌드는 Metal Toolchain 미설치 시 `continue-on-error: true` 로 실패를 건너뜁니다.

### UITest 서명 전략 (C-1 carry-over)

- **현재 구현**: Ad-hoc 서명(`CODE_SIGN_IDENTITY="-"`)으로 best-effort 실행
- **완전 해소 조건**: Apple Developer Team ID를 GitHub Secrets에 등록 후 provisioning 프로파일 구성 필요
  - 필요 Secrets: `APPLE_TEAM_ID`, `APPLE_CERTIFICATE_BASE64`, `APPLE_CERTIFICATE_PASSWORD`
  - 완전 구현 참고: [Apple Xcode signing in CI](https://developer.apple.com/documentation/xcode/signing-a-macos-app-for-distribution)
- **상태**: 부분 해소 (M3에서 완전 해소 예정)

---

## 수동/opt-in 스크립트

| 스크립트 | 목적 | 실행 조건 |
|---------|------|---------|
| `scripts/validate-claude-e2e.sh` | Claude CLI E2E 검증 (C-2) | `ANTHROPIC_API_KEY` 설정 + `claude` binary in PATH |
| `scripts/stress-test-4ws.sh` | 4-워크스페이스 RSS 스트레스 (C-3) | 빌드된 `MoAI Studio.app` 필요 |

이 스크립트들은 CI에서 **자동 실행되지 않습니다**. 로컬 또는 수동 파이프라인에서만 실행하세요.

---

## 이월 항목 (M3 예정)

| 항목 | 상태 | 비고 |
|------|------|------|
| C-1 UITest 전체 서명 | 부분 해소 | Apple Dev 계정 Secrets 설정 필요 |
| C-2 Claude CLI E2E | opt-in 스크립트 | CI 자동화는 M3 |
| C-3 10min 스트레스 | opt-in 스크립트 | CI 자동화는 M3 |
| C-4 Metal 60fps 벤치마크 | 하네스 생성 완료 | 전체 측정은 GhosttyHost wiring 완료 후 |
