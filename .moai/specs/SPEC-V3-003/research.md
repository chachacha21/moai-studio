# SPEC-V3-003 Research — Tab / Pane Split (Phase 3)

---
spec_id: SPEC-V3-003
phase: 3 (Tab/Pane split)
created: 2026-04-24
updated: 2026-04-24
author: MoAI (manager-spec, research-only delegation)
language: Korean
scope: research-only — spec.md / plan.md / acceptance.md 는 본 위임 범위 밖
depends_on: SPEC-V3-001 (scaffold, DONE), SPEC-V3-002 (Terminal Core, completed v1.1.0)
---

## 0. 본 문서의 목적과 한계

본 research.md 는 SPEC-V3-003 의 **경쟁 레퍼런스 분석 + 현재 코드베이스 경계 + 기술 후보 + 위험 + 종합 권고** 만을 담는다. EARS 요구사항, AC, Task 분해, milestone 은 **확정 질문 답변 이후** spec.md / plan.md / acceptance.md 에서 작성한다.

본 문서의 모든 기술 권고는 "권장안" 이며, §8 의 사용자 확정 질문 리스트 답변에 따라 최종 확정된다. 가상 승인 생성 금지.

### 용어 사전 (본 문서 한정)

- **Pane** — 하나의 TerminalSurface (또는 향후 다른 Surface) 가 점유하는 사각형 영역. 비분할된 leaf 또는 분할된 split 노드일 수 있다.
- **Tab** — Pane tree 1 개와 1:1 매핑되는 컨테이너. 사용자가 "탭" 으로 인식하는 단위.
- **Split** — 한 pane 을 수평/수직으로 2 개 이상의 child pane 으로 나누는 작업.
- **Focus** — 키 입력을 받는 단 하나의 pane. workspace 당 1 개 유지.
- **Session persistence** — 앱 재시작 시 pane tree, tab list, focus, scrollback 등을 복원하는 기능.

---

## 1. 경쟁 레퍼런스 분석

본 Phase 가 참고할 수 있는 선행 구현 6 종을 탭 모델, split 알고리즘, resize UX, 키 바인딩, 세션 지속성 5 축으로 요약.

### 1.1 cmux (Manaflow, 2026-02)

| 항목 | 내용 |
|------|------|
| 탭 모델 | Pane 내부에 tabs. Pane 이 tab 의 컨테이너이자 split 의 leaf. |
| Split 알고리즘 | **BondSplit** (binary tree). cmux 독자 라이브러리. macOS NSSplitView 상위 추상. |
| Resize UX | Divider drag (macOS native). 단위는 pixel + fair-share 혼합. |
| 키 바인딩 | **No prefix keys**. macOS 네이티브 Cmd+[T/W/D/Shift+D] 직접 매핑. |
| 세션 지속성 | Agent context env vars (`CMUX_WORKSPACE_ID`, `SURFACE_ID`, `SOCKET_PATH`) 로 pane 재부착 가능. |
| MoAI 채택 가능성 | **High** — `.moai/design/v3/research.md:9-31` 에서 "직접 참조 제품" 로 선언. 스택 일치 (Swift/AppKit → Rust/GPUI 치환만 다름). |

### 1.2 Zed Terminal (GPUI 기반)

| 항목 | 내용 |
|------|------|
| 탭 모델 | **Pane 이 tabs 의 container**. `Pane.items: Vec<Box<dyn ItemHandle>>` + `active_item_index: usize`. 각 item 은 ItemHandle trait 구현 (editor / search result / terminal 등). |
| Split 알고리즘 | **Binary tree** via `PaneGroup` / `PaneAxis`. Pane 이 `Event::Split { direction, mode }` 를 emit → Workspace 가 tree 조작. |
| Resize UX | GPUI `Divider` 컴포넌트 + drag 핸들. Proportional (flex) 기반. |
| 키 바인딩 | **Direct + modifier** (Cmd+T/Cmd+\\ 등). VSCode 스타일에 가깝다. Command palette 로 보완. |
| 세션 지속성 | `project_item_restoration_data` 필드 + pinned tab count 저장. 전체 workspace serialization 지원. |
| MoAI 채택 가능성 | **Very High** — 우리도 GPUI 기반. Zed 의 Pane trait 계층과 split event 패턴을 거의 그대로 참조 가능. 단, workspace crate 전체 의존은 피하고 구조만 모방. |

**핵심 인용** (zed-industries/zed `crates/workspace/src/pane.rs` WebFetch 결과): Pane 은 Workspace 에 `WeakEntity<Workspace>` 로 연결되고, split 실제 처리는 PaneGroup 으로 위임된다. `FocusHandle` + `last_focus_handle_by_item` 로 탭 전환 시 포커스 복원한다.

### 1.3 WezTerm (Rust, mux 기반)

| 항목 | 내용 |
|------|------|
| 탭 모델 | **Tab 안에 pane tree**. Tab 이 최상위, pane 은 tab 내부 split. |
| Split 알고리즘 | **Binary tree**. Direction 4 종 (Up/Down/Left/Right). |
| Resize UX | `{Percent=50}` / `{Cells=10}` / default 50-50. Lua 설정으로 resize step 커스터마이즈 가능. |
| 키 바인딩 | **Direct mapping** (prefix 없음, 기본 modifier Ctrl+Shift). Lua 로 완전 커스텀 가능. |
| 세션 지속성 | mux 서버가 pane 프로세스를 host. Client 재연결 시 scrollback 까지 유지 (resurrect). |
| MoAI 채택 가능성 | **Medium** — Rust native 로 참조 가능성 높으나 mux 서버 모델은 Phase 3 범위로 과대. pane 트리 + direction 4종만 계승. |

### 1.4 tmux (검증된 de-facto 표준)

| 항목 | 내용 |
|------|------|
| 탭 모델 | **Session → Window → Pane** 3 단계. "Tab" 에 해당하는 것이 Window. |
| Split 알고리즘 | **재귀 binary split** + 5 개의 named layout 알고리즘 (even-horizontal, even-vertical, main-horizontal, main-vertical, tiled). |
| Resize UX | `C-b C-<arrow>` (fine, 1 cell) + `C-b M-<arrow>` (coarse, 5 cells). Zoom (`C-b z`) 로 단일 pane 임시 최대화. |
| 키 바인딩 | **Prefix key (C-b)** + chord. 4 개의 key table (root / prefix / copy-mode / copy-mode-vi). 사용자 `.tmux.conf` 로 완전 재정의. |
| 세션 지속성 | **Server-side**. Detach 시에도 서버 프로세스가 panes 를 host. Client 는 attach/detach 로 연결만. |
| MoAI 채택 가능성 | **Low-direct, High-conceptual** — prefix key 철학은 우리 타겟 (native macOS app) 과 충돌. 하지만 named layout (even-horizontal 등) 은 사용자 경험으로 차용 고려 가능. **우리는 tmux 를 내부 pane 에서 실행할 수 있어야 한다** (SPEC-V3-002 C-1 제약). tmux 를 대체하지 않는다. |

### 1.5 iTerm2 (macOS native)

| 항목 | 내용 |
|------|------|
| 탭 모델 | Window → Tabs → Panes. 탭이 pane tree 를 가짐. |
| Split 알고리즘 | Binary split (Cmd+D horizontal, Cmd+Shift+D vertical 관례). |
| Resize UX | Divider drag + "Navigate Pane Shortcuts" (Cmd+Opt+arrow). |
| 키 바인딩 | Cmd+T (new tab), Cmd+D (split), Cmd+W (close), Cmd+숫자 (tab switch), Cmd+Opt+arrow (pane navigation). **macOS 사용자의 muscle memory 기준치**. |
| 세션 지속성 | **Arrangement** 기능으로 window/tab/pane 구성을 저장/복원. scrollback 포함 restoration 가능. |
| MoAI 채택 가능성 | **High for UX** — macOS 사용자 기대치. 키 바인딩 관례 (Cmd+T/D/W/숫자) 를 계승하는 것이 학습 곡선 최소화. |

### 1.6 Claude Code Desktop 2026-04-14 (경쟁사)

| 항목 | 내용 |
|------|------|
| 탭 모델 | Multi-session sidebar + tabbed workspace layout. pane types: chat / diff / preview / terminal / file / plan / tasks / subagent. |
| Split 알고리즘 | Drag-and-drop workspace layout (flex 기반 추정). |
| Resize UX | Drag divider + preset layouts. |
| 키 바인딩 | Ctrl+\` (terminal), Cmd+; (side chat). Electron 기반. |
| 세션 지속성 | Parallel sessions 유지. |
| MoAI 채택 가능성 | **Differentiation reference** — 우리는 terminal-first + native, 경쟁사는 chat-first + Electron. 단, "pane types" 확장 개념은 SPEC-V3-005 (Surfaces) 와 연계 필요. |

### 1.7 경쟁 매트릭스 요약

| 기능 | cmux | Zed | WezTerm | tmux | iTerm2 | **MoAI Studio (v3-003 후보)** |
|------|------|-----|---------|------|--------|-------------------------------|
| 탭 모델 | pane 안 tabs | pane items | tab 안 panes | session>win>pane | win>tabs>panes | **미확정** (Q1) |
| Split tree | binary | binary | binary | binary+named | binary | 권장: **binary** |
| Prefix key | ❌ | ❌ | ❌ | ✅ | ❌ | 권장: **❌ (native direct)** |
| Resize | pixel+flex | flex | percent/cells | step | drag | 권장: **flex proportional** |
| Session 복원 | env | full | mux 서버 | server | arrangement | **미확정** (Q4) |
| 최대 depth | unbounded | unbounded | unbounded | unbounded | unbounded | **미확정** (Q2) |

### 1.8 핵심 인사이트

1. **Binary tree 가 사실상 업계 표준** — cmux/Zed/WezTerm/iTerm2 모두 binary tree. Flex grid 는 Claude Code Desktop 같은 chat-centric 앱만 사용. MoAI Studio 는 terminal-first 이므로 binary tree 가 자연스럽다.
2. **Prefix key 는 tmux 외 누구도 쓰지 않는다** — macOS 네이티브 앱 관례는 Cmd+modifier 직접 매핑. 사용자가 기존에 tmux 를 pane 내부에서 돌릴 것이므로 호스트 앱까지 prefix 를 쓰면 "중첩 prefix hell" 이 발생한다. MoAI Studio 는 반드시 direct mapping 을 따른다.
3. **Tab 위계는 2 갈래만 있다** — (a) Zed/WezTerm/cmux: 1 workspace = N tabs, 각 tab 에 pane tree. (b) tmux: session > window(=tab) > pane. iTerm2 는 (a) 에 가깝다. 우리는 이미 workspace = project 개념을 가지고 있으므로 (a) 가 자연스럽다.
4. **Session 복원 범위는 비용 대비 이익 곡선** — env var 참조 (cmux) 는 저비용 고효용. Full scrollback (WezTerm mux, iTerm2 arrangement) 은 고비용 고효용. MVP 는 "open pane 만" 복원이 합리적. scrollback 복원은 libghostty-vt 의 scrollback serialization 지원 여부에 달려있다 (별도 조사 필요, §5 에서 기록).
5. **Named layout (even/main/tiled) 은 power feature** — MVP 외 stretch goal 로 분리 가능. tmux 로 충분히 커버되므로 호스트 앱이 반드시 제공할 필요 없음.

---

## 2. 현재 코드베이스 경계 분석

SPEC-V3-001 (scaffold) + SPEC-V3-002 (Terminal Core) 완료 후 현재 commit 에서 Tab/Pane split 을 추가하려 할 때 영향 받는 모듈과 수정 지점.

### 2.1 Terminal Core (확장 지점, 수정 최소화 대상)

Phase 2 에서 확립된 API 는 **변경 없이 재사용** 하는 것이 목표. 수정이 필요하면 위험 1 순위로 취급.

| 파일 | 현재 역할 | Phase 3 에서의 계획 |
|------|-----------|----------------------|
| `crates/moai-studio-terminal/src/pty/mod.rs` | `Pty` trait 정의 (fan_in ≥ 3 expected) | **변경 없음**. pane 마다 독립 Pty 인스턴스를 생성. |
| `crates/moai-studio-terminal/src/worker.rs` | PtyWorker (1 개 pty 담당) | **변경 없음**. pane 수만큼 PtyWorker 생성. 채널은 pane 별로 독립 (tokio::sync::mpsc::UnboundedSender<PtyEvent> 씩). |
| `crates/moai-studio-terminal/src/vt.rs` | VtState wrapper (!Send + !Sync) | **변경 없음**. 각 pane 이 고유 VtState 소유. GPUI entity 당 1 개. |
| `crates/moai-studio-terminal/src/events.rs` | `PtyEvent` enum | **변경 없음**. PaneId 같은 식별자는 UI 계층에서 부여 (VtState 자체는 pane 을 모름). |
| `crates/moai-studio-terminal/src/libghostty_ffi.rs` | FFI boundary anchor | **변경 없음**. |

결론: **Terminal Core 는 완전 reuse**. Phase 3 는 UI + Workspace 계층의 확장이다.

### 2.2 UI 계층 (주요 수정 대상)

#### 2.2.1 `RootView` (현재 단일 터미널 가정)

`crates/moai-studio-ui/src/lib.rs:69-76` 의 `RootView` 구조:
```rust
pub struct RootView {
    pub workspaces: Vec<Workspace>,
    pub active_id: Option<String>,
    pub storage_path: PathBuf,
    pub terminal: Option<Entity<terminal::TerminalSurface>>,  // ← 단일 터미널 (수정 필요)
}
```

**단일 터미널 가정이 깨지는 지점**:
- `crates/moai-studio-ui/src/lib.rs:75` — `terminal: Option<Entity<TerminalSurface>>` 는 1 개만 가능
- `crates/moai-studio-ui/src/lib.rs:184` — `let terminal = self.terminal.clone()` 를 main_body 에 전달
- `crates/moai-studio-ui/src/lib.rs:290-299` — `main_body(...)` 가 `Option<Entity<TerminalSurface>>` 단일 값을 받음
- `crates/moai-studio-ui/src/lib.rs:410-444` — `content_area(show_empty_state, terminal)` 가 단일 터미널 또는 empty 만 분기

**Phase 3 의 변경 방향**:
- `terminal` 필드 → `pane_tree: Option<Entity<PaneTree>>` (또는 workspace 당 tab list) 로 확장
- content_area 는 pane_tree 가 Some 이면 PaneTree 렌더, None 이면 Empty State

#### 2.2.2 `TerminalSurface` (pane 의 leaf content 로 재활용)

`crates/moai-studio-ui/src/terminal/mod.rs:101-115` 의 `TerminalSurface` 는 이미 고립된 entity. PaneTree 의 leaf content 로 **수정 없이 재활용** 가능.

단, `handle_key_down` (mod.rs:223) 이 GPUI focus 를 전제로 한다. 다중 pane 중 **어느 TerminalSurface 가 key event 를 받을지** 는 상위 PaneTree 의 focus 관리에 의존. Phase 3 에서 focus routing 구조 필요.

#### 2.2.3 Input / Clipboard (pane 내부 완결, 변경 최소)

- `crates/moai-studio-ui/src/terminal/input.rs` — Keystroke → ANSI 변환. pane 별 독립 동작, **변경 없음**.
- `crates/moai-studio-ui/src/terminal/clipboard.rs` — arboard wrapper. 시스템 수준 공유, **변경 없음**.

### 2.3 Workspace 계층

`crates/moai-studio-workspace/src/lib.rs:27-35`:
```rust
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub project_path: PathBuf,
    pub moai_config: PathBuf,
    pub color: u32,
    pub last_active: u64,
}
```

**단일 workspace = 단일 터미널 가정**:
- Workspace 구조 자체는 pane 에 무관 (project metadata 만). Phase 3 에서 **Workspace 자체는 변경 없음** 이 바람직.
- 단, session 지속성 선택 시 Workspace 에 `pane_state: Option<PaneTreeSerialized>` 같은 필드 추가 검토 (Q4 의존).

### 2.4 App 엔트리

`crates/moai-studio-app/` — workspace list 주입 후 `run_app` 호출. **변경 없음** 예상.

### 2.5 의존성 추가 후보

| Crate | 용도 | 필수? |
|-------|------|-------|
| 없음 | binary tree 는 Rust enum 으로 직접 표현 가능 | — |
| `gpui-component` (longbridge) | Resizable Panel / Dock 컴포넌트 제공 | **선택**. Zed 스타일을 직접 구현할 수도 있고 gpui-component 의 Dock/Resizable 을 reuse 할 수도 있다. 의존성 도입 비용 vs. 직접 구현 비용 비교 필요. |
| `uuid` (또는 기존 id 생성 로직 재사용) | PaneId / TabId 생성 | 권장 (workspace 가 이미 `format!("ws-{:x}", nanos)` 패턴 사용 중, 동일하게 확장) |

### 2.6 테스트 인프라

SPEC-V3-002 에서 확립된 테스트 패턴:
- `cargo test -p moai-studio-terminal --lib` (74 passed 현재)
- `cargo test -p moai-studio-ui --lib` (60 passed 현재)
- `crates/moai-studio-terminal/tests/compile_fail/` (trybuild)
- `crates/moai-studio-terminal/benches/` (criterion)

Phase 3 테스트 영역:
- PaneTree 단위 테스트 (split / close / focus 전환)
- Tab list 단위 테스트 (add / remove / activate)
- Session persistence 단위 테스트 (serialize / deserialize)
- GPUI 렌더 통합 테스트 (headless 제한 있으므로 가능한 범위만)

---

## 3. 기술 선택 후보

각 항목은 **옵션 + 장단점 + 권장안** 로 정리. 최종 결정은 §8 질문 답변 이후.

### 3.1 Split layout 자료구조

**옵션 A: 이진 트리 (binary tree, recursive enum)**
```rust
enum PaneTree {
    Leaf(Entity<TerminalSurface>),
    Split {
        direction: SplitDirection,  // Horizontal / Vertical
        ratio: f32,                 // 0.0 ~ 1.0
        left: Box<PaneTree>,
        right: Box<PaneTree>,
    },
}
```

- 장점: cmux/Zed/WezTerm/iTerm2 모두 채택, 업계 표준, 직관적. Rust enum 으로 자연스럽게 표현. 재귀 split 자연 지원.
- 단점: 3-way split 직접 불가 (2-way split 을 중첩). resize 시 sibling 비율만 조정 가능 (글로벌 균등 분배 불가).

**옵션 B: 플랫 리스트 + flex**
```rust
struct PaneFlex {
    direction: Axis,
    children: Vec<PaneChild>,  // 각 child 는 Pane 또는 nested PaneFlex
}
```

- 장점: N-way split 자연 (tmux even-horizontal 유사). Resize 시 fair share 가능.
- 단점: "왼쪽 오른쪽 분할" 의미 모호, UX 구현 복잡.

**옵션 C: CSS Grid / explicit geometry**
```rust
struct PaneGrid { cells: HashMap<(u32, u32), PaneId>, ... }
```

- 장점: 완전 자유 레이아웃.
- 단점: 분할 연산 복잡, serialize/deserialize 복잡, 업계 선례 없음.

**권장: A (binary tree)** — 업계 표준 + Rust 표현 자연 + 계속 확장 가능 (named layout 을 tree 변환 함수로 표현 가능).

### 3.2 Tab 모델

**옵션 A: Tab 없음 — 단일 pane tree 만**
- 사용자는 항상 split 으로만 영역을 늘린다.
- 단순하지만 정보 밀도 낮음. 모든 터미널이 한 화면에 동시 표시.

**옵션 B: 단일 워크스페이스 = 단일 탭 + 내부 pane split**
- 현재 RootView 의 `terminal: Option<...>` 단일 슬롯 그대로. 탭 UI 없음.
- MVP 로는 가장 가볍지만 탭 미지원은 macOS 사용자 기대치에서 후퇴.

**옵션 C: 단일 워크스페이스 = N 탭, 각 탭 = 1 pane tree** (cmux, Zed, WezTerm, iTerm2 모델)
```rust
struct WorkspacePanes {
    tabs: Vec<Tab>,
    active_tab_idx: usize,
}
struct Tab { id: TabId, title: String, pane_tree: PaneTree }
```
- 정보 밀도 높음, macOS 관례 (Cmd+T) 부합.
- 구현 복잡도 중간. 탭 UI (탭 바) 추가 필요.

**옵션 D: Session > Window > Pane (tmux 모델)**
- 과잉. MoAI Studio 는 이미 workspace = project 위계를 가짐. Window 추가는 중복.

**권장: C (N tabs per workspace, pane tree per tab)** — macOS 관례, cmux 참조, 업계 표준. MVP 로도 구현 가능 (초기엔 탭 1 개만 지원해도 구조는 Tab 가진 상태로).

### 3.3 Focus 모델

**옵션 A: 단일 글로벌 focus**
- workspace 당 1 개의 activePaneId 만 유지.
- 탭 전환 시 해당 탭의 pane_tree 에서 어느 pane 이 active 인지 기억 못함.

**옵션 B: 탭별 focus 기억**
```rust
struct Tab { ..., last_focused_pane: Option<PaneId> }
```
- Zed 의 `last_focus_handle_by_item` 패턴 참조.
- 사용자가 탭 전환 후 돌아왔을 때 마지막 active pane 복원.

**옵션 C: pane 별 focus 내부 상태까지 보존**
- TerminalSurface 내부 focus 까지 저장. 현재 GPUI FocusHandle 이 자동 처리.
- 옵션 B 와 자연 결합.

**권장: B + C 결합** — Zed 패턴. 구현 부담 작음, UX 이득 큼.

### 3.4 키 바인딩 철학

**옵션 A: tmux prefix 스타일 (Ctrl-B %, Ctrl-B ")**
- 사용자가 pane 내부에서 tmux 돌릴 때 충돌 지옥.
- 거의 모든 터미널 앱이 기피.

**옵션 B: Zed / VSCode 스타일 direct + modifier**
- Cmd+\\ (split horizontal), Cmd+Shift+\\ (split vertical), Cmd+W (close pane), Cmd+T (new tab).
- command palette 로 hidden action 보완.
- 사용자는 iTerm2/Zed 에서 이미 익숙.

**옵션 C: VSCode command palette only**
- 모든 액션을 Cmd+Shift+P 로 유도.
- 속도 낮음, 사용자 마찰 높음.

**권장: B (direct mapping)** — 경쟁 매트릭스 §1.7 결론과 일치. `.moai/design/v3/system.md:422-439` 의 플랫폼별 키바인딩 목록 (Cmd+\\ , Cmd+Shift+\\ , Cmd+T, Cmd+W 등) 이 이미 옵션 B 를 전제하고 있음. Phase 3 는 이를 실제로 구현.

### 3.5 세션 지속성 범위

**옵션 A: 없음 (MVP)**
- 앱 재시작 시 pane tree 초기화 + 단일 터미널로 시작.
- 구현 비용 0, 사용자 경험 최하.

**옵션 B: Open pane 만 복원**
- 몇 개의 pane 이 어느 구조로 열려있었는지, 각 pane 의 shell command (기본 `$SHELL`) 만 저장.
- scrollback 은 복원 안함 (새 쉘 세션).
- JSON serialize 가능 (PaneTree + Tab + PaneId).
- 구현 비용 중간, 사용자 경험 중간.

**옵션 C: scrollback 까지 복원**
- libghostty-vt 의 terminal state 를 직렬화 가능해야 함 (현재 지원 여부 미확인).
- 고비용. 실제로 WezTerm mux 처럼 별도 서버 프로세스가 필요할 가능성.
- 사용자 경험 최고.

**옵션 D: Shell session 복원 (cmux 방식)**
- Shell 프로세스를 호스트 앱과 분리해서 돌리고 재부착. 유닉스 socket + agent env vars.
- cmux 가 실제로 이 방식을 쓴다.
- 구현 비용 매우 높음, Phase 3 범위 초과.

**권장: B (open pane 만 복원)** — MVP + 확장성. 옵션 C/D 는 별도 SPEC (V3-003.1 또는 V3-세션-복원) 으로 이관.

### 3.6 Resize UX

**옵션 A: Proportional (flex) — Zed 스타일**
- 각 split 노드가 `ratio: f32` (0.0~1.0) 저장.
- Divider drag 로 두 sibling 사이 비율만 조정.
- 윈도우 resize 시 비율 유지.

**옵션 B: Pixel-based (fixed) — cmux 스타일 일부**
- 각 pane 이 최소 크기 + 실제 pixel 지정.
- 윈도우 resize 시 재배치 알고리즘 복잡.

**옵션 C: Step-based (tmux 스타일)**
- 키 입력으로 1 cell 또는 5 cell 씩 resize.
- drag 없이 키보드만.

**권장: A (proportional)** — GPUI flexbox 자연 결합. 계산 단순. 키 기반 resize (옵션 C) 는 추가 단축키로 MVP 이후 보완 가능.

### 3.7 Split 최대 깊이

**옵션 A: 제한 없음 (recursion limit 만)**
- Rust stack overflow 까지 가능 (실질 수백 depth).
- tmux/Zed/WezTerm 모두 이 방식.

**옵션 B: 하드 제한 (예: 최대 8 pane)**
- UX 보호 차원 (화면 너무 작아지지 않도록).
- 사용자가 실수로 수십개 만들 가능성 차단.

**권장: A (제한 없음) + UX 경고** — Rust enum 재귀는 실질 제한 없음. 최소 pane 크기 (가로 40 cells × 세로 10 rows 같은) 가 실질 제한으로 작용.

---

## 4. 위험 및 제약

### 4.1 GPUI Resize drag 지원 범위

- **현황**: SPEC-V3-002 는 고정 4 영역 레이아웃만 사용 (TitleBar / Sidebar / Body / StatusBar). GPUI 0.2.2 에서 drag-to-resize divider 의 공식 API 는 미확인.
- **완화책**: (a) gpui-component (longbridge) 의 Resizable Panel 컴포넌트 사용 — Context7 조회 결과 "Dock layout for panel arrangements, resizable panels" 제공 확인. (b) 직접 구현: GPUI mouse event + state + flex basis 조정.
- **위험도**: 중. gpui-component 의존 도입 여부가 Q 수준에서 결정될 사안.

### 4.2 portable-pty 다중 spawn 시 리소스 한계

- **현황**: 각 pane = 1 개 PTY pair + 1 개 reader thread + 1 개 libghostty-vt Terminal 인스턴스.
- **한계 추정**: pane 당 ~60 MB RSS (SPEC-V3-002 §5.2). 8 pane 시 ~480 MB. 일반 사용자 MBP 16GB 기준 괜찮음.
- **FD 압박**: pane 당 ptmx 2 개 (master + slave). 16 pane = 32 FD. macOS 기본 256 / Linux 기본 1024 대비 여유 있음.
- **위험도**: 저.

### 4.3 libghostty-vt 인스턴스당 메모리

- **현황**: SPEC-V3-002 §5.2 에 "scrollback 10,000 rows 기준 60 MB/terminal". Scrollback 100,000 rows 상한 허용.
- **위험**: N pane 동시 scrollback 증가 시 메모리 spike.
- **완화**: pane 별 scrollback 상한 을 workspace 설정으로 조정 가능하게 (Phase 3.1 이후).
- **위험도**: 저-중.

### 4.4 SPEC-V3-002 와의 호환성 — Pty/worker API 변경 최소화

- **원칙**: SPEC-V3-002 의 공용 API (Pty trait, PtyWorker, VtState, PtyEvent) 는 **절대 변경하지 않는다**.
- **허용 변경**: 신규 API 추가만 가능. 예) `PtyWorker::with_id(PaneId)` 같은 builder.
- **위험도**: 저 (원칙 준수 시).

### 4.5 Focus routing 과 GPUI key dispatch

- **현황**: TerminalSurface::handle_key_down 은 GPUI 가 이 entity 를 focused 로 판단할 때만 호출됨.
- **위험**: 다중 TerminalSurface 존재 시 GPUI 의 focus 이동 (click / Tab key / programmatic) 과 우리의 activePaneId 관리가 괴리될 수 있음.
- **완화**: Zed 의 `FocusHandle + last_focus_handle_by_item` 패턴 참조. GPUI FocusHandle 을 PaneId 와 1:1 매핑.
- **위험도**: 중 (구현 복잡도).

### 4.6 탭 바 UI 디자인 부재

- **현황**: `.moai/design/v3/system.md` 에 탭 바 디자인 토큰이 명시적으로 없음 (Toolbar 만 §8 에 있음). Pencil frame 01 "Main Workspace" 는 탭 바를 포함하는지 재확인 필요.
- **완화**: 본 SPEC 에서 탭 바의 최소 시각 스펙을 자체 정의하고, 별도 디자인 SPEC 에서 보강.
- **위험도**: 저-중.

### 4.7 세션 persistence 와 workspace 복구 역호환성

- **현황**: SPEC-V3-001 의 `WorkspacesStore` 는 `~/.moai/studio/workspaces.json` 에 workspace 메타데이터만 저장.
- **Phase 3 에서 session 지속성 B 선택 시**: pane tree / tab 구조를 어디에 저장할지 결정 필요. 옵션 ① workspaces.json 확장 (기존 schema 깨짐 위험). 옵션 ② 별도 파일 `~/.moai/studio/panes-{ws-id}.json`.
- **완화**: schema version 관리. `WorkspacesFile` 의 `$schema` 필드 활용 (이미 `"moai-studio/workspace-v1"` 지정됨, `lib.rs:89-91`).
- **위험도**: 중.

---

## 5. 추가 조사 필요 사항 (Phase 3 spec 작성 전)

본 research 로는 확정 못하고 spec.md 작성 전/중에 추가 spike 필요한 항목:

1. **GPUI 0.2.2 의 divider drag API 실존 여부** — gpui-component 의존 여부 결정 전제. 10 분 spike 필요.
2. **libghostty-vt 의 scrollback serialization 지원 여부** — 세션 persistence 옵션 C 가능성 판단. upstream 소스 확인 필요.
3. **gpui-component 0.x 의 Resizable/Dock 컴포넌트 API 안정성** — 실제 사용 샘플 (longbridge/gpui-component/crates/ui/src/dock) 코드 1 시간 리뷰 필요.
4. **기존 `moai-studio-app` 바이너리가 새 RootView 구조를 그대로 쓸 수 있는지** — workspace 주입 경로 일치 확인.
5. **Pencil frame 01 "Main Workspace" 의 탭 바 시각 스펙** — `.moai/design/v3/system.md:372-416` 의 IA 트리에 탭 바가 누락됨. 디자인 소스 (Pencil 파일) 에서 재확인 필요.

---

## 6. 종합 권고

### 6.1 MVP (Phase 3.0) 범위 제안

**IN SCOPE**:
- Binary tree PaneTree (§3.1 A)
- Workspace 당 N tabs, 각 tab = 1 pane tree (§3.2 C)
- Proportional resize (§3.6 A) + drag divider
- Direct mapping 키 바인딩 (§3.4 B) — Cmd+\\ (split H), Cmd+Shift+\\ (split V), Cmd+W (close pane), Cmd+T (new tab), Cmd+Shift+W (close tab), Cmd+1~9 (tab switch)
- Focus routing (§3.3 B+C) — 탭별 last-focused pane 기억
- 세션 persistence B — open pane 구조만 JSON 저장 (§3.5 B)
- Split 최대 깊이 제한 없음 (§3.7 A)
- SPEC-V3-002 Terminal Core API 무수정

**OUT OF SCOPE (Phase 3.1 이후)**:
- Named layout (even/main/tiled)
- Pane zoom (tmux C-b z)
- Drag-and-drop pane 재배치
- Scrollback 복원
- Pane 간 텍스트 복사 / 검색 / 브로드캐스트
- 탭 reordering (drag)
- 탭 detach/reattach to separate window

### 6.2 우선순위 제안

| 우선순위 | 항목 | 사유 |
|----------|------|------|
| P0 (Critical) | PaneTree 자료구조 + split/close API | 모든 후속 기능 의존 |
| P0 | TerminalSurface × PaneId 매핑 + Focus routing | 다중 pane 의 기본 |
| P0 | Divider drag (GPUI 또는 gpui-component) | 사용자 경험 필수 |
| P1 (High) | Tab list + 탭 바 UI + 탭 전환 | macOS 관례 준수 |
| P1 | 기본 키 바인딩 6-8 개 | 발견성 |
| P2 (Medium) | 세션 persistence B | 재시작 후 복구 |
| P3 (Low) | 탭 이름 편집 / 색상 지정 | nice-to-have |

### 6.3 SPEC-V3-003 의 분할 가능성

본 Phase 의 범위가 넓다면 아래 분할 고려:

- **SPEC-V3-003 (Core)**: PaneTree + split/close + resize + focus
- **SPEC-V3-003.1 (Tabs)**: Tab list + 탭 바 UI + 탭 전환
- **SPEC-V3-003.2 (Persistence)**: 세션 저장/복원

단, Tab 과 Pane 이 강결합 (Tab = pane_tree 소유자) 이므로 분할 시 설계 마찰이 발생할 수 있음. **통합 단일 SPEC 으로 진행하되 milestone 을 3 개로 나누는 것** 이 더 합리적일 수 있다. 최종 결정은 Q5 (size 제약) 답변 이후.

---

## 7. 참조 문헌

### 7.1 직접 인용한 파일 (본 레포 내)

- `.moai/design/v3/research.md:9-31` — cmux 을 MoAI Studio 의 "직접 참조 제품" 으로 지정
- `.moai/design/v3/spec.md:180-206` — Tier A Terminal Core 기능 A-1 "멀티 pane 터미널 (binary tree split)" Critical 우선순위
- `.moai/design/v3/system.md:240-250` — 3-Pane Body NSSplitView binary tree 언급
- `.moai/design/v3/system.md:422-439` — 플랫폼별 키바인딩 (Cmd+\\ , Cmd+Shift+\\ , Cmd+T, Cmd+W)
- `.moai/specs/SPEC-V3-001/progress.md:91-98` — "후속 Phase 후보" 목록 중 "Tab / Pane split" 첫 항목
- `.moai/specs/SPEC-V3-002/spec.md:278-291` — Exclusions #1 "Tab UI / Pane split — SPEC-V3-003 예정"
- `crates/moai-studio-ui/src/lib.rs:69-76` — RootView 의 단일 `terminal: Option<Entity<TerminalSurface>>` 필드
- `crates/moai-studio-ui/src/lib.rs:410-444` — `content_area` 분기 (단일 terminal vs empty state)
- `crates/moai-studio-ui/src/terminal/mod.rs:101-115` — TerminalSurface 구조 (pane leaf content 로 재활용 가능)
- `crates/moai-studio-ui/src/terminal/mod.rs:223` — handle_key_down (focus 전제 key dispatch)
- `crates/moai-studio-terminal/src/pty/mod.rs` — Pty trait (pane 별 독립 인스턴스용 재사용 대상)
- `crates/moai-studio-terminal/src/worker.rs:114-174` — PtyWorker (pane 당 1 개 spawn 대상)
- `crates/moai-studio-workspace/src/lib.rs:27-35` — Workspace 구조 (Phase 3 에서 변경 최소)
- `crates/moai-studio-workspace/src/lib.rs:89-91` — `"moai-studio/workspace-v1"` schema 버전 (session persistence 확장 시 관리 대상)

### 7.2 외부 참조

- **cmux**: [cmux.com](https://cmux.com), [cmux vs tmux (soloterm.com)](https://soloterm.com/cmux-vs-tmux), [CMUX Complete Guide (agmazon.com)](https://agmazon.com/blog/articles/technology/202603/cmux-terminal-ai-guide-en.html)
- **Zed Pane**: [zed-industries/zed/crates/workspace/src/pane.rs](https://github.com/zed-industries/zed/blob/main/crates/workspace/src/pane.rs) — ItemHandle trait, PaneGroup/PaneAxis, FocusHandle 패턴
- **WezTerm SplitPane**: [wezterm.org/config/lua/keyassignment/SplitPane.html](https://wezterm.org/config/lua/keyassignment/SplitPane.html) — binary tree, 4 direction, Percent/Cells/default 50-50
- **tmux Getting Started**: [github.com/tmux/tmux/wiki/Getting-Started](https://github.com/tmux/tmux/wiki/Getting-Started) — session > window > pane, named layouts, prefix key
- **iTerm2 Preferences Keys**: [iterm2.com/documentation-preferences-keys.html](https://iterm2.com/documentation-preferences-keys.html)
- **gpui-component**: [github.com/longbridge/gpui-component](https://github.com/longbridge/gpui-component) — Dock layout + Resizable Panel 컴포넌트 후보
- **Claude Code Desktop 2026-04-14 redesign**: [claude.com/blog/claude-code-desktop-redesign](https://claude.com/blog/claude-code-desktop-redesign) — pane types + drag-drop layout 경쟁사 패턴

---

## 8. Scope 확정 필요 질문 (사용자 확인 대상)

본 §8 은 **MoAI 오케스트레이터가 사용자에게 AskUserQuestion 으로 확인해야 하는 질문 리스트** 이다. 각 질문은 2~4 개 선택지 + 권장안 + 영향 분석을 포함. 답변 수집 후 spec.md / plan.md / acceptance.md 작성을 재위임한다.

**[HARD]** 본 agent 는 답변을 가상으로 생성하지 않는다. 답변은 오케스트레이터가 사용자로부터 받아 재위임한다.

### Q1. 탭 모델 범위

- **배경**: §3.2. 경쟁 매트릭스 §1.7 에서 4/5 가 "workspace 당 N 탭, 탭마다 pane tree" 모델 채택. 단일 탭만으로도 MVP 는 가능.
- **선택지**:
  - **A. [권장]** 단일 워크스페이스 = N 탭, 각 탭 = 1 pane tree (cmux/Zed/WezTerm/iTerm2 표준)
    - 영향: 탭 바 UI 필요. 키 바인딩 Cmd+T/W/1~9. 복잡도 중.
  - **B.** 단일 워크스페이스 = 단일 탭 + 내부 pane split 만 (MVP 단순화)
    - 영향: 탭 바 불필요. 구현 간단. 다만 향후 탭 추가 시 migration 부담.
  - **C.** 단일 워크스페이스 = 단일 pane tree 만 (탭 개념 자체 도입 연기)
    - 영향: 구조 가장 단순. macOS Cmd+T 관례 위반, 사용자 학습 곡선 증가.
- **권장 사유**: cmux 와의 스택 일치 + macOS 관례 + 경쟁 매트릭스 다수 채택. 초기 MVP 에서는 탭 1 개로 시작해도 데이터 모델만 N 탭 지원하면 후속 작업 없음.

### Q2. Pane split 최대 깊이

- **배경**: §3.7. 업계 표준은 "제한 없음". 사용자 실수 보호 차원에서 하드 제한도 옵션.
- **선택지**:
  - **A. [권장]** 제한 없음 (Rust 재귀 한계까지). 대신 최소 pane 크기 (예: 40 cols × 10 rows) 로 실질 제한.
    - 영향: tmux/Zed/WezTerm 과 동일. 사용자 자유도 최대.
  - **B.** 하드 제한: 탭당 최대 8 pane
    - 영향: UX 보호. 일부 power user 반발 가능.
  - **C.** 하드 제한: 탭당 최대 4 pane
    - 영향: 매우 보수적. 화면 작을 때 실용적.
- **권장 사유**: 최소 pane 크기 제약만으로 충분. 하드 제한은 power user 반발 소지.

### Q3. 키 바인딩 철학

- **배경**: §3.4. 디자인 문서 (`.moai/design/v3/system.md:422-439`) 가 이미 direct mapping 을 전제. 본 질문은 확정 및 미세 조정.
- **선택지**:
  - **A. [권장]** Zed/iTerm2 식 direct + modifier (Cmd+\\ / Cmd+Shift+\\ / Cmd+T / Cmd+W / Cmd+1~9)
    - 영향: macOS 관례. 사용자 muscle memory. pane 내부 tmux 와 충돌 없음.
  - **B.** tmux 식 prefix (Ctrl+B 후 keycode)
    - 영향: tmux 중첩 시 심각한 충돌. 거의 모든 native 앱 기피.
  - **C.** VSCode 식 command palette only (Cmd+Shift+P)
    - 영향: 속도 낮음. 사용자 마찰 높음.
- **권장 사유**: 이미 디자인 문서가 방향을 정한 내용. 본 질문은 **실제 키 조합의 표준 확정** 에 가깝다. 옵션 A 선택 시 세부 키 표를 spec.md 에 확정.

### Q4. 세션 지속성 범위

- **배경**: §3.5. 비용 대비 이익 곡선.
- **선택지**:
  - **A.** 없음 — 앱 재시작 시 pane tree 초기화 (MVP 단순화)
    - 영향: 개발 비용 0. 사용자 경험 최저.
  - **B. [권장]** open pane 구조만 JSON 저장/복원 — 몇 개 pane 이 어떻게 배치되어 있었는지만 복원. 각 pane 은 새 쉘 세션으로 시작 (scrollback 없음).
    - 영향: 중간 비용. 구조는 `~/.moai/studio/panes-{ws-id}.json` 별도 파일로 분리 (§4.7). 개발 부담 중간.
  - **C.** scrollback 까지 복원 — libghostty-vt terminal state 직렬화. 추가 spike 필요 (§5.2).
    - 영향: 고비용. upstream 미지원 시 구현 불가능. Phase 3 범위 초과.
  - **D.** Shell session 복원 (cmux 방식) — 호스트 앱과 쉘 분리. Unix socket 서버.
    - 영향: 매우 고비용. 별도 SPEC 필요. Phase 3 범위 크게 초과.
- **권장 사유**: B 가 MVP + 확장성 균형. C/D 는 별도 후속 SPEC 으로 이관하여 점진 진화.

### Q5. SPEC-V3-003 단일 vs 분할

- **배경**: §6.3. MVP 범위가 넓으므로 SPEC 을 더 작게 분할하는 것도 가능.
- **선택지**:
  - **A. [권장]** 단일 SPEC-V3-003 + 3 milestones (Core / Tabs / Persistence)
    - 영향: Tab/Pane 의 강결합 때문에 SPEC 분리 시 마찰. 통합 단일 SPEC 으로 milestone 만 3 개. ~~시간~~ → 우선순위 (High/Medium) 로 표현.
  - **B.** 3 SPEC 분할 — SPEC-V3-003 (Core pane) / SPEC-V3-003.1 (Tabs) / SPEC-V3-003.2 (Persistence)
    - 영향: 각 SPEC 크기 작아지나 tab 과 pane 강결합으로 설계 마찰. SPEC 간 dependency 관리 부담.
  - **C.** 2 SPEC 분할 — SPEC-V3-003 (Pane + Tabs) / SPEC-V3-003.1 (Persistence)
    - 영향: Persistence 를 완전 분리. Pane/Tabs 강결합은 통합. 중간 지점.
- **권장 사유**: A 가 가장 단순 + 설계 마찰 없음. Milestone 3 개로 실행 단위 분리. 단, 사용자가 SPEC 실행 단위를 더 작게 원하면 C 도 합리적.

### Q6. gpui-component 의존 도입 여부

- **배경**: §4.1, §2.5. longbridge/gpui-component 가 Dock + Resizable Panel 컴포넌트 제공 (Context7 조회 확인). 의존 도입 vs 직접 구현 선택.
- **선택지**:
  - **A. [권장-조건부]** gpui-component 사용 — Resizable Panel / Dock 컴포넌트 reuse
    - 영향: 개발 속도 향상. 외부 의존성 증가. 버전 호환 리스크 (GPUI 0.2.2 와). **추가 spike 필요** (§5.3) 로 실제 API 및 안정성 확인 후 최종 결정.
  - **B.** 직접 구현 — GPUI mouse event + flex basis 조정 + drag handle 자체 개발
    - 영향: 외부 의존성 없음. 개발 시간 증가. Zed 의 Divider 패턴 참조 가능.
  - **C.** 결정 연기 — spec.md 에서 양쪽 다 plan 만 세우고 plan.md 에서 implementation 직전 결정
    - 영향: 결정 지연. plan 의 task 분해에는 "split layout 컴포넌트" 추상화만 남기고 구체 구현체 미정.
- **권장 사유**: A 가 효율적이나 의존 안정성 확인 선행 필수. Q6 답변이 C 면 spec.md 에는 양쪽 모두 호환되는 추상 인터페이스로 작성.

### Q7. Milestone 순서 (Q5 가 A 일 때만 활성)

- **배경**: Q5 답변이 A (단일 SPEC + 3 milestones) 일 때 milestone 순서를 확정.
- **선택지**:
  - **A. [권장]** MS-1 Pane core (split/close/resize/focus) → MS-2 Tabs (list/bar/switch) → MS-3 Persistence
    - 영향: 가장 자연스러운 순서. MS-1 완료 시점에 단일 탭 + pane 기본 동작 시연 가능. MS-2 추가로 macOS 관례 충족. MS-3 로 재시작 복구.
  - **B.** MS-1 Pane core → MS-2 Persistence → MS-3 Tabs
    - 영향: 탭 없이 persistence 먼저 가능. 하지만 persistence 로직이 탭 미존재 전제로 짜이면 MS-3 에서 재작업.
  - **C.** MS-1 Tabs (껍데기만) → MS-2 Pane core → MS-3 Persistence
    - 영향: 비어있는 탭 바 먼저 배치. 사용자에게 시각적 진척 인상. 실질 기능은 MS-2 에 몰려 있음.
- **권장 사유**: A 가 의존성 그래프상 가장 자연스럽고, 각 milestone 마다 사용자 체감 가치 증가.

---

## 9. 처리 상태

- Research 작성 완료: 2026-04-24
- spec.md / plan.md / acceptance.md: **미착수** (본 위임 범위 밖)
- Annotation cycle: **미착수** (본 위임 범위 밖)
- 다음 단계: MoAI 오케스트레이터가 §8 의 7 개 질문을 AskUserQuestion 으로 사용자에게 제시 → 답변 수집 → 재위임으로 spec.md 초안 작성 시작

---

Version: 0.1.0 · 2026-04-24 (research-only)
