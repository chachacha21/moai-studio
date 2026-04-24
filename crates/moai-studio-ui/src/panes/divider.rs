//! `ResizableDivider` 추상 trait — drag clamp 계약 (SPEC-V3-003 T3).
//!
//! ## 모듈 역할
//!
//! divider drag 이벤트를 ratio 변경으로 변환하는 **인터페이스 계약**을 정의한다.
//!
//! ## Clamp 정책
//!
//! `on_drag` 의 반환값은 항상 `[min_ratio, 1.0 - min_ratio]` 구간으로 clamp 된다.
//! `min_ratio` 는 `PaneConstraints::{MIN_COLS, MIN_ROWS}` 에서 유도된다.
//!
//! T5 구체 구현에서 orientation (Horizontal/Vertical) 에 따라:
//! - Horizontal → MIN_COLS × px_per_col 기준
//! - Vertical   → MIN_ROWS × px_per_row 기준
//!
//! ## AC-P-17 연계
//!
//! `MockDivider` + `ResizableDivider` 결합이 구체 GPUI 구현 없이 컴파일됨을
//! `tests::abstract_traits_compile_without_impl` 단위 테스트가 검증한다.
//!
//! @MX:TODO(T5): Spike 1 결과 기반 구체 구현. Clamp 로직은 PaneConstraints::{MIN_COLS, MIN_ROWS} 준수.

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
// MockDivider — #[cfg(test)] 전용
// ============================================================

// @MX:NOTE: [AUTO] test-only-impl — drag clamp 수식의 단위 검증 + T5 refine 용 baseline

#[cfg(test)]
use crate::panes::{PaneConstraints, SplitDirection};

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
