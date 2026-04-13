# SPEC-M2-001 진행 상황

---
spec_id: SPEC-M2-001
sprint: MS-2
started: 2026-04-14
---

## MS-1 완료 현황 (이전)

| Task | 상태 | 요약 |
|------|------|------|
| T-031 | 완료 | V3 panes 테이블 마이그레이션 |
| T-032 | 완료 | V3 surfaces 테이블 마이그레이션 |
| T-033 | 완료 | pane CRUD (moai-store) |
| T-034 | 완료 | surface CRUD (moai-store) |
| T-035 | 완료 | pane FFI (moai-ffi) |
| T-036 | 완료 | surface FFI (moai-ffi) |
| T-037 | 완료 | Store + FFI 통합 테스트 (7개 신규) |

MS-1 완료 시 테스트: 208개

---

## MS-2 완료 현황

| Task | 상태 | RED | GREEN | REFACTOR |
|------|------|-----|-------|----------|
| T-038 | 완료 | - | PaneSplitView.swift (NSViewRepresentable) | 경고 수정 |
| T-039 | 완료 | - | PaneTreeModel.swift (@Observable) | let 최적화 |
| T-040 | 완료 | - | PaneSplitContainerView (Cmd+\, Cmd+Shift+\, Cmd+Shift+W) | - |
| T-041 | 완료 | - | NSSplitView 200pt 최소 + 드래그 ratio 영속 | - |
| T-042 | 완료 | list_panes_json/list_surfaces_json 테스트 RED | Rust JSON FFI + PaneTreeModel.load() | fmt/clippy clean |
| T-043 | 완료 | - | PaneContainer + RootSplitView 리팩터링 | - |

---

## 테스트 결과

- MS-1 완료 시: 208개 (Rust)
- MS-2 완료 후:
  - Rust: 213개 (+5: list_panes_json, list_surfaces_json 테스트)
  - Swift: 10개 신규 (PaneTreeModelTests)
    - test_load_createsRootLeafWhenEmpty
    - test_load_restoresExistingTree
    - test_splitActive_horizontal_convertsLeafToTwoChildren
    - test_splitActive_vertical_createsTopBottomChildren
    - test_splitActive_nonLeaf_returnsNil
    - test_closePane_promotesSimbling
    - test_closePane_lastPane_returnsFalse
    - test_updateRatio_persistsThroughBridge
    - test_children_returnsChildNodes
    - test_children_ofLeaf_returnsEmpty

---

## 파일 변경 목록

### Rust 신규/수정
- `core/crates/moai-ffi/src/pane.rs` — `list_panes_json()` 추가
- `core/crates/moai-ffi/src/surface.rs` — `list_surfaces_json()` 추가
- `core/crates/moai-ffi/src/lib.rs` — JSON FFI 메서드 + bridge 블록 추가
- `core/crates/moai-ffi/Cargo.toml` — serde_json 의존성 추가
- `core/crates/moai-ffi/tests/pane_surface_ffi.rs` — JSON FFI 테스트 5개 추가

### Swift 신규
- `app/Sources/Shell/Splits/PaneTreeModel.swift` (T-039, T-042)
- `app/Sources/Shell/Splits/PaneSplitView.swift` (T-038, T-040, T-041)
- `app/Sources/Shell/Content/PaneContainer.swift` (T-043)
- `app/Tests/PaneTreeModelTests.swift` (10개 테스트)

### Swift 수정
- `app/Sources/Bridge/RustCore+Generated.swift` — 프로토콜 확장 + Vectorizable 스텁 (PaneInfo, SurfaceInfo) + RustCoreBridge 구현
- `app/Sources/ViewModels/WorkspaceViewModel.swift` — bridge 접근 수준 internal 공개
- `app/Sources/Shell/RootSplitView.swift` — ContentArea → PaneContainer 교체
- `app/Tests/MockRustCoreBridge.swift` — pane/surface/workspace-db-id 메서드 추가
- `app/MoAIStudio.xcodeproj/project.pbxproj` — 신규 파일 등록

---

## 품질 게이트

- [x] `cargo check --workspace`: 0 errors, 0 warnings
- [x] `cargo clippy --workspace -- -D warnings`: clean
- [x] `cargo fmt --all -- --check`: clean
- [x] `cargo test --workspace`: 213/213 통과 (기존 208 + MS-2 신규 5)
- [x] Xcode build-for-testing: 0 errors, 0 warnings (deprecation 제외)
- [x] Swift 단위 테스트: PaneTreeModelTests 10/10 통과

## @MX 태그 추가 목록

| 파일 | 태그 | 설명 |
|------|------|------|
| `RustCore+Generated.swift` | ANCHOR | pane CRUD FFI 프로토콜 |
| `RustCore+Generated.swift` | WARN × 3 | Vectorizable stub (WorkspaceInfo, PaneInfo, SurfaceInfo) |
| `PaneTreeModel.swift` | ANCHOR | pane 상태 소스 (fan_in>=3) |
| `PaneTreeModel.swift` | NOTE × 3 | orientation 주의, parent_id=0 규약, ratio 클램프 설명 |
| `PaneSplitView.swift` | ANCHOR × 2 | 렌더링 진입점, 단축키 통합 진입점 |
| `PaneSplitView.swift` | NOTE × 3 | orientation 반전, 200pt 최소, MS-3 교체 예정 |
| `PaneContainer.swift` | ANCHOR | WorkspaceViewModel ↔ PaneTreeModel 허브 |
| `PaneContainer.swift` | NOTE | 워크스페이스별 캐시 |
| `pane.rs (ffi)` | NOTE | JSON FFI 역할, C-5 tech debt |
| `surface.rs (ffi)` | NOTE | JSON FFI 역할, C-5 tech debt |
| `lib.rs` | NOTE | Vectorizable 한계 우회 설명 |

## 반복 로그

| 반복 | 완료 AC | 에러 수 |
|------|---------|---------|
| 1 | 0 (Rust RED) | 15 (컴파일 오류) |
| 2 | 6 (T-038~T-043 Rust+Swift) | 0 |

## 알려진 제한 사항

- XCUITest (NSSplitView UI 상호작용): 서명 이슈 (C-1 carry-over) 로 UI 테스트 제외. 순수 모델 테스트만 검증.
- MS-3 이전까지 leaf pane 내부는 플레이스홀더 (LeafPlaceholderView) 표시. TabBarView + SurfaceProtocol 교체 예정.
- NSSplitViewItem 사용 방식: macOS AppKit 에서 NSViewRepresentable + NSSplitView 재귀 구성 시 NSHostingView 를 직접 서브뷰로 추가하는 방식 사용. Xcode 수동 검증 필요.
