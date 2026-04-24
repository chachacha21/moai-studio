//! `PaneSplitter` 추상 trait — pane split / close / focus 계약 (SPEC-V3-003 T3).
//!
//! ## 모듈 역할
//!
//! 이 모듈은 pane 분할·제거·포커스 연산의 **인터페이스 계약만** 정의한다.
//! 구체 구현체 선택 (GpuiNativeSplitter 혹은 GpuiComponentSplitter) 은 **T4 + Spike 1** 에서
//! 결정되며, 그 선택과 무관하게 이 trait + Mock 조합으로 AC-P-17 을 검증한다.
//!
//! ## AC-P-17 검증 방식 결정 (d 경로)
//!
//! doc test 는 `#[cfg(test)]` 블록 내 Mock 에 외부 crate 도달 불가 문제와
//! Cargo.toml 변경 금지 원칙 충돌로 실용성이 낮다.
//! 따라서 `tests::abstract_traits_compile_without_impl` unit test 가 AC-P-17 을 검증한다:
//! trait object + Mock 결합 선언이 컴파일되면 구체 impl 없이도 계약 사용이 가능하다.
//!
//! @MX:TODO(T4): Spike 1 PASS 시 `GpuiNativeSplitter` 구현, FAIL + Spike 2 PASS 시 `GpuiComponentSplitter` 구현. 사용자 결정 required.

use crate::panes::{PaneId, SplitError};

// ============================================================
// CloseError
// ============================================================

/// pane 닫기 연산 실패 원인.
///
/// split 경로에서 발생하는 [`SplitError`] 와 별개로, close 전용 오류 타입을 정의한다.
/// T4 구체 구현체에서 PtyWorker drop 실패 등 추가 variant 가 생길 수 있다.
#[derive(Debug, PartialEq, Eq)]
pub enum CloseError {
    /// 닫으려는 PaneId 가 트리에 존재하지 않는다.
    TargetNotFound,
}

impl From<SplitError> for CloseError {
    /// [`SplitError`] → [`CloseError`] 매핑.
    ///
    /// `SplitError::TargetNotFound` → `CloseError::TargetNotFound`.
    /// `SplitError::MinSizeViolated` 는 close 경로에서 발생하지 않으나,
    /// defensive 처리를 위해 `TargetNotFound` 로 매핑한다.
    fn from(e: SplitError) -> Self {
        match e {
            SplitError::TargetNotFound => CloseError::TargetNotFound,
            SplitError::MinSizeViolated => CloseError::TargetNotFound,
        }
    }
}

// ============================================================
// PaneSplitter trait
// ============================================================

// @MX:ANCHOR: [AUTO] pane-splitter-contract
// @MX:REASON: [AUTO] UI orchestrator 의 split/close/focus 진입점 계약.
//   plan spike (S1/S2) 구현체 선택 변화에 무관하게 AC-P-17 유지.
//   fan_in >= 3 (T4 GpuiNativeSplitter, T7 RootView, T9 키 바인딩).

/// pane 분할·제거·포커스 연산의 추상 계약.
///
/// ## 구현체 선택 (T4)
///
/// Spike 1 (GPUI 0.2.2 divider drag API) 결과에 따라:
/// - PASS → `GpuiNativeSplitter` (native GPUI drag event 사용)
/// - FAIL → Spike 2 → `GpuiComponentSplitter` (gpui-component::Resizable 사용)
///
/// T4 에서 `Entity<TerminalSurface>` 를 leaf 로 생성할 때 `PtyWorker::spawn` 을 호출한다.
/// 이 trait 은 그 선택에 무관하게 AC-P-17 을 보장한다.
pub trait PaneSplitter {
    /// `target` leaf 를 Horizontal (좌/우) 로 분할한다.
    ///
    /// 새로 생성된 leaf 의 [`PaneId`] 를 반환한다.
    ///
    /// # Errors
    ///
    /// - [`SplitError::TargetNotFound`]: `target` 가 트리에 없을 때.
    fn split_horizontal(&mut self, target: PaneId) -> Result<PaneId, SplitError>;

    /// `target` leaf 를 Vertical (상/하) 로 분할한다.
    ///
    /// 새로 생성된 leaf 의 [`PaneId`] 를 반환한다.
    ///
    /// # Errors
    ///
    /// - [`SplitError::TargetNotFound`]: `target` 가 트리에 없을 때.
    fn split_vertical(&mut self, target: PaneId) -> Result<PaneId, SplitError>;

    /// `target` leaf 를 트리에서 제거한다.
    ///
    /// 단일 leaf 인 경우 no-op (AC-P-3 계약 유지).
    ///
    /// # Errors
    ///
    /// - [`CloseError::TargetNotFound`]: `target` 가 트리에 없을 때.
    fn close_pane(&mut self, target: PaneId) -> Result<(), CloseError>;

    /// `target` leaf 에 포커스를 설정한다.
    ///
    /// T6 에서 FocusHandle 연결 시 확장.
    fn focus_pane(&mut self, target: PaneId);
}

// ============================================================
// MockPaneSplitter — #[cfg(test)] 전용
// ============================================================

// @MX:NOTE: [AUTO] test-only-impl — PaneTree<String> 을 감싸 GPUI 의존 없이 trait 계약 검증 (AC-P-17)

/// 테스트 전용 `PaneSplitter` 구현체.
///
/// 내부적으로 [`crate::panes::PaneTree<String>`] 을 사용하여 GPUI 의존 없이 동작한다.
/// payload 는 `"mock-pane-{n}"` 형식의 String 이다.
#[cfg(test)]
pub struct MockPaneSplitter {
    tree: crate::panes::PaneTree<String>,
    next_counter: u32,
    current_focus: Option<PaneId>,
}

#[cfg(test)]
impl MockPaneSplitter {
    /// 단일 leaf (`"mock-pane-0"`) 로 시작하는 MockPaneSplitter 를 생성한다.
    pub fn new() -> Self {
        let root_id = PaneId::new_from_literal("mock-pane-0");
        let tree = crate::panes::PaneTree::new_leaf(root_id.clone(), "mock-pane-0".to_string());
        Self {
            tree,
            next_counter: 1,
            current_focus: Some(root_id),
        }
    }

    /// 현재 tree 의 leaf 수 (테스트 검증용).
    pub fn leaf_count(&self) -> usize {
        self.tree.leaf_count()
    }

    /// 현재 포커스된 PaneId 참조 (테스트 검증용).
    pub fn current_focused(&self) -> Option<&PaneId> {
        self.current_focus.as_ref()
    }

    /// 다음 mock pane payload 문자열을 생성하고 counter 를 증가한다.
    fn next_payload(&mut self) -> (PaneId, String) {
        let n = self.next_counter;
        self.next_counter += 1;
        let label = format!("mock-pane-{n}");
        (PaneId::new_from_literal(&label), label)
    }
}

#[cfg(test)]
impl PaneSplitter for MockPaneSplitter {
    fn split_horizontal(&mut self, target: PaneId) -> Result<PaneId, SplitError> {
        let (new_id, new_payload) = self.next_payload();
        self.tree
            .split_horizontal(&target, new_id.clone(), new_payload)?;
        Ok(new_id)
    }

    fn split_vertical(&mut self, target: PaneId) -> Result<PaneId, SplitError> {
        let (new_id, new_payload) = self.next_payload();
        self.tree
            .split_vertical(&target, new_id.clone(), new_payload)?;
        Ok(new_id)
    }

    fn close_pane(&mut self, target: PaneId) -> Result<(), CloseError> {
        self.tree.close_pane(&target).map_err(CloseError::from)
    }

    fn focus_pane(&mut self, target: PaneId) {
        // 단순 저장 — T6 에서 FocusHandle 배선 시 확장.
        self.current_focus = Some(target);
    }
}

// ============================================================
// #[cfg(test)] 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------
    // AC-P-17: abstract_traits_compile_without_impl
    // trait object + Mock 결합이 구체 GPUI 구현 없이 컴파일됨을 검증.
    // -------------------------------------------------------

    /// Mock 을 trait object 로 사용하는 함수가 컴파일된다 → AC-P-17 충족.
    #[test]
    fn abstract_traits_compile_without_impl() {
        fn accept(_: &dyn PaneSplitter) {}
        let m = MockPaneSplitter::new();
        accept(&m);
    }

    // -------------------------------------------------------
    // MockPaneSplitter — split horizontal
    // -------------------------------------------------------

    /// split_horizontal 호출 후 새 PaneId 가 반환되고 leaf_count 가 증가한다.
    #[test]
    fn mock_splitter_splits_and_returns_new_pane_id() {
        let mut s = MockPaneSplitter::new();
        assert_eq!(s.leaf_count(), 1);

        let root = s.current_focused().cloned().unwrap();
        let new_id = s.split_horizontal(root).expect("split 성공");

        assert_eq!(s.leaf_count(), 2, "split 후 leaf 수 == 2");
        assert_ne!(new_id.0, "mock-pane-0", "새 pane id 는 달라야 함");
    }

    // -------------------------------------------------------
    // MockPaneSplitter — split vertical
    // -------------------------------------------------------

    /// split_vertical 호출 후 새 PaneId 반환 및 leaf_count 증가.
    #[test]
    fn mock_splitter_split_vertical() {
        let mut s = MockPaneSplitter::new();
        let root = s.current_focused().cloned().unwrap();
        let new_id = s.split_vertical(root).expect("split 성공");

        assert_eq!(s.leaf_count(), 2);
        assert_eq!(new_id.0, "mock-pane-1");
    }

    // -------------------------------------------------------
    // MockPaneSplitter — close propagates sibling
    // -------------------------------------------------------

    /// 3-leaf 구성 후 중간 leaf 를 close 하면 leaf_count 가 감소한다.
    #[test]
    fn mock_splitter_close_propagates_sibling() {
        let mut s = MockPaneSplitter::new();
        let root = PaneId::new_from_literal("mock-pane-0");

        // root 를 수평 분할 → [root, pane-1]
        let p1 = s.split_horizontal(root.clone()).unwrap();
        assert_eq!(s.leaf_count(), 2);

        // pane-1 을 수직 분할 → [root, pane-1, pane-2]
        let _p2 = s.split_vertical(p1.clone()).unwrap();
        assert_eq!(s.leaf_count(), 3);

        // pane-1 을 닫으면 → [root, pane-2]
        s.close_pane(p1).expect("pane-1 닫기 성공");
        assert_eq!(s.leaf_count(), 2);
    }

    // -------------------------------------------------------
    // MockPaneSplitter — close last leaf returns Ok (AC-P-3)
    // -------------------------------------------------------

    /// 단일 leaf 에 close_pane 하면 Ok(()) 반환, leaf_count 유지 (AC-P-3).
    #[test]
    fn mock_splitter_close_last_leaf_returns_ok_silently() {
        let mut s = MockPaneSplitter::new();
        assert_eq!(s.leaf_count(), 1);

        let root = PaneId::new_from_literal("mock-pane-0");
        s.close_pane(root).expect("no-op Ok 이어야 함");

        assert_eq!(s.leaf_count(), 1, "leaf_count 유지");
    }

    // -------------------------------------------------------
    // MockPaneSplitter — close target not found
    // -------------------------------------------------------

    /// 존재하지 않는 PaneId 로 close → Err(CloseError::TargetNotFound).
    #[test]
    fn mock_splitter_close_target_not_found() {
        let mut s = MockPaneSplitter::new();
        let ghost = PaneId::new_from_literal("ghost-pane");
        assert_eq!(
            s.close_pane(ghost),
            Err(CloseError::TargetNotFound),
            "존재하지 않는 pane close 는 TargetNotFound"
        );
    }

    // -------------------------------------------------------
    // MockPaneSplitter — split target not found
    // -------------------------------------------------------

    /// 존재하지 않는 PaneId 로 split → Err(SplitError::TargetNotFound).
    #[test]
    fn mock_splitter_split_target_not_found() {
        let mut s = MockPaneSplitter::new();
        let ghost = PaneId::new_from_literal("ghost-pane");
        assert_eq!(
            s.split_horizontal(ghost),
            Err(SplitError::TargetNotFound),
            "존재하지 않는 pane split 은 TargetNotFound"
        );
    }

    // -------------------------------------------------------
    // MockPaneSplitter — focus_pane updates current
    // -------------------------------------------------------

    /// focus_pane 호출 후 current_focused() 가 갱신된다.
    #[test]
    fn mock_splitter_focus_pane_updates_current() {
        let mut s = MockPaneSplitter::new();
        let root = PaneId::new_from_literal("mock-pane-0");
        let p1 = s.split_horizontal(root.clone()).unwrap();

        s.focus_pane(p1.clone());
        assert_eq!(
            s.current_focused(),
            Some(&p1),
            "focus 후 current_focused == p1"
        );

        s.focus_pane(root.clone());
        assert_eq!(
            s.current_focused(),
            Some(&root),
            "re-focus 후 current_focused == root"
        );
    }
}
