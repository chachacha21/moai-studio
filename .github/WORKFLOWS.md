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

### UITest 서명 전략 (C-1 carry-over 해소)

- **현재 구현**: `HAS_SIGNING` 조건부 분기
  - Secrets 있음: `.github/actions/install-signing` composite action → Developer ID Application 서명 → UITest 실행
  - Secrets 없음 (fork PR 등): UITest skip + 안내 메시지 출력
- **필요 Secrets**: `BUILD_CERTIFICATE_BASE64`, `P12_PASSWORD`, `KEYCHAIN_PASSWORD`, `APPLE_TEAM_ID`
- **설정 방법**: [`.github/SIGNING.md`](SIGNING.md) 참고
- **상태**: 인프라 준비 완료 — Secrets 업로드 1회 후 완전 자동화 활성

---

## 수동/opt-in 스크립트

| 스크립트 | 목적 | 실행 조건 |
|---------|------|---------|
| `scripts/validate-claude-e2e.sh` | Claude CLI E2E 검증 (C-2) | `ANTHROPIC_API_KEY` 설정 + `claude` binary in PATH |
| `scripts/stress-test-4ws.sh` | 4-워크스페이스 RSS 스트레스 (C-3) | 빌드된 `MoAI Studio.app` 필요 |

이 스크립트들은 CI에서 **자동 실행되지 않습니다**. 로컬 또는 수동 파이프라인에서만 실행하세요.

---

## 이월 항목

| 항목 | 상태 | 비고 |
|------|------|------|
| C-1 UITest 전체 서명 | 인프라 준비 완료 | Secrets 업로드 1회 후 자동화 활성 ([SIGNING.md](SIGNING.md)) |
| C-2 Claude CLI E2E | opt-in 스크립트 | CI 자동화는 M3 |
| C-3 10min 스트레스 | opt-in 스크립트 | CI 자동화는 M3 |
| C-4 Metal 60fps 벤치마크 | 하네스 생성 완료 | 전체 측정은 GhosttyHost wiring 완료 후 |
