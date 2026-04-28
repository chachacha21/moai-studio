# SPEC-V3-007 Progress

**Started**: 2026-04-28
**Branch**: main (direct commit)
**SPEC status**: complete
**Completion date**: 2026-04-28

## Implementation Timeline

- 2026-04-28 `1b8c88e` PR #61: feat(web): SPEC-V3-007 MS-1 — WebViewSurface + wry backend skeleton
- 2026-04-28 `a65acb7`: feat(ui): SPEC-V3-007 MS-1 — WebView Surface + wry 백엔드 기반 구조 (merge conflict resolved)
- 2026-04-28 `6bde34a`: feat(web): SPEC-V3-007 MS-2 — NavigationHistory + URL validation + DevTools toggle
- 2026-04-28 `38227cd`: feat(web): SPEC-V3-007 MS-3 — JS↔Rust bridge module (bridge.rs)
- 2026-04-28 `b5433ba`: feat(web): SPEC-V3-007 MS-3 — URL auto-detection from PTY output (url_detector.rs)
- 2026-04-28 `ea66777`: feat(web): SPEC-V3-007 MS-3 — WebConfig module (config.rs)
- 2026-04-28 `c5d0cb8`: feat(web): SPEC-V3-007 MS-3 — WebViewState and crash recovery (surface.rs)
- 2026-04-28 `5541672`: fix(web): UrlDetectionDebouncer 시간 기반 dedup 수정

## Milestone Status

- [x] MS-1: WebViewSurface trait + WryBackend struct + #[cfg(feature = "web")] gate — committed `a65acb7`
- [x] MS-2: URL navigation + History + DevTools + sandbox — committed `6bde34a`
- [x] MS-3: JS bridge + Auto-detect + Persistence integration — committed `38227cd`..`c5d0cb8`

## Key Files Changed

### New Files

- `crates/moai-studio-ui/src/web/mod.rs`: WebView trait + module re-exports
- `crates/moai-studio-ui/src/web/surface.rs`: WebSurface GPUI component
- `crates/moai-studio-ui/src/web/wry_backend.rs`: WryBackend wry abstraction
- `crates/moai-studio-ui/src/web/history.rs`: NavigationHistory (MS-2)
- `crates/moai-studio-ui/src/web/url.rs`: URL validation (MS-2)
- `crates/moai-studio-ui/src/web/bridge.rs`: BridgeRouter + BridgeMessage (MS-3)
- `crates/moai-studio-ui/src/web/url_detector.rs`: detect_local_urls + Debouncer (MS-3)
- `crates/moai-studio-ui/src/web/config.rs`: WebConfig struct (MS-3)
- `crates/moai-studio-ui/examples/wry_spike.rs`: wry integration spike

### Modified Files

- `crates/moai-studio-ui/Cargo.toml`: wry dependency (feature-gated)
- `crates/moai-studio-ui/src/lib.rs`: #[cfg(feature = "web")] pub mod web
- `crates/moai-studio-ui/src/viewer/mod.rs`: LeafKind::Web variant

## USER-DECISION Resolutions

- Spike 0 (wry + GPUI handshake): PASSED — wry_spike.rs confirms wry builds alongside GPUI
- webview-backend-choice: wry selected (cross-platform, Rust-native)
- linux-webkit2gtk-version: deferred to MS-2 (Linux CI)
- devtools-activation-policy: deferred to MS-2
- webview-sandbox-profile: deferred to MS-3

## Notes

- Dependencies: SPEC-V3-004 (render layer) — DONE
- wry is feature-gated: cargo build -p moai-studio-ui --features web
- GPUI-only builds unaffected (no wry dependency)
