# MoAI Studio 기능 정의 + UI/UX 재설계 스펙 (2026-04-17)

본 문서는 **MoAI Studio 전체 기능 정의 + 정보 아키텍처 + UI/UX 재설계 제안** 을 통합한다. 기존 `moai-termail.pen` 12 프레임을 기반으로, 2026 agentic coding IDE 베스트 프랙티스를 반영해 재설계한다.

- 참조 토큰: [system.md](./system.md)
- 참조 리서치: [research.md](./research.md)

---

## 1. 기능 인벤토리 (Functional Inventory)

MoAI Studio 는 총 **21 개 기능** 을 제공한다 (기존 16 + 신규 5).

### Tier 1: Core Shell (MVP, M1 ~ M2.5 이미 구현 목표)

| # | 기능 | 구현 상태 | 관련 SPEC |
|---|------|-----------|-----------|
| F-1 | Workspace CRUD + 다중 워크스페이스 | ⚠️ create 메뉴 버그 | M1-001 |
| F-2 | Ghostty Metal 60 Hz 터미널 렌더 | ✅ M2.5 해소 | M2-002 |
| F-3 | Pane Splitting (NSSplitView binary tree) | ✅ 구현 | M2-001 |
| F-4 | Tab UI | ✅ 구현 | M2-001 |
| F-5 | Command Palette (⌘K) | ⚠️ 콜백 실구현 부족 | M2-001/002 |
| F-6 | File Tree Surface | ⚠️ 1레벨만 | M2-001 |
| F-7 | Markdown Surface (EARS + KaTeX + Mermaid) | ✅ 부분 | M2-001 |
| F-8 | Image Surface | ✅ 구현 | M2-001 |
| F-9 | Browser Surface (localhost:3000 자동) | ✅ 구현 | M2-001 |

### Tier 2: Agentic Value (M3 ~ M5)

| # | 기능 | 구현 상태 | 관련 SPEC |
|---|------|-----------|-----------|
| F-10 | Code Viewer (TreeSitter + LSP + @MX + tri-pane diff + time-travel) | 📋 SPEC 있음 | M3-001 |
| F-11 | Agent Run Viewer (hook event timeline + cost + tokens) | ❌ 미구현 | M5 (DESIGN.v4) |
| F-12 | Kanban Board (SPEC ↔ worktree ↔ agent run) | ❌ 미구현 | M5 |
| F-13 | Memory Viewer (~/.claude/projects/…/memory/) | ❌ 미구현 | M5 |
| F-14 | Instructions Graph (세션 컨텍스트 디버거) | ❌ 미구현 | M5 |

### Tier 3: Integration & Config

| # | 기능 | 구현 상태 | 관련 SPEC |
|---|------|-----------|-----------|
| F-15 | New Workspace Wizard (4-step + NSOpenPanel) | ⚠️ 기본 Sheet 만 | M2-UX-001 |
| F-16 | Settings | ❌ 미구현 | M2-UX-002 |
| F-17 | Onboarding (환경 감지 + Hook consent) | 📋 Pencil 설계 완료 | M2-UX-002 |
| F-18 | CG Mode (Claude+GLM split) | 📋 Pencil 설계 완료 | M5 |

### Tier 4: 신규 추가 (리서치 기반)

| # | 기능 | 배경 | 구현 상태 |
|---|------|------|-----------|
| F-19 | **Mission Control** (parallel agent grid) | Cursor 3 Mission Control | ❌ 신규 |
| F-20 | **Hooks & MCP Panel** (설정) | Claude Code `/hooks` `/mcp` | ❌ 신규 |
| F-21 | **Agent Thread** (conversational with tool calls inline) | Zed Agent Panel | ❌ 신규 |

---

## 2. 정보 아키텍처 (IA)

### 상위 구조

```
MoAI Studio App
├── Menu Bar
│   ├── MoAI Studio (app menu) — About, Settings (⌘,), Services, Hide, Quit
│   ├── File — New Workspace (⌘N), New Tab (⌘T), Close (⌘W), Open Recent, Import SPEC
│   ├── Edit — Find in Project (⌘⇧F), Undo/Redo, standard
│   ├── View — Toggle Sidebar (⌘0), Toggle FileTree (⌘B), Toggle Agent Run (⌘⌥R), Open Palette (⌘K), Follow Agent (⌘G), Full Screen
│   ├── Pane — New Pane (⌘⇧N), Split Horizontally (⌘\), Split Vertically (⌘⇧\), Close Pane (⌘⇧W)
│   ├── Surface — Open Terminal, FileTree, Markdown, Image, Browser, Code Viewer, Agent Run (각 단축키)
│   ├── SPEC — Plan (⌘⇧M P), Run (⌘⇧M R), Sync (⌘⇧M S), Review, Coverage, E2E, MX Scan
│   ├── Agent — Mission Control (⌘⇧A), New Thread, Stop All, Follow
│   ├── Go — Go to File (⌘P), Go to Symbol (⌘⇧P), Go to SPEC, Go to Definition
│   ├── Window — Minimize, Zoom, Toggle CG Mode, Focus Palette
│   └── Help — Documentation, Keyboard Shortcuts (⌘?), Feedback, Report Issue
│
├── Toolbar (customizable, default 7 items)
│   [+ New Workspace] [⊟ ⊞ Split] [▶ Run SPEC] [⌘K Palette] [⊙ Agent Status] [⚠ Diagnostics] [? Help]
│
├── Main Window (3-pane body)
│   ├── Sidebar (260pt)
│   │   ├── Workspaces section — list + status icons + context menu
│   │   ├── Git Worktrees section — active worktrees per project
│   │   ├── SPECs section — 현재 워크스페이스의 SPEC list (draft/completed/active)
│   │   └── [+ New Workspace] button (bottom safeArea)
│   │
│   ├── Center (Main3Pane — NSSplitView binary tree)
│   │   └── Each leaf = 1 Surface (Terminal/Code/FileTree/Markdown/Image/Browser/AgentRun)
│   │
│   └── Right Panel (460pt, toggleable ⌘⌥R)
│       └── Agent Run Viewer — hook timeline + stats + chart + detail
│
├── Status Bar (28pt)
│   └── [⎇ branch] · [LSP] · [version] · ... · [⊙ Agent: idle/running/cost] · [⌘K to search]
│
└── Overlays
    ├── Command Palette (⌘K) — modal, 720pt
    ├── Sheets — New Workspace Wizard, Settings, Rename, Delete Confirm
    ├── Onboarding (first-run only) — gradient hero overlay
    └── Mission Control (⌘⇧A) — full-screen grid of agents
```

### 첫 실행 플로우 (Aha Moment)

```
Launch
 → [CLI 감지 + Claude Code 설치 확인]
 → IF 처음 실행:
     → Onboarding (environment detected + consent + "Start Sample" CTA)
     → Sample Workspace 자동 생성 ("Sample: MoAI Tour") with demo SPEC
     → 샘플 pane 구성 (Terminal + Code Viewer + Agent Run)
     → 첫 /moai plan 또는 /moai run 시범 (가이드 풍선)
 → ELSE IF 워크스페이스 0개:
     → Empty State CTA — "첫 워크스페이스 만들기" + "샘플 열기" + "이전 프로젝트 열기"
 → ELSE:
     → 마지막 활성 워크스페이스 복원
```

### 컨텍스트 파이프라인 (UI 노출)

Agent Run Viewer 상단에 이번 턴의 context sources 뱃지를 나열:

```
[CLAUDE.md] [.moai/rules/] [auto-memory] [활성 파일 3] [선택 14줄] [검색 결과 2] ⌘ 자세히
```

Windsurf Cascade 의 Flow Awareness 와 동등한 투명성.

---

## 3. 프레임별 재설계 (기존 12 → 목표 18)

Pencil 파일: `/Users/goos/MoAI/moai-adk-go/pencil/moai-termail.pen` (1600 × 1000 기본)

### 3.1 Frame 01. Main Workspace (기존 id=`6rM07`) — **리뉴얼**

**현재**: TitleBar (traffic + moai-terminal / moai-adk + tabs + search/bell/settings) + Sidebar 260 (WORKSPACE + GIT WORKTREES) + Main3Pane (Terminal | Code Viewer | Agent Run Viewer) + StatusBar.

**변경 포인트**:

1. **Sidebar 섹션 추가**: `SPECs` 라벨 + 현재 워크스페이스의 SPEC 카드 목록 (색상 뱃지: draft/active/completed).
2. **Toolbar 추가**: TitleBar 아래 36pt 높이의 Toolbar 레이어. 7개 primary action + "⋯ Customize" 오버플로우.
3. **StatusBar 왼쪽 확장**: Agent Run 상태 pill 추가 — `[⊙ Agent: idle]` 또는 `[⊙ Running · $0.024 · 3,482 tok]`.
4. **Context Burst 인디케이터**: Agent Run Viewer 상단에 이번 턴 주입 소스 뱃지 6-8 개 (작은 chip).

### 3.2 Frame 02. Kanban Board (id=`BxZi3`) — **유지 + 강화**

**현재**: kbHead + lanes.

**추가**:
- 각 카드에 `Agent Run` 미니 그래프 (최근 3회 실행 히트맵).
- SPEC 링크 뱃지 (`SPEC-AUTH-001` 타이포그래피).
- Lane 헤더에 count + WIP limit 표시.

### 3.3 Frame 03. Project Dashboard (id=`PuRCp`) — **유지 + 강화**

**현재**: dh + kpiRow + mid + recent.

**추가**:
- TRUST 5 레이더 차트 (5축 점수).
- LSP 진단 추이 스파크라인.
- @MX 태그 분포 도넛 (NOTE/WARN/ANCHOR/TODO 비율).

### 3.4 Frame 04. Code Viewer Deep Dive (id=`s3Kz3`) — **유지 + hunk-level UI**

**현재**: cvTop + triBar + triPane (HEAD | working | pending) + sliderArea (time-travel slider).

**추가**:
- `triPane` 각 열 상단에 `[Accept] [Accept All Hunks] [Reject]` hunk-level 컨트롤.
- 거터 확장: line num + LSP diag + @MX 아이콘 + git decoration + Agent edit 마커 (에이전트가 편집한 라인 표시).
- `sliderArea` 에 cost / token 오버레이 (시점별 누적 비용).

### 3.5 Frame 05. Agent Run Viewer (id=`6kYdu`) — **유지 + controls & drill-down**

**현재**: arLeft (header + hook event timeline) + arRight 460 (statGrid + chartCard + detailCard).

**추가**:
- `arH` 에 Play/Pause/Stop/Resume 컨트롤 버튼 그룹.
- `runStatus` pill 을 상태 기반 색상 변형 (running/idle/error/completed).
- `tl` 타임라인을 **Span Tree** 로 변경 (hierarchical 들여쓰기, sub-agent branching).
- `detailCard` 하단에 **"Follow this agent"** 토글 + "Open in Code Viewer" 액션.
- 새 카드: **Token Breakdown** (input / output / cached / reasoning 4분할 바).
- 새 카드: **Context Sources** (이번 turn 에 주입된 파일/메모리/규칙 리스트).

### 3.6 Frame 06. File Explorer + EARS Markdown (id=`l2JjA`) — **유지 + @-mention**

**현재**: fxTree (320pt) + mdArea.

**추가**:
- fxTree 에 재귀 expand (DisclosureGroup) + git status 색상 + LSP diag 도트.
- mdArea 상단 toolbar: `[@ Mention]` 버튼 → 선택 텍스트를 Command Palette 에 주입.
- SPEC 문서 전용 edge: YAML frontmatter 편집 위젯 + HISTORY 표 편집기.

### 3.7 Frame 07. Browser + Image Viewer (id=`4YcE1`) — **유지**

**현재**: brPane + imgPane (520pt).

**추가**:
- brPane 상단에 DevTools 토글 (WebKit inspector).
- imgPane 에 zoom / rotate / flip / measure 툴바.

### 3.8 Frame 08. Command Palette (id=`VnFRr`) — **대폭 강화 (⌘K)**

**현재**: backdrop (80% black) + palette_modal 720pt (search_row + results_body + footer).

**변경**:
- `results_body` 에 **섹션 그룹** 도입: `⭐ Favorites`, `Recent`, `Commands`, `SPECs`, `Files`, `Symbols`, `MCP Tools`, `Agents`.
- 각 row: `[icon] [label] ...................... [shortcut] [tag]` 형식 (Raycast 스타일).
- **Nested palette**: 항목 선택 → 하위 action set (예: SPEC 선택 → Plan/Run/Sync/Review/Clean).
- **@/# mentions** inline: `@file.swift` 입력 → file picker, `#grep` → search palette.
- `footer`: `↑↓ Navigate · ↵ Select · ⌘↵ Pin · ⎋ Close · Tab Nest` 힌트.
- 검색 prefix: `>` = 명령, `@` = 파일, `#` = 심볼, `/` = slash command, `:` = MCP tool.

### 3.9 Frame 09. New Workspace Wizard (id=`yDEvl`) — **강화 + NSOpenPanel**

**현재**: 4-step (stepsNav 280 + main). `st1/st2/st3/st4`.

**제안 step 구성**:
1. **Project Source** — (a) "Pick folder..." 버튼 (NSOpenPanel) / (b) "Git clone URL" / (c) "Start empty"
2. **Identity** — Workspace name + description + color tag
3. **Environment** — 감지된 스택 표시 (Swift/Rust/Node/Python) + 동의 토글
4. **Worktree** — (a) "Use current dir" / (b) "Create dedicated worktree at ~/.moai/worktrees/{name}/"
5. **Review & Create** — 요약 카드 + `[Create Workspace]` primary button

### 3.10 Frame 10. Settings (id=`Ms3D7`) — **구체화**

**제안 섹션 (sidebar)**:
- General (theme, font size, auto-update)
- Hooks (27 hook events, enable/disable per type)
- MCP Servers (⊕ add, edit, disable, test connection)
- Skills (list + enable/disable)
- Rules (CLAUDE.md viewer + .claude/rules/ tree)
- Keybindings (customizable, import/export)
- Integrations (GitHub, Slack, Linear)
- Privacy (local-only, hook egress policy)

### 3.11 Frame 11. Onboarding (id=`3x9pP`) — **유지 + 샘플 플로우**

**현재**: Hero + ENVIRONMENT DETECTED card + CONSENT card + 3-button bar (Skip/Back/Continue).

**변경**:
- Hero 카피 수정: "Native coding shell for Claude Code agents — SPEC-first, agent-native."
- CONSENT card 아래 **"Start with Sample"** pale-orange CTA 추가 — 샘플 워크스페이스 자동 생성하고 Main Workspace 로 진입.
- "Skip" 은 빈 상태로 진입 (CTA 있는 empty state 로).

### 3.12 Frame 12. CG Mode View (id=`TcmLB`) — **유지 + 가이드**

**추가**:
- Leader (Claude) vs Workers (GLM) 시각 분리.
- tmux 세션 리스트 + 각 세션 live 미리보기 (미니 터미널 썸네일).
- cost 절감 표시: "Expected: $0.42 / hour vs $1.20 (Claude-only)".

---

## 4. 신규 추가 프레임 (6 개)

### 4.1 Frame 13. Mission Control — **신규**

Cursor 3 Mission Control 모델.

```
┌─────────────────────────────────────────────────────────┐
│  MISSION CONTROL                               ⌘⇧A  ×  │
├─────────────────────────────────────────────────────────┤
│  [Active 4]  [Idle 2]  [Failed 1]       Filter: All v   │
├─────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐     │
│  │ SPEC-A.1│  │ SPEC-B.2│  │ SPEC-C.1│  │ Fix loop│     │
│  │ Running │  │ Paused  │  │ Failed  │  │ Running │     │
│  │ ███ 65%│  │ 40%     │  │ err 2   │  │ ████ 80%│     │
│  │ $0.12   │  │ $0.05   │  │ $0.08   │  │ $0.31   │     │
│  │ [open]  │  │ [open]  │  │ [retry] │  │ [follow]│     │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                  │
│  │ Review  │  │ MX Scan │  │ + New   │                  │
│  │ Done ✓  │  │ Idle    │  │         │                  │
│  └─────────┘  └─────────┘  └─────────┘                  │
├─────────────────────────────────────────────────────────┤
│  Total: $0.89 · 24,562 tok · 4 running                  │
└─────────────────────────────────────────────────────────┘
```

각 카드: SPEC/task + status + progress + cost + primary action.

### 4.2 Frame 14. Agent Thread — **신규**

Zed Agent Panel 모델.

```
┌──────────────────────────────────────────────┐
│  ← SPEC-M2-003 · TDD Implementation    ⌘T   │
├──────────────────────────────────────────────┤
│  🧑 user (you):                              │
│    /moai run SPEC-M2-003 state_json...       │
│                                              │
│  🤖 manager-tdd (Opus 4.6):                  │
│    I'll start with MS-1 (V4 migration)...   │
│    ┌─ tool call: Bash ────────────────┐    │
│    │ command: cargo test --lib        │    │
│    │ result: 289 passed · 250ms       │    │
│    └──────────────────────────────────┘    │
│    ┌─ tool call: Edit ────────────────┐    │
│    │ file: migrations/V4.sql          │    │
│    │ +12 -0 lines · diff open ↗       │    │
│    └──────────────────────────────────┘    │
│    Proceeding to MS-2...                    │
│                                              │
│  🧑 (queued): 실행 중 → 터미널 출력 보여줘  │
├──────────────────────────────────────────────┤
│  [@] [#] [Type message or /command]   ↵    │
│  ⌘↵ Send now · ⌘Q Queue · Esc cancel       │
└──────────────────────────────────────────────┘
```

**차별화**: Claude Code subprocess 의 hook 이벤트를 Thread 메시지로 변환. 기존 Cursor/Zed 와 달리 **MoAI 의 SPEC 워크플로우 전용**.

### 4.3 Frame 15. Context Panel — **신규**

@-mention picker overlay.

```
┌──────────────────────────────────────┐
│  Add Context                         │
├──────────────────────────────────────┤
│  [Search: @xxx]                      │
├──────────────────────────────────────┤
│  📁 Files (12)                       │
│    app/Sources/.../Terminal.swift    │
│    core/crates/moai-store/...        │
│  🔍 Symbols (8)                      │
│    PaneTreeModel.splitActive         │
│    WorkspaceViewModel.createWorkspace│
│  📋 SPECs (3)                        │
│    SPEC-M2-002 (current)             │
│    SPEC-M2-003 (draft)               │
│  💾 Memories (2)                     │
│    project_current_phase             │
│    feedback_testing                  │
│  🛠 MCP Tools (4)                    │
│    context7 · pencil · lsp           │
├──────────────────────────────────────┤
│  [2 selected] [Add to prompt]        │
└──────────────────────────────────────┘
```

VS Code @/# 멘션 모델의 MoAI 확장.

### 4.4 Frame 16. Diff Review — **신규**

Cursor Composer hunk-level accept/reject.

```
┌─────────────────────────────────────────────────┐
│  Review Changes · SPEC-M2-003 (12 files)  ✓6/12 │
├─────────────────────────────────────────────────┤
│  ◄  app/Sources/.../TabBarViewModel.swift  ►   │
├──────────┬──────────────────────────────────────┤
│          │ @@ -200,7 +200,10 @@                  │
│  HEAD    │   func register(...) {                │
│          │-    cache[id] = path                  │
│          │+    persistToSQLite(id: id,          │
│          │+                    path: path)      │
│          │   }                                   │
│          │  [Accept] [Reject] [Edit]             │
├──────────┼──────────────────────────────────────┤
│  AI      │ @@ -310,0 +313,12 @@                  │
│  propose │+  func loadFromSQLite() {             │
│          │+    ...                               │
│          │+  }                                   │
│          │  [Accept] [Reject] [Edit]             │
├──────────┴──────────────────────────────────────┤
│ [◄ Prev file] [File 2/12] [Next file ►]        │
│ [Accept all in file] [Reject all] [Commit 6]    │
└─────────────────────────────────────────────────┘
```

### 4.5 Frame 17. Memory Viewer — **신규**

`~/.claude/projects/{hash}/memory/` 열람.

```
┌──────────────────────────────────┐
│  Memory · MoAI Studio            │
├──────────────────────────────────┤
│  [Filter: user|project|feedback] │
├──────────────────────────────────┤
│  project_current_phase    ⭐    │
│  └─ M2.5 complete, draft SPECs   │
│     Updated: 2026-04-17 01:00    │
│                                  │
│  feedback_testing                │
│  └─ Never mock DB                │
│     Updated: 2026-03-22          │
│                                  │
│  [+ New memory] [Archive old]    │
└──────────────────────────────────┘
```

### 4.6 Frame 18. Hooks & MCP Panel — **신규**

Settings sub-view, Claude Code `/hooks` + `/mcp` UI 대체.

```
┌─────────────────────────────────────────────┐
│  Hooks & MCP                                │
├───────┬─────────────────────────────────────┤
│ Hooks │  [27 Hook events]         [Search]  │
│ MCP   │  ◉ SessionStart        [handle-...]│
│ Rules │  ◉ PreToolUse          [handle-...]│
│ Skills│  ○ PostToolUse         [disabled]  │
│       │  ◉ UserPromptSubmit    [handle-...]│
│       │  ...                                │
│       │  [+ Add hook]                       │
│       │                                     │
│       │  [Test connection: localhost:4273]  │
│       │  Status: ✓ 27/27 reachable          │
└───────┴─────────────────────────────────────┘
```

---

## 5. Empty State 재설계 (긴급)

현재 구현의 **가장 큰 문제**: 워크스페이스 0개 상태에서 완전히 빈 화면.

### 제안: Main Workspace 의 Center Pane Empty State

워크스페이스 미선택 상태에서 Center Pane (3-pane 중 중앙) 표시:

```
┌──────────────────────────────────────────────────┐
│                                                  │
│                      🌱                          │
│                                                  │
│          Welcome to MoAI Studio                  │
│      SPEC-first native shell for agents          │
│                                                  │
│   ┌─────────────────────────────────┐           │
│   │  ＋ Create First Workspace       │           │
│   │                                  │           │
│   │  Opens a project folder and      │           │
│   │  attaches a terminal + agent     │           │
│   └─────────────────────────────────┘           │
│                                                  │
│   ┌──────────────────┐ ┌──────────────────┐    │
│   │ 🚀 Start Sample  │ │ 📂 Open Recent  │    │
│   │ Guided tour      │ │ Last used       │    │
│   └──────────────────┘ └──────────────────┘    │
│                                                  │
│   Tip: ⌘K opens Command Palette anytime         │
│                                                  │
└──────────────────────────────────────────────────┘
```

Notion 모델 (pre-filled templates, helpful tooltips, prompt to create first note) 의 MoAI 적용.

---

## 6. 구현 우선순위 (재정렬)

이전 세션의 `SPEC-M2-UX-001` 스코프를 **본 재설계 기반으로 확장**:

### Priority 1 (M2-UX-001 essential, 이번 재설계의 핵심)
- **P1.1** 치명 버그: `WorkspaceViewModel.requestNewWorkspace()` 실구현 + `requestRename` 실구현
- **P1.2** Menu Bar 완전 확장 (File/Edit/View/Pane/Surface/SPEC/Agent/Go/Window/Help)
- **P1.3** Toolbar 도입 (7 primary actions)
- **P1.4** Empty State CTA (위 5 절)
- **P1.5** NewWorkspaceSheet 개선 (NSOpenPanel + 5-step wizard Frame 09 의 간략 버전)
- **P1.6** StatusBar 확장 (Agent status pill + ⌘K 힌트)

### Priority 2 (M2-UX-002 onboarding)
- **P2.1** First-run Onboarding (Frame 11 실구현)
- **P2.2** Sample Workspace 자동 생성 플로우
- **P2.3** Command Palette 섹션 그룹 + nested + @/# mentions
- **P2.4** Context Panel Frame 15 overlay
- **P2.5** XCUITest 통합 (실제 empty state → workspace create → terminal render 검증)

### Priority 3 (M3+ agentic)
- **P3.1** Code Viewer Deep Dive (SPEC-M3-001 이미 존재)
- **P3.2** Agent Run Viewer 실구현 (Frame 05 기반)
- **P3.3** Mission Control (Frame 13 신규)
- **P3.4** Agent Thread (Frame 14 신규)
- **P3.5** Memory Viewer (Frame 17 신규)
- **P3.6** Hooks & MCP Panel (Frame 18 신규)

### Priority 4 (Dashboard & polish)
- **P4.1** Kanban Board 실구현 (Frame 02)
- **P4.2** Project Dashboard 실구현 (Frame 03)
- **P4.3** Settings 섹션 (Frame 10)
- **P4.4** CG Mode View 실구현 (Frame 12)
- **P4.5** Diff Review (Frame 16)

---

## 7. SPEC 분할 (재조정)

현 재설계에 맞춰 SPEC 을 재구획한다:

| SPEC ID | 제목 | 범위 | 우선순위 |
|---------|------|------|----------|
| SPEC-M2-UX-001 | Integration Rescue & First-Run | P1.1 ~ P1.6 | High (critical) |
| SPEC-M2-UX-002 | Onboarding + Sample Workspace | P2.1 ~ P2.5 | High |
| SPEC-M2-003 | Surface State Persistence | (기존) | Medium |
| SPEC-M3-001 | Code Viewer Deep Dive | (기존) | High (P3.1) |
| SPEC-M5-001 | Agent Run Viewer + Mission Control | P3.2 ~ P3.4 | High |
| SPEC-M5-002 | Memory Viewer + Context Panel | P3.5 + F-15 Frame 15 | Medium |
| SPEC-M5-003 | Hooks & MCP Admin Panel | P3.6 | Medium |
| SPEC-M6-001 | Dashboards (Kanban + Project + Settings) | P4 | Medium |

---

## 8. 디자인 검증 기준 (Acceptance)

재설계된 Pencil frames 가 구현으로 변환될 때 검증해야 할 기준:

1. **첫 실행 시 빈 창 금지** — 반드시 Onboarding 또는 Empty State CTA.
2. **메뉴 바 모든 단축키 노출** — 21개 이상 MenuItem.
3. **Toolbar 최소 5 actions** — 항상 표시.
4. **Agent Run 상태 항상 가시** — 상태바 pill.
5. **Command Palette 5 섹션 이상** — Favorites/Recent/Commands/Files/Symbols.
6. **@/# 멘션 동작** — `@file` `#symbol` 입력 시 picker 등장.
7. **Follow Agent 토글** — Agent Run Viewer + Code Viewer 연동.
8. **TRUST 5 점수 상시** — Project Dashboard 또는 상태바에서 1-click 접근.
9. **모든 인터랙티브 요소 accessibilityIdentifier** — XCUITest.
10. **최소 XCUITest 10 시나리오** — empty → create → render → split → agent run.

---

버전: 1.0.0 · 2026-04-17
