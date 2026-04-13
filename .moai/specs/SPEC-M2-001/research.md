# SPEC-M2-001 Research — M2 Viewers Deep Codebase Analysis

---
spec_id: SPEC-M2-001
phase: research
created: 2026-04-13
---

## 1. 현재 Shell 아키텍처 (M1 상태)

### 1.1 앱 진입점
- `app/Sources/App/MoAIStudioApp.swift`: @main, @Observable WorkspaceViewModel + WindowStateStore 을 .environment() 주입
- `app/Sources/Shell/RootSplitView.swift`: NavigationSplitView (2-pane: 사이드바 + ContentArea)
- 사이드바 너비: min=200, ideal=250, max=400 (WindowStateStore 에서 관리)

### 1.2 콘텐츠 영역
- `app/Sources/Shell/Content/ContentArea.swift`: selected workspace ID 기반 TerminalSurface 또는 EmptyState 전환
- **단일 surface**: 현재는 workspace 당 1개의 TerminalSurface 만 지원
- Pane splitting, Tab UI 없음

### 1.3 Surface 패턴
- `app/Sources/Shell/Content/TerminalSurface.swift`: GhosttyKit wrapper + TerminalFallback
- TerminalBackend enum (ghostty/nstext), MOAI_TERMINAL_BACKEND 환경변수 오버라이드
- **SurfaceProtocol 없음** — M2에서 정의 필요
- onAppear 기반 초기화, @State loadFailed 로 fallback 전환

### 1.4 ViewModel 패턴
- `app/Sources/ViewModels/WorkspaceViewModel.swift`: @Observable + @MainActor
- DispatchSource.timer 기반 이벤트 폴링 (16ms ≈ 60Hz)
- WorkspaceSnapshot 구조체로 Swift 측에 데이터 전달

### 1.5 윈도우 상태
- `app/Sources/Shell/WindowStateStore.swift`: @Observable, sidebarWidth 영속

## 2. Rust Core 현황

### 2.1 moai-store (core/crates/moai-store/)
- `lib.rs`: Store (Arc<Mutex<Connection>>), V1+V2 마이그레이션
- **현재 테이블**: workspaces (V2: name, project_path, worktree_path, status, spec_id, claude_session_id), hook_events, schema_version
- **누락 테이블**: panes, surfaces — M2 V3 마이그레이션 필요
- `workspace.rs`: WorkspaceDao, WorkspaceRow, NewWorkspace, WorkspaceStoreExt trait
- `state.rs`: WorkspaceStatus enum (Created, Starting, Running, Paused, Error, Deleted), 상태 전이 검증

### 2.2 moai-ffi (core/crates/moai-ffi/)
- `lib.rs`: swift-bridge #[swift_bridge::bridge] mod ffi — 유일한 FFI 경계
- 노출 API: RustCore::new/version/create_workspace/delete_workspace/list_workspaces/send_user_message/subscribe_events/poll_event
- WorkspaceInfo struct (id, name, status)
- **sync polling 패턴**: tokio broadcast -> VecDeque -> poll_event (Swift DispatchSource.timer 호출)
- `events.rs`: 이벤트 큐 관리
- `workspace.rs`: WorkspaceRegistry
- **Pane/Surface FFI 없음** — M2에서 추가 필요

### 2.3 moai-fs (core/crates/moai-fs/)
- notify 7.x 기반 파일 감시
- FileTree surface 의 실시간 갱신에 활용 가능

### 2.4 moai-core (core/crates/moai-core/)
- 얇은 facade. version() 함수

## 3. DESIGN.v4.md 설계 사양

### 3.1 Pane DB 스키마 (§6)
```sql
CREATE TABLE panes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id),
    parent_id INTEGER,
    split TEXT,  -- horizontal|vertical|leaf
    ratio REAL
);
CREATE TABLE surfaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pane_id INTEGER NOT NULL REFERENCES panes(id),
    kind TEXT NOT NULL,  -- terminal|code|markdown|image|browser|filetree|agent_run|kanban|memory|instructions_graph
    state_json BLOB
);
```

### 3.2 Shell 디렉토리 구조 (§8)
```
app/Sources/Shell/
├── Sidebar/        # M1 완료
├── Tabs/           # M2 목표
├── Splits/         # M2 목표 (NSSplitView binary tree)
└── CommandPalette/ # M2 목표
```

### 3.3 Surface 디렉토리 구조 (§8)
```
app/Sources/Surfaces/
├── Terminal/       # M1 완료 (wrapper)
├── FileTree/       # M2 목표
├── Markdown/       # M2 목표
├── Image/          # M2 목표
└── Browser/        # M2 목표
```

## 4. CI/CD 현황

- `.github/workflows/`: 없음
- `scripts/`: build-ghostty-xcframework.sh, build-rust-xcframework.sh, check-metal-toolchain.sh
- Makefile/justfile/Taskfile: 없음
- **GitHub Actions 워크플로우 전체 신규 생성 필요**

## 5. M1 Carry-over 8건

m1-completion-report.md 기준:

| ID | 항목 | 분류 | 상태 |
|----|------|------|------|
| C-1 | Xcode UITest 서명 + E2EWorkingShellTests | 테스트 | 미해소 |
| C-2 | Claude CLI E2E 응답 수신 검증 | 테스트 | 미해소 |
| C-3 | 10min 4-ws stress + RSS <400MB | 성능 | 미해소 |
| C-4 | GhosttyKit Metal 60fps 측정 | 성능 | 미해소 |
| C-5 | swift-bridge Vectorizable workaround | 기술부채 | 미해소 |
| C-6 | Auth token rotation (hook-http) | 보안 | 미해소 |
| C-7 | Swift FFI <1ms XCTest benchmark | 검증 | 미해소 |
| C-8 | force_paused 정식 API | 기술부채 | 미해소 |

## 6. 리스크 분석

| 리스크 | 확률 | 영향 | 대응 |
|--------|------|------|------|
| NSSplitView + SwiftUI 통합 복잡도 | 중간 | 높음 | NSViewRepresentable 패턴, cmux 참조 구현 |
| Down (cmark) + WKWebView 임베딩 호환 | 낮음 | 중간 | pure WKWebView fallback |
| swift-bridge Vectorizable 이슈 지속 | 중간 | 낮음 | workaround 유지, M3 재검토 |
| GhosttyKit Metal 60fps 미달 | 낮음 | 중간 | nstext fallback 이미 존재 |
| CI macOS runner 비용/속도 | 중간 | 중간 | 캐싱 최적화, 매트릭스 최소화 |
| FileTree 대규모 디렉토리 성능 | 중간 | 중간 | lazy loading, virtual scrolling |
| Pane state 복원 정확도 | 낮음 | 높음 | 100% 재현 통합 테스트 |

## 7. 권장 구현 순서

1. **MS-1 (Rust foundation)**: Store V3 + FFI pane/surface — 모든 UI 의 기반
2. **MS-2 (Pane UI)**: NSSplitView wrapper — Shell 구조 변경
3. **MS-3 (Tab + Protocol)**: SurfaceProtocol + TabBar — Surface 통합 기반
4. **MS-4/5/6 (병렬)**: FileTree, 3 surfaces, Command Palette
5. **MS-7 (CI + Carry-over)**: CI 설정은 MS-1과 병렬 시작 가능

**신뢰도**: HIGH (Pane/Tab/CI), MEDIUM (Surfaces/CommandPalette)
