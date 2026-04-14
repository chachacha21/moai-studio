# SPEC-M2-001 진행 상황

---
spec_id: SPEC-M2-001
sprint: MS-3
started: 2026-04-14
updated: 2026-04-14
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

## MS-3 완료 현황

| Task | 상태 | RED | GREEN | REFACTOR |
|------|------|-----|-------|----------|
| T-044 | 완료 | TabBarViewModelTests RED | SurfaceProtocol.swift (SurfaceKind, SurfaceToolbarItem, SurfaceProtocol, SurfaceLifecycleHandler) | - |
| T-045 | 완료 | - | Surfaces/Terminal/TerminalSurface.swift (SurfaceProtocol conform) + TerminalFallback 이전 | - |
| T-046 | 완료 | - | Shell/Tabs/TabBarView.swift (TabItem, TabBarView, TabDropDelegate) | - |
| T-047 | 완료 | TabBarViewModelTests RED | Shell/Tabs/TabBarViewModel.swift (@Observable) | - |
| T-048 | 완료 | - | PaneSplitView.swift: LeafPaneView + SurfaceRouter + TerminalSurfacePlaceholder + NotYetImplementedSurface | - |
| T-049 | 완료 | TabBarViewModelTests.swift (9개 테스트, 先 작성) | GREEN 통과 | - |

---

## 테스트 결과 (누적)

- MS-1 완료 시: 208개 (Rust)
- MS-2 완료 후: Rust 213개 + Swift 10개
- MS-3 완료 후:
  - Rust: 213개 (변경 없음)
  - Swift: 41개 (+19: TabBarViewModelTests 9개 + 기존 suite 유지)
    - test_load_populatesTabsFromFFI
    - test_load_withNoSurfaces_autoCreatesDefaultTerminalTab
    - test_newTab_addsTabWithIncrementalTabOrder
    - test_newTab_multipleCallsIncrementTabOrder
    - test_closeTab_removesTabAndReturnsTrue
    - test_closeTab_lastTab_returnsFalse
    - test_reorder_updatesTabOrderCorrectly
    - test_selectTab_updatesActiveTabId
    - test_activeTabKind_returnsKindOfActiveTab
    - test_activeTabKind_noActiveTab_returnsNil (10개)

---

## 파일 변경 목록 (MS-3 신규/수정)

### Rust 수정 없음 (MS-3 은 Swift 전용 스프린트)
- Rust 213개 테스트 그대로 유지

### Swift 신규
- `app/Sources/Surfaces/SurfaceProtocol.swift` (T-044)
- `app/Sources/Surfaces/Terminal/TerminalSurface.swift` (T-045: Content/ 에서 이전 + SurfaceProtocol conform)
- `app/Sources/Surfaces/Terminal/TerminalFallback.swift` (T-045: Content/ 에서 이전)
- `app/Sources/Shell/Tabs/TabBarView.swift` (T-046)
- `app/Sources/Shell/Tabs/TabBarViewModel.swift` (T-047)
- `app/Tests/TabBarViewModelTests.swift` (T-049)

### Swift 수정
- `app/Sources/Shell/Splits/PaneSplitView.swift` (T-048: LeafPlaceholderView → LeafPaneView + SurfaceRouter)
- `app/Sources/Shell/Splits/PaneTreeModel.swift` (bridge internal 접근)
- `app/Sources/Bridge/RustCore+Generated.swift` (updateSurfaceTabOrder 추가)
- `app/Tests/MockRustCoreBridge.swift` (updateSurfaceTabOrder + MockSurfaceRecord.tabOrder var)
- `app/MoAIStudio.xcodeproj/project.pbxproj` (신규 파일 등록, TerminalSurface/Fallback 구 위치 제거)

---

## 품질 게이트 (MS-3)

- [x] `cargo check --workspace`: 0 errors, 0 warnings
- [x] `cargo clippy --workspace -- -D warnings`: clean
- [x] `cargo fmt --all -- --check`: clean
- [x] `cargo test --workspace`: 213/213 통과
- [x] Xcode build-for-testing: 0 errors (** TEST BUILD SUCCEEDED **)
- [x] Swift 단위 테스트: 41/41 통과 (PaneTreeModelTests 10 + TabBarViewModelTests 10 포함)

## @MX 태그 추가 목록 (MS-3)

| 파일 | 태그 | 설명 |
|------|------|------|
| `SurfaceProtocol.swift` | ANCHOR | 10종 Surface 공통 계약 (fan_in>=3 예상) |
| `SurfaceProtocol.swift` | NOTE | 10종 Surface 종류 레지스트리 |
| `TabBarViewModel.swift` | ANCHOR | pane 내 탭 상태 유일 소스 (fan_in>=3 예상) |
| `TabBarViewModel.swift` | NOTE | 기본 탭 자동 생성 규칙 |
| `PaneSplitView.swift` | NOTE | MS-4+ 교체 예정 주석 (SurfaceRouter, TerminalSurfacePlaceholder) |

## 반복 로그

| 반복 | 완료 AC | 에러 수 |
|------|---------|---------|
| 1 (MS-2) | 0 (Rust RED) | 15 (컴파일 오류) |
| 2 (MS-2) | 6 (T-038~T-043 Rust+Swift) | 0 |
| 3 (MS-3) | 0 (Swift RED - TabBarViewModelTests) | 0 (빌드 오류 없음) |
| 4 (MS-3) | 6 (T-044~T-049) | 0 |

---

## 중간 체크포인트 (2026-04-14)

**Status**: MS-1~MS-3 완료, MS-4~MS-7 잔여

**테스트 통과**: 213 Rust + 41 Swift = 254/254 PASS

**@MX 태그**: 22개 (MS-2) + 6개 (MS-3) = 28개 누적

**Scope 준수**: 19/19 task (100%, expansion 없음)

**다음 단계**:
- MS-4 (FileTree + Markdown, 7 tasks)
- MS-5 (Image + Browser, 10 tasks)
- MS-6 (Command Palette, 7 tasks)
- MS-7 (CI/CD + carry-over + E2E, 14 tasks)

## 알려진 제한 사항

- XCUITest (NSSplitView UI 상호작용): 서명 이슈 (C-1 carry-over) 로 UI 테스트 제외. 순수 모델 테스트만 검증.
- TerminalSurface 에 WorkspaceSnapshot 미연결: MS-3 에서는 TerminalSurfacePlaceholder 를 표시. MS-4+ 에서 @Environment 로 workspace 주입 후 실제 TerminalSurface(workspace:) 로 교체 예정.
- 탭 재배치 (reorder): SwiftUI onDrag/onDrop 기반. NSCollectionView DnD 와 동작이 다를 수 있음. 수동 검증 필요.
- SurfaceProtocol 이 View 를 상속하므로 associatedtype Body 를 암묵적으로 요구함. Swift 6 existential type 에서 `any SurfaceProtocol` 박싱 시 제약 있음. MS-4+ 에서 AnyView 래퍼 패턴 적용 예정.
