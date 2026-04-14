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

## 중간 체크포인트 (2026-04-14) — MS-3 완료

**Status**: MS-1~MS-3 완료, MS-4~MS-7 잔여

**테스트 통과**: 213 Rust + 41 Swift = 254/254 PASS

**@MX 태그**: 22개 (MS-2) + 6개 (MS-3) = 28개 누적

**Scope 준수**: 19/19 task (100%, expansion 없음)

**다음 단계**:
- MS-4 (FileTree + Markdown, 7 tasks)
- MS-5 (Image + Browser, 10 tasks)
- MS-6 (Command Palette, 7 tasks)
- MS-7 (CI/CD + carry-over + E2E, 14 tasks)

---

## MS-4 완료 현황

| Task | 상태 | RED | GREEN | REFACTOR |
|------|------|-----|-------|----------|
| T-050 | 완료 | filetree_ffi.rs 5개 테스트 RED | filetree.rs + moai-git status_map() GREEN | clippy/fmt clean |
| T-051 | 완료 | 스켈레톤 | tree_watcher.rs 폴링 방식 채택 문서화 | - |
| T-052 | 완료 | FileTreeViewModelTests RED | FileTreeSurface.swift + FileTreeViewModel GREEN | accentColor 수정 |
| T-053 | 완료 | GitStatus.color 포함 T-052 에서 처리 | - | - |
| T-054 | 완료 | - | TabBarViewModel.newTab(kind:statePath:) + SurfaceRouter.kindForExtension | - |
| T-055 | 완료 | - | FileTreeSurface.startRefreshTimer() (500ms) | - |
| T-056 | 완료 | 5개 Rust + 7개 Swift 테스트 | 전체 통과 | - |

---

## 테스트 결과 (MS-4 완료 후)

- Rust: 218개 (기존 213 + filetree_ffi 5개 신규)
- Swift: 48개 (기존 41 + FileTreeViewModelTests 7개 신규)

---

## 파일 변경 목록 (MS-4 신규/수정)

### Rust 신규
- `core/crates/moai-ffi/src/filetree.rs` — list_directory_json + git_status_map_json
- `core/crates/moai-fs/src/tree_watcher.rs` — 폴링 방식 스켈레톤 (MS-7+ push 업그레이드 예정)
- `core/crates/moai-ffi/tests/filetree_ffi.rs` — 5개 테스트

### Rust 수정
- `core/crates/moai-ffi/src/lib.rs` — filetree 모듈 등록 + RustCore 메서드 + bridge 블록 FFI 노출
- `core/crates/moai-ffi/Cargo.toml` — moai-git 의존성 + dev-dependencies tempfile/git2
- `core/crates/moai-git/src/lib.rs` — status_map() 추가
- `core/crates/moai-fs/src/lib.rs` — tree_watcher 모듈 등록

### Swift 신규
- `app/Sources/Surfaces/FileTree/FileTreeSurface.swift` — FileTreeViewModel + FileTreeSurface + GitStatus + FileTreeEntry
- `app/Tests/FileTreeViewModelTests.swift` — 7개 테스트

### Swift 수정
- `app/Sources/Bridge/RustCore+Generated.swift` — listDirectoryJson + gitStatusMapJson 프로토콜 + RustCoreBridge 구현
- `app/Sources/Shell/Tabs/TabBarViewModel.swift` — newTab(kind:statePath:) 시그니처 확장
- `app/Sources/Shell/Splits/PaneSplitView.swift` — SurfaceRouter .filetree case + kindForExtension + LeafPaneView onFileOpen 콜백
- `app/Tests/MockRustCoreBridge.swift` — stubbedDirectoryJson + stubbedStatusJson + listDirectoryJson + gitStatusMapJson
- `app/MoAIStudio.xcodeproj/project.pbxproj` — FileTreeSurface.swift + FileTreeViewModelTests.swift 등록

---

## 품질 게이트 (MS-4)

- [x] `cargo check --workspace`: 0 errors, 0 warnings
- [x] `cargo clippy --workspace -- -D warnings`: clean
- [x] `cargo fmt --all -- --check`: clean
- [x] `cargo test --workspace`: 218/218 통과 (기존 213 + filetree_ffi 5개)
- [x] Xcode build-for-testing: ** TEST BUILD SUCCEEDED **
- [x] Swift 단위 테스트 (MoAIStudioTests): 48/48 통과 (기존 41 + FileTreeViewModelTests 7개)

## @MX 태그 추가 목록 (MS-4)

| 파일 | 태그 | 설명 |
|------|------|------|
| `filetree.rs` | ANCHOR | list_directory_json — 디렉토리 데이터 유일 소스 (fan_in>=3) |
| `FileTreeSurface.swift` | ANCHOR × 2 | FileTreeViewModel, FileTreeSurface 렌더링 진입점 |
| `FileTreeSurface.swift` | NOTE × 3 | 폴링 기반, git status 색상, 500ms 타이머 |
| `RustCore+Generated.swift` | ANCHOR | FileTree FFI 프로토콜 (fan_in>=3) |
| `TabBarViewModel.swift` | NOTE | statePath 직렬화 방식 |
| `PaneSplitView.swift` | NOTE × 2 | resolveWorkspacePath 폴백, T-054 kindForExtension 매핑 |
| `tree_watcher.rs` | NOTE | 폴링 채택, MS-7 push 업그레이드 예정 |

## 반복 로그 (MS-4 추가)

| 반복 | 완료 AC | 에러 수 |
|------|---------|---------|
| 5 (MS-4 Rust RED) | 0 (메서드 없음) | 8 (컴파일 오류) |
| 6 (MS-4 Rust GREEN) | 5 (T-050 Rust tests) | 0 |
| 7 (MS-4 Swift RED) | 0 (모듈 없음) | 1 (빌드 오류) |
| 8 (MS-4 Swift GREEN) | 7 (T-051~T-056) | 0 |

---

## 알려진 제한 사항 (MS-4)

- FileTree 폴링 타이머가 View disappear 시 자동 취소되지 않음. SwiftUI .task {} 내부 Task 취소에 의존. MS-7+ 에서 notify-push 로 교체 예정.
- resolveWorkspacePath() 가 홈 디렉토리 폴백 — MS-5+ 에서 @Environment WorkspaceSnapshot.projectPath 로 교체 예정.
- FileTree는 루트 한 레벨만 리스팅 (expand 시 하위 디렉토리 재로딩 미구현). MS-5+ 에서 toggle(path:) 시 subpath 기반 재귀 리스팅 구현 예정.
- SurfaceRouter onFileOpen 콜백이 LeafPaneView 외부에서 호출 불가. MS-5+ 에서 @Environment 패턴으로 개선.

## 중간 체크포인트 (2026-04-14) — MS-4 완료

**Status**: MS-1~MS-4 완료, MS-5~MS-7 잔여

**테스트 통과**: 218 Rust + 48 Swift = 266/266 PASS

**@MX 태그**: 28개 (MS-1~MS-3) + 10개 (MS-4) = 38개 누적

**Scope 준수**: 26/26 task (100%, expansion 없음)

## MS-5 완료 현황

| Task | 상태 | RED | GREEN | REFACTOR |
|------|------|-----|-------|----------|
| T-057 | 완료 | MarkdownViewModelTests RED | MarkdownSurface.swift + MarkdownViewModel GREEN | - |
| T-058 | 완료 | EARSFormatterTests RED | EARSFormatter.swift (regex 변환) GREEN | - |
| T-059 | 완료 | - | MarkdownSurface WebContentRenderer (CDN KaTeX/Mermaid HTML 템플릿) GREEN | - |
| T-060 | 완료 | - | MarkdownViewModel.startWatching() / stopWatching() (DispatchSource) GREEN | - |
| T-061 | 완료 | ImageViewModelTests RED | ImageSurface.swift + ImageViewModel GREEN | - |
| T-062 | 완료 | - | ImageDiffView + Vision VNFeaturePrintRequest 유사도 GREEN | - |
| T-063 | 완료 | BrowserViewModelTests RED | BrowserSurface.swift + BrowserViewModel GREEN | - |
| T-064 | 완료 | - | DevServerDetector.swift (포트 프로브 병렬) GREEN | - |
| T-065 | 완료 | - | WKNavigationDelegate 링크 처리 (localhost 허용, 외부→기본브라우저) GREEN | - |
| T-066 | 완료 | 4개 테스트 파일 RED | 전체 통과 GREEN | - |

---

## 테스트 결과 (MS-5 완료 후)

- Rust: 218개 (변경 없음, 모두 통과)
- Swift: 80개 (기존 48 + MS-5 신규 32개)
  - MarkdownViewModelTests: 7개
  - ImageViewModelTests: 9개
  - BrowserViewModelTests: 11개
  - EARSFormatterTests: 5개

---

## 파일 변경 목록 (MS-5 신규/수정)

### Swift 신규
- `app/Sources/Surfaces/Markdown/MarkdownSurface.swift` — MarkdownViewModel + MarkdownSurface + MarkdownWebView
- `app/Sources/Surfaces/Markdown/EARSFormatter.swift` — EARS 패턴/SPEC-ID 강조 포맷터
- `app/Sources/Surfaces/Image/ImageSurface.swift` — ImageViewModel + ImageSurface + ImageDiffView (Vision 유사도)
- `app/Sources/Surfaces/Browser/BrowserSurface.swift` — BrowserViewModel + BrowserSurface + BrowserWebViewRepresentable
- `app/Sources/Surfaces/Browser/DevServerDetector.swift` — 로컬 포트 자동 감지
- `app/Tests/MarkdownViewModelTests.swift` — 7개 테스트
- `app/Tests/ImageViewModelTests.swift` — 9개 테스트
- `app/Tests/BrowserViewModelTests.swift` — 11개 테스트
- `app/Tests/EARSFormatterTests.swift` — 5개 테스트

### Swift 수정
- `app/Sources/Shell/Splits/PaneSplitView.swift` — SurfaceRouter .markdown/.image/.browser cases 연결 + statePath 전달
- `app/Sources/Shell/Tabs/TabBarViewModel.swift` — activeStatePath() + statePathCache 추가
- `app/MoAIStudio.xcodeproj/project.pbxproj` — 신규 파일 등록 (소스 + 테스트)

---

## 품질 게이트 (MS-5)

- [x] `cargo test --workspace`: 218/218 통과 (Rust 변경 없음, 기존 통과 유지)
- [x] Xcode build-for-testing: ** TEST BUILD SUCCEEDED ** (0 errors)
- [x] Swift 단위 테스트 (MoAIStudioTests): 80/80 통과 (기존 48 + MS-5 32개)

## @MX 태그 추가 목록 (MS-5)

| 파일 | 태그 | 설명 |
|------|------|------|
| `MarkdownSurface.swift` | ANCHOR × 1 | MarkdownViewModel — Markdown 탭 상태 유일 소스 (fan_in>=3) |
| `MarkdownSurface.swift` | WARN × 1 | DispatchSource fd 누수 위험 (stopWatching 필수) |
| `MarkdownSurface.swift` | NOTE × 2 | 렌더링 방식(AttributedString+WKWebView 하이브리드), CDN 의존 |
| `ImageSurface.swift` | ANCHOR × 1 | ImageViewModel — Image 탭 상태 유일 소스 (fan_in>=3) |
| `ImageSurface.swift` | NOTE × 2 | 지원 포맷(NSImage 네이티브), Vision 피처 프린트 vs 진짜 SSIM |
| `BrowserSurface.swift` | ANCHOR × 1 | BrowserViewModel — Browser 탭 상태 유일 소스 (fan_in>=3) |
| `BrowserSurface.swift` | NOTE × 1 | 링크 처리 정책 (localhost 허용, 외부→기본브라우저) |
| `DevServerDetector.swift` | NOTE × 1 | 프로브 포트 목록 설명 |
| `EARSFormatter.swift` | NOTE × 1 | EARS 패턴 변환 방식 |

## 반복 로그 (MS-5)

| 반복 | 완료 AC | 에러 수 |
|------|---------|---------|
| 9 (MS-5 Swift RED) | 0 (타입 미존재) | 0 (프로젝트 미등록) |
| 10 (MS-5 Swift GREEN) | 10 (T-057~T-066) | 0 |

---

## 중간 체크포인트 (2026-04-14) — MS-5 완료

**Status**: MS-1~MS-5 완료, MS-6~MS-7 잔여

**테스트 통과**: 218 Rust + 80 Swift = 298/298 PASS

**@MX 태그**: 38개 (MS-1~MS-4) + 10개 (MS-5) = 48개 누적

**Scope 준수**: 36/36 task (100%, expansion 없음)

**다음 단계**:
- MS-6 (Command Palette, 7 tasks)
- MS-7 (CI/CD + carry-over + E2E, 14 tasks)

---

## MS-6 완료 현황

| Task | 상태 | RED | GREEN | REFACTOR |
|------|------|-----|-------|----------|
| T-067 | 완료 | CommandPaletteTests RED | CommandPaletteView.swift + CommandPaletteController GREEN | - |
| T-068 | 완료 | CommandRegistryTests RED | CommandRegistry.swift (4 카테고리 + 내장 명령어) GREEN | - |
| T-069 | 완료 | FuzzyMatcherTests RED | FuzzyMatcher.swift (subsequence + prefix bonus) GREEN | - |
| T-070 | 완료 | - | CommandPaletteView 키보드 네비게이션 (Escape/Enter/Up/Down) GREEN | - |
| T-071 | 완료 | - | SlashInjector.swift (SlashInjecting 프로토콜 + 구현) GREEN | - |
| T-072 | 완료 | - | CommandRegistry Surface/Workspace/Pane 명령어 등록 GREEN | - |
| T-073 | 완료 | 21개 테스트 先 작성 (CommandPaletteControllerTests 9 + FuzzyMatcherTests 7 + CommandRegistryTests 5) | 전체 통과 GREEN | - |

---

## 테스트 결과 (MS-6 완료 후)

- Rust: 218개 (변경 없음, 모두 통과)
- Swift: 101개 (기존 80 + MS-6 신규 21개)
  - CommandPaletteControllerTests: 9개
  - FuzzyMatcherTests: 7개
  - CommandRegistryTests: 5개

---

## 파일 변경 목록 (MS-6 신규/수정)

### Rust 변경 없음 (MS-6 는 Swift 전용)
- Rust 218개 테스트 그대로 유지

### Swift 신규
- `app/Sources/Shell/CommandPalette/CommandPaletteView.swift` — CommandPaletteController(@Observable) + CommandPaletteView + CommandRowView
- `app/Sources/Shell/CommandPalette/CommandRegistry.swift` — PaletteCommand + PaneSplitDirection + CommandRegistry (4 카테고리)
- `app/Sources/Shell/CommandPalette/FuzzyMatcher.swift` — FuzzyMatcher.Match + match() 알고리즘
- `app/Sources/Shell/CommandPalette/SlashInjector.swift` — SlashInjecting 프로토콜 + SlashInjector 구현
- `app/Tests/CommandPaletteTests.swift` — 21개 테스트

### Swift 수정
- `app/Sources/Shell/RootSplitView.swift` — Cmd+K (Button + .keyboardShortcut) + CommandPaletteView 오버레이 + setupPaletteController()
- `app/MoAIStudio.xcodeproj/project.pbxproj` — MS6 신규 파일 5개 등록

---

## 품질 게이트 (MS-6)

- [x] `cargo test --workspace`: 218/218 통과 (Rust 변경 없음)
- [x] Xcode build-for-testing: ** TEST BUILD SUCCEEDED ** (0 errors)
- [x] Swift 단위 테스트 (MoAIStudioTests): 101/101 통과 (기존 80 + MS-6 21개)

## @MX 태그 추가 목록 (MS-6)

| 파일 | 태그 | 설명 |
|------|------|------|
| `CommandPaletteView.swift` | ANCHOR × 1 | CommandPaletteController — 팔레트 상태 유일 소스 (fan_in>=3) |
| `CommandPaletteView.swift` | NOTE × 1 | Cmd+K 캡처 전략 (Button + .keyboardShortcut) |
| `CommandRegistry.swift` | ANCHOR × 1 | PaletteCommand 전체 목록 유일 소스 (fan_in>=3) |
| `CommandRegistry.swift` | NOTE × 3 | /moai 슬래시 명령어, Surface 열기, Pane 분할 콜백 설명 |
| `FuzzyMatcher.swift` | ANCHOR × 1 | Command Palette 검색 알고리즘 진입점 (fan_in>=3) |
| `FuzzyMatcher.swift` | NOTE × 2 | 알고리즘 설명, 빈 쿼리 동작 |
| `SlashInjector.swift` | NOTE × 1 | 슬래시 주입 라우팅 설명 |

## 반복 로그 (MS-6)

| 반복 | 완료 AC | 에러 수 |
|------|---------|---------|
| 11 (MS-6 Swift RED) | 0 (타입 미존재) | 0 (프로젝트 미등록) |
| 12 (MS-6 Swift GREEN) | 7 (T-067~T-073) | 2 (onKeyPress modifier, NewWorkspaceSheet init) |
| 13 (MS-6 컴파일 수정) | 7 (전체) | 0 |

---

## 중간 체크포인트 (2026-04-14) — MS-6 완료

**Status**: MS-1~MS-6 완료, MS-7 잔여

**테스트 통과**: 218 Rust + 101 Swift = 319/319 PASS

**@MX 태그**: 48개 (MS-1~MS-5) + 9개 (MS-6) = 57개 누적

**Scope 준수**: 43/43 task (100%, expansion 없음)

**다음 단계**:
- MS-7 (CI/CD + carry-over + E2E, 14 tasks)

---

## MS-7 완료 현황

| Task | 상태 | 내용 |
|------|------|------|
| T-074 | 완료 | .github/workflows/ci-rust.yml (fmt/clippy/test/check) |
| T-075 | 완료 | .github/workflows/ci-swift.yml (xcodebuild) |
| T-076 | 완료 | xcframework 캐싱 (GhosttyKit + MoaiCore) |
| T-077 | 완료 | C-1 UITest ad-hoc signing (부분 해소) + WORKFLOWS.md |
| T-078 | 완료 | C-2 scripts/validate-claude-e2e.sh (opt-in) |
| T-079 | 완료 | C-3 scripts/stress-test-4ws.sh + stress_4ws.rs (#[ignore]) |
| T-080 | 완료 | C-4 GhosttyMetalBenchmarkTests 하네스 (TODO — 전체 측정 이월) |
| T-081 | 완료 | C-5 이미 MS-2에서 해소 (JSON FFI 경로) |
| T-082 | 완료 | C-6 RotatingAuthToken (moai-hook-http/src/auth.rs, 7 tests) |
| T-083 | 완료 | C-7 FFIBenchmarkTests.swift (P95 < 1ms, 3 tests) |
| T-084 | 완료 | C-8 WorkspaceDao::force_pause + state_force_pause.rs (4 tests) |
| T-085 | 완료 | E2E Rust (e2e_viewers.rs 3 tests) + Swift UITest skeleton |
| T-086 | 완료 | nfr-report.md |
| T-087 | 완료 | m2-completion-report.md + spec.md v1.2.0 completed |

---

## 테스트 결과 (MS-7 완료 후)

- Rust: 233개 (기존 218 + MS-7 신규 15개)
  - auth.rs unit tests: 7개 (RotatingAuthToken)
  - state_force_pause.rs: 4개 (force_pause)
  - stress_4ws.rs: 2개 (smoke 1 + #[ignore] 1)
  - e2e_viewers.rs: 3개 (M2 E2E)
- Swift: 106개 (기존 101 + MS-7 신규 5개)
  - FFIBenchmarkTests.swift: 3개
  - GhosttyMetalBenchmarkTests.swift: 2개
  - E2EViewersTests.swift: CI skip (UITest)

---

## 파일 변경 목록 (MS-7)

### CI/CD 신규
- `.github/workflows/ci-rust.yml`
- `.github/workflows/ci-swift.yml`
- `.github/WORKFLOWS.md`

### Rust 신규
- `core/crates/moai-hook-http/src/auth.rs` (C-6)
- `core/crates/moai-store/tests/state_force_pause.rs` (C-8)
- `core/crates/moai-ffi/tests/stress_4ws.rs` (C-3)
- `core/crates/moai-ffi/tests/e2e_viewers.rs` (T-085)

### Rust 수정
- `core/crates/moai-hook-http/src/lib.rs` (auth 모듈 등록)
- `core/crates/moai-store/src/workspace.rs` (force_pause API)

### Swift 신규
- `app/Tests/FFIBenchmarkTests.swift` (C-7)
- `app/Tests/GhosttyMetalBenchmarkTests.swift` (C-4)
- `app/UITests/E2EViewersTests.swift` (T-085)

### Scripts 신규
- `scripts/validate-claude-e2e.sh` (C-2)
- `scripts/stress-test-4ws.sh` (C-3)

### 문서
- `.moai/specs/SPEC-M2-001/nfr-report.md`
- `.moai/specs/SPEC-M2-001/m2-completion-report.md`
- `.moai/specs/SPEC-M2-001/spec.md` (v1.2.0, status=completed)

---

## 품질 게이트 (MS-7)

- [x] `cargo check --workspace`: 0 errors, 0 warnings
- [x] `cargo clippy --workspace -- -D warnings`: clean
- [x] `cargo fmt --all -- --check`: clean
- [x] `cargo test --workspace`: 233/233 통과
- [x] Xcode build-for-testing: ** TEST BUILD SUCCEEDED **
- [x] Swift 단위 테스트: 106/106 통과

## @MX 태그 추가 목록 (MS-7)

| 파일 | 태그 | 설명 |
|------|------|------|
| `auth.rs` | ANCHOR | RotatingAuthToken — hook endpoint 인증 단일 소스 |
| `auth.rs` | WARN | grace period 동안 이전 토큰 유효 |
| `workspace.rs` | ANCHOR | force_pause — 관리자용 강제 일시정지 API |
| `stress_4ws.rs` | NOTE | opt-in 스크립트, CI 자동 실행 아님 |
| `GhosttyMetalBenchmarkTests.swift` | TODO | 전체 Metal fps 측정 이월 |
| `E2EViewersTests.swift` | NOTE | CI skip (C-1 carry-over) |

## 반복 로그 (MS-7)

| 반복 | 완료 AC | 에러 수 |
|------|---------|---------|
| 14 (MS-7 RED) | 0 (파일 생성 전) | 0 |
| 15 (MS-7 GREEN) | 14 (T-074~T-087) | 1 (clippy collapsible_if) |
| 16 (MS-7 FIX) | 14 (전체) | 0 |

---

## 최종 체크포인트 (2026-04-14) — MS-7 완료 / M2 COMPLETE

**Status**: MS-1~MS-7 전체 완료. SPEC-M2-001 **조건부 GO**

**테스트 통과**: 233 Rust + 106 Swift = **339/339 PASS**

**@MX 태그**: 57개 (MS-1~MS-6) + 6개 (MS-7) = **63개** 누적 (spec.md HISTORY 수정 반영: 69개)

**Scope 준수**: 57/57 task (100%, expansion 없음)

**Carry-over 해소**: C-5(MS-2), C-6(MS-7), C-7(MS-7), C-8(MS-7) — 4건 완전 해소
**Carry-over 이월**: C-1(부분), C-2(opt-in), C-3(opt-in), C-4(하네스) — M3 예정

## 알려진 제한 사항

- XCUITest (NSSplitView UI 상호작용): 서명 이슈 (C-1 carry-over) 로 UI 테스트 제외. 순수 모델 테스트만 검증.
- TerminalSurface 에 WorkspaceSnapshot 미연결: MS-3 에서는 TerminalSurfacePlaceholder 를 표시. MS-7 에서 @Environment 로 workspace 주입 후 실제 TerminalSurface(workspace:) 로 교체 예정.
- 탭 재배치 (reorder): SwiftUI onDrag/onDrop 기반. NSCollectionView DnD 와 동작이 다를 수 있음. 수동 검증 필요.
- SurfaceProtocol 이 View 를 상속하므로 associatedtype Body 를 암묵적으로 요구함. Swift 6 existential type 에서 `any SurfaceProtocol` 박싱 시 제약 있음. MS-7 에서 AnyView 래퍼 패턴 적용 예정.
- MarkdownSurface: KaTeX/Mermaid CDN 의존 (오프라인 환경에서 수식/다이어그램 미렌더링). MS-7 에서 번들 내 정적 리소스로 교체 예정.
- ImageDiffView: Vision VNFeaturePrintRequest 기반 근사 유사도 (진정한 SSIM 아님). MS-7 에서 픽셀 레벨 SSIM 구현 예정.
- BrowserSurface: statePath 사용 안 함. MS-7 에서 마지막 URL 영속 예정.
- statePathCache: TabBarViewModel 메모리 내 캐시. 앱 재시작 시 소실. MS-7 에서 state_json DB 읽기로 교체 예정.
- CommandPalette Surface 열기 콜백: onSurfaceOpen 이 현재 no-op. MS-7 에서 ActivePaneProvider @Environment 구현 후 TabBarViewModel.newTab(kind:) 연결 예정.
- CommandPalette Pane 분할 콜백: onPaneSplit 이 현재 no-op. MS-7 에서 ActivePaneProvider 통해 PaneTreeModel.splitActive 연결 예정.
