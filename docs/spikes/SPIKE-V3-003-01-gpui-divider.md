# SPIKE-V3-003-01 — GPUI 0.2.2 Divider Drag API 검증

**Date**: 2026-04-24
**Context**: SPEC-V3-003 Plan Spike 1 (plan.md §3)
**Blocks**: T4 (PaneSplitter 구체 구현), T5 (ResizableDivider 구체 구현)
**Decision authority**: MoAI orchestrator (research only — no code change in main repo)

---

## 1. 목표

GPUI 0.2.2 의 mouse event 체인으로 divider 의 drag → ratio 갱신 → re-paint 왕복을 **자체 구현** 할 수 있는지 확인. 구현 불가 시 Spike 2 (longbridge/gpui-component) 로 escalate.

## 2. 성공 기준 (plan.md §3 Spike 1)

- [x] 200 LOC 이하 구현으로 2-pane 수평 split 에서 divider drag → ratio 갱신 → frame 재갱신 가능성 확인
- [x] 드래그 중 frame rate ≥ 60 fps 유지 가능성 확인 (architecture 레벨)
- [x] GPUI 0.2.2 native API only (외부 crate 의존성 없음)

---

## 3. 조사 방법

1. **Context7 MCP** — `/websites/rs_gpui_gpui` (4718 snippets, source reputation High) 조회.
2. **docs.rs WebFetch** — `https://docs.rs/gpui/0.2.2/gpui/trait.InteractiveElement.html`, `https://docs.rs/gpui/0.2.2/gpui/struct.MouseMoveEvent.html`.
3. **Repo grep** — 현재 `moai-studio-ui` 의 GPUI 사용 패턴 (`crates/moai-studio-ui/src/lib.rs:165`, `:176`) 에서 `on_mouse_down` + `MouseButton::Left` + `cx.listener(...)` 기존 패턴 확인.
4. **Cargo.lock 검증** — `gpui v0.2.2` 가 crates.io registry 에서 직접 pull (git main 이 아님). SPEC-V3-001 에서 결정된 "안정 경로" 준수.

## 4. 발견된 API 카탈로그

### 4.1 InteractiveElement trait (가장 중요)

`Div`, `Img`, `Svg`, `UniformList`, `Stateful<E>` 에 구현됨. 본 SPEC 은 `Stateful<Div>` 기반 divider 를 예상.

| 메서드 | 시그니처 | Spike 1 용도 |
|--------|----------|--------------|
| `on_mouse_down(button, listener)` | `Fn(&MouseDownEvent, &mut Window, &mut App)` | drag 시작 트리거 — 현재 `lib.rs:165` 에서 이미 사용 중 |
| `on_mouse_move(listener)` | `Fn(&MouseMoveEvent, &mut Window, &mut App)` — **Bubble phase** | drag 진행 중 ratio 갱신. frame 단위 delivery |
| `on_mouse_up(button, listener)` | `Fn(&MouseUpEvent, &mut Window, &mut App)` | drag 종료 시 최종 ratio commit |
| `on_drag_move<T: 'static>(listener)` | `Fn(&DragMoveEvent<T>, ...)` | (선택) element 내외부 모두 capture — divider handle 이탈 시에도 event 수신, payload 기반 DnD |
| `drag_over<S>(style_fn)` / `hover(style_fn)` | style refinement | cursor 변경 / hover 하이라이트 |

### 4.2 MouseMoveEvent 구조체

- `position: Point<Pixels>` — window 상대 좌표
- `pressed_button: Option<MouseButton>` — 눌린 버튼
- `modifiers: Modifiers` — Shift/Ctrl 등
- `dragging() -> bool` — 좌측 버튼 눌림 여부 (drag 활성 판정용)

### 4.3 Event Phase

- **Capture**: root → target (`capture_any_mouse_down` 계열)
- **Bubble**: target → root (`on_mouse_move`, `on_mouse_up` 기본값)

Divider drag 는 **Bubble phase** 로 충분. Capture 는 불필요 (child element 보다 divider 가 이벤트 우선권 가지지 않음).

---

## 5. 판정

**PASS** — 자체 구현 경로 확정.

근거:
1. `on_mouse_down` → `on_mouse_move` → `on_mouse_up` 체인이 Bubble phase 에서 완결됨.
2. `MouseMoveEvent::dragging()` 으로 drag 활성 여부를 명시적으로 판정 가능.
3. `on_drag_move<T>` 는 handle 이탈 시에도 이벤트 지속 수신 — divider 좌표가 총 너비의 경계를 넘어도 drag 유지.
4. GPUI render loop 가 `cx.notify()` 트리거 시 frame 재그림. mouse_move 콜백 내에서 `cx.notify()` 호출 → 다음 frame 에서 style 재계산.
5. `Styled` trait 의 `.w(px(N))` / `.flex_basis(..)` / `.flex_grow(..)` 로 pane 너비 조정 → flex layout 이 나머지 자동 분배.
6. 60 fps 유지 여부는 architecture 레벨 제약 없음 — GPUI 자체가 GPU 가속 기반이며 기존 SPEC-V3-002 의 `TerminalSurface` 가 동일 render loop 에서 60 fps 로 이미 동작 중 (moai-studio-ui 60 tests GREEN).
7. 200 LOC 한계: 본 조사 기반 스케치 (아래 §6) 가 ~80 LOC 내외로 달성 가능 — 여유 충분.

---

## 6. T4 / T5 구현 스케치 (pseudo code, 실제 구현은 각 task 범위)

### 6.1 GpuiNativeSplitter (T4)

```rust
// crates/moai-studio-ui/src/panes/splitter_gpui_native.rs (T4 신규)
use gpui::{
    div, px, Context, Entity, IntoElement, MouseButton, ParentElement,
    Render, Stateful, Styled, Window,
};

pub struct GpuiNativeSplitter {
    tree: Entity<PaneTree<Entity<TerminalSurface>>>,
    focus: Option<PaneId>,
}

impl PaneSplitter for GpuiNativeSplitter {
    fn split_horizontal(&mut self, target: PaneId) -> Result<PaneId, SplitError> {
        let new_id = PaneId::new_unique();
        self.tree.update(cx, |tree, cx| {
            let surface = cx.new(|cx| TerminalSurface::new(/*spawn PtyWorker*/));
            tree.split_horizontal(&target, new_id.clone(), surface)
        })?;
        Ok(new_id)
    }
    // ... split_vertical, close_pane, focus_pane 동일 패턴
}

impl Render for GpuiNativeSplitter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Tree 재귀 렌더: Leaf → TerminalSurface, Split → Divider + first + second
        render_tree_recursive(&self.tree.read(cx), cx)
    }
}
```

### 6.2 GpuiDivider (T5)

```rust
// crates/moai-studio-ui/src/panes/divider_impl.rs (T5 신규)
use gpui::{
    div, px, Context, IntoElement, MouseButton, ParentElement, Render,
    Stateful, Styled, Window, MouseMoveEvent, MouseDownEvent, MouseUpEvent,
};

pub struct GpuiDivider {
    split_node_id: SplitNodeId,
    orientation: SplitDirection,   // DividerOrientation 신규 금지 (strategy §3.2 YAGNI)
    total_px: f32,
    current_ratio: f32,
    drag_start_ratio: Option<f32>,
    drag_start_px: Option<f32>,
    sibling_min_px: f32,
}

impl ResizableDivider for GpuiDivider {
    fn on_drag(&mut self, delta_px: f32, total_px: f32) -> f32 {
        let min_ratio = self.min_ratio_for(self.sibling_min_px);
        let raw = self.drag_start_ratio.unwrap_or(self.current_ratio)
                + delta_px / total_px;
        raw.clamp(min_ratio, 1.0 - min_ratio)
    }
    fn min_ratio_for(&self, sibling_px: f32) -> f32 { sibling_px / self.total_px }
}

impl Render for GpuiDivider {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let min_ratio = self.min_ratio_for(self.sibling_min_px);
        div()
            .id("divider")
            .w(px(4.0))  // §6.3 최소 4pt hit area
            .bg(rgb(0x2a2a2a))
            .hover(|s, _, _, _| s.bg(rgb(0x555555)))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, _, cx| {
                this.drag_start_ratio = Some(this.current_ratio);
                this.drag_start_px = Some(event.position.x.0);
                cx.notify();
            }))
            .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                if event.dragging() && this.drag_start_px.is_some() {
                    let delta = event.position.x.0 - this.drag_start_px.unwrap();
                    this.current_ratio = this.on_drag(delta, this.total_px);
                    cx.notify();  // frame 재그림 → flex_basis 갱신
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _event, _, cx| {
                this.drag_start_ratio = None;
                this.drag_start_px = None;
                // AC-P-4 / AC-P-6: 최종 ratio 에 대해 PtyWorker::resize 호출 (caller 가 tree update)
                cx.notify();
            }))
    }
}
```

예상 LOC: T4 (GpuiNativeSplitter) ~100 LOC + T5 (GpuiDivider) ~80 LOC = ~180 LOC **이내**.

---

## 7. FAIL 경로 회피

Spike 1 PASS 이므로:

- **Spike 2 (longbridge/gpui-component) 실행 불필요** — plan.md §3 Spike 2 "S1 FAIL 조건부" 조건 불충족.
- **[USER-DECISION-REQUIRED: gpui-component-adoption]** 기본값 (자체 구현) 자동 적용 가능. 단, 사용자에게 결정 확인 요청 (의도적 경로 선택을 명시적 승인 받기 위해).

---

## 8. 추가 관찰 / T4 시작 전 확인 사항

1. **`cx.listener(...)` 패턴** — 기존 `lib.rs:165-182` 에 안정적으로 활용 중. T4/T5 도 동일 패턴 적용.
2. **`cx.notify()` vs `cx.emit(...)`** — divider drag ratio 갱신은 local state 변경이므로 `cx.notify()` 충분 (외부 event emit 불요).
3. **Layout primitive** — `.flex_basis(px(ratio * total_px))` 또는 `.w(px(...))`. flex 환경에서는 flex_basis 권장.
4. **Vertical orientation** — Horizontal split 시 `.flex().flex_row()` + width 조정 / Vertical split 시 `.flex().flex_col()` + height 조정. `event.position.y.0` 사용.
5. **accessibility** — spec.md §6.3 "Divider 는 최소 4 pt 의 drag hit area" 충족 가능 — `.w(px(4.0))` 으로 설정 (위 스케치).
6. **AC-P-18 (paint ≤ 200ms)** — GPUI render loop + PtyWorker spawn 시간의 합이 200ms 이내 — SPEC-V3-002 에서 TerminalSurface 초기화 ~50-100ms 관측, 여유 있음.
7. **Zed pane.rs 참조** — 추후 T4/T5 구현 시 Zed 의 `crates/workspace/src/pane.rs` 의 divider 처리를 선택적 참고 (research.md 기존 언급). 단 Zed 는 최신 GPUI 사용이므로 API 차이 주의.

---

## 9. 최종 판정

| 항목 | 결과 |
|------|------|
| API 존재 여부 | ✅ GPUI 0.2.2 `InteractiveElement` 완전 지원 |
| 200 LOC 한계 | ✅ 예상 ~180 LOC, 여유 |
| 60 fps 유지 | ✅ GPUI GPU 가속 + SPEC-V3-002 기존 runtime 검증 |
| Native API only | ✅ 외부 crate 불필요 |
| Platform dispatch | ✅ cross-platform (GPUI 0.2.2 = macOS + Linux) |
| **전체** | **PASS** |

**경로 확정**: T4 = `GpuiNativeSplitter` 자체 구현 / T5 = `GpuiDivider` 자체 구현. **Spike 2 미실행**.

**[USER-DECISION-REQUIRED: gpui-component-adoption]**: Spike 1 PASS 로 기본값 (a) 자동 적용 가능하나, MoAI orchestrator 가 명시적 사용자 승인 요청 (의도적 경로 선택 확인).

---

Version: 1.0.0
Source: SPEC-V3-003 Plan Spike 1 (plan.md §3, strategy.md §4.2)
Classified: Research only — no code changes in production tree
