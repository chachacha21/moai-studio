//! Focus routing — prev/next pane 단축키 + mouse click focus 순수 상태 머신.
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-3 (REQ-P-020 ~ REQ-P-024): focus 상태 관리
//! - spec.md §5 RG-P-4 (REQ-P-030 ~ REQ-P-034): platform_mod 매크로 (macOS: Cmd / Linux: Ctrl)
//!
//! ## 설계 원칙
//!
//! GPUI 의존 없는 순수 Rust 상태 머신. T7 에서 GPUI `KeyDownEvent` → [`KeyModifiers`]/[`KeyCode`] 변환 후 주입.
//!
//! - [`FocusRouter`] — pane focus 상태 + in-order prev/next 탐색 + mouse click 처리
//! - [`FocusCommand`] — 상태 변경 명령 (Prev / Next / Click)
//! - [`PlatformMod`] — 플랫폼 modifier 추상 (macOS = Cmd, Linux/기타 = Ctrl)
//! - [`PLATFORM_MOD`] — 컴파일 타임 현재 플랫폼 modifier
//! - [`dispatch_key`] — MS-1 키 이벤트 → `FocusCommand` 매핑 (AC-P-23 Ctrl+B passthrough 보장)

use crate::panes::{PaneId, PaneTree};

// ============================================================
// 플랫폼 Modifier 추상화
// ============================================================

/// 플랫폼 단축키 modifier.
///
/// `cfg(target_os = "macos")` → `Cmd`,  기타 → `Ctrl`.
/// MS-2 T9 에서 Spike 4 (Linux shell Ctrl 충돌 탐색) 결과에 따라 재검토 예정.
// @MX:NOTE: [AUTO] cmd-ctrl-platform-dispatch
// macOS 는 Cmd (Super), 나머지는 Ctrl. Spike 4 결정: default (a) 현행 유지.
// MS-2 T9 에서 Linux pty Ctrl 충돌 재평가 예정 (contract.md §4.2 AC-P-9b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformMod {
    /// macOS Command 키 (⌘).
    Cmd,
    /// Linux / 기타 Ctrl 키.
    Ctrl,
}

/// 컴파일 타임 현재 플랫폼 modifier 상수.
#[cfg(target_os = "macos")]
pub const PLATFORM_MOD: PlatformMod = PlatformMod::Cmd;

#[cfg(not(target_os = "macos"))]
pub const PLATFORM_MOD: PlatformMod = PlatformMod::Ctrl;

// ============================================================
// 키 이벤트 타입 (GPUI 독립 순수 데이터)
// ============================================================

/// GPUI 독립 modifier 비트마스크.
///
/// T7 에서 `gpui::Modifiers` → 이 타입으로 변환 후 [`dispatch_key`] 에 전달.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    /// Alt / Option 키.
    pub alt: bool,
    /// Ctrl 키.
    pub ctrl: bool,
    /// Command 키 (macOS ⌘).
    pub cmd: bool,
    /// Shift 키.
    pub shift: bool,
}

/// GPUI 독립 키 코드.
///
/// MS-1 에서 필요한 키만 열거. T7 에서 `gpui::KeyDownEvent.keystroke.key` → 이 타입으로 변환.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    /// 왼쪽 화살표 키.
    ArrowLeft,
    /// 오른쪽 화살표 키.
    ArrowRight,
    /// 기타 키 (passthrough).
    Other,
}

// ============================================================
// FocusCommand
// ============================================================

/// FocusRouter 에 전달하는 focus 변경 명령.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusCommand {
    /// 이전 pane 으로 이동 (in-order 역방향 wrap-around).
    Prev,
    /// 다음 pane 으로 이동 (in-order 정방향 wrap-around).
    Next,
    /// 특정 pane 을 마우스 클릭으로 선택.
    Click(PaneId),
}

// ============================================================
// FocusRouter
// ============================================================

// @MX:ANCHOR: [AUTO] focus-routing
// @MX:REASON: [AUTO] pane focus 상태 머신의 단일 진입점.
//   fan_in >= 3: T7 key handler (GPUI KeyDown 배선), T8 tab switch 복원, T9 MS-2 바인딩 dispatcher.
//   single_focus_invariant (AC-P-22): 동시에 focused 인 pane 은 최대 1개.
/// pane focus 상태를 관리하는 순수 Rust 상태 머신.
///
/// GPUI FocusHandle 배선은 T7 범위. T6 에서는 `PaneId` 기반 상태 추적만 수행.
#[derive(Debug, Default)]
pub struct FocusRouter {
    /// 현재 focused pane 의 ID. `None` 이면 아무 pane 도 focused 되지 않은 상태.
    current: Option<PaneId>,
}

impl FocusRouter {
    /// 새 FocusRouter 를 생성한다. 초기 focus 는 None.
    pub fn new() -> Self {
        Self { current: None }
    }

    /// 현재 focused pane 의 PaneId 를 반환한다.
    pub fn current(&self) -> Option<&PaneId> {
        self.current.as_ref()
    }

    /// 트리와 명령을 받아 focus 상태를 갱신한다.
    ///
    /// - `FocusCommand::Next`: in-order 다음 leaf 로 이동 (마지막 → 첫 번째 wrap-around).
    /// - `FocusCommand::Prev`: in-order 이전 leaf 로 이동 (첫 번째 → 마지막 wrap-around).
    /// - `FocusCommand::Click(id)`: 해당 pane 이 트리에 존재하면 focused 로 설정.
    ///   존재하지 않으면 no-op (AC-P-22 unknown-pane noop).
    pub fn apply<L: Clone>(&mut self, tree: &PaneTree<L>, cmd: FocusCommand) {
        let ids: Vec<PaneId> = tree.leaves().into_iter().map(|l| l.id.clone()).collect();
        if ids.is_empty() {
            return;
        }

        match cmd {
            FocusCommand::Next => {
                let next = match &self.current {
                    None => ids[0].clone(),
                    Some(cur) => {
                        let pos = ids.iter().position(|id| id == cur);
                        match pos {
                            None => ids[0].clone(),
                            Some(i) => ids[(i + 1) % ids.len()].clone(),
                        }
                    }
                };
                self.current = Some(next);
            }
            FocusCommand::Prev => {
                let prev = match &self.current {
                    None => ids[ids.len() - 1].clone(),
                    Some(cur) => {
                        let pos = ids.iter().position(|id| id == cur);
                        match pos {
                            None => ids[ids.len() - 1].clone(),
                            Some(0) => ids[ids.len() - 1].clone(),
                            Some(i) => ids[i - 1].clone(),
                        }
                    }
                };
                self.current = Some(prev);
            }
            FocusCommand::Click(id) => {
                // AC-P-22: 트리에 없는 pane ID 는 no-op.
                if ids.contains(&id) {
                    self.current = Some(id);
                }
            }
        }
    }
}

// ============================================================
// 키 dispatch (MS-1 바인딩)
// ============================================================

/// MS-1 키 이벤트를 [`FocusCommand`] 로 변환한다.
///
/// 매핑 테이블 (MS-1 범위):
/// - `PLATFORM_MOD + Alt + ArrowLeft`  → `FocusCommand::Prev`
/// - `PLATFORM_MOD + Alt + ArrowRight` → `FocusCommand::Next`
/// - 그 외 모든 키                     → `None` (passthrough)
///
/// # AC-P-23 Ctrl+B passthrough 보장
///
// @MX:NOTE: [AUTO] ac-p-23-ctrl-b-passthrough
// PLATFORM_MOD = Ctrl (Linux) 인 경우, 순수 Ctrl+B 는 tmux prefix 키다.
// dispatch_key 는 Ctrl+Alt+Arrow 만 소비하므로 Ctrl+B (alt=false, code=Other) 는
// 항상 None 을 반환 → caller(T7 GPUI event handler) 가 passthrough 처리.
pub fn dispatch_key(modifiers: KeyModifiers, code: KeyCode) -> Option<FocusCommand> {
    let platform_mod_active = match PLATFORM_MOD {
        PlatformMod::Cmd => modifiers.cmd,
        PlatformMod::Ctrl => modifiers.ctrl,
    };

    if !platform_mod_active || !modifiers.alt {
        return None;
    }

    match code {
        KeyCode::ArrowLeft => Some(FocusCommand::Prev),
        KeyCode::ArrowRight => Some(FocusCommand::Next),
        _ => None,
    }
}

// ============================================================
// 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panes::PaneTree;

    // -------------------------------------------------------
    // 테스트 헬퍼
    // -------------------------------------------------------

    fn pid(s: &str) -> PaneId {
        PaneId::new_from_literal(s)
    }

    /// leaf 3개짜리 트리 구성: a → b → c (in-order).
    fn three_leaf_tree() -> PaneTree<String> {
        let mut tree = PaneTree::new_leaf(pid("a"), "a".to_string());
        tree.split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .unwrap();
        tree.split_horizontal(&pid("b"), pid("c"), "c".to_string())
            .unwrap();
        tree
    }

    // -------------------------------------------------------
    // AC-P-7: next_pane_in_order
    // -------------------------------------------------------

    /// FocusRouter 가 in-order 순서로 다음 pane 을 focus 한다.
    #[test]
    fn next_pane_in_order() {
        let tree = three_leaf_tree();
        let mut router = FocusRouter::new();
        router.apply(&tree, FocusCommand::Next);
        assert_eq!(router.current(), Some(&pid("a")));

        router.apply(&tree, FocusCommand::Next);
        assert_eq!(router.current(), Some(&pid("b")));

        router.apply(&tree, FocusCommand::Next);
        assert_eq!(router.current(), Some(&pid("c")));
    }

    // -------------------------------------------------------
    // AC-P-7: prev_pane_in_order
    // -------------------------------------------------------

    /// FocusRouter 가 in-order 역방향으로 이전 pane 을 focus 한다.
    #[test]
    fn prev_pane_in_order() {
        let tree = three_leaf_tree();
        let mut router = FocusRouter::new();
        // focus 를 c 로 이동 후 역방향 탐색
        router.apply(&tree, FocusCommand::Click(pid("c")));
        assert_eq!(router.current(), Some(&pid("c")));

        router.apply(&tree, FocusCommand::Prev);
        assert_eq!(router.current(), Some(&pid("b")));

        router.apply(&tree, FocusCommand::Prev);
        assert_eq!(router.current(), Some(&pid("a")));
    }

    // -------------------------------------------------------
    // AC-P-7: wraparound_at_last_pane
    // -------------------------------------------------------

    /// 마지막 pane 에서 Next 를 호출하면 첫 번째 pane 으로 wrap-around 된다.
    #[test]
    fn wraparound_at_last_pane() {
        let tree = three_leaf_tree();
        let mut router = FocusRouter::new();
        router.apply(&tree, FocusCommand::Click(pid("c")));
        router.apply(&tree, FocusCommand::Next);
        assert_eq!(router.current(), Some(&pid("a")));
    }

    // -------------------------------------------------------
    // AC-P-22: single_focus_invariant
    // -------------------------------------------------------

    /// 동시에 focused 인 pane 은 최대 1개 — apply 후 current 는 단 하나의 ID 를 반환한다.
    #[test]
    fn single_focus_invariant() {
        let tree = three_leaf_tree();
        let mut router = FocusRouter::new();

        // 여러 번 상태 변경 후에도 current 는 하나
        router.apply(&tree, FocusCommand::Next);
        router.apply(&tree, FocusCommand::Next);
        router.apply(&tree, FocusCommand::Click(pid("c")));
        router.apply(&tree, FocusCommand::Prev);

        // current 는 정확히 하나의 Option<PaneId> 를 가진다
        assert!(router.current().is_some());
        // 그 ID 가 실제 트리에 존재하는 ID 중 하나여야 한다
        let valid_ids = [pid("a"), pid("b"), pid("c")];
        assert!(valid_ids.contains(router.current().unwrap()));
    }

    // -------------------------------------------------------
    // AC-P-22: mouse_click_focuses_pane
    // -------------------------------------------------------

    /// Click(PaneId) 명령이 해당 pane 을 focused 로 설정한다.
    #[test]
    fn mouse_click_focuses_pane() {
        let tree = three_leaf_tree();
        let mut router = FocusRouter::new();
        router.apply(&tree, FocusCommand::Click(pid("b")));
        assert_eq!(router.current(), Some(&pid("b")));
    }

    // -------------------------------------------------------
    // AC-P-22: unknown_pane_id_is_noop
    // -------------------------------------------------------

    /// 트리에 존재하지 않는 PaneId 에 Click 하면 상태 변화 없음 (no-op).
    #[test]
    fn unknown_pane_id_is_noop() {
        let tree = three_leaf_tree();
        let mut router = FocusRouter::new();
        router.apply(&tree, FocusCommand::Click(pid("b")));
        assert_eq!(router.current(), Some(&pid("b")));

        router.apply(&tree, FocusCommand::Click(pid("nonexistent")));
        // 상태 변화 없이 b 유지
        assert_eq!(router.current(), Some(&pid("b")));
    }

    // -------------------------------------------------------
    // AC-P-9a (MS-1): platform_mod_is_cmd_on_macos
    // -------------------------------------------------------

    /// macOS 에서 PLATFORM_MOD 는 Cmd 이다.
    #[cfg(target_os = "macos")]
    #[test]
    fn platform_mod_is_cmd_on_macos() {
        assert_eq!(PLATFORM_MOD, PlatformMod::Cmd);
    }

    // -------------------------------------------------------
    // AC-P-9b (MS-1): platform_mod_is_ctrl_on_non_macos
    // -------------------------------------------------------

    /// macOS 외 플랫폼에서 PLATFORM_MOD 는 Ctrl 이다.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn platform_mod_is_ctrl_on_non_macos() {
        assert_eq!(PLATFORM_MOD, PlatformMod::Ctrl);
    }

    // -------------------------------------------------------
    // AC-P-23: ctrl_b_passthrough_when_platform_is_ctrl
    // -------------------------------------------------------

    /// PLATFORM_MOD = Ctrl 시 순수 Ctrl+B 는 dispatch_key 에서 None 을 반환한다.
    /// tmux prefix 키(Ctrl+B)가 FocusRouter 에 소비되지 않는다 (AC-P-23).
    #[test]
    fn ctrl_b_passthrough_when_platform_is_ctrl() {
        // 이 테스트는 플랫폼 무관하게 Ctrl modifier 로 직접 dispatch_key 를 호출.
        // PLATFORM_MOD 가 Ctrl 인 경우의 동작을 시뮬레이션.
        //
        // Ctrl+B: ctrl=true, alt=false, cmd=false → dispatch_key 는 alt=false 이므로 None 반환.
        let mods = KeyModifiers {
            ctrl: true,
            alt: false,
            cmd: false,
            shift: false,
        };
        // KeyCode::Other 로 "B" 를 표현 (MS-1 에서 B 키는 Other)
        let result = dispatch_key(mods, KeyCode::Other);
        assert_eq!(
            result, None,
            "Ctrl+B 는 FocusRouter 에 소비되지 않아야 한다 (AC-P-23)"
        );
    }

    // -------------------------------------------------------
    // dispatch_key 매핑 검증 (플랫폼 별)
    // -------------------------------------------------------

    /// macOS: Cmd+Alt+Right → FocusCommand::Next.
    #[cfg(target_os = "macos")]
    #[test]
    fn dispatch_cmd_alt_right_is_next_on_macos() {
        let mods = KeyModifiers {
            cmd: true,
            alt: true,
            ctrl: false,
            shift: false,
        };
        assert_eq!(
            dispatch_key(mods, KeyCode::ArrowRight),
            Some(FocusCommand::Next)
        );
    }

    /// macOS: Cmd+Alt+Left → FocusCommand::Prev.
    #[cfg(target_os = "macos")]
    #[test]
    fn dispatch_cmd_alt_left_is_prev_on_macos() {
        let mods = KeyModifiers {
            cmd: true,
            alt: true,
            ctrl: false,
            shift: false,
        };
        assert_eq!(
            dispatch_key(mods, KeyCode::ArrowLeft),
            Some(FocusCommand::Prev)
        );
    }

    /// Linux: Ctrl+Alt+Right → FocusCommand::Next.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn dispatch_ctrl_alt_right_is_next_on_linux() {
        let mods = KeyModifiers {
            ctrl: true,
            alt: true,
            cmd: false,
            shift: false,
        };
        assert_eq!(
            dispatch_key(mods, KeyCode::ArrowRight),
            Some(FocusCommand::Next)
        );
    }

    /// Linux: Ctrl+Alt+Left → FocusCommand::Prev.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn dispatch_ctrl_alt_left_is_prev_on_linux() {
        let mods = KeyModifiers {
            ctrl: true,
            alt: true,
            cmd: false,
            shift: false,
        };
        assert_eq!(
            dispatch_key(mods, KeyCode::ArrowLeft),
            Some(FocusCommand::Prev)
        );
    }

    // -------------------------------------------------------
    // wrap-around: 첫 번째 pane 에서 Prev → 마지막 pane
    // -------------------------------------------------------

    /// 첫 번째 pane 에서 Prev 를 호출하면 마지막 pane 으로 wrap-around 된다.
    #[test]
    fn wraparound_at_first_pane() {
        let tree = three_leaf_tree();
        let mut router = FocusRouter::new();
        router.apply(&tree, FocusCommand::Click(pid("a")));
        router.apply(&tree, FocusCommand::Prev);
        assert_eq!(router.current(), Some(&pid("c")));
    }
}
