# SPEC-M2-002 완료 보고서 — M2.5 Polish

---
spec_id: SPEC-M2-002
version: 1.0.0
completed: 2026-04-17
methodology: TDD (RED-GREEN-REFACTOR)
---

## 요약

SPEC-M2-001 이후 남아 있던 placeholder 4건을 완전히 해소하여 MoAI Studio M2.5 를 실사용 가능한 상태로 승격.

- Swift 테스트: **130/130 PASS** (기존 106 + 신규 24)
- Rust 테스트: **289/289 PASS** (회귀 0)
- cargo clippy: PASS (0 warnings)
- cargo fmt --check: PASS
- xcodebuild build-for-testing: **BUILD SUCCEEDED** (warning 증가 0)
- LSP gates: errors 0 / type_errors 0 / lint_errors 0

---

## 신규 테스트 목록 (24건)

### ActivePaneProviderTests.swift (7건)

| 테스트 | AC |
|--------|----|
| `test_defaultActivePaneContext_hasAllNilFields` | AC-1.1 |
| `test_defaultInit_hasAllNilFields` | AC-1.1 |
| `test_environmentInjection_propagatesContext` | AC-1.2 |
| `test_nestedEnvironmentOverride_wins` | AC-1.3 |
| `test_workspaceEnvironmentKey_defaultIsNil` | AC-1.2 |
| `test_workspaceEnvironmentKey_injection` | AC-1.2 |
| `test_activePaneChange_updatesWorkspaceViewModel` | AC-1.4 |
| `test_splitNode_doesNotBecomeActive_inRelease` | AC-1.6 |

### TerminalSurfaceEnvironmentTests.swift (5건)

| 테스트 | AC |
|--------|----|
| `test_terminalSurfacePlaceholder_doesNotExistInSources` | AC-2.5 |
| `test_terminalSurface_loadFailed_showsFallback` | AC-2.4 |
| `test_nstextBackend_returnsNstext` | AC-2.3 |
| `test_invalidBackendEnv_defaultsToGhostty` | AC-2.3 |
| `test_paneContainer_injectsActiveWorkspace` | AC-2.2 |

### CommandPaletteSurfaceOpenTests.swift (6건)

| 테스트 | AC |
|--------|----|
| `test_tabModelRegistration_roundTrip` | AC-3.1 |
| `test_onSurfaceOpen_filetree_callsNewTab` | AC-3.1 |
| `test_onSurfaceOpen_markdown_callsNewTab` | AC-3.2 |
| `test_onSurfaceOpen_image_callsNewTab` | AC-3.2 |
| `test_onSurfaceOpen_browser_callsNewTab` | AC-3.2 |
| `test_onSurfaceOpen_terminal_callsNewTab` | AC-3.3 |
| `test_onSurfaceOpen_nilActivePane_noops` | AC-3.4 |

### CommandPalettePaneSplitTests.swift (4건)

| 테스트 | AC |
|--------|----|
| `test_onPaneSplit_horizontal_callsSplitActive` | AC-4.1 |
| `test_onPaneSplit_vertical_callsSplitActive` | AC-4.2 |
| `test_onPaneSplit_nilPaneIdOrModel_noops` | AC-4.3 |
| `test_onPaneSplit_newPaneId_isLeafNode` | AC-4.4 |

---

## 파일 변경 내역

### 신규 파일

| 파일 | 설명 |
|------|------|
| `app/Sources/Shell/Splits/ActivePaneProvider.swift` | `ActivePaneContext`, `ActivePaneProviderKey`, `WorkspaceEnvironmentKey`, `EnvironmentValues` extension |
| `app/Tests/ActivePaneProviderTests.swift` | MS-1 테스트 스위트 (7건) |
| `app/Tests/TerminalSurfaceEnvironmentTests.swift` | MS-2 테스트 스위트 (5건) |
| `app/Tests/CommandPaletteSurfaceOpenTests.swift` | MS-3 onSurfaceOpen 테스트 스위트 (6건) |
| `app/Tests/CommandPalettePaneSplitTests.swift` | MS-3 onPaneSplit 테스트 스위트 (4건) |

### 수정된 파일

| 파일 | 주요 변경 |
|------|----------|
| `app/Sources/ViewModels/WorkspaceViewModel.swift` | `activePane: ActivePaneContext`, `tabModels: [Int64: TabBarViewModel]`, register/unregister 메서드 추가 |
| `app/Sources/Shell/Splits/PaneSplitView.swift` | `PaneSplitContainerView` activePane 동기화, `LeafPaneView` tabModels 등록/해제, `SurfaceRouter` 실 TerminalSurface 연결, `TerminalSurfacePlaceholder` 제거, `WorkspaceUnavailablePlaceholder` 신규 |
| `app/Sources/Shell/Content/PaneContainer.swift` | `WorkspaceSnapshot` → `.environment(\.activeWorkspace, ...)` 주입 |
| `app/Sources/Surfaces/Terminal/TerminalSurface.swift` | placeholder 텍스트 제거, `GhosttyMetalView: NSViewRepresentable` 도입 (런타임 동적 로딩) |
| `app/Sources/Shell/RootSplitView.swift` | `onSurfaceOpen` / `onPaneSplit` 실구현, `os.log` Logger 추가, `paneSplitKind(from:)` 헬퍼 |
| `app/Sources/Shell/Tabs/TabBarViewModel.swift` | @MX:ANCHOR fan_in 3→4 갱신, @MX:REASON 경로 추가 |
| `app/MoAIStudio.xcodeproj/project.pbxproj` | 신규 5개 파일 빌드 페이즈 등록 |

### 삭제된 파일

| 파일 | 사유 |
|------|------|
| `app/Sources/Shell/Content/TerminalSurface.swift` | MS-3 T-045 이전 전 구 위치 파일 (TerminalSurface는 Surfaces/Terminal/로 이동됨) |
| `app/Sources/Shell/Content/TerminalFallback.swift` | 동일 이유 |

---

## @MX 태그 보고서

### 추가 (ANCHOR 2건 갱신, NOTE 6건 신규/갱신)

| 파일 | 태그 | 내용 |
|------|------|------|
| `TabBarViewModel.swift` | @MX:ANCHOR 갱신 | fan_in 3→4, RootSplitView 경로 추가 |
| `WorkspaceViewModel.swift` | @MX:NOTE 신규 2건 | activePane 프로퍼티, tabModels 사전 목적 |
| `RootSplitView.swift` | @MX:NOTE 신규 2건 | onSurfaceOpen MS-3 완료, onPaneSplit MS-3 완료 |
| `PaneSplitView.swift` | @MX:NOTE 갱신 1건 | SurfaceRouter activeWorkspace 연결 |
| `TerminalSurface.swift` | @MX:WARN 유지 | GhosttyKit Metal Toolchain 의존 |

### 제거 (NOTE 3건)

| 파일 | 제거된 태그 내용 |
|------|-----------------|
| `PaneSplitView.swift` | "MS-3 이후 leaf 탭 교체 예정" (이미 완료) |
| `PaneSplitView.swift` | "MS-4+ workspace 연결 예정" (이미 완료) |
| `PaneSplitView.swift` | "MS-6+ resolveWorkspacePath" (불필요) |

---

## 해소된 Placeholder 목록

| ID | 내용 | 해소 방법 |
|----|------|----------|
| P-1 | GhosttyHost body — placeholder 텍스트 3줄 잔존 | `GhosttyMetalView: NSViewRepresentable` 교체, 런타임 `NSClassFromString` 동적 로딩 |
| P-2 | `onSurfaceOpen: { _ in }` no-op | `vm.tabModels[paneId]?.newTab(kind:)` 실구현 |
| P-3 | `onPaneSplit: { _ in }` no-op | `model.splitActive(paneId, direction:)` 실구현 |
| P-4 | `ActivePaneProvider` / `activePaneId` 미구현 | `ActivePaneContext` struct + EnvironmentKey + `PaneSplitContainerView` onChange 동기화 |

---

## Grep 검증 결과

| 항목 | 결과 |
|------|------|
| `TerminalSurfacePlaceholder` in app/Sources/ | 0건 |
| `TODO(MS-7)` in app/Sources/ | 0건 |
| `"Ghostty Metal surface will render here"` | 0건 |

---

## 수동 UI 검증 (T-M2.5-018)

Metal Toolchain 환경에서 다음 5개 체크를 사람이 수행해야 함.

- [ ] 체크 1: 기본 워크스페이스 생성 → 실 Metal surface 렌더 (AC-2.1)
- [ ] 체크 2: Cmd+K → 5종 Surface 명령 각각 실행 → 활성 pane 새 탭 생성 (AC-3.1~3.3)
- [ ] 체크 3: Cmd+K → 수평/수직 분할 (AC-4.1, 4.2)
- [ ] 체크 4: 분할 후 다른 pane에서 Cmd+K → Surface 명령 → 올바른 pane에 탭 (AC-1.4+3.1)
- [ ] 체크 5: 앱 재시작 후 레이아웃 복원 (AC-G.4)

---

## 다음 단계

- Sync phase: `git commit` (Run phase에서 커밋 금지 원칙에 따라 이 단계에서 실행)
- Sync phase: CHANGELOG.md, README.md 업데이트
- T-M2.5-018: Metal Toolchain 환경에서 수동 UI 검증 5건 수행
