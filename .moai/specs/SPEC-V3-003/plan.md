---
spec_id: SPEC-V3-003
title: Tab/Pane Split — Implementation Plan
version: 1.0.0
created_at: 2026-04-24
updated_at: 2026-04-24
parent_spec: SPEC-V3-003
parent_version: 1.0.0
language: Korean
author: MoAI (manager-spec, v1.0.0 plan phase 산출물)
development_mode: auto_detect (quality.yaml 기준 — 본 repository 는 TDD 로 설정됨)
---

# SPEC-V3-003 Implementation Plan

## 1. 개요

SPEC-V3-003 spec.md v1.0.0 의 **37 REQ-P + 29 AC** 를 3 milestone 구조 (MS-1 Pane core → MS-2 Tabs → MS-3 Persistence) 로 달성하기 위한 작업 분해 / 기술 스택 / 위험 / MX tag 계획. 시간 추정은 사용하지 않고 **우선순위 (High / Medium / Low) + 의존성 그래프** 로 표현한다.

본 plan 은 `.moai/config/sections/quality.yaml` 의 `development_mode` (본 repository 는 TDD) 와 무관하게 공통 적용되는 task decomposition 이다. Run phase 에서 manager-ddd 또는 manager-tdd 가 동일 task 목록을 방법론별 cycle 로 풀어낸다 (@workflow-modes.md).

파일 레이아웃은 **spec.md §9 (canonical)** 을 단일 진실원으로 따른다. 본 plan 의 각 task 대상 파일은 spec.md §9 의 트리와 **line-for-line 일치** 해야 한다.

### 1.1 Task / Spike 요약

- **Task 총수**: 14 (MS-1: 7, MS-2: 5, MS-3: 2)
- **Plan Spike 총수**: 4 (S1 GPUI divider drag API, S2 gpui-component Resizable, S3 ID 생성 방식, S4 Linux shell 관례 실제 UX)
- **우선순위 분포**: High 9, Medium 4, Low 1

### 1.2 v1.0.0 annotation cycle 잔여 4건 해소 상태

| 항목 | 해소 위치 | 검증 |
|------|-----------|------|
| NM-1 REQ-P-057 → REQ-P-056 rename | spec.md §5 RG-P-6 / §8 MS-3 / §10 AC-P-13a | 본 plan T12 의 구현 대상 필드명으로 반영 |
| Nm-1 AC-P-26 신규 (REQ-P-034) | spec.md §10 AC-P-26 + acceptance.md §3 AC-P-26 | 본 plan T9 에서 integration_tmux_nested.rs 테스트 작성 |
| Nm-2 AC-P-27 신규 (REQ-P-044) | spec.md §10 AC-P-27 + acceptance.md §3 AC-P-27 | 본 plan T10 에서 탭 바 design token + bold 구현 |
| Nm-3 Linux shell 관례 (§6.4 / R-9) | spec.md §6.4 + §12 R-9 | 본 plan Spike 4 로 실행 시점 연기 |

---

## 2. 기술 스택 (확정)

| 영역 | 선택 | 근거 | Spike 의존 |
|------|------|------|-------------|
| PaneTree 자료구조 | `enum PaneTree` + `Box<PaneTree>` 재귀 | spec.md §7.1 | 없음 |
| Pane leaf content | `Entity<TerminalSurface>` (SPEC-V3-002 재사용) | spec.md §2, REQ-P-060 | 없음 |
| Split renderer (PaneSplitter) | **미정** — 추상 trait 로 선언, plan Spike 1 + 2 결과로 결정 | spec.md §7.2, §11.1 | **S1, S2** |
| Divider drag (ResizableDivider) | **미정** — 추상 trait 로 선언, plan Spike 1 결과로 결정 | spec.md §7.3 | **S1** |
| Tab bar UI | GPUI 0.2.2 native element (design token 기반) | SPEC-V3-001 상속 | 없음 |
| Design token sourcing | `.moai/design/v3/system.md` Toolbar 섹션 + plan 단계에서 `toolbar.tab.active.background` 추가 | spec.md REQ-P-044, §16 #3 | 없음 (plan 단계 직접 확정) |
| Persistence serialization | `serde` + `serde_json` + `std::fs::rename` (atomic write) | spec.md §7.5, REQ-P-052 | 없음 |
| ID 생성 (PaneId, TabId) | **미정** — `uuid` crate vs 기존 workspace ID 패턴 확장 | spec.md §11.1, §16 #4 | **S3** |
| 키 바인딩 dispatch | GPUI `KeyEvent` hook + `platform_mod` 플랫폼 분기 매크로 | spec.md RG-P-4 | 없음 |
| Linux modifier 관례 | Ctrl 기반 고정 (design 원천 유지) + Spike 4 결과에 따른 조정 여지 | spec.md §6.4, R-9 | **S4** |
| 테스트 하네스 | `cargo test` + GPUI headless + criterion bench | acceptance.md §1 | 없음 |
| CI regression gate | `.github/workflows/ci-v3-pane.yml` | acceptance.md §7 | 없음 |

---

## 3. Plan Spike 4건

본 SPEC 의 모든 AC 는 추상 trait 기반이므로, spike 결과와 무관하게 AC 재작성이 필요 없다. Spike 는 구체 구현체 선택을 위한 것이며, **Run phase 초기** 에 실행된다 (본 plan 위임 범위에서는 직접 수행하지 않음).

### Spike 1: GPUI 0.2.2 divider drag API 존재 여부

- **우선순위**: High (S2 결정의 전제)
- **목표**: GPUI 0.2.2 의 mouse event 체인 (`on_mouse_down` → `on_mouse_move` → `on_mouse_up`) 으로 divider 좌표를 추적하여 flex basis 또는 수동 레이아웃 좌표를 갱신하는 것이 가능한지 검증
- **대상 파일**: 신규 `crates/moai-studio-ui/examples/divider-spike.rs` (삭제 가능한 임시 파일)
- **성공 기준**:
  - 200 줄 이하 구현으로 2-pane 수평 split 에서 divider drag → ratio 갱신 → frame 재갱신 체인 동작
  - 드래그 중 frame rate ≥ 60 fps 유지 (spec.md §6.1)
- **산출**:
  - 양/음 판정 + 샘플 코드 스니펫
  - 플랫폼별 (macOS + Linux) 동작 차이 기록
  - plan.md §2 의 "PaneSplitter / ResizableDivider" 선택 경로 업데이트
- **실패 시 경로**: S2 (gpui-component) 평가로 전환

### Spike 2: longbridge/gpui-component Resizable / Dock 안정성

- **우선순위**: High (S1 결과에 따라 조건부 실행)
- **목표**: 외부 crate `longbridge/gpui-component` 의 `Resizable` + `Dock` 컴포넌트가 GPUI 0.2.2 와 호환되며 API churn 이 허용 가능한 수준인지 검증
- **대상 파일**:
  - `crates/moai-studio-ui/Cargo.toml` 에 일시적 의존성 추가 후 PR 머지 전 revert 가능한 branch
  - 신규 `crates/moai-studio-ui/examples/gpui-component-spike.rs`
- **성공 기준**:
  - `cargo build -p moai-studio-ui` 가 의존성 추가 후 빨간 줄 없이 통과
  - `Resizable::new([pane_a, pane_b])` 로 2 pane split 예제가 60 fps 유지
  - Git SHA 또는 버전 pinning 명확 (upstream 안정성 평가)
- **산출**:
  - import 가능성 + 샘플 코드 + commit hash pin 제안 + breaking change 감지 리포트
  - plan.md §2 / spec.md §11.1 C-1 확정
- **실패 시 경로**: S1 결과가 "직접 구현 가능" 이었다면 직접 구현으로 확정; 모두 실패 시 plan 재조정 (user escalation)

### Spike 3: PaneId / TabId 생성 방식 확정

- **우선순위**: Medium
- **목표**: `uuid` crate 추가 vs 기존 workspace ID 생성 패턴 (`format!("ws-{:x}", nanos)` 같은 수동 포맷) 중 선택
- **대상 파일**:
  - `crates/moai-studio-workspace/src/lib.rs:89-91` (기존 `"moai-studio/workspace-v1"` schema + ID 패턴 확인)
  - 신규 spike 없음 — design 결정 only
- **성공 기준**:
  - 두 경로의 trade-off 표 작성 (빌드 의존성 / 충돌 가능성 / 가독성 / serialize 크기)
  - 기존 workspace ID 생성 로직과의 consistency 확보
- **산출**:
  - 선택 + REQ-P-001 / REQ-P-040 구현 지침 (Rust type alias + constructor 샘플)
  - plan.md §2 "ID 생성" 행 확정

### Spike 4: Linux Ctrl+D / Ctrl+W / Ctrl+\\ shell 관례 실제 UX 검증 (v1.0.0 Nm-3 해소)

- **우선순위**: Medium (Linux 빌드 릴리즈 전 필수)
- **목표**: Linux 에서 host 가 Ctrl+D / Ctrl+W / Ctrl+\\ 를 전역 capture 할 때, pane 내부 shell 의 EOF / word-delete / SIGQUIT 기능이 실제 어떻게 막히는지 정량 측정
- **대상 파일**:
  - 신규 `docs/spikes/SPIKE-V3-003-04-linux-shell-conventions.md` (measurement report)
  - 수동 측정 — 실제 Ubuntu 22.04 runner 또는 개발자 머신에서 bash + zsh + fish 각 세션
- **측정 항목**:
  - (a) Ctrl+D 로 `exit` 대체 불가 → 사용자가 `exit\n` 타이핑으로 우회 가능한지
  - (b) Ctrl+W 의 `unix-word-rubout` 손실 → 실제 CLI 작업 빈도 측정 (개발자 interview or local usage log 1일)
  - (c) Ctrl+\\ 의 SIGQUIT → 실무에서 거의 사용 안 됨 확인 or 반증
- **결정 경로**:
  - **(a) 현행 유지 경로 (design 원천 유지)**: host 바인딩 기본 활성. 사용자 설정 파일 (향후 Shortcut Customization SPEC) 에서 개별 비활성화 제공. spec.md RG-P-4 Linux 컬럼 + AC-P-9b **무변경**.
  - **(b) Shift-escalation 경로**: Linux 에서 Ctrl+W → Ctrl+Shift+W, Ctrl+D → Ctrl+Shift+D, Ctrl+\\ → Ctrl+Shift+\\ 로 shift. 이 경우:
    - spec.md §5 RG-P-4 표의 Linux 컬럼 갱신 (annotation cycle 재개 필요)
    - acceptance.md AC-P-9b 의 sequence 갱신
    - plan 단계에서 이 경로 선택 시 **annotation cycle 재가동 필요** (user 재승인)
- **성공 기준**: 경로 (a) 또는 (b) 중 하나 확정 + R-8/R-9 갱신
- **산출**:
  - spike 보고서 + 결정 + 관련 문서 diff proposal
  - plan.md §2 "Linux modifier 관례" 행 확정

---

## 4. Task 분해

각 task format:

- **Priority**: High / Medium / Low
- **선행 의존성**: 이전 task ID
- **대상 파일**: spec.md §9 canonical layout 과 line-for-line 일치
- **예상 산출**: 구체적 Rust 타입 / 함수 시그니처
- **검증 AC**: acceptance.md 의 AC-P-XX 목록
- **MX 태그 계획**: ANCHOR / WARN / TODO

### 4.1 MS-1 Tasks (T1 ~ T7)

---

#### T1: PaneTree 자료구조 + unit test

- **Priority**: High (MS-1 의 foundation, 모든 후속 task blocker)
- **선행 의존성**: 없음
- **대상 파일**:
  - `crates/moai-studio-ui/src/panes/mod.rs` (신규, pub 노출)
  - `crates/moai-studio-ui/src/panes/tree.rs` (신규, PaneTree enum + in-order iterator + split/close 알고리즘)
  - `crates/moai-studio-ui/src/lib.rs` (수정, `pub mod panes;` 추가)

- **예상 산출**:
  ```rust
  // crates/moai-studio-ui/src/panes/tree.rs
  pub enum PaneTree {
      Leaf(Entity<TerminalSurface>),
      Split {
          direction: SplitDirection,
          ratio: f32,
          first: Box<PaneTree>,
          second: Box<PaneTree>,
      },
  }

  pub enum SplitDirection {
      Horizontal,  // 좌/우 배치, 수직 divider
      Vertical,    // 상/하 배치, 수평 divider
  }

  pub struct PaneId(String);  // Spike 3 결정 따름

  impl PaneTree {
      pub fn split_horizontal(&mut self, target: PaneId) -> Result<PaneId, SplitError>;
      pub fn split_vertical(&mut self, target: PaneId) -> Result<PaneId, SplitError>;
      pub fn close_pane(&mut self, target: PaneId) -> Result<(), CloseError>;
      pub fn leaves(&self) -> Vec<&PaneTree>;
      pub fn leaf_count(&self) -> usize;
      pub fn root_pane_id(&self) -> PaneId;
      pub fn get_ratio(&self, split_id: SplitNodeId) -> Option<f32>;
      pub fn set_ratio(&mut self, split_id: SplitNodeId, r: f32) -> Result<(), RatioError>;
  }
  ```
  - `#[cfg(test)] mod tests` 에 10+ unit tests:
    - `split_horizontal_from_leaf` (AC-P-1)
    - `close_promotes_sibling` (AC-P-2 의 일부)
    - `close_last_leaf_is_noop` (AC-P-3)
    - `ratio_boundary_rejected` (AC-P-20)
    - `leaves_in_order_iteration`
    - `split_direction_first_second_semantics` (spec.md §7.1 의 `Horizontal` ↔ first/왼쪽 검증)

- **검증 AC**: AC-P-1, AC-P-2 (unit 부분), AC-P-3, AC-P-20, AC-P-22 (간접)

- **MX 태그 계획**:
  - `pub enum PaneTree`: `@MX:ANCHOR(pane-tree-invariant)` — 자료구조 불변 조건 (경계 ratio 제외, leaf/split 교차 불가 등). fan_in 예상 ≥ 4 (splitter, close, render, persistence).
  - `pub fn split_horizontal`: `@MX:ANCHOR(pane-split-api)` — 외부 호출 진입점.
  - `SplitDirection` doc comment: `@MX:NOTE(horizontal-is-left-right-not-top-bottom)` — C-3 의 용어 혼동 방지.

---

#### T2: PaneConstraints associated const + public API surface test

- **Priority**: High
- **선행 의존성**: T1
- **대상 파일**:
  - `crates/moai-studio-ui/src/panes/constraints.rs` (신규)
  - `crates/moai-studio-ui/src/panes/mod.rs` (수정, `pub use constraints::PaneConstraints`)
  - `crates/moai-studio-ui/tests/integration_pane_core.rs` (신규 일부 — AC-P-21 negative API surface test)

- **예상 산출**:
  ```rust
  // crates/moai-studio-ui/src/panes/constraints.rs
  pub struct PaneConstraints;

  impl PaneConstraints {
      pub const MIN_COLS: u16 = 40;
      pub const MIN_ROWS: u16 = 10;
  }
  ```
  - 가변 API (`new`, `set_min_cols`, `set_min_rows`) 는 **부재**
  - `tests/integration_pane_core.rs::pane_constraints_has_no_mutable_api` 가 `cargo public-api --simplified` 출력을 grep 으로 검증

- **검증 AC**: AC-P-4 (간접, MIN_COLS 참조), AC-P-21 (직접, negative API surface)

- **MX 태그 계획**:
  - `impl PaneConstraints`: `@MX:ANCHOR(pane-constraints-immutable)` — spec.md REQ-P-014 의 불변성 강제.

---

#### T3: PaneSplitter / ResizableDivider 추상 trait + mock impl

- **Priority**: High
- **선행 의존성**: T1, T2
- **대상 파일**:
  - `crates/moai-studio-ui/src/panes/splitter.rs` (신규, PaneSplitter trait + Mock 구현)
  - `crates/moai-studio-ui/src/panes/divider.rs` (신규, ResizableDivider trait + Mock 구현)

- **예상 산출**:
  ```rust
  // crates/moai-studio-ui/src/panes/splitter.rs
  pub trait PaneSplitter {
      fn split_horizontal(&mut self, target: PaneId) -> Result<PaneId, SplitError>;
      fn split_vertical(&mut self, target: PaneId) -> Result<PaneId, SplitError>;
      fn close_pane(&mut self, target: PaneId) -> Result<(), CloseError>;
      fn focus_pane(&mut self, target: PaneId);
  }

  #[cfg(test)]
  pub struct MockPaneSplitter { /* in-memory tree */ }
  #[cfg(test)]
  impl PaneSplitter for MockPaneSplitter { /* delegate to PaneTree */ }

  // crates/moai-studio-ui/src/panes/divider.rs
  pub trait ResizableDivider {
      fn on_drag(&mut self, delta_px: f32, total_px: f32) -> f32;
      fn min_ratio_for(&self, sibling_px: f32) -> f32;
  }
  ```
  - `cargo check -p moai-studio-ui --lib` 가 구체 구현체 선택 없이 통과 (spec.md REQ-P-063)
  - doc test: `///` 예제 코드에 MockPaneSplitter 사용 샘플

- **검증 AC**: AC-P-17

- **MX 태그 계획**:
  - `pub trait PaneSplitter`: `@MX:ANCHOR(pane-splitter-contract)` — plan spike 결정 후 구체 구현체가 바뀌어도 contract 는 유지.
  - `pub trait ResizableDivider`: `@MX:ANCHOR(divider-contract)`.
  - `MockPaneSplitter`: `@MX:NOTE(test-only-impl)` + `#[cfg(test)]`.

---

#### T4: PaneSplitter 구체 구현 (Spike 1 + 2 결과)

- **Priority**: High
- **선행 의존성**: T3, **Spike 1**, (조건부) **Spike 2**
- **대상 파일**:
  - `crates/moai-studio-ui/src/panes/splitter_gpui_native.rs` **(spike 결과가 GPUI native 경로인 경우)** — 신규
  - 또는 `crates/moai-studio-ui/src/panes/splitter_gpui_component.rs` **(spike 결과가 gpui-component 경로인 경우)** — 신규
  - `crates/moai-studio-ui/Cargo.toml` (gpui-component 경로 시 의존성 추가)

- **예상 산출**:
  - Spike 결과 기반 실체 구현체 1개
  - GPUI `Element` 및 mouse event chain 연결
  - `TerminalSurface` entity 를 leaf 로 렌더
  - split 수행 시 새 `TerminalSurface::new` + `PtyWorker::spawn` 호출

- **검증 AC**: AC-P-1 (통합 경로), AC-P-5, AC-P-6, AC-P-18

- **MX 태그 계획**:
  - `impl PaneSplitter for GpuiNativeSplitter`: `@MX:NOTE(concrete-splitter-gpui-native)` — spike 결과 반영.
  - 외부 crate 호출 지점 (`gpui_component::Resizable::new(..)` 등): `@MX:WARN(external-dep-api-churn)` with `@MX:REASON(upstream-alpha)`.

---

#### T5: ResizableDivider 구체 구현 + drag clamping

- **Priority**: High
- **선행 의존성**: T3, T4, **Spike 1**
- **대상 파일**:
  - `crates/moai-studio-ui/src/panes/divider_impl.rs` (신규)

- **예상 산출**:
  ```rust
  pub struct GpuiDivider {
      orientation: DividerOrientation,
      total_px: f32,
      sibling_constraints: (f32, f32),  // min cols/rows in px
  }

  impl ResizableDivider for GpuiDivider {
      fn on_drag(&mut self, delta_px: f32, total_px: f32) -> f32 {
          let raw_ratio = (self.current_px() + delta_px) / total_px;
          raw_ratio.clamp(self.min_ratio, self.max_ratio)
      }
      fn min_ratio_for(&self, sibling_px: f32) -> f32 { ... }
  }
  ```
  - `PaneConstraints::MIN_COLS` / `MIN_ROWS` 를 직접 참조 (spec.md §7.3)

- **검증 AC**: AC-P-6

- **MX 태그 계획**:
  - `impl ResizableDivider for GpuiDivider::on_drag`: `@MX:NOTE(ratio-clamp-enforces-min-size)` — spec.md REQ-P-012 의 clamp 의미 문서화.

---

#### T6: Focus routing + 키 바인딩 배선 (MS-1 부분)

- **Priority**: High
- **선행 의존성**: T4
- **대상 파일**:
  - `crates/moai-studio-ui/src/panes/focus.rs` (신규)
  - `crates/moai-studio-ui/src/panes/key_bindings.rs` (신규, RG-P-4 MS-1 조합 6건)
  - `crates/moai-studio-ui/tests/integration_key_bindings.rs` (신규 일부 — MS-1 해당 부분만)

- **예상 산출**:
  ```rust
  pub struct FocusRouter {
      current: PaneId,
      tree: Entity<PaneTree>,
  }

  impl FocusRouter {
      pub fn next_in_order(&mut self) -> PaneId;
      pub fn prev_in_order(&mut self) -> PaneId;
      pub fn set_focus_by_click(&mut self, pane: PaneId);
      pub fn is_single_focused(&self) -> bool;  // AC-P-22 invariant
  }
  ```
  - `platform_mod!` 매크로로 Cmd ↔ Ctrl 분기
  - 처리된 keystroke 는 `TerminalSurface::handle_key_down` 에 전달되지 않음 (REQ-P-032 negative)

- **검증 AC**: AC-P-7, AC-P-9a (MS-1 부분), AC-P-9b (MS-1 부분), AC-P-22, AC-P-23 (Ctrl+B passthrough)

- **MX 태그 계획**:
  - `FocusRouter::next_in_order`: `@MX:ANCHOR(focus-routing)` — fan_in ≥ 3 (키 바인딩, mouse click, 탭 전환).
  - `platform_mod!` 매크로: `@MX:NOTE(cmd-ctrl-platform-dispatch)`.
  - `TerminalSurface::handle_key_down` 호출 site: `@MX:WARN(host-keystroke-intercept)` with `@MX:REASON(prevent-pty-echo)`.

---

#### T7: RootView 통합 + content_area 분기 재설계

- **Priority**: High
- **선행 의존성**: T4, T5, T6
- **대상 파일**:
  - `crates/moai-studio-ui/src/lib.rs:75` 수정 — `terminal: Option<Entity<TerminalSurface>>` → `tab_container: Option<Entity<TabContainer>>` (MS-1 에서는 `TabContainer` 가 단일 탭으로 초기화됨; MS-2 에서 다중 탭 로직 확장)
  - `crates/moai-studio-ui/src/lib.rs:184` 수정 — `main_body` 시그니처 확장
  - `crates/moai-studio-ui/src/lib.rs:290-299` 수정 — 파라미터 변경
  - `crates/moai-studio-ui/src/lib.rs:410-444` 수정 — `content_area` 분기 (tab_container.is_some + tabs.is_empty)

- **예상 산출**:
  - RootView 가 TabContainer 를 렌더 (MS-1 단계에서는 강제로 탭 1개 + 단일 leaf)
  - Empty State CTA 는 `TabContainer.tabs.is_empty()` 일 때만 표시
  - SPEC-V3-002 의 TerminalSurface 는 pane leaf 의 내용물로 그대로 재사용 (REQ-P-060 보장)

- **검증 AC**: AC-P-16 (Terminal Core regression 확인), AC-P-24 (Empty State 렌더)

- **MX 태그 계획**:
  - RootView 의 `tab_container` 필드: `@MX:ANCHOR(root-view-content-binding)` — 단일 SurfaceView → 다중 탭 컨테이너 전환 지점.
  - 기존 SPEC-V3-002 의 `TerminalSurface` 는 **수정 금지** (REQ-P-060).

---

### 4.2 MS-2 Tasks (T8 ~ T11)

---

#### T8: TabContainer 자료구조 + 전환 로직

- **Priority**: High
- **선행 의존성**: T7
- **대상 파일**:
  - `crates/moai-studio-ui/src/tabs/mod.rs` (신규, pub 노출)
  - `crates/moai-studio-ui/src/tabs/container.rs` (신규, TabContainer + Tab struct)

- **예상 산출**:
  ```rust
  pub struct TabContainer {
      pub tabs: Vec<Tab>,
      pub active_tab_idx: usize,
  }

  pub struct Tab {
      pub id: TabId,
      pub title: String,  // 초기값: cwd.file_name() 또는 "untitled"
      pub pane_tree: Entity<PaneTree>,
      pub last_focused_pane: Option<PaneId>,
  }

  impl TabContainer {
      pub fn new_tab(&mut self, cwd: Option<PathBuf>) -> TabId;
      pub fn close_tab(&mut self, tab: TabId) -> Result<(), CloseTabError>;
      pub fn switch_tab(&mut self, idx: usize) -> Result<(), SwitchError>;
      pub fn active_tab(&self) -> &Tab;
      pub fn active_tab_mut(&mut self) -> &mut Tab;
  }
  ```
  - `switch_tab` 은 REQ-P-023 의 `last_focused_pane` 복원 로직 포함
  - `new_tab` 은 단일 leaf 로 초기화 (REQ-P-042)

- **검증 AC**: AC-P-8, AC-P-10, AC-P-11, AC-P-24, AC-P-25

- **MX 태그 계획**:
  - `TabContainer::switch_tab`: `@MX:ANCHOR(tab-switch-invariant)` — last_focused_pane 복원 불변 조건.
  - `TabContainer::new_tab`: `@MX:ANCHOR(tab-create-api)`.

---

#### T9: MS-2 키 바인딩 + 탭 중첩 tmux 테스트 (AC-P-26)

- **Priority**: High
- **선행 의존성**: T8
- **대상 파일**:
  - `crates/moai-studio-ui/src/panes/key_bindings.rs` 확장 (MS-2 의 Cmd/Ctrl+T / Cmd/Ctrl+Shift+W / Cmd/Ctrl+1~9 / Cmd/Ctrl+\{/\})
  - `crates/moai-studio-ui/tests/integration_tabs.rs` (신규, AC-P-10 / AC-P-11 / AC-P-24 / AC-P-25)
  - `crates/moai-studio-ui/tests/integration_tmux_nested.rs` (신규, **AC-P-26 — v1.0.0 Nm-1 해소**)
  - `crates/moai-studio-ui/tests/integration_key_bindings.rs` 확장 (AC-P-9a / AC-P-9b 전체 커버)

- **예상 산출**:
  - 키 바인딩 dispatcher 에 TabContainer 조작 추가
  - `integration_tmux_nested.rs` 에서 PTY master 의 write stream capture 하여 host keystroke byte 부재 검증
  - 테스트 실행 전제: 실제 tmux 바이너리가 runner 에 설치되어야 함 (macOS: `brew install tmux`, Linux: `apt install tmux`)

- **검증 AC**: AC-P-9a (전체), AC-P-9b (전체), AC-P-10, AC-P-11, AC-P-25, **AC-P-26** (v1.0.0 Nm-1)

- **MX 태그 계획**:
  - 탭 키 바인딩 dispatcher: `@MX:ANCHOR(tab-key-dispatch)`.
  - `integration_tmux_nested.rs`: `@MX:NOTE(tmux-nested-os-priority-test)` — REQ-P-034 의 관측 가능한 검증.

---

#### T10: 탭 바 UI + design token `toolbar.tab.active.background` (AC-P-27)

- **Priority**: High
- **선행 의존성**: T8
- **대상 파일**:
  - `crates/moai-studio-ui/src/tabs/bar.rs` (신규, 탭 바 GPUI element)
  - `.moai/design/v3/system.md` 수정 — Toolbar 섹션에 `toolbar.tab.active.background` 토큰 추가 (spec.md §16 #3)
  - (조건부) `.moai/design/v3/tokens.json` 또는 동등 파일 — 실제 RGB 값 (plan 단계에서 확정, 기존 design 원천을 따른다)
  - `crates/moai-studio-ui/src/tabs/bar.rs` 내 `#[cfg(test)] mod tests::active_tab_has_bold_and_background_token` (**AC-P-27 — v1.0.0 Nm-2 해소**)

- **예상 산출**:
  ```rust
  pub struct TabBar<'a> {
      tabs: &'a [Tab],
      active_idx: usize,
  }

  impl Render for TabBar<'_> {
      fn render(&mut self, cx: &mut RenderContext) -> impl IntoElement {
          div().flex_row().children(
              self.tabs.iter().enumerate().map(|(idx, tab)| {
                  let is_active = idx == self.active_idx;
                  div()
                      .child(tab.title.clone())
                      .when(is_active, |el| el
                          .bg(design_token("toolbar.tab.active.background"))
                          .font_weight(FontWeight::Bold))
              })
          )
      }
  }
  ```
  - design token lookup 함수 (`design_token(path: &str) -> Hsla`) 는 기존 SPEC-V3-001 의 token 시스템 재사용 또는 helper 신규

- **검증 AC**: **AC-P-27** (v1.0.0 Nm-2), AC-P-24 (탭 바 렌더 경로)

- **MX 태그 계획**:
  - `TabBar::render` 의 `when(is_active)` 블록: `@MX:ANCHOR(active-tab-styling)` — REQ-P-044 의 "(a) AND (b) 동시 충족" 불변 조건.
  - design token 추가: `.moai/design/v3/system.md` diff 에 주석 `<!-- SPEC-V3-003 REQ-P-044 추가 -->`.

---

#### T11: 탭 성능 bench (AC-P-19)

- **Priority**: Medium
- **선행 의존성**: T8, T9
- **대상 파일**:
  - `crates/moai-studio-ui/benches/tab_switch.rs` (신규, criterion)
  - `crates/moai-studio-ui/Cargo.toml` (`[[bench]]` entry)

- **예상 산출**:
  ```rust
  use criterion::{black_box, criterion_group, criterion_main, Criterion};

  fn bench_tab_switch(c: &mut Criterion) {
      let mut harness = PaneTestHarness::new_9_tabs_mixed_panes();
      c.bench_function("tab_switch_1_to_9_50_cycles", |b| {
          b.iter(|| {
              for _ in 0..50 {
                  harness.switch_tab(black_box(0));
                  harness.switch_tab(black_box(8));
              }
          });
      });
  }

  criterion_group!(benches, bench_tab_switch);
  criterion_main!(benches);
  ```
  - 평균 탭 전환 visible frame ≤ 50 ms (spec.md §6.1)

- **검증 AC**: AC-P-19

- **MX 태그 계획**:
  - bench 파일: `@MX:NOTE(tab-switch-performance-guard)`.

---

### 4.3 MS-3 Tasks (T12 ~ T14)

---

#### T12: Persistence schema + atomic write + cwd fallback (REQ-P-056)

- **Priority**: High
- **선행 의존성**: T8 (MS-2 완료)
- **대상 파일**:
  - `crates/moai-studio-workspace/src/persistence.rs` (신규 또는 확장 — SPEC-V3-001 의 기존 workspace persistence 와 분리)
  - `crates/moai-studio-workspace/src/panes_schema.rs` (신규 — `"moai-studio/panes-v1"` schema 타입)
  - `crates/moai-studio-ui/tests/integration_persistence.rs` (신규 — AC-P-12, AC-P-13, AC-P-13a, AC-P-14, AC-P-15)

- **예상 산출**:
  ```rust
  // crates/moai-studio-workspace/src/panes_schema.rs
  #[derive(Serialize, Deserialize)]
  pub struct PanesFile {
      #[serde(rename = "$schema")]
      pub schema: String,  // "moai-studio/panes-v1"
      pub workspace_id: String,
      pub active_tab_idx: usize,
      pub tabs: Vec<SerializedTab>,
  }

  #[derive(Serialize, Deserialize)]
  pub struct SerializedTab {
      pub id: String,
      pub title: String,
      pub last_focused_pane: Option<String>,
      pub pane_tree: SerializedPaneTree,
  }

  #[derive(Serialize, Deserialize)]
  #[serde(tag = "type", content = "data")]
  pub enum SerializedPaneTree {
      Leaf { pane_id: String, cwd: PathBuf },
      Split {
          direction: SerializedDirection,
          ratio: f32,
          first: Box<SerializedPaneTree>,
          second: Box<SerializedPaneTree>,
      },
  }
  ```

  ```rust
  // crates/moai-studio-workspace/src/persistence.rs
  pub fn save_panes(ws_id: &str, container: &TabContainer) -> io::Result<()> {
      // atomic write: tempfile → fsync → rename
      let dir = home_dir().join(".moai/studio");
      fs::create_dir_all(&dir)?;
      let final_path = dir.join(format!("panes-{ws_id}.json"));
      let tmp_path = final_path.with_extension("json.tmp");

      let file = PanesFile::from(container);
      let json = serde_json::to_string_pretty(&file)?;
      fs::write(&tmp_path, json)?;
      fs::rename(&tmp_path, &final_path)?;  // atomic
      Ok(())
  }

  pub fn restore_panes(ws_id: &str) -> RestoreOutcome {
      let path = home_dir().join(format!(".moai/studio/panes-{ws_id}.json"));
      match fs::read_to_string(&path) {
          Err(e) if e.kind() == io::ErrorKind::NotFound => RestoreOutcome::Default,
          Err(e) => {
              tracing::warn!("panes file read failed: {}", e);
              RestoreOutcome::Default
          }
          Ok(s) => match serde_json::from_str::<PanesFile>(&s) {
              Ok(f) if f.schema != "moai-studio/panes-v1" => {
                  tracing::warn!("panes schema version mismatch: {}", f.schema);
                  RestoreOutcome::Default
              }
              Ok(f) => {
                  let container = apply_cwd_fallback(f);  // REQ-P-056
                  RestoreOutcome::Restored(container)
              }
              Err(e) => {
                  tracing::warn!("panes file parse failed: {}", e);
                  fs::rename(&path, path.with_extension("json.corrupt")).ok();
                  RestoreOutcome::Default
              }
          },
      }
  }

  fn apply_cwd_fallback(mut f: PanesFile) -> TabContainer {
      for tab in &mut f.tabs {
          walk_leaves_mut(&mut tab.pane_tree, |leaf| {
              if !leaf.cwd.is_dir() {
                  let reason = classify_cwd_error(&leaf.cwd);
                  let home = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"));
                  tracing::warn!(
                      "pane cwd fallback: {} → $HOME (reason: {})",
                      leaf.cwd.display(), reason
                  );
                  leaf.cwd = home;
              }
          });
      }
      TabContainer::from(f)
  }
  ```

- **검증 AC**: AC-P-12 (positive + negative), AC-P-13, **AC-P-13a (REQ-P-056 — v1.0.0 NM-1 rename)**, AC-P-14, AC-P-15

- **MX 태그 계획**:
  - `save_panes`: `@MX:WARN(race-condition-on-concurrent-write)` with `@MX:REASON(multiple-windows-may-write-same-file)` — atomic rename 이 race 를 완화하지만 단일 writer 가정.
  - `restore_panes`: `@MX:ANCHOR(persistence-restore-entry)` — fan_in 예상 ≥ 2 (앱 시작, workspace 전환).
  - `apply_cwd_fallback`: `@MX:NOTE(cwd-fallback-req-p-056)` — v1.0.0 NM-1 rename 기록.
  - `SerializedPaneTree` 에는 scrollback / selection / cursor_position 필드 **부재** (REQ-P-055 negative).

---

#### T13: Persistence 통합 + 앱 shutdown / startup hook

- **Priority**: High
- **선행 의존성**: T12
- **대상 파일**:
  - `crates/moai-studio-ui/src/lib.rs` 수정 — `on_close_requested` hook 에서 `save_panes()` 호출
  - `crates/moai-studio-app/src/main.rs` 수정 — 앱 시작 시 `restore_panes()` 호출하여 RootView 초기화

- **예상 산출**:
  - GPUI `WindowCloseEvent` hook 에서 `save_panes(&ws_id, &tab_container.read(cx))` 호출
  - 앱 시작 시 `match restore_panes(&ws_id)` 분기
  - REQ-P-052 의 "atomic write" 를 실제 앱 종료 시점에 연결

- **검증 AC**: AC-P-12 (end-to-end), AC-P-13 (end-to-end)

- **MX 태그 계획**:
  - `on_close_requested`: `@MX:WARN(shutdown-race-window)` with `@MX:REASON(crash-before-save-is-non-recoverable)` — spec.md REQ-P-052 "비정상 종료 보장 없음" 정책 명시.

---

#### T14: CI regression gate (`.github/workflows/ci-v3-pane.yml`)

- **Priority**: High
- **선행 의존성**: T1 ~ T13 (모든 구현 완료)
- **대상 파일**:
  - `.github/workflows/ci-v3-pane.yml` (신규)

- **예상 산출**:
  - acceptance.md §7.1 의 5 job 구성 (`unit-tests`, `integration-tests`, `snapshot-tests`, `benches`, `terminal-core-regression`)
  - Matrix: `macos-14` + `ubuntu-22.04`
  - 전제: SPEC-V3-002 의 Zig setup step 재사용
  - Milestone 전환 gate: 각 milestone 완료 commit 에서 해당 milestone 까지 AC 전체 실행

- **검증 AC**: AC-P-16 (CI gate), 전체 AC 의 CI 실행 경로

- **MX 태그 계획**:
  - workflow 파일에 주석으로 MX 태그 설명 (파일 내 주석):
    ```yaml
    # @MX:ANCHOR(ci-v3-pane-gate) — 본 SPEC-V3-003 의 CI gate. AC 29개 전체 실행.
    ```

---

## 5. 의존성 그래프

```
T1 (PaneTree 자료구조)
  │
  ├──► T2 (PaneConstraints)
  │     │
  │     └──► T3 (PaneSplitter / ResizableDivider trait + Mock)
  │           │
  │           ├──► [Spike 1: GPUI divider drag API]
  │           │     │
  │           │     ├──► [Spike 2: gpui-component Resizable] (S1 결과 조건부)
  │           │     │     │
  │           │     │     └──────────────┐
  │           │     │                    ▼
  │           │     └──────────────────► T4 (PaneSplitter 구체 구현)
  │           │                           │
  │           │                           └──► T5 (ResizableDivider 구체 구현)
  │           │                                 │
  │           │                                 └──► T6 (Focus routing + MS-1 키 바인딩)
  │           │                                       │
  │           │                                       └──► T7 (RootView 통합)  ← MS-1 완료
  │           │                                             │
  │           │                                             └──► T8 (TabContainer)
  │           │                                                   │
  │           │                                                   ├──► T9 (MS-2 키 바인딩 + AC-P-26 tmux)
  │           │                                                   │
  │           │                                                   ├──► T10 (탭 바 UI + AC-P-27)
  │           │                                                   │
  │           │                                                   └──► T11 (탭 성능 bench) ← MS-2 완료
  │           │                                                         │
  │           │                                                         └──► T12 (Persistence schema + cwd fallback)
  │           │                                                               │
  │           │                                                               └──► T13 (Persistence 통합 hook) ← MS-3 완료
  │           │                                                                     │
  │           │                                                                     └──► T14 (CI gate)
  │           │
  │           └──► [Spike 3: ID 생성 방식] (T1 재작업 없이 PaneId 타입만 갱신)
  │
  └──► [Spike 4: Linux shell 관례] (Spike 4 결과가 shift-escalation 경로 시 → annotation cycle 재개, T6/T9 의 키 바인딩 갱신)
```

병렬 가능:
- Spike 1, 3, 4 는 독립적으로 병렬 실행 가능
- T2 와 T3 은 T1 완료 후 병렬 착수 가능
- T9, T10, T11 은 T8 완료 후 병렬 진행 가능 (단, T11 은 T9 완료된 상태에서 9 tabs fixture 필요)

---

## 6. 위험 관리

spec.md §12 R-1 ~ R-9 와 본 plan task 의 직접 연관 매핑:

| Risk ID | 관련 Task | 완화 검증 | 비고 |
|---------|-----------|-----------|------|
| R-1 (divider drag API) | T4, T5, Spike 1, Spike 2 | Spike 1 실패 시 Spike 2 fallback, 두 경로 모두 추상 trait 기반 | spec.md §11.1 C-1 |
| R-2 (FD 압박 / 메모리) | T4, T13 | pane 당 60 MB/10K rows scrollback 상한 SPEC-V3-002 계승 | research §4.2 |
| R-3 (Terminal Core API 변경 유혹) | T7, T14 | AC-P-16 CI gate | spec.md REQ-P-060 |
| R-4 (FocusHandle 괴리) | T6 | AC-P-22 single_focus_invariant | research §4.5 |
| R-5 (탭 바 디자인 토큰 부재) | T10 | REQ-P-044 의 "bold + background" 최소 스펙 + AC-P-27 | v1.0.0 Nm-2 해소 |
| R-6 (Persistence schema 역호환) | T12 | 별도 파일 `panes-{ws-id}.json` + `"moai-studio/panes-v1"` | AC-P-14 / AC-P-15 |
| R-7 (gpui-component 의존 유지비) | T4, Spike 2 | spike 미통과 시 자체 구현 fallback | spec.md §11.1 |
| R-8 (Linux Super 키 기대) | T6 | Ctrl 고정, Shortcut Customization SPEC 연기 | Exclusion #12 |
| **R-9 (Linux shell 관례 충돌)** | Spike 4, T6, T9 | Spike 4 결정 후 RG-P-4 Linux 컬럼 확정 또는 유지 | **v1.0.0 Nm-3 해소** |

추가 위험 (plan 단계에서 발견):

- **R-P1 (tmux 바이너리 CI 의존)**: AC-P-26 integration test 가 실제 tmux 를 필요로 함. 완화: macOS runner `brew install tmux`, Linux runner `apt install tmux` step 을 `ci-v3-pane.yml` 에 추가.
- **R-P2 (design token 값 부재)**: `.moai/design/v3/system.md` 에 `toolbar.tab.active.background` 값 미정의 상태에서 T10 시작 시 blocker. 완화: T10 의 첫 단계에서 design token 값 확정 (plan 단계에서 직접 결정).

---

## 7. 품질 게이트 (per-task + per-milestone)

### 7.1 Per-task gate

각 task 완료 시 다음 모두 통과:

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` (0 warnings)
- [ ] `cargo test -p moai-studio-ui --lib` (해당 task 의 unit test 전체 GREEN)
- [ ] (해당 task 가 AC 를 건드리면) `cargo test -p moai-studio-ui --test integration_*` 해당 테스트 GREEN

### 7.2 Per-milestone gate

| Milestone | Gate |
|-----------|------|
| MS-1 완료 | AC-P-1 ~ AC-P-7, AC-P-17, AC-P-18, AC-P-20, AC-P-21, AC-P-22, AC-P-23 + AC-P-16 (Terminal Core regression) 모두 GREEN |
| MS-2 완료 | MS-1 전체 + AC-P-8, AC-P-9a, AC-P-9b, AC-P-10, AC-P-11, AC-P-19, AC-P-24, AC-P-25, **AC-P-26, AC-P-27** 모두 GREEN |
| MS-3 완료 | MS-2 전체 + AC-P-12, AC-P-13, **AC-P-13a**, AC-P-14, AC-P-15 모두 GREEN → SPEC Sync phase 진입 가능 |

### 7.3 CI job 매핑 (acceptance.md §7 재확인)

- `unit-tests` (matrix: macos-14, ubuntu-22.04): AC-P-1, 3, 4, 7, 8, 10, 17, 20, 21, 22, 24
- `integration-tests` (matrix): AC-P-2, 5, 9a (macOS only), 9b (Linux only), 11, 12, 13, 13a, 14, 15, 25, 26
- `snapshot-tests` (ubuntu-22.04): AC-P-27 (옵션 A)
- `benches` (ubuntu-22.04, PR smoke): AC-P-18, 19
- `terminal-core-regression` (ubuntu-22.04): AC-P-16

---

## 8. MX 태그 전략 요약

| Tag 유형 | 개수 예상 | 주요 위치 |
|----------|-----------|-----------|
| `@MX:ANCHOR` | 9+ | PaneTree enum, PaneSplitter trait, ResizableDivider trait, PaneConstraints impl, FocusRouter, TabContainer::switch_tab, TabContainer::new_tab, TabBar active styling, Persistence restore entry |
| `@MX:WARN` | 3+ | External dep API churn (spike 결과), shutdown race window, concurrent write (persistence) |
| `@MX:NOTE` | 7+ | SplitDirection 의미론, test-only Mock impl, platform modifier dispatch, tmux nested test, active tab styling design token, cwd fallback REQ-P-056 rename, tab switch bench |
| `@MX:TODO` | 2+ | Spike 결정 전 mock impl (T3), design token 값 미정 (T10 초기) — Run phase GREEN cycle 에서 해소 |

모든 `@MX:WARN` 는 `@MX:REASON` sub-annotation 필수 (CLAUDE.md MX protocol 준수).

---

## 9. 방법론별 실행 맵

quality.yaml `development_mode` 에 따라 Run phase 진입 시 분기:

### 9.1 TDD 경로 (RED-GREEN-REFACTOR, 본 repo 기본)

- T1 RED: `PaneTree::split_horizontal_from_leaf` 테스트 작성 → FAIL (`PaneTree` 미존재)
- T1 GREEN: `enum PaneTree` 최소 구현 → 테스트 PASS
- T1 REFACTOR: in-order iterator, split error 타입 분리 → 테스트 유지 GREEN
- (T2 ~ T14 동일 cycle 반복)
- Brownfield 주의: SPEC-V3-001/002 의 기존 RootView / TerminalSurface 는 **수정 전 read-first** (CLAUDE.md Section 7 Rule 5 + @workflow-modes.md brownfield 확장)

### 9.2 DDD 경로 (ANALYZE-PRESERVE-IMPROVE, quality.yaml 변경 시)

- ANALYZE: SPEC-V3-002 의 TerminalSurface / PtyWorker 사용 경로 전수 조사 (RG-P-7 의 "수정 금지" 확인)
- PRESERVE: SPEC-V3-002 의 74 tests + SPEC-V3-001 의 60 tests 를 characterization test 로 고정
- IMPROVE: T1 → T14 순서로 점진 구현, 각 단계 후 `cargo test --workspace` 실행하여 regression 0 확인

---

## 10. 완료 기준 (DoD)

SPEC-V3-003 는 다음 조건 **모두** 충족 시 완료 (Phase 3 종결):

- [ ] 37 REQ-P 전부 구현 확인 (acceptance.md §9 추적 가능성 매트릭스)
- [ ] 29 AC 전부 통과 (macOS + Linux 양 플랫폼, spec.md §6.4 G7)
- [ ] SPEC-V3-001 + SPEC-V3-002 의 기존 134 tests regression 0 (spec.md §6.4)
- [ ] 신규 테스트 ≥ 20 (MS 별 unit + integration + snapshot 합산)
- [ ] Coverage ≥ 85% (spec.md TRUST 5)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 warnings
- [ ] `cargo fmt --all -- --check` 통과
- [ ] `ci-v3-pane.yml` 5 job matrix (macOS + Linux × unit/integration/snapshot/benches/terminal-core) 모두 Green, wall-clock ≤ 10 분
- [ ] MX tag 총 20+ 개 추가 (ANCHOR 9, WARN 3, NOTE 7, TODO 2 이상)
- [ ] **plan-auditor iter 2 CONDITIONAL PASS 잔여 4건 전체 해소 확인**:
  - NM-1 REQ-P-057 → REQ-P-056 rename 구현 반영 (T12 의 `SerializedPaneTree` 및 `apply_cwd_fallback`)
  - Nm-1 AC-P-26 통과 (T9 의 `integration_tmux_nested.rs`)
  - Nm-2 AC-P-27 통과 (T10 의 탭 바 active styling)
  - Nm-3 Spike 4 완료 + R-9 최종 해소 (spec.md §6.4 / R-9 갱신 or 유지)
- [ ] spec.md §16 열린 결정 사항 모두 해소:
  - #1 gpui-component → Spike 2 완료 후 T4 결과로 확정
  - #2 iTerm2 horizontal 명칭 → research.md §2 외부 검증 완료
  - #3 `toolbar.tab.active.background` 색상 값 → T10 완료 시점에 system.md 반영
  - #4 PaneId / TabId 생성 → Spike 3 완료 후 T1 재검토
  - #5 MX 태그 적용 지점 → Run phase RED/ANALYZE 에서 최종 확정
  - #6 Milestone CI regression gate → T14 완료
  - #7 Linux Super 키 → SPEC 범위 유지 (Exclusion #12)
  - #8 Linux Ctrl+D/W/\\ → Spike 4 완료
- [ ] `.moai/specs/SPEC-V3-003/spec.md` / `plan.md` / `acceptance.md` / `research.md` 4 문서 정합성 유지

---

## 11. 참조

- `.moai/specs/SPEC-V3-003/spec.md` v1.0.0 — EARS 요구사항 (RG-P-1 ~ RG-P-7, 37 REQ-P)
- `.moai/specs/SPEC-V3-003/acceptance.md` v1.0.0 — Given/When/Then 29 AC
- `.moai/specs/SPEC-V3-003/research.md` — deep research 근거 (6 경쟁 레퍼런스 + 기술 후보)
- `.moai/specs/SPEC-V3-002/spec.md` / `plan.md` / `acceptance.md` — Terminal Core 공개 API (REQ-P-060 전제), 본 plan 의 구조 참조
- `.moai/specs/SPEC-V3-001/progress.md` — scaffold 전제
- `.moai/design/v3/spec.md:420-438` — 플랫폼별 키 바인딩 이원 표
- `.moai/design/v3/system.md` — Toolbar 토큰 (T10 에서 `toolbar.tab.active.background` 추가)
- `.claude/rules/moai/workflow/workflow-modes.md` — TDD / DDD 선택
- plan-auditor iter 2 CONDITIONAL PASS (2026-04-24, 잔여 4건: NM-1, Nm-1, Nm-2, Nm-3)

---

Version: 1.0.0 · 2026-04-24 · approved (iter 2 CONDITIONAL PASS + annotation cycle)
