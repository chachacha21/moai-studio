//! `PaneConstraints` 최소 pane 크기 불변 상수 (40 cols × 10 rows).
//!
//! ## 스펙 참조
//!
//! - spec.md §5 RG-P-2 REQ-P-010, REQ-P-014
//! - spec.md M-2 (v0.2.0 iter2 해소): associated const 공개 값으로 단일화, runtime 변경 불가
//! - AC-P-21 negative API surface
//!
//! ## 설계 계약
//!
//! - unit struct: `pub struct PaneConstraints;` — 의도적 non-instantiable 마커 타입
//! - associated const 2개 (MIN_COLS, MIN_ROWS) — 외부 변경 불가, runtime 수정 금지
//! - 가변 API 절대 금지 (new / with_* / set_* / Builder 패턴) — doc test compile_fail 로 enforce
//! - `#[derive]` 없음 — 인스턴스 활용 자체를 사용하지 않으므로 불필요
//!
//! ## AC-P-21 Negative API Surface
//!
//! 아래 코드 블록은 **컴파일되어서는 안 된다**.
//! `compile_fail` doc test 가 `cargo test --doc -p moai-studio-ui` 에서 실행되며,
//! 실제로 컴파일 실패해야 PASS 처리된다.
//!
//! ```compile_fail
//! use moai_studio_ui::panes::PaneConstraints;
//! let _ = PaneConstraints::new(50, 15);
//! ```
//!
//! ```compile_fail
//! use moai_studio_ui::panes::PaneConstraints;
//! let mut pc = PaneConstraints;
//! pc.set_min_cols(50);
//! ```
//!
//! ```compile_fail
//! use moai_studio_ui::panes::PaneConstraints;
//! let _: &str = PaneConstraints::MIN_COLS;
//! ```

// @MX:ANCHOR: [AUTO] pane-constraints-immutable
// @MX:REASON: [AUTO] 최소 pane 크기 제약 (REQ-P-010) 의 단일 진실원.
//   fan_in >= 3 예상 (T4 split 거부 판정, T5 divider drag clamp, T7 RootView 최소 크기 계산).
//   runtime 변경 가능 API 노출은 AC-P-21 에 의해 금지된다.
pub struct PaneConstraints;

impl PaneConstraints {
    /// 최소 pane 폭 (cols). 하위 경계 판정은 strict `< MIN_COLS` (40 cols 는 허용).
    pub const MIN_COLS: u16 = 40;

    /// 최소 pane 높이 (rows). 하위 경계 판정은 strict `< MIN_ROWS` (10 rows 는 허용).
    pub const MIN_ROWS: u16 = 10;
}

#[cfg(test)]
mod tests {
    use super::PaneConstraints;

    #[test]
    fn const_values_are_40_and_10() {
        // AC-P-21: 상수 값 검증
        assert_eq!(PaneConstraints::MIN_COLS, 40);
        assert_eq!(PaneConstraints::MIN_ROWS, 10);
    }

    #[test]
    fn const_types_are_u16() {
        // 컴파일 타임 타입 강제: u16 로 할당 가능해야 한다
        const _C: u16 = PaneConstraints::MIN_COLS;
        const _R: u16 = PaneConstraints::MIN_ROWS;
    }

    #[test]
    fn instantiation_via_unit_struct() {
        // unit struct 는 값 생성 가능 (가변 API 없이 마커로만 사용)
        let _pc = PaneConstraints;
    }
}
