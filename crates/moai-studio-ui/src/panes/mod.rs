//! Pane tree 와 split 관리 (SPEC-V3-003 MS-1).
//!
//! 공개 API:
//! - [`PaneTree`] — 이진 트리 pane 자료구조 (Leaf / Split)
//! - [`PaneId`] — pane 식별자 (Spike 3 결정 따름)
//! - [`SplitDirection`] — Horizontal (좌/우) / Vertical (상/하)
//!
//! @MX:TODO: [AUTO] T2 PaneConstraints, T3 PaneSplitter/ResizableDivider 는 별도 파일에 추가 예정.

pub mod tree;
pub use tree::{Leaf, PaneId, PaneTree, RatioError, SplitDirection, SplitError, SplitNodeId};
