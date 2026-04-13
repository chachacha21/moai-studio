# SPEC-M2-001 Implementation Plan

---
spec_id: SPEC-M2-001
version: 1.0.0
status: draft
created: 2026-04-13
---

## 1. 개요

M2 "Viewers" 마일스톤 구현 계획. M1 의 T-030 에서 이어 T-031 부터 번호를 부여한다. 9개 RG 를 7개 스프린트 (MS-1 ~ MS-7) 으로 나눈다.

---

## 2. 스프린트 구성

### MS-1: DB Schema + FFI Foundation (RG-M2-1 일부 + RG-M2-2 일부)

**목표**: panes/surfaces 테이블 V3 마이그레이션 + FFI pane/surface CRUD

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-031 | `moai-store` V3 마이그레이션: panes 테이블 추가 | `core/crates/moai-store/src/migrations/v3.rs`, `mod.rs` | - |
| T-032 | `moai-store` V3 마이그레이션: surfaces 테이블 추가 (tab_order 포함) | `core/crates/moai-store/src/migrations/v3.rs` | T-031 |
| T-033 | `moai-store` pane CRUD (create, list_by_workspace, update_ratio, delete, get_tree) | `core/crates/moai-store/src/pane.rs` | T-031 |
| T-034 | `moai-store` surface CRUD (create, list_by_pane, update_kind, update_tab_order, delete) | `core/crates/moai-store/src/surface.rs` | T-032 |
| T-035 | `moai-ffi` pane FFI 함수 노출 (swift-bridge) | `core/crates/moai-ffi/src/lib.rs`, bridge 정의 | T-033 |
| T-036 | `moai-ffi` surface FFI 함수 노출 (swift-bridge) | `core/crates/moai-ffi/src/lib.rs` | T-034 |
| T-037 | Store + FFI 단위/통합 테스트 (pane tree 빌드, surface CRUD, 영속 복원) | `core/crates/moai-store/tests/`, `moai-ffi/tests/` | T-035, T-036 |

---

### MS-2: Pane Splitting UI (RG-M2-1 완성)

**목표**: NSSplitView binary tree wrapper + 단축키 + 드래그 리사이즈

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-038 | `PaneSplitView` (NSViewRepresentable NSSplitView wrapper) | `app/Sources/Shell/Splits/PaneSplitView.swift` | T-035 |
| T-039 | `PaneTreeModel` (binary tree 데이터 모델, SwiftUI ObservableObject) | `app/Sources/Shell/Splits/PaneTreeModel.swift` | T-035 |
| T-040 | 단축키 바인딩 (Cmd+\ 수평, Cmd+Shift+\ 수직, Cmd+Shift+W pane 닫기) | `app/Sources/Shell/Splits/PaneSplitView.swift`, KeyboardShortcuts | T-038, T-039 |
| T-041 | 드래그 리사이즈 + 최소 pane 크기 200pt 제약 | `app/Sources/Shell/Splits/PaneSplitView.swift` | T-038 |
| T-042 | Pane 레이아웃 영속/복원 (앱 재시작 시 DB 로부터 tree 재구성) | `app/Sources/Shell/Splits/PaneTreeModel.swift` | T-039, T-037 |
| T-043 | `RootSplitView.swift` 리팩터링: 기존 단일 ContentArea 를 PaneSplitView 로 교체 | `app/Sources/Shell/RootSplitView.swift` | T-038, T-042 |

---

### MS-3: Tab UI + Surface Protocol (RG-M2-2 + RG-M2-4 일부)

**목표**: Tab bar 컴포넌트 + SurfaceProtocol 정의 + 기존 TerminalSurface 마이그레이션

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-044 | `SurfaceProtocol.swift` (SurfaceKind enum, lifecycle, toolbar) | `app/Sources/Surfaces/SurfaceProtocol.swift` | - |
| T-045 | 기존 `TerminalSurface` 를 SurfaceProtocol conform 으로 리팩터링 | `app/Sources/Surfaces/Terminal/TerminalSurface.swift` | T-044 |
| T-046 | `TabBarView` 컴포넌트 (탭 추가, 닫기, 활성 표시, 드래그 reorder) | `app/Sources/Shell/Tabs/TabBarView.swift` | T-044, T-036 |
| T-047 | `TabBarViewModel` (surface 목록 관리, FFI surface CRUD 연동) | `app/Sources/Shell/Tabs/TabBarViewModel.swift` | T-046, T-036 |
| T-048 | PaneSplitView 와 TabBarView 통합 (leaf pane = tab bar + active surface) | `app/Sources/Shell/Splits/PaneSplitView.swift` | T-043, T-046 |
| T-049 | Tab/Surface 영속/복원 통합 테스트 | `app/UITests/` 또는 unit tests | T-047, T-048 |

---

### MS-4: FileTree Surface (RG-M2-4 완성)

**목표**: FileTree surface 구현 + moai-fs 연동 + git status

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-050 | `moai-ffi` FileTree FFI: 디렉토리 리스팅 + git status 데이터 | `core/crates/moai-ffi/src/filetree.rs` | T-036 |
| T-051 | `moai-fs` FileTree 전용 watcher (디렉토리 변경 이벤트 -> FFI 전달) | `core/crates/moai-fs/src/tree_watcher.rs` | - |
| T-052 | `FileTreeSurface` SwiftUI 뷰 (트리 렌더링, expand/collapse, 파일 아이콘) | `app/Sources/Surfaces/FileTree/FileTreeSurface.swift` | T-044, T-050 |
| T-053 | Git status 색상 (modified=노랑, added=초록, untracked=회색) | `app/Sources/Surfaces/FileTree/FileTreeSurface.swift` | T-052 |
| T-054 | 파일 더블클릭 -> 확장자별 surface 열기 로직 | `app/Sources/Surfaces/FileTree/FileTreeSurface.swift`, TabBarViewModel | T-052, T-047 |
| T-055 | 실시간 갱신 (moai-fs notify -> FFI event -> SwiftUI 반영) | `app/Sources/Surfaces/FileTree/FileTreeSurface.swift` | T-051, T-052 |
| T-056 | FileTree 단위/통합 테스트 | tests | T-052~T-055 |

---

### MS-5: Markdown + Image + Browser Surfaces (RG-M2-5 + RG-M2-6 + RG-M2-7)

**목표**: 3종 Surface 구현

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-057 | `MarkdownSurface` 기본 (Down cmark 렌더링, 다크/라이트 테마) | `app/Sources/Surfaces/Markdown/MarkdownSurface.swift` | T-044 |
| T-058 | EARS SPEC 특수 포매팅 (requirement ID 강조, Given/When/Then 블록) | `app/Sources/Surfaces/Markdown/EARSFormatter.swift` | T-057 |
| T-059 | KaTeX + Mermaid WKWebView 임베드 | `app/Sources/Surfaces/Markdown/WebContentRenderer.swift` | T-057 |
| T-060 | Markdown 파일 변경 감지 + 자동 리로드 (moai-fs 연동) | `app/Sources/Surfaces/Markdown/MarkdownSurface.swift` | T-057, T-051 |
| T-061 | `ImageSurface` (PNG/JPEG/GIF/SVG/WebP, zoom, pan, fit-to-window) | `app/Sources/Surfaces/Image/ImageSurface.swift` | T-044 |
| T-062 | Image diff 모드 (side-by-side + SSIM via Vision framework) | `app/Sources/Surfaces/Image/ImageDiffView.swift` | T-061 |
| T-063 | `BrowserSurface` (WKWebView, URL bar, back/forward/reload) | `app/Sources/Surfaces/Browser/BrowserSurface.swift` | T-044 |
| T-064 | Dev server auto-detect (localhost 포트 스캔: 3000, 5173, 8080) | `app/Sources/Surfaces/Browser/DevServerDetector.swift` | T-063 |
| T-065 | 링크 클릭 핸들링 (internal navigation vs 외부 브라우저) | `app/Sources/Surfaces/Browser/BrowserSurface.swift` | T-063 |
| T-066 | 3종 Surface 단위 테스트 | tests | T-057~T-065 |

---

### MS-6: Command Palette (RG-M2-3)

**목표**: Command Palette 오버레이 + fuzzy search + slash injection

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-067 | `CommandPaletteView` SwiftUI 오버레이 (Cmd+K 토글, Escape 닫기) | `app/Sources/Shell/CommandPalette/CommandPaletteView.swift` | - |
| T-068 | `CommandRegistry` (명령 카테고리 등록: /moai, surface, workspace, pane) | `app/Sources/Shell/CommandPalette/CommandRegistry.swift` | T-067 |
| T-069 | Fuzzy matching 엔진 (텍스트 입력 -> 필터링 + 점수 정렬) | `app/Sources/Shell/CommandPalette/FuzzyMatcher.swift` | T-068 |
| T-070 | 키보드 네비게이션 (Arrow Up/Down, Enter, Escape) | `app/Sources/Shell/CommandPalette/CommandPaletteView.swift` | T-067 |
| T-071 | Slash injection: `/moai *` 명령 -> Rust core -> SDKUserMessage -> Claude | `app/Sources/Shell/CommandPalette/SlashInjector.swift` | T-068 |
| T-072 | Surface 열기 명령 (FileTree, Markdown, Image, Browser 선택 -> 새 탭) | `app/Sources/Shell/CommandPalette/CommandRegistry.swift` | T-068, T-047 |
| T-073 | Command Palette 테스트 (fuzzy matching, 명령 실행, keyboard nav) | tests | T-067~T-072 |

---

### MS-7: CI/CD + M1 Carry-over + E2E (RG-M2-8 + RG-M2-9)

**목표**: CI 파이프라인 구축 + carry-over 해소 + 종합 E2E

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-074 | GitHub Actions: Rust CI (cargo check + test + clippy + fmt) | `.github/workflows/ci-rust.yml` | - |
| T-075 | GitHub Actions: Swift CI (xcodebuild build-for-testing + test) | `.github/workflows/ci-swift.yml` | - |
| T-076 | GhosttyKit + Rust xcframework 캐싱 설정 | `.github/workflows/ci-swift.yml` cache section | T-075 |
| T-077 | C-1: Xcode UITest 서명 자동화 (CI provisioning 또는 ad-hoc) | `.github/workflows/ci-swift.yml`, signing 설정 | T-075 |
| T-078 | C-2: Claude CLI E2E validation 스크립트 | `scripts/validate-claude-e2e.sh` | - |
| T-079 | C-3: 10min 4-ws stress + RSS <400MB 자동화 스크립트 | `scripts/stress-test-4ws.sh` 또는 Rust 테스트 | - |
| T-080 | C-4: GhosttyKit Metal 60fps XCTest benchmark | `app/UITests/` 또는 `app/Tests/` | T-075 |
| T-081 | C-5: swift-bridge Vectorizable workaround 제거 (조건부) | `core/crates/moai-ffi/` | - |
| T-082 | C-6: Auth token rotation (moai-hook-http) | `core/crates/moai-hook-http/src/auth.rs` | - |
| T-083 | C-7: Swift FFI <1ms XCTest benchmark | `app/Tests/FFIBenchmarkTests.swift` | T-075 |
| T-084 | C-8: force_paused 정식 API 승격 + 문서화 | `core/crates/moai-store/src/state_machine.rs` | - |
| T-085 | M2 종합 E2E 테스트 (pane split -> tab -> surface -> command palette 전체 흐름) | `core/tests/e2e_viewers.rs`, `app/UITests/` | 전체 |
| T-086 | M2 NFR 검증 보고서 | `.moai/specs/SPEC-M2-001/nfr-report.md` | T-085 |
| T-087 | M2 완료 보고서 + Go/No-Go 판정 | `.moai/specs/SPEC-M2-001/m2-completion-report.md` | T-086 |

---

## 3. 의존성 다이어그램

```
MS-1 (DB + FFI)
  |
  ├── MS-2 (Pane Splitting UI) ── depends on T-035, T-037
  │     |
  │     └── MS-3 (Tab UI + SurfaceProtocol) ── depends on T-043
  │           |
  │           ├── MS-4 (FileTree) ── depends on T-044, T-047
  │           │
  │           ├── MS-5 (Markdown + Image + Browser) ── depends on T-044
  │           │
  │           └── MS-6 (Command Palette) ── depends on T-047
  │
  └── MS-7 (CI/CD + Carry-over + E2E) ── 독립 시작 가능 (T-074~T-084)
                                           최종 E2E (T-085~T-087) 는 전체 의존
```

**병렬 가능 구간**:
- MS-4, MS-5, MS-6 은 MS-3 완료 후 병렬 실행 가능
- MS-7 의 CI/CD 설정 (T-074~T-076) 및 carry-over (T-078~T-084) 는 MS-1 과 병렬 시작 가능
- MS-7 의 E2E (T-085~T-087) 는 모든 스프린트 완료 후

---

## 4. 리스크 분석

| 리스크 | 영향 | 완화 전략 |
|--------|------|-----------|
| NSSplitView + SwiftUI 통합 복잡도 | NSViewRepresentable 래핑 시 레이아웃 이슈 | PaneSplitView spike 를 MS-2 초반에 실행, fallback 으로 순수 SwiftUI GeometryReader 검토 |
| Down (cmark) 의존성 macOS 14 호환 | Down 최신 버전이 macOS 15+ 만 지원할 수 있음 | Down 버전 고정 또는 swift-cmark 직접 사용 fallback |
| KaTeX/Mermaid WKWebView 임베드 성능 | 무거운 수식/다이어그램 렌더링 지연 | lazy loading + 비동기 렌더링, 로딩 인디케이터 표시 |
| GhosttyKit Metal Toolchain CI 가용성 | GitHub Actions runner 에 Metal Toolchain 미설치 | self-hosted runner 또는 pre-built artifact 캐싱 |
| swift-bridge Vectorizable 상위 호환 | 호환 버전 미출시 시 workaround 잔존 | C-5 를 조건부 이월 허용 (M3 최종 해소) |
| CI macOS runner 비용 | macOS runner 는 Linux 대비 10x 비용 | 캐싱 최적화 + 최소 매트릭스 (macOS 14 only 초기) |
| Pane 영속/복원 edge case | 비정상 종료 시 불완전 tree | 복원 실패 시 기본 레이아웃 (단일 pane) 으로 fallback |

---

## 5. 기술 접근

### 5.1 DB Schema V3

기존 V1 (workspaces), V2 (hook_events) 에 이어 V3 에서 panes + surfaces 추가. `moai-store` 의 마이그레이션 체인에 v3.sql 삽입.

### 5.2 NSSplitView Binary Tree

SwiftUI 에는 native split view 재귀 구조가 없으므로, `NSViewRepresentable` 로 `NSSplitView` 를 래핑한다. binary tree 노드를 재귀적으로 `NSSplitView` subview 로 구성.

### 5.3 Surface Protocol

Swift protocol 로 공통 인터페이스를 정의하고, 각 surface 가 conform. `@ViewBuilder` body 패턴 대신 `NSView`/`NSViewController` 래핑도 허용 (WKWebView 등).

### 5.4 Command Palette

SwiftUI `.overlay()` 로 구현. Fuzzy matching 은 문자열 subsequence 매칭 + 가중 점수. CommandRegistry 는 각 surface/module 이 명령을 등록하는 패턴.

### 5.5 CI/CD

GitHub Actions 의 macOS runner 사용. Rust 와 Swift 를 별도 워크플로우로 분리하여 변경 범위별 선택 실행 (path filter).

---

## 6. 산출물 요약

### Rust (신규/수정)
- `moai-store`: V3 마이그레이션, pane.rs, surface.rs
- `moai-ffi`: pane FFI, surface FFI, filetree FFI
- `moai-fs`: tree_watcher.rs
- `moai-hook-http`: auth.rs (token rotation)
- `moai-store`: state_machine.rs (force_paused API)

### Swift (신규)
- `app/Sources/Shell/Splits/`: PaneSplitView, PaneTreeModel
- `app/Sources/Shell/Tabs/`: TabBarView, TabBarViewModel
- `app/Sources/Shell/CommandPalette/`: CommandPaletteView, CommandRegistry, FuzzyMatcher, SlashInjector
- `app/Sources/Surfaces/SurfaceProtocol.swift`
- `app/Sources/Surfaces/FileTree/`: FileTreeSurface
- `app/Sources/Surfaces/Markdown/`: MarkdownSurface, EARSFormatter, WebContentRenderer
- `app/Sources/Surfaces/Image/`: ImageSurface, ImageDiffView
- `app/Sources/Surfaces/Browser/`: BrowserSurface, DevServerDetector

### CI/CD (신규)
- `.github/workflows/ci-rust.yml`
- `.github/workflows/ci-swift.yml`

### Scripts (신규/수정)
- `scripts/validate-claude-e2e.sh`
- `scripts/stress-test-4ws.sh`

### 테스트
- Rust: Store pane/surface 테스트, FFI 통합 테스트, E2E viewers 테스트
- Swift: UITest (pane split, tab, command palette), XCTest benchmark (Metal 60fps, FFI <1ms)
