# SPEC-V3-LINK-001 Progress

**Started**: 2026-04-27
**Branch**: feature/SPEC-V3-LINK-001 (squash merged into main)
**SPEC status**: complete (all ACs including click wiring)
**Completion date**: 2026-04-28 (final AC-LK-4/5 wiring)

## Implementation Timeline

- 2026-04-27 SPEC v0.1.0 created — initial SPEC for B-2 + B-3 + OSC 8
- 2026-04-27 Implementation: Smart Link Detection (B-2/B-3) + audit — PR #55 (`8391689`)
- 2026-04-28 AC-LK-4/5 GPUI click wiring — commit `3776c40` (TerminalSurface on_mouse_down → handle_click → resolve_click → TerminalClickEvent) + RootView wire_terminal_click_callback

## Task Status

- [x] AC-LK-1: File path detection (regex-based `src/main.rs:42:10` pattern)
- [x] AC-LK-2: URL auto-detection (http/https schemes)
- [x] AC-LK-3: OSC 8 hyperlink precedence (OSC 8 links take priority over regex matches)
- [x] AC-LK-6: O(n) performance via OnceLock-based compiled regex
- [x] AC-LK-7: 15 unit tests (>= 12 required)
- [x] OpenCodeViewer / OpenUrl structs defined
- [x] AC-LK-4: GPUI click wiring for file paths — TerminalSurface.handle_click → resolve_click → TerminalClickEvent::OpenFile → RootView.wire_terminal_click_callback
- [x] AC-LK-5: GPUI click wiring for URLs — TerminalSurface.handle_click → resolve_click → cx.open_url() + TerminalClickEvent::OpenUrl

## Key Files Changed

- `crates/moai-studio-terminal/Cargo.toml`: regex crate promoted to direct dependency
- `crates/moai-studio-terminal/src/lib.rs`: link module registration
- `crates/moai-studio-terminal/src/link.rs`: detect_links() + detect_links_with_osc8() + LinkSpan model + 15 tests
- `crates/moai-studio-ui/src/terminal/mod.rs`: TerminalSurface.handle_click + on_mouse_down wiring + EventEmitter<TerminalClickEvent> + ClickAction dispatch (AC-LK-4/5)
- `crates/moai-studio-ui/src/lib.rs`: RootView.wire_terminal_click_callback (subscribe to TerminalClickEvent, dispatch OpenFile/OpenUrl/OpenSpec) + v0.1.1 hotfix + v0.1.2 audit fixes
- `Cargo.toml`: Dependency version adjustment
- `.moai/specs/RELEASE-V0.1.1/checklist.md`: UX audit checklist
- `.moai/specs/RELEASE-V0.1.1/feature-audit.md`: Feature audit matrix
- `.moai/specs/RELEASE-V0.1.2/feature-audit.md`: v0.1.2 feature audit

## Test Coverage

- 15 unit tests in `crates/moai-studio-terminal/src/link.rs` (cargo test -p moai-studio-terminal link)
- Tests cover: file path patterns, URL patterns, OSC 8 precedence, overlap resolution, O(n) performance validation
- clippy -D warnings PASS, cargo fmt --check PASS

## Known Limitations

- B-4 SPEC-ID pattern detection, B-5 @MX tag detection, B-6 Mermaid, B-7 Hover preview: all out of scope for this SPEC
