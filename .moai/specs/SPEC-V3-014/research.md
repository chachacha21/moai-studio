# SPEC-V3-014 Research — Banners Surface

> 본 문서는 SPEC-V3-014 (Banners Surface) 의 **사전 조사 (research)** 이다.
> spec.md 가 가벼운 EARS 정의에 집중할 수 있도록, 코드베이스 분석 / UX 패턴 / severity priority / GPUI render 전략 / 위험 요소를 본 문서에 분리 수록한다.

---

## 1. 배경 — 왜 Banners Surface 가 필요한가

### 1.1 사용자 가치

moai-studio 는 IDE 형 데스크톱 앱이지만 현재 **모달 영역 외 비차단(non-blocking) 알림 채널이 없다**. 다음 시스템 이벤트가 발생해도 사용자는 인지할 방법이 없다:

- agent process (moai-supervisor 가 spawn 한 child) 의 crash / sigsegv
- moai-adk 신규 버전 출시 (자동 update check 결과)
- LSP 서버 spawn 실패 (예: rust-analyzer 가 PATH 에 없음)
- PTY fd leak 감지 / spawn 실패 (terminal 시작 불가)
- workspace 상태 복원 실패 (corrupted state JSON)

이 5가지는 모두 **사용자에게 즉시 알려야 하지만, 작업을 차단하지 않아야 하는** 정보이다. modal dialog 는 부적합 (workflow 차단), toast 는 부적합 (위치 일관성 / multi-stack 어려움). **상단 슬림 banner stack** 이 IDE 카테고리의 표준 UX 다 (VS Code, Zed, Sublime Merge 모두 동일 패턴).

### 1.2 IMPLEMENTATION-NOTES.md §13.8 / §14 F 정합

`.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md` v1.1 의 명시적 후속 작업:

- §13.8 — 5 banner 변종 (Crash / Update / LSP / PTY / Workspace) 색상 + 위치 정의
- §14 F — `crates/moai-studio-ui/src/banners/` 신규 모듈, P1 우선도

Round 2 시안 `moai-revisions.jsx` 라인 552~635 가 5 컴포넌트 prototype 보유 (icon + body + meta + actions 패턴 확인).

### 1.3 v0.1.0 release 와의 관계

본 SPEC 은 v0.1.0 release-blocker 가 **아니다** (Settings/Palette 와 달리 modern IDE 의 minimum bar 가 아닌 polish 영역). 그러나 다음 이유로 v0.1.0 직전/직후 정렬:

- moai-supervisor crash 는 사용자가 "왜 Agent 가 응답을 멈췄는지" 알 수 있는 유일한 채널
- LSP 실패는 V3-006 MS-3 이후 자주 발생할 수 있어 silent failure 차단 필요
- update banner 는 v0.1.0 → v0.1.1 hotfix 사이클의 사용자 인지 채널

---

## 2. 코드베이스 분석

### 2.1 기존 surface 모듈 패턴

moai-studio-ui 는 surface 별 모듈화 패턴이 확립되어 있다:

| Module | 역할 | 진입점 | mount 지점 |
|--------|------|-------|------------|
| `panes/` (V3-001) | TabContainer 분할 화면 | `PanesContainer` | RootView 메인 영역 |
| `tabs/` (V3-001) | Tab 관리 | `TabContainer` | PanesContainer 내 |
| `terminal/` | PTY terminal | `TerminalView` | Tab 내 |
| `viewer/` (V3-007) | Code/MD viewer | `ViewerView` | Tab 내 |
| `palette/` (V3-012) | 3 variant overlay | `PaletteOverlay` (`active_variant: Option<PaletteVariant>`) | RootView overlay |
| `settings/` (V3-013) | SettingsModal | `SettingsModal` | RootView overlay (Scrim) |
| `agent/` | Agent panel | `AgentView` | Tab 내 |
| `explorer/` | File explorer | `ExplorerView` | sidebar |

본 SPEC 의 `banners/` 는 `palette/` 의 mutual-exclusion 패턴이 아니라 **stacked overlay** 패턴 (최대 3개 동시 표시). 가장 가까운 reference 는 palette/ 의 PaletteOverlay 가 RootView 에 단일 필드로 존재하는 패턴.

### 2.2 design::tokens 활용

`crates/moai-studio-ui/src/design/tokens.rs` 가 본 SPEC 에 필요한 모든 색상 토큰을 이미 보유:

| Severity | Token | hex |
|----------|------|-----|
| critical | `semantic::DANGER` | #c44a3a |
| error | `semantic::DANGER` | #c44a3a (동일, 구분은 priority 만) |
| warn | `semantic::WARNING` | #c47b2a |
| info | `semantic::INFO` | #2a8a8c |
| success | `semantic::SUCCESS` | #1c7c70 (모두의AI 청록 계열) |
| brand action | `brand::PRIMARY` / `PRIMARY_DARK` | #144a46 / #22938a |

dimension 은 `design::layout::ide::TOPBAR_HEIGHT_PX (38.0)` 과 시각적으로 정렬 — banner height 32~40px 범위는 **신규 토큰 BANNER_HEIGHT_PX = 36.0** (양 surface 의 중간) 로 추가 권장.

`design::layout::spacing::S_2 (8.0)` / `S_3 (12.0)` 이 banner padding 으로 직접 사용 가능. radius 는 NONE (banner 는 full-width slim, corner 없음).

### 2.3 design::runtime::ActiveTheme 통합

V3-013 MS-3 가 도입한 `ActiveTheme` global (cx.global::<ActiveTheme>()) 이 dark/light 분기 + accent 처리. Banner 는 ActiveTheme 의 `theme` (Dark/Light) 만 참조 — severity color 는 dark/light 동일 hex 사용 가능 (semantic 토큰은 mode-agnostic).

ActiveTheme 변경 시 cx.notify() 가 RootView 전체 re-render 트리거 — banner 도 자동 따라옴 (별도 listener 불필요).

### 2.4 RootView 통합 지점

`crates/moai-studio-ui/src/lib.rs:82` 의 `pub struct RootView` 가 본 SPEC 의 `BannerStack` slot 을 추가받을 후보:

- 현재 RootView 필드: TabContainer / PaletteOverlay / SettingsModal (option) / etc.
- 신규 필드: `banner_stack: Entity<BannerStack>` (always present, 0~3 banners 보유)
- render 순서: banner stack 은 **TabContainer 위, palette overlay 아래** (palette mount 시 backdrop 이 banner 를 가리지만 banner 는 dismissable 이라 OK)

위치 권장: top-of-window, 메뉴바 아래 (현재 메뉴바 미구현이라면 RootView 최상단). 즉 `BannerStack` 은 vertical flex 로 banner 들을 stack 하고, 아래로 TabContainer 가 이어진다 → TabContainer 는 banner stack 높이만큼 push down.

### 2.5 moai-supervisor crash event

`crates/moai-supervisor/` 는 child process 관리. crash event 발행 패턴은 (현재 미확인이지만) 다음 중 하나:

- channel (tokio::sync::mpsc) → UI thread 에서 BannerStack 에 push
- callback / observer pattern
- 직접 Entity 참조 (gpui Context 통해 publish)

본 SPEC MS-3 는 **mock event source** 로 wire-up 하고 (BannerStack 의 public API `push(banner: Banner)` 만 노출), 실제 supervisor 통합은 별도 SPEC (V3-015 또는 v0.2.0+ hotfix) 으로 분리. 이로써 본 SPEC 은 banners/ 모듈 자체의 정합성에만 집중.

### 2.6 LSP failure (V3-006 MS-3a) 연계

V3-006 MS-3a 의 `LspProvider` mock 이 spawn 실패 시 `LspError` 이벤트 발생. `LspBanner` 는 이 이벤트를 mock subscribe → MS-3 에서는 mock 호출로 wire-up demo 만 검증.

---

## 3. UX 패턴 — Banner Severity & Stack Policy

### 3.1 Severity 5 단계와 priority

`Severity` enum 은 strict ordering 보유:

```
Critical (4) > Error (3) > Warning (2) > Info (1) > Success (0)
```

높은 priority 가 stack 에서 우선 표시. 동일 priority 내에서는 FIFO (먼저 push 된 것이 위).

### 3.2 Stack 정책 (최대 3개)

```
Stack capacity: 3
Push 시:
  - 빈 자리 있으면 priority 정렬 후 삽입
  - Full 인 경우:
    - 새 banner priority > stack 최저 priority → 최저 priority 의 가장 오래된 것 evict
    - 새 banner priority <= 최저 → 무시 (drop)
Dismiss 시:
  - 해당 banner 제거, 나머지 자동 위로 shift
Auto-dismiss 시:
  - 타이머 만료 시 자동 dismiss 동일
```

근거: 동시 3개 초과는 인지 부담 (UX 연구 — Miller's law 7±2 의 short-term cap 의 절반). VS Code 도 동시 3 banner cap.

### 3.3 Auto-dismiss 정책

| Severity | Auto-dismiss | 근거 |
|----------|-------------|------|
| Critical | manual only | 사용자 결정 필요 (예: Reopen) |
| Error | manual only | 동일 |
| Warning | manual only | 사용자 인지 보장 |
| Info | 8초 | 정보성, 충분히 읽을 시간 |
| Success | 5초 | 결과 확인만, 짧게 |

Auto-dismiss 는 BannerView 의 `mounted_at: Instant` + `auto_dismiss_after: Option<Duration>` 으로 구현. tick 은 GPUI 의 timer (cx.spawn 또는 tokio interval) — 정확한 timer 메커니즘은 MS-1 RED 단계에서 확정.

### 3.4 시각 구조 (layout)

```
┌─────────────────────────────────────────────────────────────────────┐
│ [icon] [body — strong + meta]              [action] [action] [×]    │  36px
└─────────────────────────────────────────────────────────────────────┘
```

- 좌측 padding S_3 (12px)
- icon (16x16) + S_2 (8px) gap
- body flex-grow (strong text + optional meta sub-line in muted)
- actions 우측 정렬, 각 button 사이 S_2 (8px), 마지막 dismiss × 버튼
- 우측 padding S_3 (12px)

### 3.5 5 Variant 의 default text/icon

| Variant | Icon | Strong text | Meta | Actions |
|---------|------|------------|------|---------|
| Crash | alert-triangle | "Agent crashed" | log path + last alive | "Reopen" (pri), "Dismiss" |
| Update | zap | "Update v{x.y.z} available" | size + changelog link | "Update" (pri), "Later" |
| LSP | zap | "{server} failed to start" | error reason | "Configure" (pri), "Dismiss" |
| PTY | terminal | "Terminal failed to spawn" | error code + cwd | "Restart Terminal" (pri), "Dismiss" |
| Workspace | folder-open | "Workspace state corrupted" | bak path | "Reset Workspace" (pri), "Continue" |

**.pri** = primary action button (brand.primary 색상), 그 외는 default neutral border button.

---

## 4. GPUI Render 전략

### 4.1 Banner trait 패턴

```
trait Banner {
    fn severity(&self) -> Severity;
    fn icon(&self) -> Icon;          // 또는 SVG path
    fn strong_text(&self) -> String;
    fn meta(&self) -> Option<String>;
    fn actions(&self) -> Vec<ActionButton>;
    fn auto_dismiss_after(&self) -> Option<Duration>;
    fn id(&self) -> BannerId;        // unique, 동일 source 의 중복 push 차단
}
```

각 variant 는 struct (`CrashBanner { log_path: PathBuf, ... }`) + Banner trait 구현. trait object (`Box<dyn Banner>`) 로 BannerStack 의 Vec 에 저장. (대안: enum Banner — 5 variant 고정인 경우 더 간단. 본 SPEC 권장은 enum 으로 시작, 6번째 variant 추가 시 trait 로 리팩토 — 더 적은 boilerplate).

**최종 결정**: enum BannerKind 으로 시작 (5 variant fixed). v0.2.0+ 에서 trait 로 마이그레이션 가능성 열어둠. MS-1 RED 단계에서 enum vs trait 최종 확정.

### 4.2 BannerView (개별 banner UI)

GPUI Entity. fields: kind (BannerKind), state (mounted_at, dismissed: bool), action handlers (Box<dyn Fn(&mut Context)>).

render: severity → bg color + icon color, body + actions row, dismiss × 버튼.

### 4.3 BannerStack (Entity)

fields:
- `banners: Vec<BannerEntity>` (max 3)
- `dedupe_keys: HashSet<BannerId>` — 동일 id 중복 push 차단

methods:
- `push(banner: BannerKind, cx: &mut Context)` → 정렬 + evict + 새 Entity 생성
- `dismiss(id: BannerId, cx)` → 제거 + cx.notify()
- `tick(cx)` — auto-dismiss 만료 검사 (timer 또는 frame pulse)

render: vertical Flex. banner 가 0개면 height 0 (TabContainer 가 최상단 차지).

### 4.4 RootView 통합

```
RootView {
    banner_stack: Entity<BannerStack>,
    tab_container: ...,
    palette_overlay: ...,
    settings_modal: Option<...>,
    ...
}

impl Render for RootView {
    fn render(...) -> ... {
        v_flex()
            .child(self.banner_stack.clone())   // top
            .child(self.tab_container.clone())  // main
            .child(self.palette_overlay)        // overlay (z-index 위)
            .child(self.settings_modal)         // overlay
    }
}
```

Banner stack 은 TabContainer 와 normal layout flow 공유 (overlay 가 아닌 push-down). Palette/Settings overlay 는 z-index 위로 banner 도 가릴 수 있음 — OK (overlay dismiss 시 banner 는 다시 보임).

### 4.5 Event source wiring (MS-3 mock)

MS-3 은 다음 mock 만 wire-up:

- `BannerStack::push_crash(log_path, last_alive, cx)` — 실제 supervisor 통합 없이 직접 호출하는 helper. 통합 테스트가 이 helper 호출 후 banner 가 표시되는지 검증.
- LSP/PTY/Update/Workspace 동일 패턴 (push_update, push_lsp_error, ...).
- Action 핸들러는 mock — 실제 dispatch (Reopen / Update / etc.) 는 별도 SPEC.

이 분리로 본 SPEC 은 **UI surface 자체의 완결성** 만 검증, 실제 시스템 통합은 v0.2.0+ 로 미룸.

---

## 5. 위험 요소 및 완화

### 5.1 GPUI timer 정확성

Auto-dismiss timer 는 GPUI 의 어떤 메커니즘을 쓰는가? cx.spawn(async) + tokio::time::sleep? 또는 frame-based polling?

완화: MS-1 RED 단계에서 GPUI 0.1 의 timer 패턴 조사 (palette/ 의 dismiss timer 가 있는지 확인). 없다면 cx.spawn + sleep 가 표준. tick interval 은 정확도 250ms 면 충분 (5초/8초 단위라 fine-grained 불필요).

### 5.2 동시 push race

여러 thread 에서 동시 push (e.g. supervisor crash + LSP failure 동시) — Vec 에 동시 mutation 위험.

완화: BannerStack 은 GPUI Entity (single-threaded UI context 보장). 모든 push 는 cx.update_entity() 통해 직렬화. multi-thread source 는 channel 로 UI thread 에 forward.

### 5.3 Severity priority 와 FIFO 의 충돌

3개 stack full 상태에서 동일 priority 추가 push 시 FIFO 로 가장 오래된 것 evict — 사용자가 막 push 된 것을 못 보고 사라지는 것은 UX 문제 아닌가?

완화: 동일 priority drop 정책 = "신규 무시" (오래된 것 보호). priority 상승 시에만 evict. 단위 테스트로 명시.

### 5.4 Action 핸들러의 mock vs real

본 SPEC 은 mock action 만. 사용자가 "Reopen" 클릭하면 아무 일도 안 일어남 — disabled 처럼 느낄 수 있음.

완화: Action 핸들러 시그니처는 `Box<dyn Fn(&mut Context)>` 로 두되, MS-3 의 mock 은 println! / log::info! 만 출력. 실제 wire-up 은 별도 SPEC. spec.md / acceptance.md 의 "비목표" 섹션에 명시.

### 5.5 Local 5 quality gates

cargo test / clippy 0 / fmt / bench / cargo check --release.

완화:
- timer 의 async 처리는 unit test 어렵 → State machine 만 테스트 (mounted_at + auto_dismiss_after → "should_dismiss(now: Instant) -> bool" 순수 함수). Real timer 은 통합 테스트만.
- bench 회귀: BannerStack 은 cold path (이벤트 발생 시에만 활성) — render/scroll bench 영향 없을 것으로 예상. MS-3 통합 후 bench smoke check.

### 5.6 Brand 정합성

[FROZEN] 모두의AI 청록 (#144a46/#22938a) 은 brand.primary action button 에만 사용. severity color 는 semantic 토큰 (DANGER/WARNING/INFO/SUCCESS) — brand 와 독립. Crash banner 는 DANGER 이지 brand 가 아님.

---

## 6. 단위 테스트 설계 (MS-1 / MS-2 / MS-3)

### 6.1 MS-1 (Banner trait + BannerView + BannerStack core)

- `severity_ordering`: Critical > Error > Warning > Info > Success (Ord impl 검증)
- `stack_push_under_capacity`: 0~2개 push → 모두 보유
- `stack_evict_on_priority_increase`: 3개 full 에 더 높은 priority push → 최저 priority evict
- `stack_drop_on_same_priority_full`: 3개 full 에 동일 priority push → drop
- `stack_dismiss_by_id`: 특정 id dismiss → 해당 항목만 제거
- `auto_dismiss_state`: should_dismiss(mounted_at, auto_dismiss_after, now) 순수 함수 검증
- `dedupe_same_id`: 동일 id push → 무시 (중복 차단)

### 6.2 MS-2 (5 variants)

각 variant 별:
- severity 검증 (Crash=Critical, Update=Info, LSP=Warning, PTY=Error, Workspace=Info or Warning)
- 기본 strong text / meta / actions 검증
- auto_dismiss_after Option 검증 (Crash/PTY/LSP=None, Update=8s, Workspace=manual or 8s, Success 가 없음 — 모든 variant 가 critical/error/warn/info)

### 6.3 MS-3 (RootView 통합 + mock wiring)

- `rootview_has_banner_stack`: RootView 가 BannerStack Entity 보유
- `push_crash_helper_displays_crash_banner`: helper 호출 → stack 에 CrashBanner 1개
- `push_update_helper_displays_update_banner`: 동일 패턴
- `push_lsp_helper_displays_lsp_banner`: 동일
- `push_pty_helper_displays_pty_banner`: 동일
- `push_workspace_helper_displays_workspace_banner`: 동일
- `multiple_banners_render_in_priority_order`: 3개 다른 severity push → 순서 검증
- `dismiss_action_invokes_handler`: 통합 테스트 — × 클릭 시 dismiss 핸들러 실행

---

## 7. 디렉토리 구조 (최종)

```
crates/moai-studio-ui/src/banners/
├── mod.rs                  // Banner trait/enum, Severity, ActionButton, BannerId, exports
├── banner_view.rs          // BannerView Entity (개별 banner UI)
├── banner_stack.rs         // BannerStack Entity (max 3, priority queue, dedup)
└── variants/
    ├── mod.rs              // re-export 5 variants
    ├── crash.rs            // CrashBanner
    ├── update.rs           // UpdateBanner
    ├── lsp.rs              // LspBanner
    ├── pty.rs              // PtyBanner
    └── workspace.rs        // WorkspaceBanner
```

총 9 파일 (mod.rs 2개 + 7 구현). 각 파일 200~400 LOC 예상 (mod.rs/variants/mod.rs 는 50 LOC 미만).

`design::tokens` 신규 module 추가 (선택, 별도 PR 가능):
```
design::tokens::banner {
    BANNER_HEIGHT_PX: f32 = 36.0
    BANNER_PADDING_X_PX: f32 = 12.0   // = spacing::S_3
    BANNER_ICON_SIZE_PX: f32 = 16.0
    BANNER_ACTION_GAP_PX: f32 = 8.0   // = spacing::S_2
}
```

---

## 8. 의존 SPEC 및 상태

| SPEC | 상태 | 본 SPEC 에서 사용 |
|------|------|----------------|
| SPEC-V3-001 | implemented | RootView, TabContainer, design module |
| SPEC-V3-004 | implemented | RootView keymap (action 핸들러 dispatch 인프라) |
| SPEC-V3-006 MS-3a | implemented | LspProvider mock — LspBanner 의 source |
| SPEC-V3-012 | implemented | palette/ 의 overlay mount 패턴 reference |
| SPEC-V3-013 | implemented | design::runtime::ActiveTheme 통합 — banner 가 ActiveTheme 으로 dark/light 분기 |

본 SPEC 은 V3-013 까지의 인프라를 사용하며, V3-013 이후 (현재 develop 브랜치 5649385 시점) 에 작업 가능.

---

Version: 1.0.0
Last Updated: 2026-04-26
Author: MoAI (manager-spec)
