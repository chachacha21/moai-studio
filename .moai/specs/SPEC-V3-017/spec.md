---
id: SPEC-V3-017
version: 1.0.0
status: ready
created_at: 2026-04-29
updated_at: 2026-04-29
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-004, SPEC-V3-010]
parallel_with: []
optional_integration: [SPEC-V3-015]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, ui, gpui, quality, trust5, visualization, radar-chart]
revision: v1.0.0 (initial draft, TRUST 5 Quality Dashboard)
---

# SPEC-V3-017: TRUST 5 Quality Dashboard -- 5-axis radar chart visualization + quality history + gate threshold display

## HISTORY

| Version | Date | Change |
|---------|------|--------|
| 1.0.0-draft | 2026-04-29 | Initial draft. TRUST 5 Quality Dashboard SPEC. Data model + scoring engine (MS-1), radar chart GPUI Canvas rendering (MS-2), dashboard integration + quality history (MS-3). 28 EARS requirements across 7 requirement groups. 12 acceptance criteria. 3 milestones. Depends on SPEC-V3-004 (Render Layer), SPEC-V3-010 (Agent Dashboard). Optional integration with SPEC-V3-015. |

---

## 1. Overview

### 1.1 Purpose

This SPEC defines the TRUST 5 Quality Dashboard -- a real-time quality visualization surface that renders project health across five dimensions (Tested, Readable, Unified, Secured, Trackable) as an interactive radar chart, complemented by per-project quality history trends and quality gate threshold indicators.

The dashboard aggregates scoring data from multiple sources:

1. **LSP diagnostics** -- type errors, lint errors, warnings (from `quality.yaml` thresholds)
2. **Test runner output** -- coverage percentage, test count, pass/fail rates
3. **Git metadata** -- conventional commit compliance, SPEC reference coverage, MX tag coverage
4. **Computed metrics** -- cyclomatic complexity, naming convention adherence, dependency audit status

### 1.2 TRUST 5 Dimensions

| Dimension | Abbreviation | Primary Metric | Source |
|-----------|-------------|----------------|--------|
| **Tested** | T | Coverage %, test pass rate | `cargo test` output, `quality.yaml` `test_coverage_target` |
| **Readable** | R | Lint errors = 0, naming score | `cargo clippy`, naming convention heuristic |
| **Unified** | U | Format compliance % | `cargo fmt --check`, style consistency score |
| **Secured** | S | Security scan pass, dependency audit | `cargo audit`, OWASP checklist |
| **Trackable** | K | Conventional commit %, SPEC ref %, MX tag % | `git log` analysis, `.moai/specs/` scan, `@MX` grep |

### 1.3 Relationship to SPEC-V3-010

V3-010 (Agent Progress Dashboard) visualizes agent runtime activity (events, costs, instructions). V3-017 (TRUST 5 Quality Dashboard) visualizes project quality health. They share:

- The same `AgentDashboardView` container -- V3-017 is integrated as a tab or panel within the existing 5-pane layout
- Design tokens (`crate::design::tokens`)
- GPUI Render trait patterns

```
SPEC-V3-010 AgentDashboardView (5-pane container)
  └── [NEW] Quality tab/panel
       ├── RadarChartView (MS-2, RG-QD-3)
       ├── QualityHistoryView (MS-3, RG-QD-5)
       ├── QualityGateView (MS-3, RG-QD-6)
       └── DimensionDetailView (MS-3, RG-QD-7)
```

### 1.4 Reference Documents

- `.moai/config/sections/quality.yaml` -- Quality gate thresholds and LSP integration settings
- `.claude/rules/moai/core/moai-constitution.md` -- TRUST 5 framework definition
- `crates/moai-studio-ui/src/agent/dashboard_view.rs` -- Existing 5-pane dashboard layout
- `crates/moai-studio-ui/src/banners/mod.rs` -- Severity enum pattern (5 levels)
- `crates/moai-studio-ui/src/design/tokens.rs` -- Design token constants

---

## 2. Background and Motivation

- **Quality visibility gap**: moai-studio displays agent events and costs (V3-010) but provides no visual summary of project quality health. The TRUST 5 framework is enforced in CI but has no GUI representation.
- **Threshold blindspot**: `quality.yaml` defines quality gate thresholds (max_errors=0, max_type_errors=0, max_lint_errors=0) but these are invisible until a CI failure occurs.
- **Trend deficit**: Without historical quality tracking, regressions are detected only by CI failure, not by gradual degradation trends.
- **Design v3 E-2 mandate**: The TRUST 5 Quality Dashboard is explicitly listed as a Design v3 feature requiring implementation.

---

## 3. Goals and Non-Goals

### 3.1 Goals

- **G1.** The system computes a TRUST 5 score (0.0--1.0) for each of the five dimensions based on measurable project metrics.
- **G2.** The system renders a 5-axis radar chart using GPUI Canvas custom drawing, displaying all five dimension scores simultaneously.
- **G3.** The system displays quality gate thresholds from `quality.yaml` with pass/fail indicators per dimension.
- **G4.** The system maintains per-project quality history (score snapshots over time) and renders trend lines.
- **G5.** The system integrates into the existing `AgentDashboardView` container without modifying the terminal/panes/tabs core.
- **G6.** The radar chart renders at 60fps with smooth score transitions (no jank on score updates).
- **G7.** The dashboard is responsive to real-time metric changes (LSP diagnostics, test results) within 1 second.

### 3.2 Non-Goals

- **N1.** Custom threshold editing from the GUI -- thresholds remain in `quality.yaml` and are edited via text.
- **N2.** Cross-project quality comparison -- v1.0.0 is single-project scope.
- **N3.** Quality score export or API -- follow-up SPEC if needed.
- **N4.** Machine learning-based quality prediction -- manual heuristic scoring only.
- **N5.** Integration with external CI/CD platforms -- only local metrics sources.
- **N6.** Adding new design tokens -- reuses existing `status.{success,warning,error,info}` and `chart.{1..8}` from V3-010.
- **N7.** terminal/panes/tabs core modifications -- RG-P-7 carry.
- **N8.** Windows build -- per V3-002/003/004 N carry.
- **N9.** New external crate for chart rendering -- GPUI Canvas custom drawing only (no SVG/Canvas2D dependency).
- **N10.** Per-file quality drilldown -- dimension-level detail only.

---

## 4. User Stories

- **US-QD-1**: A developer opens the Quality Dashboard and sees a radar chart showing that their project scores 0.92 Tested, 0.85 Readable, 0.95 Unified, 0.70 Secured, 0.88 Trackable, immediately identifying "Secured" as the weakest dimension.
- **US-QD-2**: A developer runs a test suite, and within 1 second the "Tested" axis on the radar chart animates from 0.78 to 0.85, while the overall score badge updates from "B" to "A".
- **US-QD-3**: A developer opens the Quality History view and sees a line chart showing the "Tested" score gradually increasing from 0.60 to 0.85 over the past 20 commits, confirming that the testing effort is paying off.
- **US-QD-4**: A developer hovers over the "Secured" axis of the radar chart and sees a tooltip listing: "2 dependency vulnerabilities, 0 OWASP violations, cargo audit: FAIL".
- **US-QD-5**: A developer sees red gate indicators on the "Readable" dimension with the detail: "3 clippy warnings, threshold: 0 -- GATE FAIL".
- **US-QD-6**: A developer with no active agent run sees the Quality Dashboard as a standalone view, while a developer with an active agent run sees it as a tab within the Agent Dashboard.

---

## 5. Requirements (EARS)

### RG-QD-1 -- Trust5Score Data Model

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-QD-001 | Ubiquitous | The system shall maintain a `Trust5Score` struct with five `f32` fields (tested, readable, unified, secured, trackable), each in the range [0.0, 1.0]. | The system **shall** maintain a `Trust5Score` struct with five dimension scores, each bounded [0.0, 1.0]. |
| REQ-QD-002 | Ubiquitous | The system shall compute an overall quality score as the arithmetic mean of the five dimension scores. | The system **shall** compute the overall score as the mean of all five dimensions. |
| REQ-QD-003 | Event-Driven | When a metric source provides updated data (LSP diagnostics, test output, git log), the system shall recompute the affected dimension score and emit a `ScoreUpdated` event. | When a metric source updates, the system **shall** recompute the affected dimension and emit `ScoreUpdated`. |
| REQ-QD-004 | Unwanted | The system shall not allow any dimension score to exceed 1.0 or fall below 0.0; values outside range are clamped. | The system **shall not** allow scores outside [0.0, 1.0]; out-of-range values are clamped. |

### RG-QD-2 -- Metric Scoring Engine

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-QD-005 | State-Driven | While LSP diagnostics are available, the system shall compute the "Readable" score as `1.0 - (lint_errors / max(1, total_lines / 100))` and the "Unified" score as `1.0 - (fmt_errors / max(1, total_files))`. | While LSP diagnostics are available, the system **shall** compute Readable and Unified scores from error counts normalized by file/line counts. |
| REQ-QD-006 | State-Driven | While test coverage data is available, the system shall compute the "Tested" score as `min(1.0, actual_coverage / target_coverage)` where `target_coverage` is read from `quality.yaml` `test_coverage_target`. | While test data is available, the system **shall** compute Tested score as actual/target coverage ratio. |
| REQ-QD-007 | State-Driven | While git history is available, the system shall compute the "Trackable" score as the weighted average of: (a) conventional commit compliance % (weight 0.4), (b) SPEC reference coverage % (weight 0.3), (c) MX tag coverage of high-fan-in functions % (weight 0.3). | While git history is available, the system **shall** compute Trackable as a weighted average of commit, SPEC, and MX tag metrics. |
| REQ-QD-008 | State-Driven | While security scan data is available, the system shall compute the "Secured" score as: if critical/high vulnerabilities > 0 then 0.0; else `1.0 - (medium_vulns * 0.1 + low_vulns * 0.05)`, clamped to [0.0, 1.0]. | While security data is available, the system **shall** compute Secured score penalizing vulnerabilities by severity. |
| REQ-QD-009 | Ubiquitous | The system shall provide a `ScoringEngine` trait that accepts metric snapshots and produces `Trust5Score`, enabling alternative scoring strategies via trait implementation. | The system **shall** define a `ScoringEngine` trait for swappable scoring strategies. |

### RG-QD-3 -- Radar Chart GPUI Canvas Rendering

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-QD-010 | Ubiquitous | The system shall render a 5-axis radar chart using GPUI Canvas element with custom drawing primitives (lines, filled polygons, text labels). No external SVG or Canvas2D crate dependency. | The system **shall** render a 5-axis radar chart via GPUI Canvas custom drawing, without external chart libraries. |
| REQ-QD-011 | Ubiquitous | The system shall label each radar axis with the TRUST 5 dimension name (T, R, U, S, K) and display the numeric score (0.00--1.00) at each axis endpoint. | The system **shall** label each axis with dimension abbreviation and numeric score. |
| REQ-QD-012 | Event-Driven | When a `ScoreUpdated` event arrives, the system shall animate the radar chart polygon from old vertices to new vertices over 300ms using GPUI animation easing. | When `ScoreUpdated` fires, the system **shall** animate the radar polygon transition over 300ms. |
| REQ-QD-013 | Ubiquitous | The system shall render the quality gate threshold as a dashed reference polygon on the radar chart, using the pass threshold value from `quality.yaml`. | The system **shall** render threshold reference polygon as a dashed overlay on the radar chart. |
| REQ-QD-014 | Event-Driven | When the user hovers over a radar axis, the system shall display a tooltip with the dimension name, score, gate status (PASS/FAIL), and top contributing metrics. | When the user hovers an axis, the system **shall** show a detail tooltip. |

### RG-QD-4 -- Quality Gate Display

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-QD-015 | Ubiquitous | The system shall display a quality gate status bar with 5 indicators (one per dimension), each showing PASS (green) or FAIL (red) based on whether the dimension score meets the configured threshold. | The system **shall** display 5 gate indicators showing PASS/FAIL per dimension. |
| REQ-QD-016 | State-Driven | While any dimension gate is in FAIL state, the system shall display an overall "GATE: FAIL" badge in the status bar with the failing dimension names listed. | While any gate fails, the system **shall** show an overall "GATE: FAIL" badge. |
| REQ-QD-017 | Event-Driven | When a gate transitions from PASS to FAIL or vice versa, the system shall emit a `GateStatusChanged` event and update the indicator within 100ms. | When a gate status changes, the system **shall** emit `GateStatusChanged` and update UI within 100ms. |

### RG-QD-5 -- Quality History

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-QD-018 | Ubiquitous | The system shall maintain a `QualityHistory` log as a ring buffer of up to 100 `Trust5Score` snapshots, each tagged with a timestamp and an optional commit hash. | The system **shall** maintain a `QualityHistory` ring buffer of up to 100 score snapshots. |
| REQ-QD-019 | Event-Driven | When a git commit is detected (via `FileChanged` or equivalent hook), the system shall snapshot the current `Trust5Score` and append it to `QualityHistory`. | When a commit is detected, the system **shall** snapshot the current score to history. |
| REQ-QD-020 | Ubiquitous | The system shall render a sparkline-style history chart for the selected dimension, plotting score over time using GPUI Canvas line drawing. | The system **shall** render sparkline history charts per dimension via GPUI Canvas. |
| REQ-QD-021 | Event-Driven | When the user clicks a radar axis, the system shall switch the history chart to show that dimension's trend line. | When the user clicks a radar axis, the system **shall** switch history to that dimension. |

### RG-QD-6 -- Dashboard Integration

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-QD-022 | Ubiquitous | The system shall integrate `RadarChartView`, `QualityHistoryView`, and `QualityGateView` into a single `QualityDashboardView` container. | The system **shall** provide a `QualityDashboardView` container for all quality views. |
| REQ-QD-023 | State-Driven | While an agent run is active, the system shall display `QualityDashboardView` as a tab within `AgentDashboardView`. While no agent run is active, the system shall display `QualityDashboardView` as a standalone surface. | While an agent is active, the system **shall** embed quality as a tab in the agent dashboard; otherwise show standalone. |
| REQ-QD-024 | Ubiquitous | The system shall preserve the existing `AgentDashboardView` 5-pane layout when the quality tab is not selected, with zero visual changes to existing V3-010 functionality. | The system **shall** not alter existing V3-010 layout when quality tab is inactive. |

### RG-QD-7 -- Dimension Detail Panel

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-QD-025 | Event-Driven | When the user selects a dimension (via radar axis click or gate indicator click), the system shall display a `DimensionDetailView` panel listing the contributing metrics for that dimension. | When the user selects a dimension, the system **shall** show a detail panel with contributing metrics. |
| REQ-QD-026 | Ubiquitous | The system shall display each contributing metric in `DimensionDetailView` as: metric name, current value, threshold (if applicable), pass/fail status. | The system **shall** list each metric with name, value, threshold, and pass/fail status. |
| REQ-QD-027 | Unwanted | The system shall not show more than 10 contributing metrics per dimension; if more exist, show the top 10 by severity/impact. | The system **shall not** show more than 10 metrics per dimension; limit to top 10. |
| REQ-QD-028 | Event-Driven | When the user clicks a metric in `DimensionDetailView`, the system shall navigate to the relevant source (file, test, or spec) in the editor surface. | When the user clicks a metric, the system **shall** navigate to the relevant source. |

---

## 6. Non-Functional Requirements (NFR)

- **NFR-QD-1**: Radar chart render latency <= 16ms p95 (60fps, consistent with V3-010 NFR-AD-2).
- **NFR-QD-2**: Score recomputation latency <= 200ms p95 from metric source update to `ScoreUpdated` event.
- **NFR-QD-3**: Animation frame rate >= 30fps during radar polygon transitions (300ms animation).
- **NFR-QD-4**: Peak memory for quality history: <= 50KB for 100 snapshots (100 * 5 * f32 + timestamps).
- **NFR-QD-5**: Gate status update latency <= 100ms from score change to indicator update (REQ-QD-017).
- **NFR-QD-6**: macOS 14+ / Ubuntu 22.04+ identical behavior (Windows out of scope per N8).
- **NFR-QD-7**: All code comments in English per CLAUDE.local.md Section 9.1 HARD rule.
- **NFR-QD-8**: terminal/panes/tabs core git diff = 0 (G5, verified by AC-QD-12).
- **NFR-QD-9**: Radar chart canvas drawing uses only GPUI primitives (no external crate dependency per N9).

---

## 7. Interfaces

### 7.1 Rust Crate Boundaries

- New module: `crates/moai-studio-ui/src/quality/` -- Quality Dashboard UI components
- New module: `crates/moai-studio-agent/src/quality/` -- Scoring engine and history domain logic
- External crate dependencies: `moai-studio-agent` (existing), GPUI (existing), `serde` (existing)

### 7.2 Shared Types with SPEC-V3-010

- `moai_studio_agent::events::AgentEvent` -- Quality events follow the existing event pattern
- `AgentDashboardView` -- Extended with tab switching capability (V3-010 container)

### 7.3 Configuration Interface

- Input: `.moai/config/sections/quality.yaml` -- Thresholds, LSP gate settings, coverage targets
- Input: `.moai/config/sections/harness.yaml` -- Harness level (affects scoring depth)
- Output: In-memory `QualityHistory` ring buffer (no disk persistence in v1.0.0)

---

## 8. USER-DECISION Gates

This SPEC assumes the following default decisions. Override requires plan.md update.

- **USER-DECISION-QD-A** (History persistence): **A1** in-memory ring buffer only (100 snapshots). Disk persistence is a follow-up SPEC.
- **USER-DECISION-QD-B** (Scoring algorithm): **B1** heuristic-based (formulas in REQ-QD-005 through REQ-QD-008). Machine learning scoring is N4 (non-goal).
- **USER-DECISION-QD-C** (Integration mode): **C1** tab within AgentDashboardView (when agent active) + standalone surface (when no agent). C2 (always standalone) rejected to maximize screen real estate reuse.

---

## 9. Acceptance Criteria

Detailed scenarios in `acceptance.md`. This section lists IDs only.

| AC ID | Area | RG | Priority |
|-------|------|-----|----------|
| AC-QD-1 | Trust5Score computed from LSP metrics | RG-QD-1, RG-QD-2 | P0 |
| AC-QD-2 | Trust5Score computed from test coverage | RG-QD-2 | P0 |
| AC-QD-3 | Radar chart renders 5 axes with correct polygon | RG-QD-3 | P0 |
| AC-QD-4 | Radar chart animates on score update (300ms) | RG-QD-3 | P1 |
| AC-QD-5 | Quality gate indicators show PASS/FAIL per dimension | RG-QD-4 | P0 |
| AC-QD-6 | Quality history records score on commit detection | RG-QD-5 | P1 |
| AC-QD-7 | History sparkline renders selected dimension trend | RG-QD-5 | P1 |
| AC-QD-8 | QualityDashboardView integrates into AgentDashboardView | RG-QD-6 | P0 |
| AC-QD-9 | DimensionDetailView shows contributing metrics | RG-QD-7 | P1 |
| AC-QD-10 | Axis hover displays tooltip with detail | RG-QD-3 | P2 |
| AC-QD-11 | Gate threshold dashed polygon renders on radar | RG-QD-3 | P2 |
| AC-QD-12 | terminal/panes/tabs core git diff = 0 | (RG-P-7 carry) | P0 |

---

## 10. Milestone Mapping

- **MS-1** (Data model + scoring engine): RG-QD-1, RG-QD-2, AC-QD-1/2
  - `Trust5Score` struct with 5 dimensions
  - `ScoringEngine` trait + default heuristic implementation
  - Metric snapshot types (LSP, test, git, security)
  - Score computation with clamping
  - Unit tests for all scoring formulas

- **MS-2** (Radar chart GPUI Canvas rendering): RG-QD-3, RG-QD-4, AC-QD-3/4/5/10/11
  - GPUI Canvas-based 5-axis radar chart drawing
  - Score labels on axes
  - Threshold dashed reference polygon
  - Axis hover tooltips
  - 300ms polygon transition animation
  - Quality gate indicator bar
  - Visual tests (screenshot comparison or rendering assertions)

- **MS-3** (Dashboard integration + history): RG-QD-5, RG-QD-6, RG-QD-7, AC-QD-6/7/8/9/12
  - `QualityHistory` ring buffer (100 snapshots)
  - Commit-triggered score snapshot
  - Sparkline history chart per dimension
  - `QualityDashboardView` container
  - Tab integration with `AgentDashboardView`
  - `DimensionDetailView` metric detail panel
  - Standalone surface mode (no active agent)
  - Integration tests verifying V3-010 coexistence

---

## 11. Dependencies and Impact

### 11.1 Upstream (prerequisites)

- **SPEC-V3-004** (Render Layer) -- GPUI Render trait pattern (mandatory)
- **SPEC-V3-010** (Agent Dashboard) -- `AgentDashboardView` container integration (mandatory)

### 11.2 Parallel

- None -- this SPEC modifies the same `dashboard_view.rs` area as V3-010.

### 11.3 Optional Integration

- **SPEC-V3-015** (if applicable) -- LSP diagnostics data source could feed the "Readable" and "Unified" scoring directly.

### 11.4 Downstream (unblocked by this SPEC)

- (Hypothetical) SPEC-V3-Future-QualityAlerts -- Banner notifications when quality gate fails
- (Hypothetical) SPEC-V3-Future-QualityExport -- Export quality history to JSON/CSV

---

## 12. Risks

| Risk | Mitigation |
|------|------------|
| R1: Scoring formula inaccuracy | `ScoringEngine` trait enables formula replacement; default formulas validated against known project states |
| R2: Radar chart performance on low-end hardware | GPUI Canvas drawing is GPU-accelerated; polygon vertex count is fixed at 5; NFR-QD-1 mandates 16ms |
| R3: Quality history ring buffer overflow | 100-snapshot cap with oldest eviction (REQ-QD-018); NFR-QD-4 limits memory to 50KB |
| R4: Integration conflict with V3-010 layout | Tab-based integration preserves existing 5-pane layout when quality tab inactive (REQ-QD-024) |
| R5: Metric source unavailability | Graceful degradation: dimensions without data sources display as "N/A" with 0.0 score and warning indicator |
| R6: Animation jank during rapid score updates | Debounce score updates to max 1 per 300ms animation cycle; latest score wins |
| R7: GPUI Canvas API limitations | Radar chart uses only basic primitives (lines, filled polygons, text); no advanced path operations needed |

---

## 13. Change Impact

### 13.1 New Files

- `crates/moai-studio-agent/src/quality/mod.rs` -- Quality domain module entry point
- `crates/moai-studio-agent/src/quality/score.rs` -- `Trust5Score` struct, `ScoringEngine` trait
- `crates/moai-studio-agent/src/quality/engine.rs` -- Default heuristic scoring implementation
- `crates/moai-studio-agent/src/quality/metrics.rs` -- Metric snapshot types
- `crates/moai-studio-agent/src/quality/history.rs` -- `QualityHistory` ring buffer
- `crates/moai-studio-ui/src/quality/mod.rs` -- Quality UI module entry point
- `crates/moai-studio-ui/src/quality/radar_chart_view.rs` -- GPUI Canvas radar chart
- `crates/moai-studio-ui/src/quality/quality_gate_view.rs` -- Gate indicator bar
- `crates/moai-studio-ui/src/quality/quality_history_view.rs` -- Sparkline history chart
- `crates/moai-studio-ui/src/quality/dimension_detail_view.rs` -- Metric detail panel
- `crates/moai-studio-ui/src/quality/quality_dashboard_view.rs` -- Container view

### 13.2 Modified Files

- `crates/moai-studio-agent/src/lib.rs` -- Add `quality` module export
- `crates/moai-studio-ui/src/agent/dashboard_view.rs` -- Add tab switching for quality panel
- `crates/moai-studio-ui/src/lib.rs` -- Add `quality` module export

### 13.3 Unchanged (HARD)

- `crates/moai-studio-terminal/**` (RG-P-7 carry)
- `crates/moai-studio-ui/src/panes/**` (RG-P-7 carry)
- `crates/moai-studio-ui/src/tabs/**` (RG-P-7 carry)

AC-QD-12 verifies via git diff.

---

## 14. Exclusions (What NOT to Build)

- Quality threshold editing GUI (N1)
- Cross-project quality comparison (N2)
- Quality score export/API (N3)
- ML-based quality prediction (N4)
- External CI/CD integration (N5)
- New design tokens (N6)
- Terminal/panes/tabs modifications (N7)
- Windows build support (N8)
- External chart rendering crate (N9)
- Per-file quality drilldown (N10)
- Disk persistence of quality history (USER-DECISION-QD-A)
- Multi-project quality aggregation

---

Version: 1.0.0 (initial draft)
Last Updated: 2026-04-29
Author: MoAI (manager-spec)
Language: ko
