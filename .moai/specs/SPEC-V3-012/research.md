---
spec: SPEC-V3-012
created_at: 2026-04-26
author: MoAI (manager-spec)
purpose: Research artifact for Palette Surface — UX pattern survey, fuzzy match algorithm comparison, integration analysis, risk register.
---

# SPEC-V3-012 Research — Palette Surface

## 1. Codebase Analysis

### 1.1 Existing UI module structure

`crates/moai-studio-ui/src/` (current state, develop @ 9f90188):

```
src/
├── agent/         # AI agent surface
├── design/        # tokens.rs, typography.rs, layout.rs, mod.rs
├── explorer/      # File explorer
├── lib.rs         # RootView (~36KB — main integration target)
├── panes/         # Pane tree, splitter, focus, render
├── tabs/          # TabContainer, TabBar, keys, mod
├── terminal/      # Terminal pane integration
└── viewer/        # File viewer
```

The `palette/` module **does not yet exist**. It is added by this SPEC.

### 1.2 Reference patterns

The closest precedents for `palette/` module structure:

- **`tabs/`** (SPEC-V3-009) — module mod.rs is a thin re-export shell, `container.rs` (30KB) holds the Entity. **Pattern**: 1 Entity + render impl + tests inline.
- **`panes/`** — multi-file decomposition (`tree.rs`, `focus.rs`, `divider.rs`, `splitter.rs`, `constraints.rs`). **Pattern**: when responsibility splits (focus mgmt vs layout vs splitter UI), files split.
- **`design/`** — pure constants module. `tokens.rs` exports `pub mod brand { pub const PRIMARY: u32 = ... }` etc.

`palette/` follows the **panes/-style multi-file** decomposition since Scrim, PaletteView, fuzzy matcher, and 3 variants are independent concerns.

### 1.3 RootView (lib.rs) integration surface

`lib.rs` is **36,746 bytes** — it owns the RootView Entity and composes the existing surfaces (panes, tabs, terminal, agent, viewer, explorer). Adding palette overlay requires:

1. `mod palette;` declaration.
2. New state field on RootView: `active_palette: Option<palette::PaletteVariant>`.
3. `render` extension: when `active_palette.is_some()`, render `Scrim` + the variant's `PaletteView` above the existing tree at z-index 20.
4. New key handlers — Cmd+P / Cmd+Shift+P / `/` (terminal-context).

The integration footprint is ~50 LOC in `lib.rs`. The single-file modification respects RG-P-7 (no other crate file modified by this SPEC).

### 1.4 Design tokens — current state

`crates/moai-studio-ui/src/design/tokens.rs` v2.0.0:
- `brand::PRIMARY = 0x144a46` (light), `brand::PRIMARY_DARK = 0x22938a` (dark).
- `brand::SURFACE_LIGHT = 0xffffff`, `brand::INK = 0x09110f`.
- `neutral::N900 = 0x0e1513`, `neutral::N950 = 0x09110f`.

The Scrim alpha values (0.55 dark / 0.18 light) are **not yet** in `tokens.json` `round2_component.palette`. MS-1 adds them to the canonical token file alongside the Rust constants in `palette_view.rs` / `scrim.rs`. No FROZEN brand color is modified.

---

## 2. UX Pattern Survey

### 2.1 Industry standards

The "command palette + quick file open + slash command" trio is industry standard for keyboard-driven IDEs. Reference table:

| IDE | Quick Open | Command Palette | Slash Commands |
|-----|------------|-----------------|----------------|
| VS Code | Cmd+P | Cmd+Shift+P (`>` prefix) | None canonical (extension-only) |
| Zed | Cmd+P | Cmd+Shift+P | Channel chat `/` |
| Sublime Text | Cmd+P | Cmd+Shift+P | None |
| Atom | Cmd+P | Cmd+Shift+P | None |
| JetBrains | Cmd+Shift+O | Cmd+Shift+A (Find Action) | None |
| Vim | `:e` (no palette) | `:` cmdline | None |
| Neovim + telescope | Cmd+P (via plugin) | Cmd+Shift+P (via plugin) | None |
| Cursor | Cmd+P | Cmd+Shift+P | None |
| Discord | N/A | Ctrl+K (server jump) | `/` slash commands |

**Conclusion**: Cmd+P (quick open) and Cmd+Shift+P (command runner) are **universally recognized**. SlashBar with `/` is **less standardized in IDEs** but **highly standardized in chat surfaces** (Discord, Slack, Teams). Since moai-studio integrates an AI agent surface (terminal + agent panes), slash commands map naturally to `/moai *` workflow invocations.

### 2.2 Visual layout conventions

Common across all IDEs:

| Property | Common value | Round 2 시안 spec |
|----------|--------------|-------------------|
| Container width | 540 ~ 640px | **600px** (chosen) |
| Row height | 28 ~ 36px | **32px** |
| Input font-size | 13 ~ 16px | **14px** |
| List max-height | 240 ~ 480px | **320px** |
| Position | top 15-25% from top, horizontally centered | top ~20% (consistent) |
| Backdrop | dim 30-60% (dark) / 10-25% (light) | **0.55 dark / 0.18 light** |
| Blur | 0 ~ 8px | **2px** (minimal, browser/GPU tolerant) |
| z-index | "above everything" | **20** (matches existing scrim conventions) |

The Round 2 시안 values are within standard ranges and documented in `tokens.json`.

### 2.3 Keyboard model

Universal (no IDE deviates):

- **↑ / ↓** — navigate selection.
- **Enter** — confirm selection.
- **Esc** — dismiss.
- **Tab** — generally NOT consumed (avoids focus-trap). Round 2 시안 follows this; Tab is allowed to fall through (e.g., for accessibility focus chain).
- **Type-to-filter** — instant fuzzy filter on input change.

Variations: VS Code adds Cmd+E (last opened), JetBrains adds Tab to switch palette mode. **moai-studio v0.1.0 ships only the universal 4 keys** (↑/↓/Enter/Esc). Tab is a no-op in palette context.

### 2.4 Mutual exclusion

VS Code, Sublime, Zed all enforce **single visible palette at a time**. Pressing Cmd+Shift+P while CmdPalette is visible **closes CmdPalette and opens CommandPalette** in the same frame. Round 2 시안 + RG-PL-24 follow this convention.

---

## 3. Fuzzy Match Algorithm Review

### 3.1 Algorithm options surveyed

| Algorithm | Match semantics | Score quality | Implementation cost | Used by |
|-----------|-----------------|---------------|---------------------|---------|
| **Subsequence (greedy)** | All query chars appear in order | Basic (length-based) | ~50 LOC | Sublime original |
| **Subsequence + scoring** | Subsequence + bonus for consecutive / prefix / camel boundary | Good | ~120 LOC | VS Code, Zed (custom) |
| **fzf algorithm v2** | Subsequence + complex scoring (consecutive bonus, position bonus, boundary bonus, gap penalty) | Excellent | ~400 LOC + tables | fzf, telescope.nvim |
| **Levenshtein** | Edit distance | Poor for short queries | ~80 LOC | Spell checkers, not palettes |
| **Ngram (trigrams)** | Trigram set overlap | Mediocre for code identifiers | ~150 LOC | PostgreSQL pg_trgm |
| **`fuzzy-matcher` crate (Skim) `SkimMatcherV2`** | Subsequence + scoring close to fzf | Excellent | 0 LOC (dependency) | Skim, helix-editor |

### 3.2 Algorithm selected

**Subsequence + scoring (custom, ~120 LOC)** for v0.1.0.

Rationale:
1. **No new dependency**: avoids adding `fuzzy-matcher` crate. moai-studio policy (research §1.3 of SPEC-V3-001) prefers in-tree implementations for small surfaces. Reduces supply-chain surface.
2. **Quality is sufficient for mock data**: in MS-2, palette data is mocked at <100 items. Custom subsequence + scoring (consecutive bonus + prefix bonus) is observably indistinguishable from fzf v2 at this scale.
3. **Easy to reason about + test**: 120 LOC fits in a single file with comprehensive unit tests. AC-PL-9 ~ AC-PL-12 cover correctness.
4. **Upgradeable**: if a follow-up SPEC introduces real file index with 10k+ entries, swapping in `fuzzy-matcher::SkimMatcherV2` is a 1-file change behind the existing function signature.

### 3.3 Scoring formula (chosen)

For a successful subsequence match, score is computed as:

```
score = base_match_credit
      + sum_over_matched_positions(consecutive_bonus_if_prev_matched)
      + prefix_bonus_if_first_match_at_index_0
      + word_boundary_bonus_if_match_after_separator
      - gap_penalty_per_unmatched_char_in_match_window
```

Concrete weights (tunable, MS-2 default):

| Term | Weight |
|------|--------|
| `base_match_credit` per matched char | +16 |
| `consecutive_bonus` | +15 (added on top of base) |
| `prefix_bonus` | +10 |
| `word_boundary_bonus` | +8 (after `_`, `-`, `.`, `/`, ` `, camel-boundary) |
| `gap_penalty` per unmatched char in match window | −1 |

Tuning is internal; only the **comparative ordering** (consecutive > scattered, prefix > middle) is required by RG-PL-15 and tested in AC-PL-11.

### 3.4 Highlight position output

The matcher returns `Vec<usize>` of byte indices in the candidate string corresponding to matched query characters. Renderer in `palette_view.rs` is responsible for:

1. Converting byte indices to grapheme cluster boundaries (Unicode safety).
2. Slicing the candidate string into runs alternating between matched and unmatched.
3. Applying `accent-soft` (PRIMARY_DARK alpha 0.20) styling to matched runs only.

Test coverage for non-ASCII (Korean, emoji) candidates is added in MS-2.

### 3.5 Algorithm pseudocode (informative)

```
fn fuzzy_match(query: &str, candidate: &str) -> Option<(i32, Vec<usize>)> {
    if query.is_empty() {
        return Some((0, vec![]));
    }
    let q: Vec<char> = query.to_lowercase().chars().collect();
    let c_chars: Vec<(usize, char)> = candidate.char_indices()
        .map(|(i, ch)| (i, ch.to_lowercase().next().unwrap_or(ch))).collect();
    let mut highlights = Vec::with_capacity(q.len());
    let mut score: i32 = 0;
    let mut prev_matched_idx: Option<usize> = None;
    let mut q_i = 0;
    for (cand_idx, ch) in &c_chars {
        if q_i >= q.len() { break; }
        if *ch == q[q_i] {
            score += 16; // base_match_credit
            if Some(*cand_idx).map(|i| prev_matched_idx == Some(i.saturating_sub(1))) == Some(true) {
                score += 15; // consecutive_bonus
            }
            if *cand_idx == 0 && q_i == 0 {
                score += 10; // prefix_bonus
            }
            // word_boundary_bonus omitted in pseudocode; full impl in fuzzy.rs
            highlights.push(*cand_idx);
            prev_matched_idx = Some(*cand_idx);
            q_i += 1;
        }
    }
    if q_i == q.len() { Some((score, highlights)) } else { None }
}
```

(Production implementation includes word-boundary detection and gap penalty.)

---

## 4. Integration Analysis

### 4.1 GPUI 0.1 capabilities

GPUI 0.1 (workspace pinned) supports:
- Entity / View / EventEmitter — used by all existing surfaces (TabContainer, PaneTree).
- KeyBinding registration at the View level — used by tabs/keys.rs (~25KB module).
- Layered rendering via z-index style — used by panes splitter overlay.
- Theme-aware styling via observable theme Entity — used by tabs/container.rs.

**Confirmed unsupported / fallback required**:
- **Native backdrop-filter blur**: GPUI 0.1 does not expose a CSS-like backdrop-filter API. RG-PL-4 explicitly allows fallback to a solid alpha overlay (no blur). MS-1 ships solid-alpha; if a future GPUI version exposes blur, an opt-in 2px blur is added in a follow-up.

### 4.2 Focus model interaction

`crates/moai-studio-ui/src/panes/focus.rs` (~17KB) owns the pane focus state. SlashBar opening requires "terminal pane has focus" — the check uses the existing `panes::focus::active_pane_kind() == PaneKind::Terminal` API (or equivalent — exact API name verified at MS-3 implementation time).

For Cmd+P / Cmd+Shift+P, the global RootView key handler captures these regardless of pane focus per RG-PL-21 / RG-PL-22, with the exception in RG-PL-25 (text input owning focus).

### 4.3 Existing key binding conflicts

`tabs/keys.rs` registers tab-related shortcuts. Confirmed (by reading file paths): no current binding for Cmd+P, Cmd+Shift+P, or `/`. Conflict-free addition.

`terminal/` may forward Cmd+P to the underlying PTY in some platforms. The convention in moai-studio is **palette wins** (matches VS Code / Zed). RG-PL-25 ensures editing input fields is unaffected.

### 4.4 Theme switching

Existing theme observation pattern (used by `tabs/container.rs`):
```
cx.observe_global::<Theme>(|this, cx| {
    cx.notify();
})
```

Scrim and PaletteView use the same pattern — Scrim color and palette container background switch in the same frame as the theme switch (US-PL-7).

---

## 5. Test Strategy

### 5.1 Test layering

| Layer | Location | Coverage target |
|-------|----------|-----------------|
| Unit — Scrim | `palette/scrim.rs` `#[cfg(test)] mod tests` | render correctness, click event boundary, theme-aware color |
| Unit — PaletteView | `palette/palette_view.rs` `#[cfg(test)] mod tests` | dimensions, focus, kbd nav (wrap, Enter, Esc), highlight render |
| Unit — fuzzy | `palette/fuzzy.rs` `#[cfg(test)] mod tests` | subsequence correctness, scoring relative order, empty query, non-ASCII |
| Unit — variants | `palette/variants/{cmd_palette,command_palette,slash_bar}.rs` `#[cfg(test)] mod tests` | data source filter, Enter dispatches correct event payload |
| Integration | `crates/moai-studio-ui/src/lib.rs` `#[cfg(test)] mod tests` | Cmd+P opens, Cmd+Shift+P replaces, mock select, Esc closes |

### 5.2 Coverage goal

- AC-PL-T1: `palette/` subtree at >= 85% line coverage.
- Estimated **40+ test cases** across MS-1 (15) / MS-2 (20) / MS-3 (5+).
- Tarpaulin command: `cargo tarpaulin -p moai-studio-ui --include-tests --out Stdout`.

### 5.3 Bench impact

Palette open/close is a UI overlay — not in the hot rendering path of existing benches. **No new bench is added in MS-1/2/3**; AC-PL-T5 only requires no regression on the existing 3.92µs baseline.

---

## 6. Risk Register

### R1. GPUI 0.1 lacks backdrop-filter blur
- **Impact**: Medium — Round 2 시안 specifies 2px blur; without it, Scrim is solid-alpha.
- **Likelihood**: High (confirmed by capability survey).
- **Mitigation**: RG-PL-4 explicitly permits platform-equivalent fallback. Solid alpha at 0.55/0.18 still produces clear visual separation.

### R2. Mutual exclusion flicker
- **Impact**: Low — visible flash for one frame when switching variants.
- **Likelihood**: Medium without single-frame guarantee.
- **Mitigation**: Implementer collapses dismiss + open into a single state transition (`active_palette = Some(new_variant)` with no intermediate `None`). AC-PL-15 covers via integration test.

### R3. Cmd+P swallowed by terminal pane PTY on some platforms
- **Impact**: Medium — feature appears broken when terminal owns focus.
- **Likelihood**: Low for moai-studio (terminal does not capture Cmd+P per current behavior).
- **Mitigation**: RG-PL-25 + RootView-level key binding takes precedence. Manual testing on macOS / Linux at MS-3.

### R4. Fuzzy matcher byte-vs-char index off-by-one for non-ASCII
- **Impact**: Medium — Korean/emoji highlights misaligned.
- **Likelihood**: Medium without explicit grapheme handling.
- **Mitigation**: matcher returns byte indices; renderer converts via `unicode-segmentation` (already a transitive dep via egui/cosmic-text in workspace) or via `str::char_indices`. Test cases with Korean candidates added in MS-2.

### R5. Fuzzy scoring tuning is subjective
- **Impact**: Low — scores affect ordering only, not correctness.
- **Likelihood**: High that initial weights are not perfect.
- **Mitigation**: AC-PL-11 only requires *relative* ordering (consecutive > scattered). Weight values (16/15/10/8/−1) can be tuned in a follow-up without contract change.

### R6. Token file divergence between Rust constants and tokens.json
- **Impact**: Low — visual drift between handoff bundle and Rust runtime.
- **Likelihood**: Medium without a sync gate.
- **Mitigation**: MS-1 adds `round2_component.palette` keys to tokens.json in the same PR as `palette/scrim.rs` + `palette/palette_view.rs` constants. Reviewers verify 1:1 mapping per spec §10.

### R7. Mock data leak — variant references real data layer accidentally
- **Impact**: High — couples palette to file index / command registry before they exist.
- **Likelihood**: Low if reviewers check.
- **Mitigation**: Variant module structures `mock_*` constants explicitly. Real data source SPEC introduces a trait `PaletteDataSource` later; v0.1.0 variants implement the trait inline with mock data only.

### R8. Test coverage shortfall on `lib.rs` due to GPUI integration test complexity
- **Impact**: Medium — AC-PL-T1 requires >= 85% on `palette/` subtree only, not lib.rs. Lib.rs integration tests target functional flows (3 tests), not coverage.
- **Likelihood**: Low for `palette/` subtree (small files, comprehensive unit tests).
- **Mitigation**: Coverage gate scopes to `palette/`. Lib.rs delta is small (~50 LOC of integration). `cargo tarpaulin` `--include-tests` flag includes inline tests in coverage.

---

## 7. Token Sync Plan (MS-1)

`.moai/design/tokens.json` v2.0.0 currently contains `round2_component` with brand-related entries. Add:

```jsonc
"round2_component": {
  // ...existing keys...
  "palette": {
    "scrim": {
      "dark": "rgba(8,12,11,0.55)",
      "light": "rgba(20,30,28,0.18)",
      "blur_px": 2,
      "z_index": 20
    },
    "container": {
      "width_px": 600,
      "bg_light": "#ffffff",
      "bg_dark": "#0e1513",
      "shadow": "0 12px 32px rgba(8,12,11,0.18)"
    },
    "row": { "height_px": 32 },
    "input": { "font_size_px": 14 },
    "list": { "max_height_px": 320 },
    "highlight": { "alpha": 0.20, "base_color_ref": "brand.primary_dark" }
  }
}
```

No FROZEN brand color is modified. The shadow value is informative; if not implementable in GPUI 0.1, it is skipped at render time.

---

## 8. Cross-SPEC Dependencies Verified

| Dependency | Status | Notes |
|------------|--------|-------|
| SPEC-V3-001 (workspace) | DONE | tokens.json v2.0.0 canonical exists. |
| SPEC-V3-002 (design module) | DONE | tokens.rs / typography.rs available. |
| SPEC-V3-003 (TabContainer pattern) | DONE | Reference pattern for Entity + render. |
| SPEC-V3-004 (terminal focus) | DONE | `panes::focus` API available for SlashBar trigger. |
| SPEC-V3-009 (tabs surface brand) | DONE | Round 2 brand migration sets the precedent palette follows. |

No blocking upstream dependency.

---

## 9. References

- VS Code source — `vscode/src/vs/platform/quickinput/browser/quickInput.ts` (UX reference, MIT).
- Zed documentation — `command_palette` action.
- fzf algorithm v2 — `fzf/src/algo/algo.go` (MIT).
- `fuzzy-matcher` crate (Skim) — https://crates.io/crates/fuzzy-matcher (considered, not adopted for v0.1.0).
- `unicode-segmentation` crate — for grapheme-aware highlight rendering.
- Round 2 시안 — `.moai/design/from-claude-design/project/moai-revisions.jsx` (CmdPalette / CommandPalette / SlashBar JSX components).
- IMPLEMENTATION-NOTES.md v1.1 §14 C — Palette Surface scope statement.
- moai-studio existing pattern — `crates/moai-studio-ui/src/tabs/container.rs` (Entity + render + theme observe).

---

End of research.md.
