# SPEC-M2-001 MS-1 진행 상황

---
spec_id: SPEC-M2-001
sprint: MS-1
started: 2026-04-14
---

## 진행 현황

| Task | 상태 | RED | GREEN | REFACTOR |
|------|------|-----|-------|----------|
| T-031 | 완료 | panes_surfaces_integration.rs 실패 | V3__panes_surfaces.sql + migrate() | fmt/clippy clean |
| T-032 | 완료 | (T-031 과 동일 파일) | V3 surfaces 테이블 (동일 SQL) | fmt/clippy clean |
| T-033 | 완료 | panes_surfaces_integration.rs 컴파일 오류 | pane.rs PaneDao | fmt/clippy clean |
| T-034 | 완료 | panes_surfaces_integration.rs 컴파일 오류 | surface.rs SurfaceDao | fmt/clippy clean |
| T-035 | 완료 | pane_surface_ffi.rs 컴파일 오류 | pane.rs FFI + lib.rs bridge | fmt/clippy clean |
| T-036 | 완료 | pane_surface_ffi.rs 컴파일 오류 | surface.rs FFI + lib.rs bridge | fmt/clippy clean |
| T-037 | 완료 | 위 테스트 파일들이 RED | 구현 후 모두 GREEN | fmt/clippy clean |

## 테스트 결과

- 시작 전: 186개
- 완료 후: 208개 (+22개)
  - moai-store panes_surfaces_integration: 15개 신규
  - moai-ffi pane_surface_ffi: 7개 신규

## 품질 게이트

- [x] `cargo check --workspace`: 0 errors, 0 warnings
- [x] `cargo clippy --workspace -- -D warnings`: clean
- [x] `cargo fmt --all -- --check`: clean
- [x] `cargo test --workspace`: 208/208 통과 (기존 186 + 신규 22)

## @MX 태그

| 파일 | 태그 | 설명 |
|------|------|------|
| `pane.rs` | ANCHOR | PaneDao::insert (fan_in>=3) |
| `pane.rs` | NOTE | SplitKind 설명 |
| `surface.rs` | ANCHOR | SurfaceDao::insert (fan_in>=3) |
| `surface.rs` | NOTE | SurfaceKind M2 구현 범위 |
| `moai-ffi/pane.rs` | ANCHOR | create_pane FFI 진입점 |
| `moai-ffi/surface.rs` | ANCHOR | create_surface FFI 진입점 |
| `lib.rs` | NOTE | V3 마이그레이션 설명 |
| `lib.rs` | NOTE | conn_for_test 용도 |

## 반복 로그

| 반복 | 완료 AC | 에러 수 |
|------|---------|---------|
| 1 | 0 | 0 |
| 2 | 7 (T-031~T-037) | 0 |
