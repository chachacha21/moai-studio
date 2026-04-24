---
spec_id: SPEC-V3-003
title: Tab/Pane Split — Acceptance Criteria
version: 1.0.0
created_at: 2026-04-24
updated_at: 2026-04-24
parent_spec: SPEC-V3-003
parent_version: 1.0.0
language: Korean
author: MoAI (manager-spec, v1.0.0 plan phase 산출물)
---

# SPEC-V3-003 Acceptance Criteria (상세)

## 0. 개요

본 문서는 `.moai/specs/SPEC-V3-003/spec.md` v1.0.0 §10 AC 표의 각 항목에 대한 **실행 가능한 테스트 시나리오** 를 기술한다. Given / When / Then 3 블록은 spec.md 의 간략 표 버전을 **실제 Rust 코드 또는 의사 코드 수준** 으로 확장하며, 각 AC 는 테스트 하네스 / 검증 코드 위치 / 실패 모드 (flakiness 사례) 까지 명시된다.

[HARD] 본 문서의 모든 AC metric 은 spec.md §10 에 대응하는 **AC-P-NNN** 번호 체계를 그대로 유지한다. 새 AC 번호를 부여하지 않는다.

AC 총수: **29** (AC-P-1 ~ AC-P-25 + AC-P-9a / AC-P-9b / AC-P-13a + AC-P-26 + AC-P-27)

- AC-P-9 는 플랫폼 이원화로 AC-P-9a (macOS) / AC-P-9b (Linux) 로 분할되어 있음 — AC 카운트는 각각 1건.
- AC-P-13a 는 cwd fallback 검증 (REQ-P-056, v1.0.0 Nm-1 해소로 REQ-P-057 에서 rename).
- AC-P-26 는 REQ-P-034 (Optional, tmux 중첩 OS-level 우선 처리, v1.0.0 Nm-1 해소).
- AC-P-27 는 REQ-P-044 (탭 바 active 시각 구분, v1.0.0 Nm-2 해소).

테스트 실행 환경:

- macOS 14+ (Apple Silicon + Intel runner 각 1회)
- Ubuntu 22.04+ (x86_64 runner)
- Rust stable 1.93+
- Zig 0.15.2 (libghostty-vt 의존, SPEC-V3-002 계승)

테스트 하네스:

- **Unit**: `cargo test -p moai-studio-ui --lib` / `cargo test -p moai-studio-workspace --lib`
- **Integration**: `cargo test -p moai-studio-ui --test integration_*`
- **GPUI snapshot**: `cargo test -p moai-studio-ui --features snapshot`
- **Benchmark**: `cargo bench -p moai-studio-ui` (criterion, AC-P-18 / AC-P-19)
- **Manual verification**: `cargo run --example ghostty-spike` + 본 문서 §8 체크리스트

---

## 1. Test Harness 구조

### 1.1 테스트 파일 레이아웃

```
crates/moai-studio-ui/
├── src/
│   ├── panes/
│   │   ├── tree.rs          # PaneTree enum (AC-P-1 ~ AC-P-7, AC-P-20, AC-P-22)
│   │   ├── splitter.rs      # PaneSplitter trait (AC-P-17)
│   │   ├── divider.rs       # ResizableDivider (AC-P-6)
│   │   ├── focus.rs         # focus routing (AC-P-22)
│   │   └── constraints.rs   # PaneConstraints (AC-P-4, AC-P-21)
│   ├── tabs/
│   │   ├── container.rs     # TabContainer (AC-P-8, AC-P-10, AC-P-11, AC-P-24, AC-P-25)
│   │   └── bar.rs           # 탭 바 rendering (AC-P-27)
│   └── lib.rs
├── tests/
│   ├── integration_pane_core.rs       # AC-P-1 ~ AC-P-7 (통합)
│   ├── integration_key_bindings.rs    # AC-P-9a / AC-P-9b / AC-P-23
│   ├── integration_tabs.rs            # AC-P-10 / AC-P-11 / AC-P-24 / AC-P-25
│   ├── integration_persistence.rs     # AC-P-12 / AC-P-13 / AC-P-13a / AC-P-14 / AC-P-15
│   ├── integration_tmux_nested.rs     # AC-P-26 (v1.0.0 신규)
│   └── integration_terminal_core_regression.rs  # AC-P-16
├── benches/
│   ├── pane_split.rs        # AC-P-18
│   └── tab_switch.rs        # AC-P-19
└── examples/
    └── ghostty-spike.rs     # Manual (§8 checklist)
```

### 1.2 공통 fixture

```rust
// crates/moai-studio-ui/tests/common/mod.rs
pub struct PaneTestHarness {
    pub cx: TestAppContext,           // GPUI headless context
    pub temp_dir: TempDir,             // cwd / persistence 용
    pub tracing_buf: TracingTestBuf,   // warn! 로그 capture
}

impl PaneTestHarness {
    pub fn new_single_pane() -> Self { ... }
    pub fn new_3_level_split() -> Self { ... }
    pub fn new_9_tabs_mixed_panes() -> Self { ... }
    pub fn drive_key(&mut self, key: Keystroke) { ... }
    pub fn drive_mouse_click(&mut self, px: (f32, f32)) { ... }
    pub fn assert_tracing_warn_once(&self, substring: &str) { ... }
}
```

### 1.3 플랫폼 분기 매크로

```rust
#[cfg(target_os = "macos")]
macro_rules! platform_mod { () => { Modifiers::COMMAND }; }

#[cfg(target_os = "linux")]
macro_rules! platform_mod { () => { Modifiers::CONTROL }; }
```

---

## 2. Milestone MS-1 (Pane Core) AC 상세

### AC-P-1 — PaneTree 자료구조 기본 split

- **관련 REQ**: REQ-P-002 (group RG-P-1)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/src/panes/tree.rs` 내 `#[cfg(test)] mod tests` (test fn: `split_horizontal_from_leaf`)

**Given**:
- `let mut tree = PaneTree::Leaf(cx.new_entity(TerminalSurface::new_test()));`
- `let focused_id: PaneId = tree.root_pane_id();`

**When**:
- `tree.split_horizontal(focused_id).expect("split should succeed");`

**Then**:
```rust
match tree {
    PaneTree::Split { direction, ratio, first, second } => {
        assert_eq!(direction, SplitDirection::Horizontal);
        assert!((ratio - 0.5).abs() < f32::EPSILON);
        assert!(matches!(*first, PaneTree::Leaf(_)));
        assert!(matches!(*second, PaneTree::Leaf(_)));
    }
    _ => panic!("expected Split after split_horizontal"),
}
```
- `assert_eq!(tree.leaf_count(), 2);`
- `second` leaf 의 `TerminalSurface::pty_worker().is_spawned() == true` (검증 가능하려면 PtyWorker mock 또는 test hook 필요)

**Failure modes**:
- `ratio` 가 `0.5` 가 아닌 다른 기본값으로 설정되면 REQ-P-002 위반
- `first` 와 `second` 가 모두 leaf 가 아니면 (`Split` 가 중첩되면) REQ-P-002 위반 (split 은 leaf 만 대상)

---

### AC-P-2 — 3-level split 후 close propagation

- **관련 REQ**: REQ-P-003 (group RG-P-1)
- **Test category**: Unit + Integration (FD count assert)
- **Test location**:
  - Unit: `crates/moai-studio-ui/src/panes/tree.rs::tests::close_promotes_sibling`
  - Integration: `crates/moai-studio-ui/tests/integration_pane_core.rs::close_frees_pty_fds_within_1s`

**Given**:
- `let mut harness = PaneTestHarness::new_3_level_split();` (8 leaf)
- `let before_fd_count = count_open_ptmx_fds();`

**When**:
- `let target: PaneId = harness.tree().leaves()[3];`  // 4번째 leaf
- `harness.tree_mut().close_pane(target).expect("close should succeed");`
- `std::thread::sleep(Duration::from_secs(1));`  // drop propagation 대기

**Then**:
- `assert_eq!(harness.tree().leaf_count(), 7);`
- sibling 이 parent 위치로 승격되어 tree depth 감소 (rotation 검증)
- `let after_fd_count = count_open_ptmx_fds(); assert!(after_fd_count < before_fd_count);`
- 닫힌 pane 의 `PtyWorker::is_alive() == false`

**Failure modes**:
- 1초 지연에도 FD 가 회수 안 되면 PtyWorker drop 누락
- sibling 승격이 아니라 parent 가 통째로 삭제되면 tree 구조 불일치

---

### AC-P-3 — 단일 leaf close 시 무시

- **관련 REQ**: REQ-P-004 (group RG-P-1)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/src/panes/tree.rs::tests::close_last_leaf_is_noop`

**Given**:
- `let mut harness = PaneTestHarness::new_single_pane();`
- `let tree_before = harness.tree().clone();`

**When**:
- `let only_pane = harness.tree().root_pane_id();`
- `let result = harness.tree_mut().close_pane(only_pane);`

**Then**:
- `assert_eq!(result, Ok(()));`  // 경고 없이 무시
- `assert_eq!(harness.tree(), &tree_before);`  // 상태 변경 없음
- `harness.tracing_buf.assert_no_entries_matching("pane close");`  // 로그 없음

**Failure modes**:
- Err 반환 시 REQ-P-004 위반 ("경고 없이 무시")
- 로그 기록 시 REQ-P-004 위반

---

### AC-P-4 — 최소 pane 크기 위반 시 split 거부

- **관련 REQ**: REQ-P-011 (group RG-P-2)
- **Test category**: Unit + tracing subscriber
- **Test location**: `crates/moai-studio-ui/src/panes/constraints.rs::tests::split_rejected_on_min_size_violation`

**Given**:
```rust
let mut harness = PaneTestHarness::new_single_pane_with_size(60, 20);
// 60 cols × 20 rows. horizontal split 시 좌/우 각 30 cols → < MIN_COLS (40) → strict less
```

**When**:
- `let result = harness.tree_mut().split_horizontal(harness.tree().root_pane_id());`

**Then**:
- `assert!(matches!(result, Err(SplitError::MinSizeViolation { .. })));`
- `harness.tracing_buf.assert_warn_once_matching(r"split rejected: pane size constraint violated");`
- `assert_eq!(harness.tree().leaf_count(), 1);`  // 변경 없음

**경계 판정 (spec.md REQ-P-011)**:
- strict `< MIN_COLS` 또는 `< MIN_ROWS` 일 때만 거부
- 정확히 40 cols 가 되는 경우는 허용 (80 cols → 40/40 split 허용)
- 추가 케이스: `new_single_pane_with_size(80, 20)` 에서 split 성공 검증

**Failure modes**:
- 정확히 40 cols 경계에서 거부되면 strict 판정 위반
- warn 로그 미발생 시 REQ-P-011 위반

---

### AC-P-5 — 윈도우 resize 진행 중 깊은 pane 숨김

- **관련 REQ**: REQ-P-013 (group RG-P-2)
- **Test category**: Integration (headless resize simulation)
- **Test location**: `crates/moai-studio-ui/tests/integration_pane_core.rs::window_resize_hides_deepest_pane`

**Given**:
- `let mut harness = PaneTestHarness::new_3_level_split_with_window(800, 600);`

**When**:
```rust
harness.resize_window(400, 300);   // half size
```

**Then**:
- `assert_eq!(harness.tree().leaf_count(), 8);`  // 자료구조 유지
- `assert!(harness.tree().deepest_leaf_is_hidden());`  // 가장 깊은 pane 은 시각적으로 숨김
- `harness.resize_window(800, 600);`
- `assert!(!harness.tree().deepest_leaf_is_hidden());`  // 복원 시 재표시

**Failure modes**:
- 자료구조 축소되면 REQ-P-013 위반
- 윈도우 복원 후 재표시 안 되면 restore 로직 결함

---

### AC-P-6 — Divider drag clamping (REQ-P-005 + REQ-P-012 합동)

- **관련 REQ**: REQ-P-012 (group RG-P-2) + REQ-P-005 (group RG-P-1, boundary)
- **Test category**: Unit + Manual
- **Test location**: `crates/moai-studio-ui/src/panes/divider.rs::tests::drag_clamps_ratio_within_min_size`

**Given**:
- 2 sibling leaf (horizontal split, 총 100 cols, ratio = 0.5 → 50/50)

**When**:
```rust
// divider 를 오른쪽 끝까지 drag (왼쪽 pane 이 아주 커지도록)
let new_ratio = divider.on_drag(delta_px = 200.0, total_px = 200.0);
// 결과: 왼쪽 pane 100 cols, 오른쪽 pane 0 cols 가 되려는 상황
```

**Then**:
- `assert!((MIN_COLS as f32 / 200.0) <= new_ratio && new_ratio <= (1.0 - MIN_COLS as f32 / 200.0));`
  - 구체적으로: `0.2 <= new_ratio <= 0.8` (200 cols 총 폭, MIN=40)
- `assert!(new_ratio > 0.0 && new_ratio < 1.0);`  // REQ-P-005: 경계 제외
- 양 pane 모두 `MIN_COLS` (40) 이상 유지

**Failure modes**:
- `new_ratio == 1.0` 이면 REQ-P-005 위반
- `new_ratio * 200 < 40` 이면 REQ-P-012 clamp 실패

---

### AC-P-7 — next pane focus 이동 (in-order)

- **관련 REQ**: REQ-P-021 (group RG-P-3)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/src/panes/focus.rs::tests::next_pane_in_order`

**Given**:
- 3 leaf in-order: `[pane-A, pane-B, pane-C]`, `pane-A` focused

**When**:
- `harness.drive_key(Keystroke::mod_shift_bracket_right());`  // Cmd/Ctrl+Shift+]

**Then**:
- `assert_eq!(harness.focused_pane(), pane_B);`
- 추가 1회 더 입력: `assert_eq!(harness.focused_pane(), pane_C);`
- 추가 1회 더 (wrap around): `assert_eq!(harness.focused_pane(), pane_A);`
- `TerminalSurface::handle_key_down` 은 매 step 에서 **정확히 한 pane 에서만** 호출 (call counter 검증)

**Failure modes**:
- wrap around 미작동
- focused 아닌 pane 에서 `handle_key_down` 호출 시 REQ-P-020 위반

---

### AC-P-17 — PaneSplitter / ResizableDivider 추상 trait 존재

- **관련 REQ**: REQ-P-061 (group RG-P-7)
- **Test category**: Doc test + `cargo check`
- **Test location**: `crates/moai-studio-ui/src/panes/splitter.rs` (trait 정의 + doc test) + `crates/moai-studio-ui/tests/integration_pane_core.rs::abstract_traits_compile_without_impl`

**Given**:
- `panes` 모듈의 public API

**When**:
- `cargo check -p moai-studio-ui --lib`
- doc test: `cargo test -p moai-studio-ui --doc`

**Then**:
- `PaneSplitter` trait 이 public 으로 노출되고 `split_horizontal`, `split_vertical`, `close_pane`, `focus_pane` 메서드를 포함
- `ResizableDivider` trait 이 public 으로 노출되고 `on_drag`, `min_ratio_for` 포함
- Mock impl 이 존재하여 `cargo check` 통과 — **구체 구현체 (Custom vs gpui-component) 선택 없이도 빌드 가능**

**중요 (spec.md §16 #1 에 따라)**:
- 본 AC 는 plan spike 결정 **이전** 에도 통과 가능해야 함
- 검증은 "추상 trait 이 존재하는지" 까지만. 구체 구현체 교체 시 AC-P-1 ~ AC-P-7 재실행으로 동등성 확인

**Failure modes**:
- trait 이 crate-private 이면 REQ-P-061 위반
- trait 메서드 시그니처 누락 시 AC 실패

---

### AC-P-18 — Split 성능 (≤ 200ms)

- **관련 REQ**: spec.md §6.1
- **Test category**: Benchmark (criterion)
- **Test location**: `crates/moai-studio-ui/benches/pane_split.rs::bench_split_9_leaf`

**Given**:
- 9-leaf PaneTree (이미 3-level split 완료 상태)

**When**:
- `c.bench_function("split_9_leaf", |b| b.iter(|| { harness.split_horizontal(target_id).unwrap(); }))`

**Then**:
- criterion 의 **p99 latency ≤ 200 ms** (새 TerminalSurface 첫 프레임 paint 까지)
- `cargo bench -p moai-studio-ui --bench pane_split` 이 `200ms` threshold 를 넘지 않음

**Failure modes**:
- p99 > 200ms → PtyWorker spawn 또는 GPUI paint 병목. 프로파일링 후 최적화 필요

---

### AC-P-20 — ratio 경계값 거부 (negative assertion)

- **관련 REQ**: REQ-P-005 (group RG-P-1)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/src/panes/tree.rs::tests::ratio_boundary_rejected`

**Given**:
- 2-leaf horizontal split, ratio = 0.5

**When**:
- `let r1 = tree.set_ratio(split_node_id, 0.0);`
- `let r2 = tree.set_ratio(split_node_id, 1.0);`
- `let r3 = tree.set_ratio(split_node_id, -0.1);`
- `let r4 = tree.set_ratio(split_node_id, 1.5);`

**Then**:
- `assert!(matches!(r1, Err(RatioError::OutOfBounds)));`
- `assert!(matches!(r2, Err(RatioError::OutOfBounds)));`
- `assert!(matches!(r3, Err(RatioError::OutOfBounds)));`
- `assert!(matches!(r4, Err(RatioError::OutOfBounds)));`
- `assert_eq!(tree.get_ratio(split_node_id), Some(0.5));`  // 기존 값 유지

**Failure modes**:
- `Ok(())` 반환 시 REQ-P-005 위반 (negative assertion 실패)

---

### AC-P-21 — PaneConstraints 공개 API 불변성 (negative assertion)

- **관련 REQ**: REQ-P-014 (group RG-P-2)
- **Test category**: Public API surface check
- **Test location**: `crates/moai-studio-ui/tests/integration_pane_core.rs::pane_constraints_has_no_mutable_api`

**Given**:
- `moai-studio-ui` crate 의 public API

**When**:
- `cargo public-api -p moai-studio-ui --simplified > api.txt`
- 또는 수동 rustdoc HTML 파싱

**Then**:
```rust
// 존재해야 함
assert_api_contains("pub const PaneConstraints::MIN_COLS: u16 = 40");
assert_api_contains("pub const PaneConstraints::MIN_ROWS: u16 = 10");

// 존재하면 안 됨 (negative assertion)
assert_api_not_contains("pub fn PaneConstraints::new");
assert_api_not_contains("pub fn PaneConstraints::set_min_cols");
assert_api_not_contains("pub fn PaneConstraints::set_min_rows");
```

**Failure modes**:
- 가변 API 노출 시 REQ-P-014 위반 → MS-1 범위 외 기능이 유출됨

---

### AC-P-22 — 단일 focus 불변 (negative assertion)

- **관련 REQ**: REQ-P-024 (group RG-P-3)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/src/panes/focus.rs::tests::single_focus_invariant`

**Given**:
- 3-leaf PaneTree

**When**:
```rust
let sequence = vec![
    Event::Key(Keystroke::mod_shift_bracket_right()),  // next
    Event::MouseClick((200.0, 300.0)),                  // click pane-2
    Event::Key(Keystroke::mod_shift_bracket_left()),   // prev
    Event::MouseClick((500.0, 300.0)),                  // click pane-3
];
for ev in sequence {
    harness.drive(ev);
    let focused_count = harness.tree().leaves().iter().filter(|l| l.is_focused()).count();
    assert_eq!(focused_count, 1, "single focus invariant violated");
}
```

**Then**:
- 모든 시퀀스 단계에서 `focused_count == 1`

**Failure modes**:
- 2개 이상 pane 이 동시 focused → REQ-P-024 위반

---

### AC-P-23 — tmux prefix Ctrl+B 통과 (호환성)

- **관련 REQ**: REQ-P-033 (group RG-P-4)
- **Test category**: Manual + PTY feed 검증
- **Test location**: `crates/moai-studio-ui/tests/integration_key_bindings.rs::tmux_prefix_passes_to_pane`

**Given**:
- pane 내부에서 `tmux new-session` 실행 중
- pane focused

**When**:
- 사용자가 `Ctrl+B` 입력

**Then**:
- `Ctrl+B` 에 해당하는 byte (`\x02`) 가 pane 의 PTY master 로 쓰여짐을 assert
- host 앱은 `Ctrl+B` 를 소비하지 않음 (TabContainer / PaneTree 상태 변경 없음)
- tmux 프로세스는 다음 키를 대기 상태로 전환 (manual 검증)

**Failure modes**:
- host 가 `Ctrl+B` 를 intercept 하면 REQ-P-033 위반
- PTY feed 에 `\x02` 가 기록되지 않으면 키 이벤트 경로 결함

---

### AC-P-20 ~ AC-P-22 의 일관된 결과 (교차 참조)

AC-P-20 (ratio boundary), AC-P-21 (constraints immutable), AC-P-22 (single focus) 는 모두 MS-1 단계의 **negative assertion** AC 로, MS-1 구현 완료 시점에 동시 통과해야 한다. 어느 하나라도 실패하면 MS-1 → MS-2 전환 차단.

---

## 3. Milestone MS-2 (Tabs) AC 상세

### AC-P-8 — 탭 전환 시 last-focused-pane 복원

- **관련 REQ**: REQ-P-023 (group RG-P-3)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/src/tabs/container.rs::tests::tab_switch_restores_focus`

**Given**:
- Tab A: 2 pane (pane-A1, pane-A2), pane-A2 focused
- Tab B: 2 pane (pane-B1, pane-B2), pane-B1 focused
- `active_tab_idx = 0` (Tab A)

**When**:
```rust
harness.drive_key(Keystroke::mod_digit(2));  // Cmd/Ctrl+2 → Tab B
assert_eq!(harness.active_tab_idx(), 1);
assert_eq!(harness.focused_pane_in_active_tab(), pane_B1);

harness.drive_key(Keystroke::mod_digit(1));  // Cmd/Ctrl+1 → Tab A
```

**Then**:
- `assert_eq!(harness.active_tab_idx(), 0);`
- `assert_eq!(harness.focused_pane_in_active_tab(), pane_A2);`  // 복원

**Failure modes**:
- 전환 후 탭의 첫 leaf 로 focus 가 초기화되면 REQ-P-023 위반
- `last_focused_pane` 필드가 업데이트 안 되면 복원 실패

---

### AC-P-9a — macOS 키 바인딩 전체

- **관련 REQ**: REQ-P-030 / REQ-P-031 / REQ-P-032 (group RG-P-4, macOS 컬럼)
- **Test category**: Integration (macOS CI job only)
- **Test location**: `crates/moai-studio-ui/tests/integration_key_bindings.rs::macos_key_bindings` (gated by `#[cfg(target_os = "macos")]`)

**Given**:
- macOS 14 runner, `PaneTestHarness::new_single_pane()`

**When**:
```rust
let sequence = vec![
    Keystroke::cmd('t'),               // new tab
    Keystroke::cmd('\\'),              // horizontal split
    Keystroke::cmd_shift('\\'),        // vertical split
    Keystroke::cmd('w'),               // close pane
    Keystroke::cmd_shift('['),         // prev pane
];
for k in sequence {
    harness.drive_key(k);
    harness.assert_no_pty_echo(k);  // keystroke 가 pane 내부로 전달 안 됨
}
```

**Then**:
- Cmd+T 후 `assert_eq!(harness.tabs().len(), 2);`
- Cmd+\\ 후 PaneTree 가 horizontal Split 로 변환
- Cmd+Shift+\\ 후 한 leaf 가 vertical Split 로 재분할
- Cmd+W 후 pane 1개 감소
- Cmd+Shift+\[ 후 focus 이동
- **Negative assertion**: 각 keystroke 에 대응하는 byte sequence 가 PTY feed 에 기록되지 않음

**Failure modes**:
- host 가 intercept 못 하면 REQ-P-032 위반

---

### AC-P-9b — Linux 키 바인딩 전체

- **관련 REQ**: REQ-P-030 / REQ-P-031 / REQ-P-032 (group RG-P-4, Linux 컬럼)
- **Test category**: Integration (Linux CI job only)
- **Test location**: `crates/moai-studio-ui/tests/integration_key_bindings.rs::linux_key_bindings` (gated by `#[cfg(target_os = "linux")]`)

**Given**:
- Ubuntu 22.04 runner, `PaneTestHarness::new_single_pane()`

**When**:
```rust
let sequence = vec![
    Keystroke::ctrl('t'),              // new tab
    Keystroke::ctrl('\\'),             // horizontal split
    Keystroke::ctrl_shift('\\'),       // vertical split
    Keystroke::ctrl('w'),              // close pane
    Keystroke::ctrl_shift('['),        // prev pane
];
```

**Then** (AC-P-9a 와 동일 동작 + 플랫폼 modifier 치환):
- Ctrl+T → 새 탭
- Ctrl+\\ → horizontal split
- Ctrl+Shift+\\ → vertical split
- Ctrl+W → close pane (pane 내부 readline word-delete 는 수신 안 함, R-9 trade-off)
- Ctrl+Shift+\[ → prev pane
- **Negative assertion**: 각 keystroke 에 대응하는 byte sequence 가 PTY feed 에 기록되지 않음

**Trade-off 주석 (v1.0.0 Nm-3, spec.md §6.4 / R-9)**:
- Ctrl+W 는 shell 의 `unix-word-rubout` 과 충돌. 본 AC 는 host 바인딩 우선 경로를 전제함
- Spike 4 결과가 "shift-escalation" 경로를 선택한 경우, plan 단계에서 이 AC 의 Ctrl+W 항목이 Ctrl+Shift+W 로 갱신됨

**Failure modes**:
- Linux 에서 host 가 intercept 못 하면 REQ-P-032 위반
- macOS 키 조합 (Cmd) 이 Linux runner 에서 활성화되면 플랫폼 분기 실패

---

### AC-P-10 — 9개 탭 생성

- **관련 REQ**: REQ-P-042 (group RG-P-5)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/src/tabs/container.rs::tests::create_nine_tabs`

**Given**:
- `PaneTestHarness::new_empty_workspace()` (탭 0개)

**When**:
```rust
for _ in 0..9 {
    harness.drive_key(Keystroke::mod_char('t'));
}
```

**Then**:
- `assert_eq!(harness.tabs().len(), 9);`
- `assert_eq!(harness.active_tab_idx(), 8);`  // 0-based 마지막
- 각 탭의 `pane_tree` 는 단일 leaf
- 각 탭의 `title` 은 `"untitled"` 또는 초기 cwd.file_name() (spec.md REQ-P-040)

**Failure modes**:
- `active_tab_idx` 가 0 에 머무르면 REQ-P-042 위반 ("새 탭으로 변경")

---

### AC-P-11 — 탭 전환 시 PaneTree 보존

- **관련 REQ**: REQ-P-041 (group RG-P-5)
- **Test category**: Integration (GPUI headless)
- **Test location**: `crates/moai-studio-ui/tests/integration_tabs.rs::tab_switch_preserves_pane_tree_identity`

**Given**:
- Tab A: 2 pane, Tab B: 4 pane, Tab C: 3 pane, ..., 총 9 탭 생성 (각 탭마다 다른 pane 구조)

**When**:
- Cmd/Ctrl+1 → Cmd/Ctrl+5 → Cmd/Ctrl+9 → Cmd/Ctrl+1 순서 전환

**Then**:
```rust
// 전환 전/후 각 탭의 PaneTree 구조 해시 + TerminalSurface Entity ID 동일성 검증
let before = harness.snapshot_all_tab_trees();
// ... 전환 시퀀스 ...
let after = harness.snapshot_all_tab_trees();
assert_eq!(before, after, "tab switch must preserve PaneTree structure and Entity identity");
```

**Failure modes**:
- Entity ID 가 전환 시마다 재생성되면 TerminalSurface 의 scrollback 손실 + REQ-P-041 위반

---

### AC-P-24 — Empty State CTA (REQ-P-041 negative assertion)

- **관련 REQ**: REQ-P-041 (group RG-P-5)
- **Test category**: Unit + GPUI headless render
- **Test location**: `crates/moai-studio-ui/src/tabs/container.rs::tests::empty_state_cta_shown_when_no_tabs`

**Given**:
- 새로 생성된 workspace, `TabContainer::tabs.is_empty() == true`

**When**:
- RootView 렌더 (GPUI headless)

**Then**:
- `assert!(harness.has_element_with_id("empty-state-cta"));`
- `assert!(!harness.has_element_with_id("tab-bar"));`  // 탭 바 미렌더
- Cmd/Ctrl+T 1회 입력:
  - `assert!(!harness.has_element_with_id("empty-state-cta"));`
  - `assert!(harness.has_element_with_id("tab-bar"));`

**Failure modes**:
- `tabs.is_empty()` 인데도 탭 바 렌더 시 REQ-P-041 위반

---

### AC-P-25 — 10번째 이상 탭은 1~9 단축키로 접근 불가

- **관련 REQ**: REQ-P-045 (group RG-P-5)
- **Test category**: Integration
- **Test location**: `crates/moai-studio-ui/tests/integration_tabs.rs::ninth_plus_tabs_accessible_only_via_mouse_or_brace`

**Given**:
- 12개 탭 생성 상태, `active_tab_idx = 0`

**When**:
```rust
for digit in 1..=9 {
    harness.drive_key(Keystroke::mod_digit(digit));
    assert_eq!(harness.active_tab_idx(), digit - 1);  // 0~8 탭만
}

// 10번째 탭 (index 9) 은 Cmd/Ctrl+0 또는 1~9 로 접근 불가
// 마우스 클릭으로 시도
harness.drive_mouse_click_on_tab(9);
assert_eq!(harness.active_tab_idx(), 9);

// Cmd/Ctrl+} 로 11번째 탭 (index 10) 으로 이동
harness.drive_key(Keystroke::mod_right_brace());
assert_eq!(harness.active_tab_idx(), 10);
```

**Then**:
- 위 assertion 모두 통과

**Failure modes**:
- Cmd/Ctrl+1 ~ Cmd/Ctrl+9 로 10번째 이상 탭에 접근 가능하면 REQ-P-045 위반

---

### AC-P-26 — tmux 중첩 시 OS-level 우선 처리 (v1.0.0 Nm-1 해소)

- **관련 REQ**: REQ-P-034 (group RG-P-4, Optional)
- **Test category**: Integration (macOS + Linux 양 플랫폼)
- **Test location**: `crates/moai-studio-ui/tests/integration_tmux_nested.rs::nested_tmux_does_not_receive_host_keystroke`

**Given**:
- 단일 탭 + 단일 pane
- pane 의 shell 에서 `tmux new-session -s nested` 실행 (tmux 가 PTY 내부에서 동작 중)
- pane focused, tmux 세션 active

**When**:
- 사용자가 새 탭 단축키 입력 (macOS: Cmd+T, Linux: Ctrl+T)

**Then**:
```rust
// (a) pane 내부 tmux 는 key event 를 수신하지 않음 — byte-level assertion
let pty_master_writes: Vec<u8> = harness.captured_pty_writes_since_last_checkpoint();
// Cmd+T / Ctrl+T 에 해당하는 escape sequence 패턴이 존재하지 않아야 함
// 일반적으로 'T' (0x54) 또는 Ctrl+T (0x14) 가 stream 에 포함되지 않음
let has_ctrl_t = pty_master_writes.iter().any(|&b| b == 0x14);
let has_raw_t = pty_master_writes.iter().any(|&b| b == b'T');
assert!(!has_ctrl_t && !has_raw_t,
    "pane PTY received host keystroke bytes: {:?}", pty_master_writes);

// (b) host 앱은 새 탭을 생성
assert_eq!(harness.tabs().len(), 2);
assert_eq!(harness.active_tab_idx(), 1);
```

**검증 절차 상세**:
1. Harness 가 PTY master 의 write stream 을 buffer 에 기록 (checkpoint 기반)
2. Keystroke drive 직전 checkpoint
3. Keystroke drive 후 checkpoint
4. 두 checkpoint 사이 byte stream 에서 Cmd+T / Ctrl+T byte 패턴 검색
5. 동시에 host 앱의 `TabContainer` 상태 검사

**Failure modes**:
- tmux 가 Cmd+T / Ctrl+T 를 수신하여 tmux 내부 새 윈도우 생성 명령 대기 상태로 진입 시 REQ-P-034 위반 (host 우선 처리 실패)
- host 탭이 생성되지 않으면 dual failure: tmux 는 못 받고 host 도 못 받음
- **Flakiness 방지**: tmux session 초기화 완료 대기 (`sleep 500ms` 또는 tmux status response polling) 후 keystroke drive

**주의사항**:
- Linux 에서 pane 내부 tmux 가 이미 존재한다면 host 의 Ctrl+W (close pane) 도 동일한 원리로 OS-level 우선. AC-P-9b 와 의미상 중첩되지만, 본 AC 는 **tmux 가 PTY 내부에서 동작 중인 특수 상황** 을 명시적으로 검증.

---

### AC-P-27 — 탭 바 active 시각 구분 (v1.0.0 Nm-2 해소)

- **관련 REQ**: REQ-P-044 (group RG-P-5, State-Driven)
- **Test category**: GPUI snapshot test 또는 Unit (styled element 속성 검증)
- **Test location**: `crates/moai-studio-ui/src/tabs/bar.rs::tests::active_tab_has_bold_and_background_token`

**Given**:
- `TabContainer.tabs.len() >= 2`, 예: 3개 탭 생성
- `active_tab_idx = 0`
- design token `toolbar.tab.active.background` 가 `.moai/design/v3/system.md` Toolbar 섹션에 정의되어 있음 (plan 단계에서 추가)

**When**:
- RootView 가 탭 바를 렌더 (GPUI element tree 생성)

**Then**:
```rust
let tab_bar_elements: Vec<StyledTab> = harness.extract_tab_bar_styled();
assert_eq!(tab_bar_elements.len(), 3);

// 활성 탭 (index 0): 두 속성 동시 충족
let active_tab = &tab_bar_elements[0];
assert_eq!(active_tab.background_color, design_token("toolbar.tab.active.background"),
    "active tab must use toolbar.tab.active.background design token");
assert_eq!(active_tab.font_weight, FontWeight::Bold,
    "active tab must use bold font weight");

// 비활성 탭 (index 1, 2): 두 속성 모두 해당하지 않음
for inactive_tab in &tab_bar_elements[1..] {
    assert_ne!(inactive_tab.background_color, design_token("toolbar.tab.active.background"),
        "inactive tab must NOT use active background token");
    assert_ne!(inactive_tab.font_weight, FontWeight::Bold,
        "inactive tab must NOT use bold font weight");
}
```

**검증 옵션** (plan spike 1 결과에 따라 택1):
- **옵션 A (GPUI snapshot)**: `cargo test -p moai-studio-ui --features snapshot` 로 snapshot 비교 (RGB 색상 + font weight 메타데이터 포함)
- **옵션 B (Unit, element 속성 추출)**: GPUI `Element::styled_values()` 또는 테스트 hook 으로 속성 직접 추출

**Failure modes**:
- (a) background color 는 맞고 bold 는 아니면 → REQ-P-044 의 "(a) AND (b) 두 조건 동시 충족" 위반 (AND 조건)
- (b) 비활성 탭이 active 토큰을 사용하면 → 시각적 구분 상실
- **Flakiness 방지**: design token 값 하드코딩 금지 — 반드시 `design_token()` 함수로 lookup

**주의사항 (spec.md REQ-P-044)**:
- 정확한 RGB 값은 plan 단계에서 `.moai/design/v3/system.md` 에 추가되며, 본 AC 는 **토큰 참조 경로** 만 고정함
- 추후 toolbar 토큰 값이 변경되어도 AC 는 재작성 불필요 (design token indirection)

---

## 4. Milestone MS-3 (Persistence) AC 상세

### AC-P-12 — 저장 (schema v1 + atomic write, negative: no VT state)

- **관련 REQ**: REQ-P-050 / REQ-P-051 / REQ-P-052 / REQ-P-055 (group RG-P-6)
- **Test category**: Integration (tempdir + fs assertion, macOS + Linux 각각)
- **Test location**: `crates/moai-studio-ui/tests/integration_persistence.rs::save_tabs_atomic_without_vt_state`

**Given**:
- `PaneTestHarness::new_with_tempdir()` — 임시 `$HOME` 에 3 탭 × 각 2 pane 구성

**When**:
- 정상 종료 sequence trigger (`harness.trigger_normal_shutdown()`)

**Then**:
```rust
let path = harness.tempdir().join(".moai/studio/panes-{ws_id}.json");
assert!(path.exists(), "panes JSON must be created");

let json = fs::read_to_string(&path)?;
let parsed: serde_json::Value = serde_json::from_str(&json)?;

// Positive assertions
assert_eq!(parsed["$schema"], "moai-studio/panes-v1");
assert_eq!(parsed["tabs"].as_array().unwrap().len(), 3);
assert_eq!(parsed["active_tab_idx"], harness.active_tab_idx());
// cwd, last_focused_pane 검증

// Atomic write 검증: 임시 파일은 rename 후 존재하지 않아야 함
let tmp_files: Vec<_> = fs::read_dir(path.parent().unwrap())?
    .filter_map(|e| e.ok())
    .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
    .collect();
assert!(tmp_files.is_empty(), "atomic write leftover .tmp files: {:?}", tmp_files);

// Negative assertions (REQ-P-055)
let json_str = json.as_str();
assert!(!json_str.contains("scrollback"), "scrollback must not be serialized");
assert!(!json_str.contains("selection"), "selection must not be serialized");
assert!(!json_str.contains("cursor_position"), "cursor_position must not be serialized");
assert!(!json_str.contains("vt_state"), "vt_state must not be serialized");
```

**플랫폼 분기**:
- macOS / Linux runner 에서 동일한 AC 가 각각 실행 (spec.md §6.4 G7)

**Failure modes**:
- `.tmp` 파일이 남으면 atomic rename 누락
- `scrollback` 등의 key 포함 시 REQ-P-055 위반

---

### AC-P-13 — 복원 (정상 cwd)

- **관련 REQ**: REQ-P-053 (group RG-P-6)
- **Test category**: Integration
- **Test location**: `crates/moai-studio-ui/tests/integration_persistence.rs::restore_from_valid_persistence`

**Given**:
- AC-P-12 에서 생성된 JSON 파일
- 모든 leaf pane 의 `cwd` 는 유효한 디렉터리 (tempdir 하위 실재 경로)

**When**:
- `let restored_harness = PaneTestHarness::restore_from_tempdir(saved_tempdir);`

**Then**:
```rust
assert_eq!(restored_harness.tabs().len(), 3);
assert_eq!(restored_harness.active_tab_idx(), saved_active_tab_idx);

for (saved_tab, restored_tab) in saved.tabs().iter().zip(restored_harness.tabs().iter()) {
    assert_eq!(saved_tab.pane_tree_structure_hash(), restored_tab.pane_tree_structure_hash());
    assert_eq!(saved_tab.last_focused_pane, restored_tab.last_focused_pane);
    for (saved_leaf, restored_leaf) in saved_tab.leaves().zip(restored_tab.leaves()) {
        assert_eq!(saved_leaf.cwd(), restored_leaf.cwd());
        assert!(restored_leaf.pty_worker().is_spawned(),
            "each leaf must have a fresh PtyWorker spawned with saved cwd");
    }
}
```

**Failure modes**:
- cwd 가 복원 안 되면 REQ-P-053 위반
- Entity ID 는 재생성되지만 구조 hash 는 동일해야 함

---

### AC-P-13a — cwd fallback (REQ-P-056, v1.0.0 NM-1 해소 REQ-P-057 → REQ-P-056 rename)

- **관련 REQ**: REQ-P-056 (group RG-P-6, v1.0.0 rename)
- **Test category**: Unit + tempdir + tracing subscribe
- **Test location**: `crates/moai-studio-ui/tests/integration_persistence.rs::cwd_fallback_to_home_on_missing_dir`

**Given**:
```rust
let saved_cwd = tempdir.path().join("will-be-deleted");
fs::create_dir_all(&saved_cwd)?;
// ... 저장 ...
fs::remove_dir_all(&saved_cwd)?;  // 재시작 전 삭제
```

**When**:
- `let restored = PaneTestHarness::restore_from_tempdir(tempdir);`

**Then**:
```rust
let target_leaf = restored.find_leaf_with_original_cwd(&saved_cwd);
assert_eq!(target_leaf.cwd(), std::env::var("HOME")?);
restored.tracing_buf.assert_warn_once_matching(
    r"pane cwd fallback: .* → \$HOME \(reason: not_found\)"
);

// 다른 pane 의 복원은 영향받지 않음
let other_leaves = restored.leaves_excluding(&target_leaf);
for leaf in other_leaves {
    assert_eq!(leaf.cwd(), saved_cwds[&leaf.id]);
}
```

**추가 케이스 (permission_denied, not_a_dir)**:
- `chmod 000` 으로 접근 권한 제거 → `reason: permission_denied`
- 경로를 파일로 대체 → `reason: not_a_dir`

**Failure modes**:
- 복원 자체가 실패 (panic / error) 하면 REQ-P-056 위반 (계속 진행해야 함)
- warn 로그 없으면 관측성 요구사항 §6.5 위반

---

### AC-P-14 — Schema version mismatch

- **관련 REQ**: REQ-P-054 (group RG-P-6)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/tests/integration_persistence.rs::schema_mismatch_triggers_fallback`

**Given**:
- Tempdir 에 `panes-ws-test.json` 을 수동 작성: `{"$schema": "moai-studio/panes-v2", ...}` (미래 버전)

**When**:
- `let restored = PaneTestHarness::restore_from_tempdir(tempdir);`

**Then**:
- `assert_eq!(restored.tabs().len(), 1);`  // 단일 탭 fallback
- `assert_eq!(restored.tabs()[0].leaf_count(), 1);`  // 단일 leaf
- `restored.tracing_buf.assert_warn_once_matching(r"schema version mismatch");`

---

### AC-P-15 — JSON parse failure + .corrupt rename

- **관련 REQ**: REQ-P-054 (group RG-P-6)
- **Test category**: Unit
- **Test location**: `crates/moai-studio-ui/tests/integration_persistence.rs::parse_failure_triggers_fallback_and_corrupt_rename`

**Given**:
- Tempdir 에 손상된 JSON: `{"broken": incomplete`

**When**:
- `let restored = PaneTestHarness::restore_from_tempdir(tempdir);`

**Then**:
- `assert_eq!(restored.tabs().len(), 1);`  // 단일 탭 fallback
- `restored.tracing_buf.assert_warn_once_matching(r"panes file parse failed");`
- 원본 파일은 `panes-ws-test.json.corrupt` 로 rename 됨:
  ```rust
  let corrupt_path = tempdir.path().join(".moai/studio/panes-ws-test.json.corrupt");
  assert!(corrupt_path.exists());
  let primary_path = tempdir.path().join(".moai/studio/panes-ws-test.json");
  assert!(!primary_path.exists() || primary_path.metadata()?.len() < 100,
      "primary path should be absent or replaced with fresh default");
  ```

---

## 5. 교차 Milestone AC

### AC-P-16 — Terminal Core regression 없음

- **관련 REQ**: REQ-P-060 (group RG-P-7)
- **Test category**: CI gate (mandatory on every merge)
- **Test location**: `.github/workflows/ci-rust.yml` 의 `terminal-core-regression` job

**Given**:
- MS-1 / MS-2 / MS-3 구현 완료 상태

**When**:
- `cargo test -p moai-studio-terminal`

**Then**:
- SPEC-V3-002 의 74 tests 모두 exit 0
- 기존 공개 API (Pty trait, PtyWorker, VtState, PtyEvent, TerminalSurface) 시그니처 변경 없음 (`cargo public-api --simplified` diff 0)

**Failure modes**:
- SPEC-V3-002 API 변경 시 본 SPEC PR 자동 rejection

---

## 6. Acceptance 실행 순서 및 의존성

### 6.1 Milestone 전환 Gate

| 전환 | 실행해야 할 AC |
|------|----------------|
| MS-0 → MS-1 | AC-P-16 (Terminal Core regression) + AC-P-17 (추상 trait 존재) |
| MS-1 → MS-2 | AC-P-1 ~ AC-P-7, AC-P-17, AC-P-18, AC-P-20, AC-P-21, AC-P-22, AC-P-23, AC-P-9a (MS-1 해당 부분만), AC-P-9b (MS-1 해당 부분만), AC-P-16 |
| MS-2 → MS-3 | MS-1 전체 AC + AC-P-8, AC-P-9a (전체), AC-P-9b (전체), AC-P-10, AC-P-11, AC-P-19, AC-P-24, AC-P-25, AC-P-26, AC-P-27 |
| MS-3 → Sync | 전체 29 AC + AC-P-12 ~ AC-P-15, AC-P-13a |

### 6.2 Regression gate (매 CI run)

모든 milestone commit 은 이전 milestone 의 AC 전체를 재실행해야 한다 (spec.md §11.3 A-2).

### 6.3 플랫폼별 재실행

AC-P-9a / AC-P-9b / AC-P-12 / AC-P-13 / AC-P-13a / AC-P-26 은 macOS runner + Linux runner 각각 실행 (spec.md §6.4 G7).

---

## 7. CI 통합

### 7.1 GitHub Actions workflow 매핑

```
.github/workflows/ci-v3-pane.yml
├── job: unit-tests (matrix: [macos-14, ubuntu-22.04])
│   ├── cargo test -p moai-studio-ui --lib
│   └── 실행 AC: AC-P-1, AC-P-3, AC-P-4, AC-P-7, AC-P-8, AC-P-10,
│                AC-P-17, AC-P-20, AC-P-21, AC-P-22, AC-P-24
│
├── job: integration-tests (matrix: [macos-14, ubuntu-22.04])
│   ├── cargo test -p moai-studio-ui --test 'integration_*'
│   └── 실행 AC: AC-P-2, AC-P-5, AC-P-9a (macOS only),
│                AC-P-9b (Linux only), AC-P-11, AC-P-12, AC-P-13,
│                AC-P-13a, AC-P-14, AC-P-15, AC-P-25, AC-P-26
│
├── job: snapshot-tests (ubuntu-22.04)
│   ├── cargo test -p moai-studio-ui --features snapshot
│   └── 실행 AC: AC-P-27 (옵션 A 선택 시)
│
├── job: benches (ubuntu-22.04, PR 에서는 smoke only)
│   ├── cargo bench -p moai-studio-ui
│   └── 실행 AC: AC-P-18, AC-P-19
│
└── job: terminal-core-regression (ubuntu-22.04)
    ├── cargo test -p moai-studio-terminal
    └── 실행 AC: AC-P-16
```

### 7.2 Manual verification CI (비자동)

AC-P-6 (divider drag manual), AC-P-23 (tmux prefix manual) 은 CI 가 아닌 개발자 로컬 + release 전 QA 에서 실행.

---

## 8. 수동 검증 체크리스트

다음 항목은 `cargo run --example ghostty-spike` 로 실제 앱을 띄운 후 사람이 확인:

- [ ] macOS 에서 Cmd+\\ 로 좌/우 분할 후 우측 pane 에서 Cmd+Shift+\\ 로 상/하 분할
- [ ] Linux 에서 Ctrl+\\ / Ctrl+Shift+\\ 동작 확인 (R-9 trade-off 수용)
- [ ] 9개 탭 생성 후 각 Cmd/Ctrl+1~9 로 탭 전환
- [ ] divider 를 마우스 drag 하여 비율 조정, 최소 크기에서 clamp 발생 (shake 애니메이션 시각 확인, spec.md §6.3)
- [ ] VoiceOver (macOS) 또는 Orca (Linux) 에서 pane focus 변경 시 음성 안내
- [ ] 3탭 × 각 2pane 상태로 종료 후 재시작 — 배치 복원 확인
- [ ] cwd 를 의도적으로 삭제 후 재시작 — `$HOME` fallback + warn 로그 확인
- [ ] pane 내부에서 `tmux new-session` 실행 후 Cmd/Ctrl+T 입력 — tmux 는 반응 없음, host 가 새 탭 생성 (AC-P-26 수동 재현)
- [ ] 탭 바 렌더 — active 탭이 bold + 별도 background color 로 시각 구분 (AC-P-27 수동 재현)

### 8.1 스크린샷 비교 기준

- Active tab 배경색: design token `toolbar.tab.active.background` (plan 단계에서 RGB 확정)
- Inactive tab 배경색: design token `toolbar.tab.inactive.background`
- Font weight 차이: bold (weight ≥ 600) vs regular (weight ≤ 500)

---

## 9. 추적 가능성 (REQ → AC 매핑)

| Milestone | Requirement Group | REQ | 대응 AC |
|-----------|-------------------|-----|---------|
| MS-1 | RG-P-1 | REQ-P-001 | AC-P-1 (간접, PaneTree 자료구조 정의) |
| MS-1 | RG-P-1 | REQ-P-002 | AC-P-1 |
| MS-1 | RG-P-1 | REQ-P-003 | AC-P-2 |
| MS-1 | RG-P-1 | REQ-P-004 | AC-P-3 |
| MS-1 | RG-P-1 | REQ-P-005 | AC-P-6, AC-P-20 |
| MS-1 | RG-P-2 | REQ-P-010 | AC-P-4 (간접, MIN_COLS/MIN_ROWS 상수 참조) |
| MS-1 | RG-P-2 | REQ-P-011 | AC-P-4 |
| MS-1 | RG-P-2 | REQ-P-012 | AC-P-6 |
| MS-1 | RG-P-2 | REQ-P-013 | AC-P-5 |
| MS-1 | RG-P-2 | REQ-P-014 | AC-P-21 |
| MS-1 | RG-P-3 | REQ-P-020 | AC-P-7, AC-P-22 (간접) |
| MS-1 | RG-P-3 | REQ-P-021 | AC-P-7 |
| MS-1 | RG-P-3 | REQ-P-022 | AC-P-22 (mouse click 경로 포함) |
| MS-2 | RG-P-3 | REQ-P-023 | AC-P-8 |
| MS-1 | RG-P-3 | REQ-P-024 | AC-P-22 |
| MS-1+MS-2 | RG-P-4 | REQ-P-030 | AC-P-9a, AC-P-9b |
| MS-1+MS-2 | RG-P-4 | REQ-P-031 | AC-P-9a, AC-P-9b |
| MS-1+MS-2 | RG-P-4 | REQ-P-032 | AC-P-9a, AC-P-9b (negative assert) |
| MS-1 | RG-P-4 | REQ-P-033 | AC-P-23 |
| MS-2 | RG-P-4 | REQ-P-034 | **AC-P-26** (v1.0.0 신규) |
| MS-2 | RG-P-5 | REQ-P-040 | AC-P-10 (간접) |
| MS-2 | RG-P-5 | REQ-P-041 | AC-P-11, AC-P-24 |
| MS-2 | RG-P-5 | REQ-P-042 | AC-P-10 |
| MS-2 | RG-P-5 | REQ-P-043 | (간접, MS-3 의 종료 sequence 로 검증) |
| MS-2 | RG-P-5 | REQ-P-044 | **AC-P-27** (v1.0.0 신규) |
| MS-2 | RG-P-5 | REQ-P-045 | AC-P-25 |
| MS-3 | RG-P-6 | REQ-P-050 | AC-P-12 |
| MS-3 | RG-P-6 | REQ-P-051 | AC-P-12 |
| MS-3 | RG-P-6 | REQ-P-052 | AC-P-12 |
| MS-3 | RG-P-6 | REQ-P-053 | AC-P-13 |
| MS-3 | RG-P-6 | REQ-P-054 | AC-P-14, AC-P-15 |
| MS-3 | RG-P-6 | REQ-P-055 | AC-P-12 (negative assert) |
| MS-3 | RG-P-6 | REQ-P-056 | AC-P-13a (v1.0.0 NM-1 rename) |
| 전체 | RG-P-7 | REQ-P-060 | AC-P-16 |
| MS-1 | RG-P-7 | REQ-P-061 | AC-P-17 |
| MS-1 | RG-P-7 | REQ-P-062 | AC-P-6 (간접, ResizableDivider 경로) |
| 전체 | RG-P-7 | REQ-P-063 | AC-P-17 (추상 trait 기반 통과성) |
| MS-1 | §6.1 | 성능 (split) | AC-P-18 |
| MS-2 | §6.1 | 성능 (탭 전환) | AC-P-19 |

### 9.1 Orphan REQ (직접 AC 없음, 간접 검증)

- **REQ-P-001** (PaneTree 자료형 존재): AC-P-1 의 `match tree { PaneTree::Split { .. } => ... }` 패턴 매칭으로 간접 검증
- **REQ-P-010** (PaneConstraints 존재): AC-P-4 의 MIN_COLS/MIN_ROWS 상수 참조로 간접 검증
- **REQ-P-020** (단일 active pane): AC-P-7 + AC-P-22 로 간접 검증
- **REQ-P-022** (mouse click → active): AC-P-22 의 sequence 내 `MouseClick` 이벤트로 간접 검증
- **REQ-P-040** (TabContainer 자료형 존재): AC-P-10 의 탭 생성 경로로 간접 검증
- **REQ-P-043** (탭 close 경로): AC-P-12 의 종료 sequence 전 탭 close 동작으로 간접 검증
- **REQ-P-062** (ResizableDivider trait 존재): AC-P-6 의 divider drag 경로로 간접 검증

이들 orphan REQ 는 자료구조 선언적 요구사항이므로 구현 존재 자체가 AC 에서 자동 검증된다.

---

## 10. 변경 이력

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0 | 2026-04-24 | 초안 작성. spec.md v1.0.0 §10 AC 표 (29 개) 를 실행 가능한 Given/When/Then 시나리오로 확장. AC-P-26 / AC-P-27 의 검증 코드 상세 기술 (v1.0.0 Nm-1 / Nm-2 해소). AC-P-13a 의 REQ 참조를 REQ-P-057 → REQ-P-056 갱신 (NM-1 해소). SPEC-V3-002 acceptance.md 구조 (section 0 ~ 14) 에 기반하되 본 SPEC 의 milestone 3 분할 특성에 맞게 §2 / §3 / §4 로 재조직. Manual verification 체크리스트 (§8) 및 CI 매핑 (§7) 추가. |

---

Version: 1.0.0 · 2026-04-24 · approved
