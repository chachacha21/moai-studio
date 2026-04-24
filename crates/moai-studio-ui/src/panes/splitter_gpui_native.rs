//! `GpuiNativeSplitter` — PaneSplitter 구체 구현 (SPEC-V3-003 T4, MS-1).
//!
//! ## 구현 경로 결정: 경로 A (Generic Factory)
//!
//! ### 결정 근거
//!
//! GPUI 0.2.2 의 `TestAppContext` 는 `gpui` crate 의 `test-support` feature 활성화가
//! 필요하다 (gpui-0.2.2/src/app/test_context.rs:535 의 `#[cfg(feature = "test-support")]`).
//! 그러나 `crates/moai-studio-ui/Cargo.toml` 에는 `gpui = "0.2"` 만 선언되어 있고,
//! T4 scope 제약 (`Cargo.toml 변경 금지`) 으로 feature 추가가 불가하다.
//!
//! ### Trade-off
//!
//! - **장점**: GPUI 의존 없이 단위 테스트 완전 격리. `factory: Box<dyn FnMut(&PaneId) -> L>`
//!   덕분에 prod 에서는 `cx.new(|cx| TerminalSurface::new(...))` 주입, test 에서는
//!   `Arc<Mutex<TestPane>>` 또는 `String` 주입 가능.
//! - **단점**: prod 타입 바인딩이 T7 (RootView wire-up) 으로 지연된다. 본 task 에서는
//!   `GpuiNativeSplitter<String>` 으로 컴파일 및 동작 검증만 수행.
//! - **AC-P-1 통합**: Factory 인터페이스가 `Entity<TerminalSurface>` 생성을 포함하므로
//!   T7 에서 `GpuiNativeSplitter<Entity<TerminalSplitter>>` 로 타입 파라미터 확정 시
//!   AC-P-1 완전 충족. 본 task 기준으로는 PARTIAL (factory 설계 확정, prod wire 연기).
//!
//! ### MockPaneSplitter 와의 설계 일관성
//!
//! `splitter.rs` 의 `MockPaneSplitter` 가 `PaneTree<String>` + counter 를 감싸듯,
//! `GpuiNativeSplitter<L>` 도 `PaneTree<L>` + `factory` + `focus` 로 구성한다.
//! 단, `MockPaneSplitter` 는 payload 를 내부에서 생성하고,
//! `GpuiNativeSplitter` 는 **외부 factory 주입** 으로 payload 생성을 위임한다.

use crate::panes::{CloseError, PaneId, PaneSplitter, PaneTree, SplitError};

// ============================================================
// GpuiNativeSplitter
// ============================================================

// @MX:ANCHOR: [AUTO] concrete-splitter-gpui-native
// @MX:REASON: [AUTO] GpuiNativeSplitter 는 PaneSplitter 의 구체 구현체로
//   T7 RootView, T9 키 바인딩 dispatcher, T11 bench harness 에서 호출된다 (fan_in >= 3).
//   factory closure 주입으로 prod/test 간 payload 생성 전략을 분리.

/// GPUI native 이벤트 기반 PaneSplitter 구체 구현체.
///
/// ## 제네릭 파라미터
///
/// - `L: Clone + 'static`: leaf payload 타입.
///   - prod:  `Entity<TerminalSurface>` (T7 에서 RootView wire-up 시 확정)
///   - test:  `String` 또는 `Arc<Mutex<TestPane>>`
///
/// ## 생성 방법
///
/// ```ignore
/// let splitter = GpuiNativeSplitter::new_with_factory(
///     root_id,
///     root_payload,
///     Box::new(|_id| String::from("test-pane")),
/// );
/// ```
pub struct GpuiNativeSplitter<L: Clone + 'static> {
    /// 이진 트리 pane 자료구조.
    tree: PaneTree<L>,
    /// 현재 포커스된 pane ID.
    focus: Option<PaneId>,
    // @MX:ANCHOR: [AUTO] pane-leaf-factory-injection
    // @MX:REASON: [AUTO] split 시 새 leaf 를 생성하는 전략을 외부에서 주입받는다.
    //   prod 에서는 `cx.new(|cx| TerminalSurface::new(...))` 형태의 closure.
    //   test 에서는 String literal 반환 closure. T7 wire-up 포인트.
    /// 새 leaf payload 를 생성하는 factory closure.
    ///
    /// `split_horizontal` / `split_vertical` 호출 시 새 `PaneId` 를 인수로 받아
    /// 해당 pane 의 payload 를 생성한다.
    factory: Box<dyn FnMut(&PaneId) -> L>,
}

impl<L: Clone + 'static> GpuiNativeSplitter<L> {
    /// 지정된 root pane 과 factory 로 새 GpuiNativeSplitter 를 생성한다.
    ///
    /// # Arguments
    ///
    /// - `root_id`: 루트 pane 의 식별자.
    /// - `root_payload`: 루트 pane 의 초기 payload.
    /// - `factory`: 새 split 시 호출되어 신규 leaf payload 를 생성하는 closure.
    pub fn new_with_factory(
        root_id: PaneId,
        root_payload: L,
        factory: Box<dyn FnMut(&PaneId) -> L>,
    ) -> Self {
        let focus = Some(root_id.clone());
        Self {
            tree: PaneTree::new_leaf(root_id, root_payload),
            focus,
            factory,
        }
    }

    /// 현재 PaneTree 에 대한 참조를 반환한다 (테스트 검증용).
    pub fn tree(&self) -> &PaneTree<L> {
        &self.tree
    }

    /// 현재 포커스된 PaneId 참조를 반환한다 (테스트 검증용).
    pub fn focused(&self) -> Option<&PaneId> {
        self.focus.as_ref()
    }
}

// ============================================================
// PaneSplitter impl
// ============================================================

// @MX:WARN: [AUTO] gpui-api-churn-risk
// @MX:REASON: [AUTO] GPUI 0.2.2 는 crates.io 공식판이나 Zed 팀이 main 브랜치에서
//   렌더 API 를 지속 변경하고 있다 (SPEC-V3-001 에서도 동일 위험 관찰).
//   Phase 7+ 업그레이드 시 `Entity<TerminalSurface>` prod 타입 바인딩 (T7) 에서
//   API 변경이 발생할 수 있다. factory closure 경계가 변경 격리 역할을 한다.

impl<L: Clone + 'static> PaneSplitter for GpuiNativeSplitter<L> {
    /// `target` leaf 를 Horizontal (좌/우) 로 분할한다.
    ///
    /// factory 로 새 payload 를 생성 후 PaneTree::split_horizontal 에 위임한다.
    fn split_horizontal(&mut self, target: PaneId) -> Result<PaneId, SplitError> {
        let new_id = PaneId::new_unique();
        let new_payload = (self.factory)(&new_id);
        self.tree
            .split_horizontal(&target, new_id.clone(), new_payload)?;
        Ok(new_id)
    }

    /// `target` leaf 를 Vertical (상/하) 로 분할한다.
    ///
    /// factory 로 새 payload 를 생성 후 PaneTree::split_vertical 에 위임한다.
    fn split_vertical(&mut self, target: PaneId) -> Result<PaneId, SplitError> {
        let new_id = PaneId::new_unique();
        let new_payload = (self.factory)(&new_id);
        self.tree
            .split_vertical(&target, new_id.clone(), new_payload)?;
        Ok(new_id)
    }

    /// `target` leaf 를 트리에서 제거한다.
    ///
    /// SplitError → CloseError 변환 포함. 단일 leaf no-op (AC-P-3) 은 PaneTree 가 보장.
    fn close_pane(&mut self, target: PaneId) -> Result<(), CloseError> {
        self.tree.close_pane(&target).map_err(CloseError::from)
    }

    /// `target` leaf 에 포커스를 설정한다.
    ///
    /// T6 에서 FocusHandle 배선 시 확장 예정. 현재는 내부 상태만 갱신.
    fn focus_pane(&mut self, target: PaneId) {
        self.focus = Some(target);
    }
}

// ============================================================
// #[cfg(test)] 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // -------------------------------------------------------
    // 헬퍼: 테스트용 GpuiNativeSplitter<String> 생성
    // -------------------------------------------------------

    /// 테스트용 GpuiNativeSplitter<String> 을 생성한다.
    ///
    /// factory 는 "native-pane-{id}" 형식 String 을 반환한다.
    fn make_splitter() -> GpuiNativeSplitter<String> {
        let root_id = PaneId::new_from_literal("root-pane");
        GpuiNativeSplitter::new_with_factory(
            root_id,
            "native-pane-root".to_string(),
            Box::new(|id| format!("native-pane-{}", id.0)),
        )
    }

    // -------------------------------------------------------
    // 정적 컴파일 검증 — PaneSplitter trait 구현 확인
    // -------------------------------------------------------

    /// GpuiNativeSplitter<String> 이 PaneSplitter 를 구현함을 컴파일 타임 검증.
    ///
    /// `fn _assert<T: PaneSplitter>(_: T) {}` 패턴으로 타입 체크.
    #[test]
    fn compile_time_pane_splitter_impl() {
        fn _assert<T: PaneSplitter>(_: &T) {}
        let s = make_splitter();
        _assert(&s);
    }

    // -------------------------------------------------------
    // split_horizontal 기본 동작
    // -------------------------------------------------------

    /// split_horizontal 호출 후 leaf_count 가 증가하고 새 PaneId 가 반환된다.
    #[test]
    fn split_horizontal_creates_new_pane_and_increments_leaf_count() {
        let mut s = make_splitter();
        assert_eq!(s.tree().leaf_count(), 1, "초기 leaf_count == 1");

        let root_id = PaneId::new_from_literal("root-pane");
        let new_id = s
            .split_horizontal(root_id.clone())
            .expect("split_horizontal 성공해야 함");

        assert_eq!(s.tree().leaf_count(), 2, "split 후 leaf_count == 2");
        assert_ne!(new_id.0, "root-pane", "새 pane id 는 root 와 달라야 함");
    }

    // -------------------------------------------------------
    // split_vertical 기본 동작
    // -------------------------------------------------------

    /// split_vertical 호출 후 leaf_count 가 증가하고 새 PaneId 가 반환된다.
    #[test]
    fn split_vertical_creates_new_pane_and_increments_leaf_count() {
        let mut s = make_splitter();
        let root_id = PaneId::new_from_literal("root-pane");
        let new_id = s
            .split_vertical(root_id.clone())
            .expect("split_vertical 성공해야 함");

        assert_eq!(s.tree().leaf_count(), 2, "split 후 leaf_count == 2");
        assert_ne!(new_id.0, "root-pane", "새 pane id 는 root 와 달라야 함");
    }

    // -------------------------------------------------------
    // close_pane — leaf drop + strong_count 감소 (AC-P-2)
    // -------------------------------------------------------

    /// close_pane 호출 시 leaf_count 가 감소하고 Arc strong_count 가 줄어든다.
    ///
    /// Arc<Mutex<T>> payload 로 strong_count 를 통해 drop 을 검증한다 (AC-P-2).
    #[test]
    fn close_pane_decrements_leaf_count_and_drops_payload() {
        // Arc 페이로드로 GpuiNativeSplitter 생성
        let root_arc: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        let root_id = PaneId::new_from_literal("arc-root");
        let root_payload = Arc::clone(&root_arc);

        let factory_arc: Arc<Mutex<i32>> = Arc::new(Mutex::new(1));
        let factory_arc_clone = Arc::clone(&factory_arc);
        let mut s: GpuiNativeSplitter<Arc<Mutex<i32>>> = GpuiNativeSplitter::new_with_factory(
            root_id.clone(),
            root_payload,
            Box::new(move |_id| Arc::clone(&factory_arc_clone)),
        );

        // split_horizontal 으로 새 leaf (factory_arc 의 clone) 생성
        let new_id = s.split_horizontal(root_id.clone()).expect("split 성공");

        // tree 에 2개 leaf 가 있고, factory_arc strong_count == 2 (factory 원본 + leaf payload)
        assert_eq!(s.tree().leaf_count(), 2);
        // factory_arc 는 factory closure 내부 + leaf payload 에서 참조됨 (strong_count >= 2)
        assert!(
            Arc::strong_count(&factory_arc) >= 2,
            "split 후 strong_count >= 2"
        );

        // new_id leaf 를 close → drop → factory_arc 의 leaf 참조가 해제됨
        s.close_pane(new_id).expect("close 성공");
        assert_eq!(s.tree().leaf_count(), 1, "close 후 leaf_count == 1");

        // leaf payload drop 확인: factory_arc 의 strong_count 가 감소해야 함
        // factory closure 내부의 Arc 참조만 남아 있어야 함 (== 2 가 아닌 1 + factory closure 1)
        let count_after = Arc::strong_count(&factory_arc);
        assert!(
            count_after < 3,
            "close 후 strong_count 감소해야 함, 실제: {count_after}"
        );
    }

    // -------------------------------------------------------
    // close_pane — TargetNotFound
    // -------------------------------------------------------

    /// 존재하지 않는 PaneId 로 close_pane → Err(CloseError::TargetNotFound).
    #[test]
    fn close_target_not_found_returns_err() {
        let mut s = make_splitter();
        let ghost = PaneId::new_from_literal("ghost-pane");
        assert_eq!(
            s.close_pane(ghost),
            Err(CloseError::TargetNotFound),
            "존재하지 않는 pane close 는 TargetNotFound"
        );
    }

    // -------------------------------------------------------
    // split — TargetNotFound
    // -------------------------------------------------------

    /// 존재하지 않는 PaneId 로 split_horizontal → Err(SplitError::TargetNotFound).
    #[test]
    fn split_target_not_found_returns_split_error_not_found() {
        let mut s = make_splitter();
        let ghost = PaneId::new_from_literal("ghost-pane");
        assert_eq!(
            s.split_horizontal(ghost),
            Err(SplitError::TargetNotFound),
            "존재하지 않는 pane split 은 TargetNotFound"
        );
    }

    // -------------------------------------------------------
    // focus_pane — 상태 갱신
    // -------------------------------------------------------

    /// focus_pane 호출 후 focused() 가 해당 PaneId 를 반환한다.
    #[test]
    fn focus_pane_updates_focus_state() {
        let mut s = make_splitter();
        let root_id = PaneId::new_from_literal("root-pane");

        // split 후 새 pane 생성
        let new_id = s.split_horizontal(root_id.clone()).unwrap();

        // 새 pane 으로 포커스 이동
        s.focus_pane(new_id.clone());
        assert_eq!(s.focused(), Some(&new_id), "focus 후 focused == new_id");

        // 다시 root 로 포커스 복귀
        s.focus_pane(root_id.clone());
        assert_eq!(
            s.focused(),
            Some(&root_id),
            "re-focus 후 focused == root_id"
        );
    }

    // -------------------------------------------------------
    // close 단일 leaf — no-op (AC-P-3)
    // -------------------------------------------------------

    /// 단일 leaf 를 close_pane 하면 Ok(()) 반환하고 leaf_count 유지 (AC-P-3).
    #[test]
    fn close_single_leaf_is_noop() {
        let mut s = make_splitter();
        assert_eq!(s.tree().leaf_count(), 1);

        let root_id = PaneId::new_from_literal("root-pane");
        s.close_pane(root_id)
            .expect("단일 leaf close 는 Ok(()) no-op");

        assert_eq!(s.tree().leaf_count(), 1, "leaf_count 유지");
    }

    // -------------------------------------------------------
    // 3-leaf 구성 후 close
    // -------------------------------------------------------

    /// 3-leaf 구성 후 중간 leaf close 시 leaf_count 가 감소한다.
    #[test]
    fn close_middle_pane_in_three_leaf_tree() {
        let mut s = make_splitter();
        let root_id = PaneId::new_from_literal("root-pane");

        // root 수평 분할 → [root, p1]
        let p1 = s.split_horizontal(root_id.clone()).unwrap();
        assert_eq!(s.tree().leaf_count(), 2);

        // p1 수직 분할 → [root, p1, p2]
        let _p2 = s.split_vertical(p1.clone()).unwrap();
        assert_eq!(s.tree().leaf_count(), 3);

        // p1 close → [root, p2]
        s.close_pane(p1).expect("p1 close 성공");
        assert_eq!(s.tree().leaf_count(), 2, "close 후 leaf_count == 2");
    }
}
