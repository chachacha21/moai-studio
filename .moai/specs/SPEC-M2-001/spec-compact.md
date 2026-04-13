# SPEC-M2-001 Compact: M2 Viewers

> id: SPEC-M2-001 | version: 1.0.0 | status: draft | priority: High

---

## 요구사항 요약

### RG-M2-1: Pane Splitting (NSSplitView binary tree)
- Rust Store `panes` 테이블 V3 마이그레이션
- NSSplitView binary tree wrapper (NSViewRepresentable)
- Cmd+\ 수평 / Cmd+Shift+\ 수직 분할
- 드래그 리사이즈 (최소 200pt)
- Pane 닫기 (마지막 pane 보호)
- 레이아웃 영속/복원
- FFI pane CRUD

### RG-M2-2: Tab UI
- Pane 상단 tab bar (각 탭 = surface)
- Cmd+T 새 탭, Cmd+W / X 버튼 닫기
- 드래그 reorder (같은 pane 내)
- 활성 탭 시각 표시
- `surfaces` 테이블 (V3), tab_order 영속

### RG-M2-3: Command Palette
- Cmd+K 열기, Escape 닫기
- Fuzzy search (명령 필터링)
- /moai slash injection -> Rust core -> Claude
- Surface 열기 / Workspace 조작 / Pane 조작 명령
- 키보드 네비게이션

### RG-M2-4: Surface Protocol + FileTree
- `SurfaceProtocol` 공통 인터페이스 (10종 surface 타입)
- FileTree: 디렉토리 트리 + expand/collapse + 파일 아이콘
- Git status 색상 (modified 노랑, added 초록, untracked 회색)
- 더블클릭 -> 확장자별 surface 열기
- moai-fs (notify) 실시간 갱신
- FFI filetree 데이터 전달

### RG-M2-5: Markdown Surface
- Down (cmark) 렌더링
- EARS SPEC 특수 포매팅
- KaTeX 수식 + Mermaid 다이어그램 (WKWebView)
- 파일 변경 자동 리로드
- 다크/라이트 테마

### RG-M2-6: Image Surface
- PNG/JPEG/GIF/SVG/WebP 지원
- Zoom in/out, pan, fit-to-window
- Diff 모드 (side-by-side + SSIM via Vision)

### RG-M2-7: Browser Surface
- WKWebView wrapper + URL bar + 네비게이션
- Dev server auto-detect (localhost 포트 스캔)
- 외부 도메인 링크 -> 시스템 브라우저

### RG-M2-8: CI/CD Pipeline
- GitHub Actions: Rust CI (check + test + clippy + fmt)
- GitHub Actions: Swift CI (xcodebuild)
- GhosttyKit/Rust xcframework 캐싱
- macOS 14+ / Xcode 15+ 매트릭스

### RG-M2-9: M1 Carry-over (8건)
- C-1: UITest 서명 CI 통합
- C-2: Claude CLI E2E 검증
- C-3: 4-ws 10min stress RSS <400MB
- C-4: Metal 60fps 벤치마크
- C-5: Vectorizable workaround 제거 (조건부)
- C-6: Auth token rotation
- C-7: FFI <1ms XCTest
- C-8: force_paused 정식 API

---

## Exclusions
1. Code Viewer -- M3
2. Agent Run Viewer -- M5
3. Kanban board -- M5
4. Memory Viewer -- M5
5. InstructionsGraph -- M5
6. LSP integration -- M4
7. Native Permission Dialog -- M4
8. 16+ 동시 워크스페이스 -- M6
9. Auto-update (Sparkle) -- M6
10. Onboarding wizard -- M4
11. Cross-pane 탭 drag-and-drop -- M3+
12. Surface 간 통신 프로토콜 -- M3

---

## 핵심 Acceptance Criteria

| AC | 시나리오 | 기준 |
|----|----------|------|
| AC-1.1 | Cmd+\ 수평 분할 | 좌우 pane 생성, ratio 0.5 |
| AC-1.6 | 레이아웃 영속 | 재시작 후 pane tree 100% 복원 |
| AC-2.1 | Cmd+T 새 탭 | 탭 추가 + EmptyState |
| AC-3.1 | Cmd+K Palette | 오버레이 <200ms |
| AC-3.3 | Slash injection | /moai -> Claude subprocess 도달 |
| AC-4.1 | FileTree 렌더 | 파일/디렉토리 트리 표시 |
| AC-4.3 | Git status 색상 | modified 노랑, added 초록 |
| AC-5.1 | Markdown 렌더 | H1/단락/리스트 정상 |
| AC-5.3 | KaTeX 수식 | LaTeX 수식 렌더링 |
| AC-6.1 | 이미지 표시 | <500ms fit-to-window |
| AC-7.2 | Dev server detect | localhost 자동 감지 |
| AC-8.1 | Rust CI | check+test+clippy+fmt 통과 |
| AC-8.2 | Swift CI | xcodebuild 통과 |

---

## NFR 목표

| 항목 | 목표 |
|------|------|
| Pane split | <100ms |
| Tab 전환 | <50ms |
| Palette 열기 | <200ms |
| FileTree 로드 | <500ms (1K files) |
| Markdown 렌더 | <1s (100KB) |
| Image 로드 | <500ms (10MB) |
| RSS (8 pane) | <600MB |
| CI Rust | <10min |
| CI Swift | <15min |

---

## 태스크 (T-031 ~ T-087)

| Sprint | Tasks | 범위 |
|--------|-------|------|
| MS-1 | T-031~T-037 | DB V3 + FFI pane/surface |
| MS-2 | T-038~T-043 | Pane Splitting UI |
| MS-3 | T-044~T-049 | Tab UI + SurfaceProtocol |
| MS-4 | T-050~T-056 | FileTree Surface |
| MS-5 | T-057~T-066 | Markdown + Image + Browser |
| MS-6 | T-067~T-073 | Command Palette |
| MS-7 | T-074~T-087 | CI/CD + Carry-over + E2E |
