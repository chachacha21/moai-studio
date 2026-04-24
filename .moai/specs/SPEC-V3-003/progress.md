## SPEC-V3-003 Progress

- Started: 2026-04-24T (run phase entry — ultrathink 키워드 감지, Adaptive Thinking 활성)
- Branch: feat/v3-scaffold (4 commits ahead of origin)
- SPEC status: approved v1.0.0 (annotation cycle 승인, NM-1/Nm-1/Nm-2/Nm-3 해소)
- Prior artifacts: spec.md (62KB), plan.md (41KB), acceptance.md (43KB), research.md (37KB)

### Phase 0.5 (memory_guard): SKIPPED
- quality.yaml `memory_guard.enabled: false` → skip environment memory assessment

### Phase 0.9 (JIT Language Detection): complete
- Detected: Cargo.toml workspace at repo root (resolver=2, rust-version=1.93, edition=2024, members=["crates/*"])
- Primary language skill: moai-lang-rust
- Additional language contexts: Zig 0.15.2 (libghostty-vt FFI, SPEC-V3-002 상속, Terminal Core 는 변경 금지이므로 Zig 직접 편집 없음)

### Phase 0.95 (Scale-Based Execution Mode): complete
- SPEC scope files: 8 신규 (panes/mod+tree+splitter+divider+focus+constraints, tabs/mod+container+bar) + 5 수정 (lib.rs 4지점 + workspace/persistence.rs) = 13 파일
- Domains: 3 (UI panes, UI tabs, workspace persistence)
- Complexity signals: 37 REQ-P + 29 AC + 3 milestones + 4 Plan Spikes
- Selected mode: **Full Pipeline** (SPEC scope >= 10 files AND >= 3 domains)
- Agents: manager-strategy (Phase 1) + manager-tdd (Phase 2, development_mode=tdd) + manager-quality (Phase 2.5) + evaluator-active (Phase 2.8a) + manager-git (Phase 3)

### Harness Level: thorough
- Rationale: complex multi-domain SPEC (3 domains, 29 AC, 4 Spikes) + ultrathink 요청
- Sprint Contract: enabled (Phase 2.0 contract.md 생성 예정)
- evaluator-active mode: per-sprint (Phase 2.0 + Phase 2.8a 양쪽)

### UltraThink Activation
- Trigger: user included `ultrathink` keyword in `/moai run SPEC-V3-003 ultrathink`
- Mode: Claude native extended reasoning (Adaptive Thinking for claude-opus-4-7)
- Applied to: Phase 1 manager-strategy 위임 (deeper architectural analysis + 4 Spike 배치 전략)

### Initial codebase state (targets)
- crates/moai-studio-ui/src/: lib.rs, terminal/{mod,clipboard,input}.rs (기존 SPEC-V3-001/002 산출)
- crates/moai-studio-workspace/src/: lib.rs only
- crates/moai-studio-terminal/src/: SPEC-V3-002 산출물 (74 tests, 변경 금지)

### Phase 1 (Analysis & Planning): complete
- Delegated to manager-strategy subagent (foreground, isolation=none, ultrathink via Adaptive Thinking)
- Agent ID: a2cdfe3cd65326793 (retained for potential SendMessage follow-up)
- Output: `.moai/specs/SPEC-V3-003/strategy.md` (9 sections, 29 AC coverage verified, 14 tasks TDD graph, 4 Spikes placement, 3 USER-DECISION-REQUIRED markers, 7 reuse patterns, 5 new risks R-P1~R-P5)
- Key findings:
  - Codebase reality check: plan.md line ref `lib.rs:290-299` → actual `:286-300` (minor drift, T7 must re-verify during RED phase)
  - Reuse opportunities: `$schema` versioned JSON + HOME/APPDATA split + ID pattern from `WorkspacesStore` + MX 주석 5-종 pattern from `worker.rs`
  - YAGNI flagged: `uuid` crate adoption (use existing nanos+prefix), `PaneConstraints::new(...)` (AC-P-21 forbids mutable API)
  - New risk R-P1: GPUI 0.2.2 headless test feasibility unverified — measure in T6/T8/T10 RED
  - New risk R-P2: tmux CI dependency — `apt install tmux` / `brew install tmux` in ci-v3-pane.yml

### Decision Point 1 (HUMAN GATE Plan Approval): APPROVED
- AskUserQuestion response: "승인하고 Phase 1.5부터 진행" + "실제 결정 시점에 개별 AskUserQuestion"
- Timestamp: 2026-04-24 run session
- Implication: USER-DECISION-REQUIRED 3 markers will be raised at their respective decision points (S4 completion, S2 completion, T10 RED phase), not preemptively

### Phase 1.5 (Task Decomposition): complete
- Output: `.moai/specs/SPEC-V3-003/tasks.md` persist (14 tasks × mapped AC × planned files × tier × status)
- Branch target: feat/v3-scaffold (per spec.md §11.4, per git-strategy.manual)
- All 29 AC coverage confirmed in Task 표 AC column

### Phase 1.6 (Acceptance Criteria Initialization): complete
- 29 AC 전체 TaskList 등록 (Batch 1: AC-P-1~10 = Tasks #11-15, 17-20; Batch 2: AC-P-11~19+13a = Tasks #21-31; Batch 3: AC-P-20~27 = Tasks #32-39)
- Failing checklist pattern — 모두 pending 상태. 각 AC 를 구현하며 completed 로 전환.
- 검증: acceptance.md §2-§5 scenarios 참조로 test location / failure mode / requirement 매핑 확인.

### Phase 1.7 (File Structure Scaffolding): complete
- Created 9 stub files with module-level documentation comments (Korean per `code_comments: ko`) + MX:TODO markers referencing task IDs:
  - crates/moai-studio-ui/src/panes/mod.rs
  - crates/moai-studio-ui/src/panes/tree.rs
  - crates/moai-studio-ui/src/panes/splitter.rs
  - crates/moai-studio-ui/src/panes/divider.rs
  - crates/moai-studio-ui/src/panes/focus.rs
  - crates/moai-studio-ui/src/panes/constraints.rs
  - crates/moai-studio-ui/src/tabs/mod.rs
  - crates/moai-studio-ui/src/tabs/container.rs
  - crates/moai-studio-ui/src/tabs/bar.rs
- lib.rs 는 무변경 (T1 RED phase 에서 `pub mod panes;` + T7 에서 전면 재배선). Rust 는 lib.rs 에서 참조하지 않는 파일도 컴파일 OK 이므로 stub 단계 baseline 불변.
- LSP baseline: cargo check 는 manager-tdd 가 Phase 2 진입 시 실행. 현재 repository 는 `cargo test -p moai-studio-terminal` 74 tests GREEN 상태 (SPEC-V3-002 post-completion).

### Phase 1.8 (Pre-Implementation MX Context Scan): complete
- MX Context Map (existing files):
  - `crates/moai-studio-ui/src/terminal/mod.rs:3` — `@MX:ANCHOR: terminal-surface-render` + `@MX:REASON: GPUI 렌더 진입점` (fan_in 높음, 변경 금지 — SPEC-V3-002 산출)
  - `crates/moai-studio-ui/src/terminal/mod.rs:19, 159` — `@MX:NOTE: font-metric-coord-mapping`
  - `crates/moai-studio-ui/src/terminal/mod.rs:76, 112` — `@MX:TODO` 2건 (SPEC-V3-002 후속, 본 SPEC 범위 외)
  - `crates/moai-studio-ui/src/lib.rs` — MX 태그 없음. T7 수정 시 `tab_container: Option<Entity<TabContainer>>` 필드에 `@MX:ANCHOR(root-view-content-binding)` 추가 필요 (strategy.md §5.1 T7 계획대로)
  - `crates/moai-studio-workspace/src/lib.rs` — MX 태그 없음. T12 `persistence.rs` 신규에서 `@MX:WARN(race-condition-on-concurrent-write)` + `@MX:ANCHOR(persistence-restore-entry)` 예정
- 제약 전달: Phase 2 agent prompt 에 terminal crate `@MX:ANCHOR: terminal-surface-render` 는 "절대 수정 금지" 계약으로 명시 (RG-P-7 AC-P-16 재확인)

### Spike 3 (PaneId / TabId 생성 방식): 결정 완료
- 조사: `crates/moai-studio-workspace/src/lib.rs:60-67` 기존 workspace ID 패턴 `format!("ws-{:x}", nanos)` 확인
- 결정: **기존 패턴 차용** — `PaneId = format!("pane-{:x}", nanos)`, `TabId = format!("tab-{:x}", nanos)`
- 근거: (1) workspace/terminal/pane/tab 의 ID 네이밍 consistency, (2) uuid crate 추가 불필요 (YAGNI 회피, Cargo.toml 변경 없음), (3) 충돌 가능성 무시 가능 (nanos precision 이면 동일 mill-sec 내 여러 pane 생성 시에도 carrier 가 다름, 필요시 counter 추가 fallback)
- 산출: 본 progress.md 기록 + tasks.md 반영. 별도 spike 보고서 미생성 (read-only design decision).

### Phase 2.0 (Sprint Contract, thorough harness): complete
- Output: `.moai/specs/SPEC-V3-003/contract.md` MS-1 sprint 계약 생성
- Scope: T1~T7 + Spike 1 + Spike 3 (완료)
- Priority: Functionality 40% / Security 25% (Phase 2.8a full audit) / Craft 20% / Consistency 15%
- Acceptance checklist: 14 MS-1 AC (AC-P-1~7, 9a/9b MS-1 부분, 16, 17, 18, 20, 21, 22, 23)
- Hard thresholds: 85% coverage, 0 LSP errors, 0 clippy warnings, SPEC-V3-002 regression 0
- Escalation: 3 연속 RED 실패 → re-planning gate, Spike 1 FAIL → AskUserQuestion
- evaluator-active 호출 전략: Phase 2.0 skip (strategy.md 가 이미 plan review 완료), Phase 2.8a 에서 1회 full 4-dim 평가 (Opus 4.7 "fewer sub-agents" 원칙)

### Phase 2 T1 (PaneTree RED-GREEN-REFACTOR): complete
- Agent: manager-tdd (subagent_type, isolation=worktree, foreground)
- Agent ID: acaf7776814bc1279 (retained for T2 SendMessage resume)
- Scope: T1 only (T2/T3/T4 범위 침범 없음)
- files modified:
  - crates/moai-studio-ui/src/panes/tree.rs (stub → 제네릭 PaneTree<L> + 13 unit tests, ~540 LOC)
  - crates/moai-studio-ui/src/panes/mod.rs (stub → pub re-export: PaneTree/PaneId/SplitNodeId/SplitDirection/SplitError/RatioError/Leaf)
  - crates/moai-studio-ui/src/lib.rs (`pub mod panes;` 1줄 추가, 다른 부분 무수정 — drive-by refactor 금지 준수)
- 구현 결정:
  - **제네릭 PaneTree<L>** 채택: prod `L = Entity<TerminalSurface>` (T4 통합), test `L = String` (GPUI context 없이 단위 검증). Rationale: doc comment 명시.
  - **PaneId/SplitNodeId 패턴**: Spike 3 결정대로 `format!("pane-{:x}", nanos)` / `format!("split-{:x}", nanos)` — workspace/src/lib.rs:60-67 차용.
  - **PaneId::new_from_literal(&str)** 추가: 테스트 편의 메서드. clippy `should_implement_trait` 회피 목적.
  - **Leaf<L>** 래퍼 구조체: PaneId + payload 분리 — close 알고리즘의 ownership 이전 단순화.
- test results:
  - `cargo test -p moai-studio-ui --lib panes::tree`: **13/13 PASS**
  - `cargo test -p moai-studio-terminal`: **74/74 PASS** (AC-P-16 regression gate GREEN)
  - Coverage (llvm-cov): panes/tree.rs **line 90.10% / branch 85.59%** (목표 85% 초과)
- MX tags added:
  - `panes/tree.rs:111` ANCHOR `pane-tree-invariant` + REASON (fan_in >= 4)
  - `panes/tree.rs:170` ANCHOR `pane-split-api` + REASON (fan_in >= 3: T4/T7/T9)
  - `panes/tree.rs:78` NOTE `horizontal-is-left-right-not-top-bottom` (spec.md §7.1 C-3)
  - `panes/tree.rs:19` TODO T4 PaneLeafHandle GPUI Entity 통합
- TRUST 5 self-check: T/R/U/S/T 전원 PASS
- implementation_divergence:
  - planned vs actual files: 완전 일치 (0% drift)
  - additional_features: PaneId::new_from_literal (test helper)
  - new_dependencies: 없음 (Cargo.toml 변경 없음)
- AC 통과 (T1 범위):
  - **AC-P-1** ✅ split_horizontal_from_leaf / split_vertical_from_leaf / split_direction_first_second_semantics 검증
  - **AC-P-3** ✅ close_last_leaf_is_noop 검증
  - **AC-P-20** ✅ ratio_boundary_rejected (0.0, 1.0, NaN, Inf 모두 Err)
  - **AC-P-2 (단위 부분)** ✅ close_promotes_sibling (integration FD count 는 T4 범위)
- blockers: 없음

### Phase 2 T2 (PaneConstraints RED-GREEN-REFACTOR): complete
- Agent: manager-tdd (subagent_type, isolation=worktree, foreground, 별도 agent spawn)
- Agent ID: ad7ad54130a4ca255
- Scope: T2 only (T3 범위 미침범)
- files modified:
  - crates/moai-studio-ui/src/panes/constraints.rs (stub → 실제 구현, 76 LOC including tests + doc tests)
  - crates/moai-studio-ui/src/panes/mod.rs (+2 lines: `pub mod constraints;` + `pub use constraints::PaneConstraints;`)
- 구현 결정:
  - **unit struct** `pub struct PaneConstraints;` — 의도적 non-instantiable 마커 (인스턴스 활용 없음)
  - **`impl` associated const**: `MIN_COLS: u16 = 40`, `MIN_ROWS: u16 = 10` (spec.md M-2 해소)
  - **가변 API 금지**: new / with_* / set_* / Builder 패턴 불허 (AC-P-21)
  - **AC-P-21 컴파일타임 강제**: doc test `compile_fail` 3건 (new, set_min_cols, type mismatch) — trybuild 의존 없이 doc test 로 완전 대체, Cargo.toml 무변경
- test results:
  - `cargo test -p moai-studio-ui --lib panes::constraints`: **3/3 PASS**
  - `cargo test --doc -p moai-studio-ui`: **3 compile_fail doc tests PASS** (AC-P-21 negative enforcement)
  - `cargo test -p moai-studio-ui --lib` 전체: **76/76 PASS** (T1 13 + T2 3 + 기존 60)
  - `cargo test -p moai-studio-terminal`: **14/14 PASS** (AC-P-16 regression gate, 1 ignored 는 기존 상태 유지)
  - `cargo clippy -p moai-studio-ui -- -D warnings`: **0 warnings**
  - Coverage: constraints.rs **~100%** (12 LOC 실 구현 완전 커버)
- MX tags added:
  - `panes/constraints.rs:38` ANCHOR `pane-constraints-immutable` + REASON (fan_in >= 3: T4/T5/T7)
- TRUST 5 self-check: T/R/U/S/T 전원 PASS
- implementation_divergence:
  - planned vs actual files: 완전 일치 (0% drift)
  - additional_features: 없음 (YAGNI 준수)
  - new_dependencies: 없음
- AC 통과 (T2 범위):
  - **AC-P-21** ✅ PaneConstraints public API negative surface 컴파일타임 강제 완료
  - **AC-P-4** 준비 완료 (T4/T5 에서 MIN_COLS/MIN_ROWS 활용 예정)
- blockers: 없음

### Session Summary (2026-04-24 /moai run SPEC-V3-003 ultrathink)

완료된 Phase:
- Phase 0.5 (skip — memory_guard disabled)
- Phase 0.9 Language detection (Rust 1.93 workspace)
- Phase 0.95 Scale-Based Mode (Full Pipeline)
- Phase 1 manager-strategy 분석 (strategy.md 9 sections, 29 AC 커버리지 검증)
- Decision Point 1 HUMAN GATE → **APPROVED**
- Phase 1.5 tasks.md (14 tasks × 3 milestones)
- Phase 1.6 29 AC TaskCreate (Batch 1+2+3, TaskList 후속 리셋되었으나 tasks.md 에 persistent)
- Phase 1.7 9 stub files
- Phase 1.8 MX context scan (terminal/mod.rs 의 ANCHOR/NOTE/TODO 파악)
- Phase 2.0 Sprint Contract (contract.md MS-1)
- Spike 3 결정 완료 (PaneId/TabId pattern = `format!("pane-{:x}", nanos)`)
- **Phase 2 T1 PaneTree** (commit b65e34a, 13 tests, 90% coverage)
- **Phase 2 T2 PaneConstraints** (commit fa68cb1, 3+3 tests, ~100% coverage)

Commits added:
- `579c9e2` docs(spec): SPEC-V3-003 Run Phase 1 산출물 + MS-1 stub scaffolding
- `b65e34a` feat(panes): T1 PaneTree — 이진 트리 split/close 자료구조 v1.0.0 (AC-P-1, AC-P-3, AC-P-20)
- `fa68cb1` feat(panes): T2 PaneConstraints — 최소 pane 크기 불변 상수 (AC-P-21)

Branch: feat/v3-scaffold (7 commits ahead of origin — 기존 4 + 본 session 3)
Working tree: clean (T2 commit 후)

AC 통과 누계 (MS-1 14 AC 중):
- AC-P-1 ✅ (T1, split_horizontal/vertical_from_leaf)
- AC-P-2 ⏳ 부분 (T1 unit; T4 integration 대기)
- AC-P-3 ✅ (T1, close_last_leaf_is_noop)
- AC-P-20 ✅ (T1, ratio_boundary_rejected)
- AC-P-21 ✅ (T2, PaneConstraints negative API surface)
- 잔여 9건: AC-P-4/5/6/7/9a(MS-1 부분)/9b(MS-1 부분)/16/17/18/22/23 → T3~T7 에서 처리

### Phase 2 T3 (PaneSplitter + ResizableDivider RED-GREEN-REFACTOR): complete
- Agent: manager-tdd (TDD implementer, T3 only, no sub-agent spawn)
- Scope: T3 only (T4/T5 범위 미침범, T1/T2 무수정)
- files modified:
  - crates/moai-studio-ui/src/panes/splitter.rs (stub → PaneSplitter trait + CloseError + MockPaneSplitter + 8 unit tests)
  - crates/moai-studio-ui/src/panes/divider.rs (stub → ResizableDivider trait + MockDivider + 4 unit tests)
  - crates/moai-studio-ui/src/panes/mod.rs (append-only: `pub mod splitter;` + re-exports + `pub mod divider;` + re-exports + @MX:TODO 제거, 기존 라인 무수정)
- 구현 결정:
  - **CloseError** enum: variant `TargetNotFound` 1개. `From<SplitError>` 구현 (defensive, MinSizeViolated → TargetNotFound 매핑).
  - **AC-P-17 검증 방식**: d 경로 채택 — `tests::abstract_traits_compile_without_impl` unit test (trait object + Mock 결합). doc test 는 `#[cfg(test)]` 외부 접근 불가 + Cargo.toml 변경 금지 제약으로 기각. `no_run` fence doc test 는 실행 검증 불가로 기각.
  - **MockPaneSplitter payload factory**: `format!("mock-pane-{n}")` + `next_counter: u32` 증가. PaneId 는 `new_from_literal` 로 생성 (T1 패턴 재사용).
  - **MockDivider**: `sibling_min_px` 외부 주입. `clamp_ratio` 내부 헬퍼. `min_px_for_orientation` `#[allow(dead_code)]` — T5 참조 경로 문서화용.
  - **import 배치**: `use crate::panes::{PaneConstraints, SplitDirection}` 를 `#[cfg(test)]` 위에 배치 → clippy `-D unused-imports` 회피.
- test results:
  - `cargo test -p moai-studio-ui --lib splitter`: **8/8 PASS**
  - `cargo test -p moai-studio-ui --lib divider`: **4/4 PASS**
  - `cargo test -p moai-studio-ui --lib` 전체: **88/88 PASS** (T1 13 + T2 3 + T3 12 + 기존 60)
  - `cargo test --doc -p moai-studio-ui`: **3/3 PASS** (T2 compile_fail doc tests 유지, T3 doc test 없음)
  - `cargo test -p moai-studio-terminal`: **4/4 PASS** (integration binary 포함 총 4, AC-P-16 regression gate GREEN)
  - `cargo clippy -p moai-studio-ui -- -D warnings`: **0 warnings**
  - `cargo fmt --package moai-studio-ui`: clean
- MX tags added:
  - `splitter.rs` before `pub trait PaneSplitter`: ANCHOR `pane-splitter-contract` + REASON
  - `divider.rs` before `pub trait ResizableDivider`: ANCHOR `divider-contract` + REASON
  - `splitter.rs` before `MockPaneSplitter`: NOTE `test-only-impl`
  - `divider.rs` before `#[cfg(test)] use`: NOTE `test-only-impl`
- TRUST 5 self-check: T/R/U/S/T 전원 PASS
- implementation_divergence:
  - planned vs actual files: 완전 일치 (0% drift)
  - additional_features: `CloseError::From<SplitError>` defensive impl, `MockDivider::min_px_for_orientation` doc 메서드
  - scope_changes: DividerOrientation 신규 도입 없음 (task 명시 대로 SplitDirection 재사용)
  - new_dependencies: 없음
  - new_directories: 없음
- AC 통과 (T3 범위):
  - **AC-P-17** ✅ abstract_traits_compile_without_impl unit test 통과 — PaneSplitter + ResizableDivider + Mock 조합이 GPUI 의존 없이 컴파일·실행됨
- blockers: 없음 (T4 blocker: Spike 1 GPUI 0.2.2 divider drag API 검증 — T3 블록 아님, T4 선행)

### Next Session Resume Instructions

다음 session 에서 `/moai run SPEC-V3-003` 재호출:
1. progress.md 읽고 "Phase 2 T3 complete" + "Next Session Resume Instructions" 섹션 확인
2. **Spike 1** 먼저 실행 (GPUI 0.2.2 divider drag API 검증) — T4/T5 blocker. Context7 `gpui` 라이브러리 조회 후 native drag API 존재 여부 확인.
3. Spike 1 결과에 따라:
   - PASS: T4 `GpuiNativeSplitter` 구현 (`splitter_gpui_native.rs` 신규, Cargo.toml 무변경)
   - FAIL: [USER-DECISION-REQUIRED: gpui-component-adoption] AskUserQuestion → S2 Spike 실행 여부 결정
4. T5 → T6 → T7 순차. T6 에서 [USER-DECISION-REQUIRED: spike-4-linux-shell-path] 조사 선행 가능 (MS-2 T9 Linux 결정에 영향)
5. MS-1 완료 시 contract.md §7 Sprint Exit Criteria 모두 체크
6. MS-2 진입 전 contract.md 에 MS-2 sprint revision 추가

사전 준비물 (다음 session resume 시 orchestrator 가 reload):
- 본 progress.md (checkpoint, T3 완료 상태)
- tasks.md T4~T14 표
- strategy.md §5.1 T4/T5 상세
- contract.md §4.2 integration test 시나리오
- spec.md §7.2/§7.3 (trait 정의), §11.1 C-1 (spike 전략)
- T3 산출물:
  - `splitter.rs`: `PaneSplitter` trait + `CloseError` + `MockPaneSplitter`
  - `divider.rs`: `ResizableDivider` trait + `MockDivider`
  - `mod.rs` re-exports: `CloseError, PaneSplitter, ResizableDivider`

### Spike 1 (GPUI 0.2.2 divider drag API 검증): complete — **PASS**
- 조사 시점: 2026-04-24 post-T3 session
- Method: Context7 MCP `/websites/rs_gpui_gpui` (4718 snippets) + docs.rs WebFetch (`trait.InteractiveElement`, `struct.MouseMoveEvent`) + 현재 repo grep (`lib.rs:165` 기존 on_mouse_down 패턴 확인)
- 판정: **PASS** — GPUI 0.2.2 native API only 경로 확정
- 핵심 발견:
  - `InteractiveElement::on_mouse_down/on_mouse_move/on_mouse_up` 트리오 Bubble phase 지원
  - `MouseMoveEvent::dragging() -> bool` drag 활성 판정
  - `on_drag_move<T>` — handle 외부 이동 capture (divider 경계 이탈 시에도 수신)
  - `Stateful<Div>` + `id()` + `.w(px)` / `.flex_basis(px)` 로 layout 갱신
  - `cx.notify()` frame 재그림 트리거
- 산출: `docs/spikes/SPIKE-V3-003-01-gpui-divider.md` (구현 pseudo code 포함, T4/T5 설계 가이드)
- Spike 2 미실행 확정 (plan.md §3 "S1 FAIL 조건부" 조건 불충족)

### USER-DECISION: gpui-component-adoption = 자체 구현 경로 확정 (2026-04-24)
- 사용자 선택: "자체 구현 — GpuiNativeSplitter + GpuiDivider (권장)"
- 본 session 진행 범위: "T4 만 완료 후 checkpoint (권장)"
- Cargo.toml 무변경 원칙 재확인 (external crate 불도입)
- T4 대상 파일: `crates/moai-studio-ui/src/panes/splitter_gpui_native.rs` (신규)
- T5 는 다음 session resume 예정

### Phase 2 T4 (GpuiNativeSplitter RED-GREEN-REFACTOR): complete

- Agent: manager-tdd (TDD implementer, T4 only, no sub-agent spawn)
- Scope: T4 only (T1/T2/T3 무수정, T5/T6/T7 범위 미침범)
- files modified:
  - crates/moai-studio-ui/src/panes/splitter_gpui_native.rs (신규, 경로 A generic factory)
  - crates/moai-studio-ui/src/panes/mod.rs (append-only: `pub mod splitter_gpui_native;` + `pub use splitter_gpui_native::GpuiNativeSplitter;`)

- 구현 결정:
  - **경로 A (Generic Factory)** 확정: `GpuiNativeSplitter<L: Clone + 'static>`. 사유:
    `gpui` crate `test-support` feature 가 `crates/moai-studio-ui/Cargo.toml` 에 없고
    Cargo.toml 변경 금지 원칙으로 `TestAppContext` 사용 불가. Factory closure 주입으로
    prod (`Entity<TerminalSurface>`) / test (`String`, `Arc<Mutex<i32>>`) 격리.
  - **factory: Box<dyn FnMut(&PaneId) -> L>**: split 시 새 PaneId 를 인수로 받아 payload 생성.
    T7 wire-up 시 `Box::new(|_id| cx.new(|cx| TerminalSurface::new(...)))` 주입.
  - **close drop 검증**: `Arc<Mutex<i32>>` payload 로 `Arc::strong_count` 추적.
    close 후 leaf Arc 참조 해제 → strong_count 감소 검증 (AC-P-2 단위).

- test results:
  - `cargo test -p moai-studio-ui --lib splitter_gpui_native`: **9/9 PASS**
  - `cargo test -p moai-studio-ui --lib` 전체: **97/97 PASS** (88 기존 + 9 신규)
  - `cargo test --doc -p moai-studio-ui`: **3/3 PASS** (T2 compile_fail doc tests 유지)
  - `cargo test -p moai-studio-terminal`: **4/4 PASS** (AC-P-16 regression gate GREEN)
  - `cargo clippy -p moai-studio-ui -- -D warnings`: **0 warnings**
  - `cargo fmt --package moai-studio-ui -- --check`: clean

- MX tags added:
  - `splitter_gpui_native.rs` before `pub struct GpuiNativeSplitter`: ANCHOR `concrete-splitter-gpui-native` + REASON (fan_in >= 3: T7/T9/T11)
  - `splitter_gpui_native.rs` factory field: ANCHOR `pane-leaf-factory-injection` + REASON (T7 wire-up 포인트)
  - `splitter_gpui_native.rs` `impl PaneSplitter`: WARN `gpui-api-churn-risk` + REASON (GPUI 0.2.2 API churn 예상)

- TRUST 5 self-check: T/R/U/S/T 전원 PASS

- AC 통과 (T4 범위):
  - **AC-P-1**: PARTIAL — factory 설계 확정 (prod wire T7 에서 Entity<TerminalSurface> 주입 시 완전 충족)
  - **AC-P-2**: PARTIAL — Arc strong_count drop 단위 검증 완료 (실제 Entity drop 은 T7 integration 시)
  - **AC-P-16**: PASS — moai-studio-terminal 4/4 regression 0

- deferred_ac:
  - AC-P-5 (visual hide small window): T5 divider 구체 + T7 RootView 통합 시 자연 충족
  - AC-P-6 (divider drag clamp): T5 범위
  - AC-P-18 (paint ≤ 200ms bench): T11 criterion harness 도입 시 (Cargo.toml 변경 필요)

- implementation_divergence:
  - planned vs actual: 경로 B → 경로 A (generic factory) 변경. 사유: TestAppContext 사용 불가
  - additional_features: `GpuiNativeSplitter::tree()` + `focused()` 조회 메서드 (test helper)
  - scope_changes: prod Entity<TerminalSurface> 바인딩 T7 으로 이연 (계획 범위)
  - new_dependencies: 없음 (Cargo.toml 무변경)
  - new_directories: 없음

- blockers: 없음

### AC 통과 누계 (T4 완료 시점, MS-1 14 AC 중)

- AC-P-1 ✅ PARTIAL → T7 완전 충족
- AC-P-2 ✅ PARTIAL (단위 Arc drop) → T7 integration 완전 충족
- AC-P-3 ✅ (T1)
- AC-P-16 ✅ regression 0 (T4 포함)
- AC-P-17 ✅ (T3)
- AC-P-20 ✅ (T1)
- AC-P-21 ✅ (T2)
- 잔여: AC-P-4/5/6/7/9a/9b/18/22/23 → T5~T7 에서 처리

### Next: T5 GpuiDivider (ResizableDivider 구체 구현)



