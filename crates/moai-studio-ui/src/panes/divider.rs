//! `ResizableDivider` 추상 trait + `GpuiDivider` 구체 구현 (SPEC-V3-003 T3/T5).
//!
//! ## 모듈 역할
//!
//! divider drag 이벤트를 ratio 변경으로 변환하는 **인터페이스 계약** 과
//! `PaneConstraints` 기반 clamp 를 수행하는 **프로덕션 구체 구현** 을 제공한다.
//!
//! ## Clamp 정책
//!
//! `on_drag` 의 반환값은 항상 `[min_ratio, 1.0 - min_ratio]` 구간으로 clamp 된다.
//! `min_ratio` 는 `PaneConstraints::{MIN_COLS, MIN_ROWS}` 에서 유도된다.
//!
//! orientation 별 분기:
//! - `Horizontal` → `MIN_COLS × px_per_col` 기준
//! - `Vertical`   → `MIN_ROWS × px_per_row` 기준
//!
//! ## AC-P-17 연계
//!
//! `MockDivider` + `ResizableDivider` 결합이 구체 GPUI 구현 없이 컴파일됨을
//! `tests::abstract_traits_compile_without_impl` 단위 테스트가 검증한다.
//!
//! ## GPUI 배선 (T7 범위)
//!
//! `GpuiDivider` 는 순수 Rust 계산만 포함한다.
//! GPUI `on_mouse_move` → `GpuiDivider::on_drag` 연결은 T7 RootView wire-up 에서 수행한다.

use crate::panes::{PaneConstraints, SplitDirection};

// ============================================================
// ResizableDivider trait
// ============================================================

// @MX:ANCHOR: [AUTO] divider-contract
// @MX:REASON: [AUTO] sibling ratio 갱신 책임 추상. 구체 구현 (T5 GpuiDivider) 은 plan spike 결과로 결정.
//   PaneConstraints::{MIN_COLS,MIN_ROWS} 참조 계약 명시.
//   fan_in >= 3 (T5 GpuiDivider, T7 RootView drag callback, T4 headless resize test).

/// divider drag 이벤트를 ratio 변경으로 변환하는 추상 계약.
///
/// ## 수식 (MockDivider 기준, T5 에서 정밀화)
///
/// ```text
/// raw_ratio = (current_ratio * total_px + delta_px) / total_px
/// result    = clamp(raw_ratio, [min_ratio, 1.0 - min_ratio])
/// min_ratio = min_ratio_for(sibling_min_px) / total_px  (근사)
/// ```
///
/// T5 에서 orientation 에 따라 `sibling_min_px` 계산 방식이 달라진다.
pub trait ResizableDivider {
    /// drag delta 를 반영한 새 ratio 를 반환한다.
    ///
    /// ## 인자
    ///
    /// - `delta_px`: drag 이동 픽셀 (양수 = first 방향으로 확대).
    /// - `total_px`: 두 sibling 을 합한 전체 픽셀 크기.
    ///
    /// ## 반환
    ///
    /// clamp 된 ratio. 항상 `(0.0, 1.0)` 범위 내, 최소 sibling 크기 준수.
    fn on_drag(&mut self, delta_px: f32, total_px: f32) -> f32;

    /// sibling 이 `sibling_px` 크기일 때 필요한 최소 ratio 를 반환한다.
    ///
    /// ## 수식 (MockDivider 기준)
    ///
    /// ```text
    /// min_ratio = sibling_min_px / total_px
    /// ```
    ///
    /// `sibling_px` 는 total 에 대한 sibling 의 픽셀 크기이다.
    /// T5 에서 `total_px` 를 명시적으로 받도록 시그니처가 변경될 수 있다.
    ///
    /// ## 가정 (MockDivider)
    ///
    /// `sibling_px` 를 total 로 간주하여 `self.sibling_min_px / sibling_px` 를 반환한다.
    /// T5 refine 시 수식과 가정을 docstring 에 업데이트한다.
    fn min_ratio_for(&self, sibling_px: f32) -> f32;
}

// ============================================================
// GpuiDivider — 프로덕션 구체 구현체 (T5)
// ============================================================

// @MX:ANCHOR: [AUTO] concrete-divider-gpui
// @MX:REASON: [AUTO] GpuiDivider 는 ResizableDivider 의 프로덕션 구체 구현체.
//   fan_in >= 2 예상: T7 RootView drag callback, T11 future bench.
//   순수 Rust 계산만 포함 — GPUI on_mouse_move 배선은 T7 범위.

/// GPUI native 이벤트 기반 `ResizableDivider` 구체 구현체.
///
/// ## 역할
///
/// drag delta 를 orientation 에 맞는 min_px 제약으로 clamp 하여 새 ratio 를 반환한다.
///
/// ## GPUI 배선 (T7 범위)
///
/// 이 구조체는 순수 Rust 계산만 제공한다. GPUI `on_mouse_move` → `GpuiDivider::on_drag`
/// 연결은 T7 RootView wire-up 에서 수행한다.
///
/// ## orientation 별 min_px 계산
///
/// - `Horizontal` → `PaneConstraints::MIN_COLS × px_per_col`
/// - `Vertical`   → `PaneConstraints::MIN_ROWS × px_per_row`
pub struct GpuiDivider {
    /// 분할 방향: Horizontal = 좌/우 (수직 divider), Vertical = 상/하 (수평 divider).
    orientation: SplitDirection,
    /// 현재 ratio (0.0 < ratio < 1.0).
    current_ratio: f32,
    /// 열(col) 당 픽셀 크기 — Horizontal split 에서 MIN_COLS 환산에 사용.
    px_per_col: f32,
    /// 행(row) 당 픽셀 크기 — Vertical split 에서 MIN_ROWS 환산에 사용.
    px_per_row: f32,
}

impl GpuiDivider {
    /// 새 GpuiDivider 를 생성한다.
    ///
    /// # Arguments
    ///
    /// - `orientation`: 분할 방향 (`SplitDirection::Horizontal` 또는 `Vertical`).
    /// - `initial_ratio`: 초기 ratio (`0.0 < ratio < 1.0`).
    /// - `px_per_col`: 열 당 픽셀 크기 (Horizontal split 최소 크기 계산).
    /// - `px_per_row`: 행 당 픽셀 크기 (Vertical split 최소 크기 계산).
    pub fn new(
        orientation: SplitDirection,
        initial_ratio: f32,
        px_per_col: f32,
        px_per_row: f32,
    ) -> Self {
        Self {
            orientation,
            current_ratio: initial_ratio,
            px_per_col,
            px_per_row,
        }
    }

    /// orientation 에 따른 최소 sibling 픽셀 크기를 반환한다.
    ///
    /// - `Horizontal` → `PaneConstraints::MIN_COLS × px_per_col`
    /// - `Vertical`   → `PaneConstraints::MIN_ROWS × px_per_row`
    fn min_px_for_orientation(&self) -> f32 {
        match self.orientation {
            SplitDirection::Horizontal => PaneConstraints::MIN_COLS as f32 * self.px_per_col,
            SplitDirection::Vertical => PaneConstraints::MIN_ROWS as f32 * self.px_per_row,
        }
    }
}

impl ResizableDivider for GpuiDivider {
    /// drag delta 를 반영한 새 ratio 를 반환한다.
    ///
    /// # @MX:NOTE: [AUTO] ratio-clamp-enforces-min-size
    /// REQ-P-012 / AC-P-6: ratio 는 항상 `[min_ratio, 1.0 - min_ratio]` 로 clamp 된다.
    /// min_ratio = min_px_for_orientation() / total_px.
    /// 양 sibling 모두 최소 `PaneConstraints::MIN_COLS` (Horizontal) 또는
    /// `PaneConstraints::MIN_ROWS` (Vertical) 크기를 보장한다.
    fn on_drag(&mut self, delta_px: f32, total_px: f32) -> f32 {
        // @MX:NOTE: [AUTO] ratio-clamp-enforces-min-size
        // REQ-P-012 / AC-P-6: raw ratio 에 clamp 적용.
        let min_ratio = self.min_ratio_for(total_px);
        let max_ratio = 1.0 - min_ratio;

        // raw ratio = (현재 ratio × total_px + delta_px) / total_px
        let raw = (self.current_ratio * total_px + delta_px) / total_px;

        // clamp 적용
        let clamped = raw.clamp(min_ratio, max_ratio);
        self.current_ratio = clamped;
        clamped
    }

    /// `total_px` 에 대한 최소 ratio 를 반환한다.
    ///
    /// `min_ratio = min_px_for_orientation() / total_px`
    fn min_ratio_for(&self, total_px: f32) -> f32 {
        self.min_px_for_orientation() / total_px
    }
}

// ============================================================
// MockDivider — #[cfg(test)] 전용
// ============================================================

// @MX:NOTE: [AUTO] test-only-impl — drag clamp 수식의 단위 검증 + T5 refine 용 baseline

/// 테스트 전용 `ResizableDivider` 구현체.
///
/// ## 필드
///
/// - `orientation`: 분할 방향 (Horizontal/Vertical). T5 에서 MIN_COLS vs MIN_ROWS 분기 기준.
/// - `current_ratio`: 현재 ratio (0.0 < ratio < 1.0).
/// - `sibling_min_px`: 한 쪽 sibling 의 최소 픽셀 크기.
///   MockDivider 생성 시 외부에서 주입 (`PaneConstraints::MIN_COLS × px_per_col` 가정).
#[cfg(test)]
pub struct MockDivider {
    pub orientation: SplitDirection,
    pub current_ratio: f32,
    pub sibling_min_px: f32,
}

#[cfg(test)]
impl MockDivider {
    /// MockDivider 를 생성한다.
    ///
    /// ## 인자
    ///
    /// - `orientation`: 분할 방향.
    /// - `initial_ratio`: 초기 ratio (0.0 < ratio < 1.0).
    /// - `sibling_min_px`: 최소 sibling 픽셀 크기 (예: `MIN_COLS × px_per_col = 40 × 3 = 120.0`).
    pub fn new(orientation: SplitDirection, initial_ratio: f32, sibling_min_px: f32) -> Self {
        Self {
            orientation,
            current_ratio: initial_ratio,
            sibling_min_px,
        }
    }

    /// ratio 를 `[min_ratio, 1.0 - min_ratio]` 로 clamp 한다.
    fn clamp_ratio(&self, raw: f32, total_px: f32) -> f32 {
        let min_r = self.sibling_min_px / total_px;
        let max_r = 1.0 - min_r;
        raw.clamp(min_r, max_r)
    }

    /// PaneConstraints 상수 접근 예시 (orientation 별 분기 — T5 refine 기준).
    ///
    /// 현재 MockDivider 는 `sibling_min_px` 를 외부에서 주입받으므로 이 메서드는
    /// T5 구체 구현의 참조 경로를 문서화하기 위한 것이다.
    #[allow(dead_code)]
    fn min_px_for_orientation(&self) -> f32 {
        match self.orientation {
            SplitDirection::Horizontal => PaneConstraints::MIN_COLS as f32,
            SplitDirection::Vertical => PaneConstraints::MIN_ROWS as f32,
        }
    }
}

#[cfg(test)]
impl ResizableDivider for MockDivider {
    fn on_drag(&mut self, delta_px: f32, total_px: f32) -> f32 {
        // raw ratio: 현재 ratio 에 delta 를 픽셀 단위로 더한 뒤 total 로 정규화.
        let raw = (self.current_ratio * total_px + delta_px) / total_px;
        let clamped = self.clamp_ratio(raw, total_px);
        self.current_ratio = clamped;
        clamped
    }

    fn min_ratio_for(&self, sibling_px: f32) -> f32 {
        // 수식: sibling_min_px / sibling_px (sibling_px 를 total 로 간주한 근사).
        // T5 에서 total_px 를 별도 인자로 받아 정밀화.
        self.sibling_min_px / sibling_px
    }
}

// ============================================================
// #[cfg(test)] 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------
    // AC-P-17 연계: abstract_traits_compile_without_impl
    // -------------------------------------------------------

    /// MockDivider 를 trait object 로 사용 가능함 → AC-P-17 충족.
    #[test]
    fn abstract_traits_compile_without_impl() {
        fn accept(_: &dyn ResizableDivider) {}
        let d = MockDivider::new(SplitDirection::Horizontal, 0.5, 120.0);
        accept(&d);
    }

    // -------------------------------------------------------
    // GpuiDivider T5 테스트 (AC-P-6, AC-P-4)
    // -------------------------------------------------------

    /// AC-P-6: on_drag 은 ratio 를 항상 [min_ratio, 1.0 - min_ratio] 범위로 clamp 한다.
    ///
    /// 설정: Horizontal, px_per_col=10.0, px_per_row=10.0, total=800px
    /// min_px = MIN_COLS(40) × 10.0 = 400px
    /// min_ratio = 400 / 800 = 0.5
    /// → valid range = [0.5, 0.5] (극단 케이스이지만 boundary 테스트용)
    ///
    /// 실제 테스트: total=1200px, px_per_col=5.0
    /// min_px = 40 × 5.0 = 200px
    /// min_ratio = 200 / 1200 ≈ 0.1667
    /// initial_ratio=0.5, delta=+700px → raw=(0.5*1200+700)/1200=1300/1200=1.0833 → clamped=0.8333
    #[test]
    fn drag_clamps_ratio() {
        // Horizontal, px_per_col=5.0, px_per_row=10.0
        // MIN_COLS=40 → min_px = 200.0
        // total=1200px → min_ratio = 200/1200 ≈ 0.1667
        let mut d = GpuiDivider::new(SplitDirection::Horizontal, 0.5, 5.0, 10.0);

        // (1) 극단 양수 delta — max clamp
        let max_clamped = d.on_drag(700.0, 1200.0);
        let expected_min = 200.0_f32 / 1200.0; // ≈ 0.1667
        let expected_max = 1.0 - expected_min; // ≈ 0.8333
        assert!(
            (max_clamped - expected_max).abs() < 1e-4,
            "극단 양수 delta → max clamp 예상 {expected_max:.4}, 실제 {max_clamped:.4}"
        );

        // (2) 극단 음수 delta — min clamp
        let mut d2 = GpuiDivider::new(SplitDirection::Horizontal, 0.5, 5.0, 10.0);
        let min_clamped = d2.on_drag(-700.0, 1200.0);
        assert!(
            (min_clamped - expected_min).abs() < 1e-4,
            "극단 음수 delta → min clamp 예상 {expected_min:.4}, 실제 {min_clamped:.4}"
        );

        // (3) 범위 내 정상 delta
        let mut d3 = GpuiDivider::new(SplitDirection::Horizontal, 0.5, 5.0, 10.0);
        let normal = d3.on_drag(120.0, 1200.0);
        // raw = (0.5 * 1200 + 120) / 1200 = 720/1200 = 0.6
        assert!(
            (normal - 0.6).abs() < 1e-4,
            "정상 delta → 0.6 예상, 실제 {normal}"
        );
        // clamp 범위 내임을 확인
        assert!(
            normal >= expected_min && normal <= expected_max,
            "결과 ratio 가 [min_ratio, max_ratio] 내에 있어야 함"
        );
    }

    /// AC-P-4 부분: Horizontal orientation → min_px 는 MIN_COLS × px_per_col 로 계산된다.
    ///
    /// GpuiDivider::min_ratio_for(total_px) = (MIN_COLS × px_per_col) / total_px
    #[test]
    fn horizontal_uses_min_cols() {
        // px_per_col=3.0 → min_px = 40 × 3.0 = 120.0
        let d = GpuiDivider::new(SplitDirection::Horizontal, 0.5, 3.0, 10.0);
        let min_ratio = d.min_ratio_for(400.0);
        // 120.0 / 400.0 = 0.3
        assert!(
            (min_ratio - 0.3).abs() < 1e-5,
            "Horizontal min_ratio 예상 0.3, 실제 {min_ratio}"
        );
    }

    /// AC-P-4 부분: Vertical orientation → min_px 는 MIN_ROWS × px_per_row 로 계산된다.
    ///
    /// GpuiDivider::min_ratio_for(total_px) = (MIN_ROWS × px_per_row) / total_px
    #[test]
    fn vertical_uses_min_rows() {
        // px_per_row=5.0 → min_px = 10 × 5.0 = 50.0
        let d = GpuiDivider::new(SplitDirection::Vertical, 0.5, 10.0, 5.0);
        let min_ratio = d.min_ratio_for(400.0);
        // 50.0 / 400.0 = 0.125
        assert!(
            (min_ratio - 0.125).abs() < 1e-5,
            "Vertical min_ratio 예상 0.125, 실제 {min_ratio}"
        );
    }

    /// delta 가 ratio 를 min 이하로 낮추면 min_ratio 로 clamp 된다.
    #[test]
    fn delta_below_min_clamps_to_min_ratio() {
        // Horizontal, px_per_col=6.0 → min_px=240, total=800 → min_ratio=0.3
        let mut d = GpuiDivider::new(SplitDirection::Horizontal, 0.5, 6.0, 10.0);
        // delta=-300 → raw=(0.5*800-300)/800=100/800=0.125 < 0.3 → clamp to 0.3
        let result = d.on_drag(-300.0, 800.0);
        let min_ratio = 240.0_f32 / 800.0; // 0.3
        assert!(
            (result - min_ratio).abs() < 1e-5,
            "min clamp 예상 {min_ratio}, 실제 {result}"
        );
    }

    /// delta 가 ratio 를 1.0-min 이상으로 올리면 1.0-min_ratio 로 clamp 된다.
    #[test]
    fn delta_above_max_clamps_to_max_ratio() {
        // Horizontal, px_per_col=6.0 → min_px=240, total=800 → min_ratio=0.3, max_ratio=0.7
        let mut d = GpuiDivider::new(SplitDirection::Horizontal, 0.5, 6.0, 10.0);
        // delta=+300 → raw=(0.5*800+300)/800=700/800=0.875 > 0.7 → clamp to 0.7
        let result = d.on_drag(300.0, 800.0);
        let max_ratio = 1.0 - 240.0_f32 / 800.0; // 0.7
        assert!(
            (result - max_ratio).abs() < 1e-5,
            "max clamp 예상 {max_ratio}, 실제 {result}"
        );
    }

    // -------------------------------------------------------
    // on_drag — 정상 범위 내 clamp
    // -------------------------------------------------------

    /// delta 가 clamp 범위 내일 때 on_drag 결과가 정확히 계산된다.
    ///
    /// total=400px, min_px=120px → min_ratio=0.3, max_ratio=0.7
    /// 초기 ratio=0.5, delta=+40px → raw=0.6 → clamped=0.6
    #[test]
    fn mock_divider_on_drag_returns_clamped_ratio() {
        let mut d = MockDivider::new(SplitDirection::Horizontal, 0.5, 120.0);
        let result = d.on_drag(40.0, 400.0);

        // raw = (0.5 * 400 + 40) / 400 = 240/400 = 0.6
        assert!((result - 0.6).abs() < 1e-5, "예상 0.6, 실제 {result}");
        assert!((d.current_ratio - 0.6).abs() < 1e-5, "상태 갱신 확인");
    }

    // -------------------------------------------------------
    // on_drag — sibling 최소 크기 미만 clamp
    // -------------------------------------------------------

    /// drag 로 한쪽 sibling 이 MIN 미만이 되면 clamp 된 max ratio 반환.
    ///
    /// total=400px, min_px=120px → min_ratio=0.3, max_ratio=0.7
    /// 초기 ratio=0.5, delta=+200px → raw=1.0 → clamped=0.7
    #[test]
    fn mock_divider_drag_clamps_below_min_sibling() {
        let mut d = MockDivider::new(SplitDirection::Horizontal, 0.5, 120.0);
        let result = d.on_drag(200.0, 400.0);

        // max_ratio = 1.0 - 120/400 = 0.7
        assert!(
            (result - 0.7).abs() < 1e-5,
            "max clamp 예상 0.7, 실제 {result}"
        );
    }

    // -------------------------------------------------------
    // min_ratio_for — PaneConstraints::MIN_COLS × px_per_col 기준
    // -------------------------------------------------------

    /// min_ratio_for(sibling_px=120.0) 에서 MIN_COLS(40) × 3px/col = 120px 기준
    /// min_ratio = 120 / 120 = 1.0 이지만,
    /// sibling_min_px=120 이 sibling 자체의 minimum 이므로
    /// min_ratio_for(sibling_px) = sibling_min_px / sibling_px = 120/120 = 1.0.
    ///
    /// 현실적 사용: sibling_px 는 total_px 보다 크므로 비율은 < 1.0 이 됨.
    /// 예: total=400, sibling_min=120 → min_ratio = 120/400 = 0.3
    /// min_ratio_for(400.0) = 0.3
    #[test]
    fn mock_divider_min_ratio_for_sibling() {
        // 생성: sibling_min_px = 40(MIN_COLS) × 3(px/col) = 120
        let d = MockDivider::new(SplitDirection::Horizontal, 0.5, 120.0);

        // min_ratio_for(400.0) = 120 / 400 = 0.3
        let ratio = d.min_ratio_for(400.0);
        assert!(
            (ratio - 0.3).abs() < 1e-5,
            "예상 min_ratio=0.3, 실제 {ratio}"
        );
    }
}
