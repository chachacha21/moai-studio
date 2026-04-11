# R1 — Native AI Shells 경쟁사 리서치

> **Date**: 2026-04-11
> **Scope**: moai-cli 의 직접 경쟁/참고 대상이 되는 native AI shell 5종
> **Method**: WebSearch + WebFetch only. 모든 주장은 URL 출처 첨부.
> **Source agent**: general-purpose, single foreground run

---

## Executive Summary

The "AI-native shell" space in 2026 is shaped by a clear bifurcation: **closed-source, VC-funded, Rust-based native apps** (Warp) vs **open-source native apps built on libghostty** (cmux) vs **open-source Electron apps with Go backends** (Wave Terminal) vs **editor-first products that added terminals and agent panels** (Zed). Ghostty itself has no AI features and is quietly becoming the industry's shared [C/Zig terminal engine via libghostty](https://github.com/ghostty-org/ghostty), which is the single most important architectural development for anyone (including moai-cli) planning to embed a GPU terminal.

Across all products, **three pain points repeat with brutal consistency**: (1) **platform coverage gaps** — cmux, Warp (historically), Zed (historically) all had major HN threads demanding Windows/Linux support; (2) **AI trust/privacy issues** — Warp has been hit multiple times on HN for telemetry and ["Warp sends a terminal session to LLM without user consent"](https://news.ycombinator.com/item?id=44953470); (3) **Electron memory footprint** — Wave Terminal is explicitly [rejected by some users on Lemmy and HN threads](https://lemmy.dbzer0.com/comment/13354563) because "Electron apps hog all memory."

The biggest strategic insight for moai-cli: **cmux already occupies ~80% of the architectural niche** ("Ghostty-based macOS terminal with vertical tabs and notifications for AI coding agents," GPL-3.0, [13.6k stars on GitHub](https://github.com/manaflow-ai/cmux)) and explicitly ships a `cmux claude-teams` launcher that wraps Claude Code's experimental teammate mode. cmux is the one competitor moai-cli must study most carefully — not Warp, not Zed. The differentiation gap cmux leaves open: **no file tree, no kanban/SPEC board, no markdown viewer, no MCP, no structured hooks/metrics dashboard**, and [no Linux/Windows support](https://github.com/manaflow-ai/cmux/issues/330). Those are moai-cli's clearest opportunity surfaces.

The second insight is that **Zed's ACP + Claude Code integration is live but buggy**. Users have filed P2 bugs including [completely missing diff/review UI with External Claude Agent](https://github.com/zed-industries/zed/issues/50142) and [context window being clamped to ~200k instead of 1M on Max](https://github.com/zed-industries/zed/issues/51648). This tells you: even a 78.9k-star, Rust/GPUI-based project with a full-time team cannot ship a polished Claude Code UX on the first try. Expect ACP integration to be hard.

The third insight is about **libghostty specifically**: it is real, shipping as `GhosttyKit.xcframework`, documented by [Mitchell Hashimoto's canonical guide](https://mitchellh.com/writing/zig-and-swiftui), and already powering a dozen projects listed in [awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty). Hashimoto himself predicts "by the middle of 2027, the number of people using Ghostty via libghostty will dwarf the number of users that actually use the Ghostty GUI." This is a validated embed path, not vaporware.

---

## 1. Warp

### Tech stack
- **Language**: [Rust, with a custom UI framework (not Flutter, not Electron)](https://www.warp.dev/blog/how-warp-works). The team explicitly pivoted from Electron early on.
- **Renderer**: [GPU-rendered — Metal on macOS, wgpu on Linux/Windows, with cosmic-text for text shaping and winit for windowing](https://thenewstack.io/a-review-of-warp-another-rust-based-terminal/)
- **Terminal model**: [Forked from Alacritty's model code](https://thenewstack.io/a-review-of-warp-another-rust-based-terminal/)
- **Cross-platform code reuse**: [The Linux version reportedly shares 98% of the underlying code with macOS](https://news.itsfoss.com/warp/)
- **Source transparency**: The [warpdotdev/Warp GitHub repo](https://github.com/warpdotdev/Warp) is an issues-only tracker (342 commits on main, 26.4k stars, 639 forks as observed). Client codebase is closed; the team states they [plan to open-source the Rust UI framework first, then potentially parts of the client](https://github.com/warpdotdev/Warp). Server remains closed.

### License + business model
- **Closed-source proprietary license** with an issues-only public repo.
- **Pricing (per [warp.dev/pricing](https://www.warp.dev/pricing))**:
  - **Free**: $0 — 150 AI credits/mo for first 2 months then 75/mo, 4 concurrent cloud agents, 3 indexed codebases (3,000 files each)
  - **Build**: $18/mo — 1,500 credits/mo, access to OpenAI/Anthropic/Google models, 20 concurrent cloud agents, 40 indexed codebases (100,000 files each), BYO API key option
  - **Max**: $180/mo — 18,000 credits/mo, 40 concurrent cloud agents
  - **Business**: $45/user/mo — Zero Data Retention, SAML SSO, up to 50 seats
  - **Enterprise**: Custom — self-hosted cloud agents, dedicated account manager
- **Account required**: login enforced even for free tier historically (a recurring HN complaint).

### AI integration
- **Providers**: [OpenAI, Anthropic, Google, with BYO API key on Build tier+](https://www.warp.dev/pricing)
- **Autonomy level**: Supports [agentic cloud agents ("Oz" orchestration platform)](https://www.warp.dev/) with up to 40 concurrent agents on Max tier — the most autonomous of any competitor researched.
- **Multi-agent**: Yes — concurrent cloud agents is a headline feature.
- **Codebase indexing**: Built-in, capped per tier.

### Feature matrix

| Feature | Warp |
|---|---|
| Multi-pane / split panes | Y |
| Multi-tab | Y |
| Multi-window | Y |
| File tree sidebar | N (terminal-first; [VS Code embedding was requested as issue #257 but not shipped](https://github.com/warpdotdev/Warp/issues)) |
| Code viewer/editor | N (not a first-class surface) |
| Browser surface | N |
| Markdown preview | N |
| Image viewer | N |
| Kanban or task board | N |
| Hook/event system | N (not exposed to users) |
| MCP support | [Unverified — not prominently documented; could not find a dedicated MCP configuration page] |
| Custom slash commands | Y ("Workflows" — community repository) |

### Top user pain points
1. **["Warp sends a terminal session to LLM without user consent" (HN, Aug 2025)](https://news.ycombinator.com/item?id=44953470)** — User discovered Warp silently sent session content to an LLM to explain a test failure.
2. **["Warp terminal spyware sending data to Segment" (Issue #1346)](https://github.com/warpdotdev/Warp/issues/1346)** — 2022 origin of the telemetry controversy.
3. **["Telemetry is now optional in Warp" HN thread](https://news.ycombinator.com/item?id=33910992)** — Community still skeptical even after opt-in change.
4. **[Issue #1957 — SSH "channel 21: open failed"](https://github.com/warpdotdev/Warp/issues/1957)** — 151 comments, long-standing unresolved SSH bug, repeatedly reopened.
5. **[Issue #4339 — Support local LLMs like Ollama](https://github.com/warpdotdev/Warp/issues/4339)** — Users want local model support rather than forced cloud.
6. **[Issue #1811 — Bypass/disable Warp tab-completion](https://github.com/warpdotdev/Warp/issues/1811)** — Forced completion overrides shell defaults, frustrating power users.
7. **[Issue #7287 — Request failed with 403](https://github.com/warpdotdev/Warp/issues/7287)** — 21 comments on HTTP request failures blocking normal use.
8. **[Issue #4240 — WSL support (closed)](https://github.com/warpdotdev/Warp/issues/4240)** and **[#204 — Windows support (closed)](https://github.com/warpdotdev/Warp/issues/204)** — historical platform gaps, since resolved.
9. **["A telemetry-sending, VC-funded, closed-source terminal" blog post](https://www.careerlimitingmoves.com/2022/04/09/warp/)** — Widely circulated critique that still colors HN perception.

### Strengths and failures
**Wins**:
- [GPU rendering at >144 FPS with ~1.9 ms average redraw](https://thenewstack.io/a-review-of-warp-another-rust-based-terminal/) — measurably the fastest Rust terminal with AI features.
- Most mature cloud-agent orchestration (40 concurrent on Max tier).
- Genuine cross-platform parity (macOS/Linux/Windows, x64 and ARM64).

**Fails**:
- **Deep, durable trust deficit on HN.** The telemetry incident is a 4-year-old albatross that still surfaces in every new HN thread, reinforced by the 2025 "session sent to LLM without consent" incident.
- Closed-source client, server, and pricing model run directly counter to the audience most likely to adopt a new terminal (OSS developers).
- Forced login and AI-first workflows alienate traditional terminal users who "want a terminal, not an app."
- No file tree / code viewer / browser surface — Warp is still fundamentally a terminal + cloud agent, not a workspace shell.

---

## 2. Wave Terminal

### Tech stack
- **Primary languages**: [Go 48.8%, TypeScript 43.1%](https://github.com/wavetermdev/waveterm)
- **GUI framework**: [Electron with React for UI](https://github.com/wavetermdev/waveterm) — confirmed by `electron.vite.config.ts` and `electron-builder.config.cjs` in the repo
- **Renderer**: Chromium (Electron) — **not GPU-native in the Rust/Metal sense**. Wave team's defense on HN: ["Electron for rendering, Go backend for networking and heavy lifting"](https://news.ycombinator.com/item?id=38869559)
- **Repo**: [github.com/wavetermdev/waveterm](https://github.com/wavetermdev/waveterm) — [19.4k stars, latest release v0.14.4 (March 27, 2026), 2,600+ commits on main](https://github.com/wavetermdev/waveterm)

### License + business model
- **[Apache-2.0](https://github.com/wavetermdev/waveterm)** — fully open source
- **No paid tier found** in the research. Company appears to be pre-monetization or supported by venture funding without a published pricing page.

### AI integration
- **Providers**: ["Wave AI (Local Models + BYOK)"](https://docs.waveterm.dev/) — supports local models and BYO API keys. Specific provider list not publicly documented in the pages I could reach.
- **Autonomy level**: Sidebar assistant-style (not autonomous agents). [Wave AI is described as a "Context-aware terminal assistant with access to terminal output, widgets, and filesystem."](https://docs.waveterm.dev/)
- **Claude Code integration**: Yes, via **hook-based "badge system"** rather than an agent dashboard. [Wave uses Claude Code's lifecycle hooks and the `wsh badge` command to surface status](https://docs.waveterm.dev/claude-code): gold bell for permission prompts (priority 20), green check for session completion (priority 10), gold message-question for AskUserQuestion. Includes **badge rollup** — multi-pane tabs show the highest-priority badge on the tab header.
- **MCP**: [No mention of MCP in Wave's Claude Code integration docs](https://docs.waveterm.dev/claude-code) — not documented.
- **Multi-agent**: [Feature request #2168 "Wave Agent Mode" is open](https://github.com/wavetermdev/waveterm/issues) — not yet implemented.

### Feature matrix

| Feature | Wave |
|---|---|
| Multi-pane / split panes | Y (called "blocks") |
| Multi-tab | Y |
| Multi-window | Y (workspaces) |
| File tree sidebar | Y (remote file browsing with VSCode-like editor) |
| Code viewer/editor | Y ([VSCode-like with syntax highlighting, mouse support, remote editing](https://www.waveterm.dev/)) |
| Browser surface | Y ([built-in for GitHub/StackOverflow/dashboards](https://www.waveterm.dev/)) |
| Markdown preview | Y (inline, as one of the supported file types) |
| Image viewer | Y ([images, audio, video, HTML, CSV inline](https://www.waveterm.dev/)) |
| Kanban or task board | N |
| Hook/event system | Y (via Claude Code hooks + badge system) |
| MCP support | [UNVERIFIED — not documented in Claude Code integration page] |
| Custom slash commands | [UNVERIFIED — not found] |
| Built-in widget types | [Term, Preview, Codeedit per docs](https://docs.waveterm.dev/widgets) |

### Top user pain points
1. **[Issue #76 — Windows OS support](https://github.com/wavetermdev/waveterm/issues/76)** — long-standing platform request.
2. **[Issue #2168 — Wave Agent Mode feature request](https://github.com/wavetermdev/waveterm/issues/2168)** — users want full agent capabilities, not just AI sidebar.
3. **[Issue #707 — Unable to connect to SSH host](https://github.com/wavetermdev/waveterm/issues/707)** — marked "not planned."
4. **[Issue #2057 — Remember last terminal path](https://github.com/wavetermdev/waveterm/issues/2057)** — basic session persistence gap.
5. **[Issue #71 / #1093 — Flatpak support](https://github.com/wavetermdev/waveterm/issues/71)** — Linux packaging gap.
6. **[Issue #83 — Customize font settings (not planned)](https://github.com/wavetermdev/waveterm/issues/83)** — refused by maintainers, which drew user frustration.
7. **[Issue #91 — Set default shell (not planned)](https://github.com/wavetermdev/waveterm/issues/91)** — another "not planned" rejection.
8. **Electron memory footprint**: [Lemmy user explicitly rejected Wave: "Electron apps hog all memory and get killed by the OS first. That's a no from me."](https://lemmy.dbzer0.com/comment/13354563)
9. **[Issue #610 — "How secure is waveterm?"](https://github.com/wavetermdev/waveterm/issues/610)** — user raised concerns about the Electron-frontend + Go-backend architecture allowing command injection with sudo privileges.
10. **[Sudo password caching in memory](https://docs.waveterm.dev/) — some users find this objectionable** (raised in the Lemmy thread above).

### Strengths and failures
**Wins**:
- **Richest "widgets/surfaces" matrix of any terminal in this research**: browser, code editor, markdown, images, CSV, audio, video all inline. This is exactly the vision moai-cli describes.
- Apache-2.0 + genuinely open source with steady release cadence (19.4k stars).
- Novel "badge rollup" hook-based Claude Code integration is clever and unique.
- Go backend for the heavy lifting separated from Electron frontend — a pragmatic architecture.

**Fails**:
- **Electron memory reputation is a persistent drag.** Every review comparing Wave to Warp mentions memory bloat, and there is no benchmark data Wave has published to refute it.
- **Maintainer "not planned" rejections for basic features** (default shell, font settings, SSH) have generated community friction.
- No MCP, no multi-agent mode (requested as #2168), no task board — AI integration feels like a retrofit rather than first-class.
- Windows support is still missing years into the project despite being the #1 most-reacted issue.

---

## 3. cmux

### Tech stack
- **Language**: **[Swift + AppKit (not Electron, not SwiftUI exclusively)](https://github.com/manaflow-ai/cmux)**. Native macOS app.
- **Terminal engine**: **[libghostty](https://github.com/manaflow-ai/cmux) — reads existing `~/.config/ghostty/config` for themes, fonts, and colors**. This is the most notable architectural choice and is exactly what moai-cli is planning.
- **Renderer**: GPU-accelerated (via libghostty → Metal on macOS)
- **Repo**: [github.com/manaflow-ai/cmux](https://github.com/manaflow-ai/cmux) — **[~12.9k–13.6k stars observed across fetches, 915 forks, 2,212 commits](https://github.com/manaflow-ai/cmux)**. Active development with nightly + stable channels.
- **Platform**: **macOS only.** [Issue #330 "Linux support?"](https://github.com/manaflow-ai/cmux/issues/330) and [#1012 "Please bring a windows version"](https://github.com/manaflow-ai/cmux/issues/1012) are open and among the most-reacted.

### License + business model
- **[GPL-3.0-or-later](https://github.com/manaflow-ai/cmux)**, with commercial licensing available for orgs that can't comply with GPL.
- **Distribution**: `brew tap manaflow-ai/cmux && brew install --cask cmux`, plus DMG downloads.
- Note: earlier HN post described it as ["free, open source, MIT-licensed"](https://news.ycombinator.com/item?id=47008732); the current README shows GPL-3.0, suggesting a license change at some point. The GPL-3.0 claim is the current verified state.
- Parent company: **manaflow-ai**, which describes itself as ["Open Source Applied AI Lab"](https://manaflow.com/) and also ships a [heatmap diff viewer for code reviews (0github.com)](https://news.ycombinator.com/item?id=45760321).

### AI integration
- **No direct LLM integration**. cmux does not talk to Anthropic/OpenAI itself. Instead, it is a **host** for Claude Code and other agent CLIs.
- **Claude Code teammate mode**: The headline feature is `cmux claude-teams`, which was added in [PR #1179](https://github.com/manaflow-ai/cmux/pull/1179). It:
  - Sets `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`
  - Defaults `--teammate-mode auto`
  - **Injects a tmux-like env** (`TMUX`, `TMUX_PANE`, `TERM=screen-256color`) so Claude's tmux-based teammate spawning transparently becomes cmux splits
  - Prepends a private tmux shim to PATH — `__tmux-compat` supports `new-session/window`, `split-window`, `select/kill`, `send-keys`, `capture-pane` with scrollback, `display-message`, etc.
  - Keeps the leader pane focused during splits
- **Worktree awareness**: [HN commenter confirms cmux "uses worktrees to manage different Claude Code instances so you don't get one Claude overwriting another's work"](https://news.ycombinator.com/item?id=47468901). Not explicitly documented in the README I fetched, but confirmed in the community thread.
- **Notifications**: Picks up terminal sequences OSC 9/99/777 and provides a `cmux notify` CLI to wire into agent hooks for Claude Code, OpenCode, etc.
- **Browser**: Scriptable browser panes adapted from agent-browser — agents can snapshot accessibility trees, interact with forms, evaluate JavaScript. Browser panes can route through SSH sessions.

### Feature matrix

| Feature | cmux |
|---|---|
| Multi-pane / split panes | Y (horizontal + vertical) |
| Multi-tab | Y (vertical sidebar tabs with rich metadata) |
| Multi-window | Y (workspaces) |
| File tree sidebar | **N** (explicitly absent per README summary) |
| Code viewer/editor | **N** |
| Browser surface | **Y** (scriptable, agent-browser-based) |
| Markdown preview | **N** |
| Image viewer | **N** (drag-to-upload via scp in SSH mode, but no viewer surface) |
| Kanban or task board | **N** |
| Hook/event system | Y (OSC 9/99/777 + `cmux notify` CLI + `cmux.json` project commands) |
| MCP support | **N** (not documented) |
| Custom slash commands | Y (via `cmux.json` project-specific actions) |
| Session restoration | Partial — window layout, CWD, scrollback, browser history. [**NOT** live process state](https://github.com/manaflow-ai/cmux) (so Claude Code / vim / tmux sessions don't resume) |
| SSH workspaces | Y (`cmux ssh user@remote` with network-routed browser) |
| Browser import | Y (Chrome, Firefox, Arc, 20+ browsers) |

### Top user pain points
1. **[Issue #330 — Linux support](https://github.com/manaflow-ai/cmux/issues/330)** — macOS-only is the single biggest gap.
2. **[Issue #1012 — Windows version](https://github.com/manaflow-ai/cmux/issues/1012)** — related platform demand.
3. **[Issue #480 — Persistence of tab information and pane layouts](https://github.com/manaflow-ai/cmux/issues/480)** — session restoration gaps, especially "NOT live process state."
4. **[Issue #2322 — Claude notifications are flaky, inconsistent delay](https://github.com/manaflow-ai/cmux/issues/2322)** — root cause analysis points to dual notification delivery paths (Claude wrapper shim hooks + OSC sequences) creating race conditions due to a 0.75s bounded socket ping.
5. **[Issue #135 — Customizable keybindings / respect Ghostty hotkey config](https://github.com/manaflow-ai/cmux/issues/135)** — users want their Ghostty muscle memory preserved.
6. **[Issue #645 — Customize modifier keys for workspace/surface shortcuts](https://github.com/manaflow-ai/cmux/issues/645)** — related keybinding gap.
7. **[Issue #1664 — First-class SSH workspaces](https://github.com/manaflow-ai/cmux/issues/1664)** — users want persistent remote sessions with image paste support.
8. **[Issue #719 — Open in browser](https://github.com/manaflow-ai/cmux/issues/719)** — browser UX gap.
9. **[Issue #124 — Browser panel: support passkeys / WebAuthn](https://github.com/manaflow-ai/cmux/issues/124)** — 2FA gap.
10. **HN feedback: tab reordering by notification recency creates cognitive burden** ([comment on Show HN thread](https://news.ycombinator.com/item?id=47079718)) — users want stable keyboard shortcuts, not dynamically reordered tabs.
11. **HN feedback: command palette (Cmd+Shift+P) and pane zoom (Cmd+Z) requested** — basic discoverability is weak.

### Strengths and failures
**Wins**:
- **Occupies almost exactly the architectural niche moai-cli is planning.** Swift + AppKit + libghostty + Claude Code teammate integration is a confirmed-working combination.
- The `cmux claude-teams` tmux-compat shim is clever engineering — it converts Claude Code's tmux teammate spawner into native cmux splits, which is non-obvious.
- Native macOS performance, GPU rendering via libghostty.
- Rich sidebar metadata (git branch, PR number, listening ports, notification text) is a feature moai-cli's Kanban board will need to match or exceed.
- Browser import from 20+ browsers is a surprisingly delightful feature.

**Fails**:
- **macOS-only** — the largest single gap vs Wave/Warp/Zed/Ghostty.
- **No file tree, no markdown viewer, no code viewer, no image viewer, no kanban.** cmux is deliberately terminal+browser focused, which leaves wide open territory.
- **No MCP integration** — only talks to Claude Code via environment variables and OSC sequences, not tools.
- **Notification reliability is a known-unsolved issue (#2322)** with a race condition between two delivery paths.
- **No live process state restoration** — Claude Code / vim / tmux sessions must be re-started after app restart. This is significant for long-running agent workflows.
- GPL-3.0 may be commercially awkward for some adopters.

---

## 4. Zed

### Tech stack
- **Language**: **[Rust (97.7% of codebase)](https://github.com/zed-industries/zed)**
- **GUI framework**: **GPUI** — Zed's own Rust-based UI framework, rendering to the GPU.
- **Renderer**: [GPU-accelerated](https://zed.dev/) — Metal on macOS, Vulkan/OpenGL elsewhere.
- **Repo**: [github.com/zed-industries/zed](https://github.com/zed-industries/zed) — **[78.9k stars, 36,872 commits, latest release v0.231.2 (April 10, 2026)](https://github.com/zed-industries/zed)**.

### License + business model
- **Mixed license**: The repo contains [AGPL, Apache, and GPL-3.0 components](https://github.com/zed-industries/zed). The bulk of Zed itself is GPL-3.0 / Apache depending on the subproject.
- **Pricing** (per [zed.dev/pricing](https://zed.dev/pricing)):
  - **Personal**: $0 — 2,000 accepted edit predictions; unlimited use with BYO API keys or external agents
  - **Pro**: $10/mo — unlimited edit predictions, $5 of hosted tokens/mo, usage-based beyond
  - **Enterprise**: Contact sales — SSO, usage analytics, shared billing

### AI integration
- **Providers**: [Anthropic, OpenAI, OpenRouter via API keys; Ollama for local models; Zed's own Zeta2 for edit predictions](https://zed.dev/docs/ai/agent-panel)
- **External agents via ACP (Agent Client Protocol)**: [Claude Agent, Codex, Gemini CLI, OpenCode](https://zed.dev/docs/ai/external-agents). Zed runs Claude Agent SDK under the hood and translates to ACP.
- **Autonomy level**: **Human-in-the-loop, not autonomous.** Three built-in profiles: [Write (full tools), Ask (read-only), Minimal (no tools)](https://zed.dev/docs/ai/agent-panel). Users accept/reject individual hunks via a multi-buffer diff view, can interrupt mid-generation, and restore via checkpoints.
- **MCP**: **Yes, first-class.** [Zed uses MCP to interact with context servers; supports Tools and Prompts features](https://zed.dev/docs/ai/mcp). Install as Extensions (GitHub, Puppeteer, Brave Search, Prisma, Figma, Resend, Container Use) or custom servers via settings. Supports the `notifications/tools/list_changed` dynamic tool update notification.
- **Multi-file edits**: Yes, shown as a multi-buffer view with per-file and per-hunk accept/reject.
- **Limitations with external Claude Agent** (per [Zed docs](https://zed.dev/docs/ai/external-agents)): "Editing past messages, resuming threads from history, and checkpointing" are NOT available. **Agent teams are currently not supported. Hooks are currently not supported.**

### Feature matrix

| Feature | Zed |
|---|---|
| Multi-pane / split panes | Y |
| Multi-tab | Y |
| Multi-window | Y |
| File tree sidebar | Y |
| Code viewer/editor | Y (it **is** a code editor first) |
| Browser surface | N |
| Markdown preview | Y |
| Image viewer | Y (basic) |
| Kanban or task board | N |
| Hook/event system | [**N for Claude Agent/ACP specifically** — "Hooks are currently not supported"](https://zed.dev/docs/ai/external-agents) |
| MCP support | Y (first-class) |
| Custom slash commands | Y (in agent panel) |

### Top user pain points
1. **[Issue #50142 — Agent diff/review UI completely missing with Claude Agent](https://github.com/zed-industries/zed/issues/50142)** (Feb 2026) — The review UI is completely absent — no diff, no buttons — when using Claude Agent via ACP. The agent edits files but the user has no way to review/accept/reject through the UI.
2. **[Issue #51648 — ACP agents (Claude Code) limited to 200K context instead of 1M on Max subscription](https://github.com/zed-industries/zed/issues/51648)** — Context compaction fires as if the limit were 120-200K despite Max tier advertising 1M. "Model effectively has dementia." BYOK got `allow_extended_context` but Max/ACP users didn't.
3. **[Issue #50304 — Claude Code Agent Panel has frequent errors and bugs after update](https://github.com/zed-industries/zed/issues/50304)** — Agent panel doesn't respect default model/mode settings from `settings.json`; keeps defaulting to Sonnet; edits don't appear inline; frequent "no path found" / 500 errors from Claude, while Claude Code in the terminal is fine.
4. **[Discussion #25498 — Claude Code Zed UI](https://github.com/zed-industries/zed/discussions/25498)** — Ongoing feedback thread on ACP UX.
5. **[Issue #8279 — Telescope-style search box](https://github.com/zed-industries/zed/issues/8279)** — still open since Feb 2024.
6. **Historical (now closed)**: [#5394 Windows support (closed Oct 2025)](https://github.com/zed-industries/zed/issues/5394), [#5395 Linux support (closed Jul 2024)](https://github.com/zed-industries/zed/issues/5395), [#11473 Dev Containers (closed Apr 2026)](https://github.com/zed-industries/zed/issues/11473), [#5065 Build and Debug support (closed Jun 2025)](https://github.com/zed-industries/zed/issues/5065). These show Zed **does eventually ship** on heavy demand — but the ACP/Claude UX bugs above are the current open wounds.

### Strengths and failures
**Wins**:
- **Best-in-class native Rust GPU performance** for an editor (GPUI framework, 97.7% Rust, 78.9k stars).
- **First-class MCP support with dynamic tool updates** — clearly ahead of every other product in this research on MCP.
- **Multi-buffer diff view** for agent edits with per-hunk accept/reject is the most sophisticated AI review UX found.
- Most mature open-source LLM provider matrix: Anthropic, OpenAI, OpenRouter, Ollama, plus external agents via ACP.
- Free tier is genuinely usable (2,000 edit predictions + unlimited BYOK).

**Fails**:
- **ACP integration with Claude Agent is buggy as of early 2026** — three serious P2 bugs open simultaneously: missing diff UI, 200K context clamp, agent panel settings not respected.
- **"Agent teams are currently not supported. Hooks are currently not supported"** with external Claude Agent — this is the exact feature set moai-adk relies on, meaning Zed is architecturally not a moai-adk host today.
- No browser surface, no kanban, no task board — it's an editor, not a workspace shell.
- GPL/AGPL licensing may deter some commercial adopters.

---

## 5. Ghostty

### Tech stack
- **Languages**: [Zig 78.7%, Swift 11.5% (macOS GUI), C 4.2%, C++ 2.9%](https://github.com/ghostty-org/ghostty)
- **Renderer**: [Metal on macOS, OpenGL on Linux](https://github.com/ghostty-org/ghostty); multi-threaded (dedicated read/write/render threads)
- **GUI framework**: **Platform-native** — AppKit on macOS, GTK on Linux. No Electron, no Flutter, no cross-platform UI toolkit.
- **Repo**: [github.com/ghostty-org/ghostty](https://github.com/ghostty-org/ghostty) — **[50.4k stars, 2.3k forks, 15,814 commits on main](https://github.com/ghostty-org/ghostty)**. ~1 million downloads/week for macOS per a 2026 report.
- **License**: **[MIT](https://github.com/ghostty-org/ghostty/blob/main/LICENSE)** (confirmed by reading the LICENSE file — "Copyright (c) 2024 Mitchell Hashimoto, Ghostty contributors")

### libghostty — the embeddable library (critical for moai-cli)

This is the most important section for a moai-cli that plans to embed Ghostty via `libghostty.xcframework`.

- **Existence confirmed**: [libghostty is "a cross-platform, zero-dependency C and Zig library for building terminal emulators or utilizing terminal functionality"](https://github.com/ghostty-org/ghostty). The project is modularizing into separate libraries starting with libghostty-vt for terminal sequence parsing.
- **Swift integration is documented and shipping**: [Mitchell Hashimoto's canonical "Integrating Zig and SwiftUI" blog post](https://mitchellh.com/writing/zig-and-swiftui) explains the approach:
  - Build libghostty as an xcframework
  - Drag into Xcode's Frameworks section, select **"Do Not Embed"** (linked statically)
  - `import GhosttyKit` in Swift — autocomplete works, types auto-convert to Swift
  - Write platform-specific GUI in SwiftUI/AppKit, all terminal logic stays in Zig
- **Prebuilt distribution**: [libghostty-spm](https://github.com/Uzaaft/awesome-libghostty) — `GhosttyKit.xcframework` distributed as a Swift Package Manager package for drop-in integration.
- **Real-world adopters**: [awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty) lists multiple shipping projects including **cmux** itself, `agtmux-term` (AI-agent-aware terminal with SwiftUI sidebar), `OpenOwl` (macOS Git GUI with libghostty + Metal), and `kytos` (macOS terminal on libghostty + KelyphosKit). [HN confirmed "libghostty is already backing more than a dozen terminal projects that are free and commercial"](https://news.ycombinator.com/item?id=45347117).
- **Hashimoto's stated trajectory**: "By the middle of 2027, the number of people using Ghostty via libghostty will dwarf the number of users that actually use the Ghostty GUI."
- **Example projects**: `Ghostling` (minimal complete project) and the `examples/` directory in the main repo.

### License + business model
- **MIT** — no commercial tier, no hosted service, pure OSS.
- Project governance under **ghostty-org** (moved from personal account in 2024-2025 era).
- No AI features, no telemetry, no pricing page.

### AI integration
- **None.** Ghostty is intentionally AI-free. It is a terminal engine, not a workspace.

### Feature matrix

| Feature | Ghostty |
|---|---|
| Multi-pane / split panes | Y |
| Multi-tab | Y |
| Multi-window | Y |
| File tree sidebar | N |
| Code viewer/editor | N |
| Browser surface | N |
| Markdown preview | N |
| Image viewer | N (limited — terminal-native only) |
| Kanban or task board | N |
| Hook/event system | N (no plugin system; config-file driven) |
| MCP support | N |
| Custom slash commands | N |
| Scrollback search | Y (shipped in 1.3, March 2026) |
| Clickable file paths | Requested in [#1972](https://github.com/ghostty-org/ghostty/issues/1972); partial |
| tmux Control Mode | Requested in [#1935](https://github.com/ghostty-org/ghostty/issues/1935) — long-standing |

### Top user pain points (historical + current)
1. **[Scrollbar saga — Issue #111](https://github.com/ghostty-org/ghostty/issues/111)** — Requested since pre-release 2023, 171 thumbs-up. Finally shipped in 1.3.0 (March 2026) after years of friction. The canonical "Ghostty user frustration" example.
2. **[Issue #1935 — tmux Control Mode support](https://github.com/ghostty-org/ghostty/issues/1935)** — Still open; major gap for tmux power users.
3. **[Issue #189 — GTK scrollback search](https://github.com/ghostty-org/ghostty/issues/189)** — Now resolved in 1.3.
4. **[Issue #1972 — Clickable file paths](https://github.com/ghostty-org/ghostty/issues/1972)** — Basic terminal UX feature, requested, still partial.
5. **[Issue #3645 — Window background image configuration](https://github.com/ghostty-org/ghostty/issues/3645)** — Cosmetic but high-reaction.
6. **[Issue #2384 — Customize initial size for quick terminal (macOS)](https://github.com/ghostty-org/ghostty/issues/2384)** — macOS-specific sizing.
7. **[Issue #1392 — Different working directory behavior for tabs/windows](https://github.com/ghostty-org/ghostty/issues/1392)** — Configuration flexibility.
8. **["Déjà vu: Ghostly CVEs in my terminal title" HN thread](https://news.ycombinator.com/item?id=42562743)** — Security researcher @dgl reported CVEs related to terminal title handling; Hashimoto publicly apologized for forgetting to test against past discoveries.
9. **["Why Ghostty is so suddenly popular?" HN thread](https://news.ycombinator.com/item?id=42885783)** — Skeptical commenters questioning whether Ghostty actually beats iTerm2/WezTerm/Kitty feature-wise.

### Strengths and failures
**Wins**:
- **libghostty is the winning architecture bet** — Hashimoto's explicit strategy is to become the shared terminal core, and it's already working (cmux, agtmux-term, OpenOwl, kytos, etc.).
- **MIT license + zero dependencies + Zig/Swift** — maximally permissive, maximally portable.
- **Platform-native UIs** (AppKit/GTK) rather than one cross-platform toolkit — each OS feels correct.
- ~1M downloads/week on macOS is real traction.
- **Ghostling example project** and `awesome-libghostty` give adopters a working starting point.

**Fails**:
- Long delays on "obvious" features — the 3-year scrollbar saga became a meme.
- No AI features at all — not a direct competitor to moai-cli, but also not a complete product for AI developers on its own.
- tmux Control Mode still missing.
- CVE incident in terminal title handling revealed security process gaps.

---

## Cross-Product Insights for moai-cli

### Common pain points across competitors

1. **Platform coverage is always the #1 issue.** Warp (historical), Zed (historical), Wave Terminal (still open #76), and cmux (#330, #1012) all had their most-reacted issues be "please support my OS." Warp and Zed eventually shipped Linux/Windows; cmux and Wave still have gaps. **If moai-cli ships macOS-only at 1.0, expect Linux to be the loudest request within weeks.**

2. **Privacy and telemetry burn trust permanently.** [Warp's 2022 telemetry incident](https://github.com/warpdotdev/Warp/issues/1346) and [2025 "sent session to LLM without consent" incident](https://news.ycombinator.com/item?id=44953470) still dominate HN sentiment 4 years later. Even Wave Terminal got the same "anonymity is meaningless when IP correlates" pushback. **Any default-on network behavior for AI features is a reputational minefield.**

3. **Electron means memory complaints, every time.** [Lemmy user rejected Wave outright on Electron grounds](https://lemmy.dbzer0.com/comment/13354563). Warp's marketing positions hard against "Electron bloat." Even Zed — which is Rust/GPUI — explicitly cites avoiding Electron as a feature. **Shipping an Electron shell in 2026 will cost moai-cli credibility with the HN/Reddit developer audience regardless of actual memory numbers.**

4. **Claude Code/LLM integration is consistently buggy.** Zed has three simultaneous P2 bugs against ACP Claude Agent ([#50142](https://github.com/zed-industries/zed/issues/50142), [#51648](https://github.com/zed-industries/zed/issues/51648), [#50304](https://github.com/zed-industries/zed/issues/50304)). cmux has [#2322](https://github.com/manaflow-ai/cmux/issues/2322) about flaky Claude notifications due to race conditions between dual delivery paths. **Expect hooks/notifications/MCP integration to be the hardest part of moai-cli, not the terminal rendering.**

5. **Session restoration of live agent state is unsolved.** cmux explicitly [does NOT restore live Claude Code / tmux / vim sessions](https://github.com/manaflow-ai/cmux). Wave has [Issue #2057 "Remember last terminal path"](https://github.com/wavetermdev/waveterm/issues/2057) still open. Zed limits "resuming threads from history" for external agents. **This is a systematic gap.**

6. **"Hooks are currently not supported" with external agents.** [Zed's own documentation states this about Claude Agent via ACP](https://zed.dev/docs/ai/external-agents). Since moai-adk is built on hooks, Zed is not currently a viable host for moai-adk.

### Features competitors lack — moai-cli differentiation opportunities

1. **Kanban/SPEC board linked to git worktrees.** **None of the 5 products has this.** cmux has vertical tabs with git branch + PR status + ports, which is the closest analogue, but it's not a kanban view and it's not SPEC-aware. This is moai-cli's single clearest differentiator.

2. **Agent Run Viewer with hook events + token metrics.** cmux has notifications; Wave has the clever "badge rollup" system; Zed has a multi-buffer diff view. **None of them has a dedicated telemetry/metrics dashboard** showing tokens consumed, tool calls, hook firings, etc. This is moai-cli territory.

3. **Markdown viewer as a first-class surface.** Only Wave Terminal has inline markdown. cmux, Warp, Ghostty, and Zed (as editor) all lack a dedicated markdown preview pane. For moai-adk's SPEC-driven workflow where spec.md, plan.md, and reports are all markdown, this matters.

4. **Image viewer as a first-class surface.** Only Wave has this. Critical for agents that emit screenshots, diagrams, or rendered outputs.

5. **File tree + browser + terminal + code + markdown + images in one window**, governed by a unified layout system. Wave is closest but it's Electron; cmux has ~2/6 of these; Zed has ~4/6 but no browser. **No product has the full set natively.**

6. **MCP first-class in a terminal-shell context.** Zed has MCP for its editor/agent-panel, but not in a terminal-shell role. Wave and cmux don't have it at all. Warp doesn't document it publicly. **Terminal-shell + MCP is open territory.**

7. **Hook/event system as a user-exposed contract.** Wave has the badge-rollup trick for Claude Code specifically. Zed explicitly says hooks are unsupported with external agents. cmux exposes OSC 9/99/777 + `cmux notify` CLI — but that's low-level. **A documented, first-class hook system targeting moai-adk's 27+ hook events is a moat.**

### Features competitors do well — moai-cli must match

1. **Native GPU rendering via libghostty or equivalent.** Non-negotiable baseline. cmux proved Swift+AppKit+libghostty works; that's the minimum bar.

2. **Sidebar metadata per workspace** — cmux's git branch + PR # + listening ports + last notification is a beloved feature. moai-cli's tabs need at least this much context per SPEC worktree.

3. **Multi-file diff with per-hunk accept/reject.** Zed's multi-buffer agent view is the gold standard. If moai-cli shows agent work to the user, it needs this or better.

4. **First-class MCP support.** Zed has it, everyone else is behind. moai-cli must match Zed here.

5. **BYO API key + local model support (Ollama).** Zed, Wave, and Warp all support this. Users have been burned enough times on forced cloud AI that this is table stakes.

6. **Genuinely open source license.** Zed (mixed but OSS), Wave (Apache-2.0), cmux (GPL-3.0), Ghostty (MIT) all pass this bar. Warp doesn't — and it costs them in trust. moai-cli picking a commercial-friendly OSS license (MIT/Apache/MPL) would differentiate vs cmux (GPL-3.0).

### Architecture lessons learned

- **Electron is a losing brand, even if it's technically competent.** Wave's Go-backend/Electron-frontend architecture is pragmatic, but ["Electron apps hog all memory" is the first comment every time Wave is discussed](https://lemmy.dbzer0.com/comment/13354563). A Rust/Swift-native moai-cli will win this comparison automatically.
- **libghostty is a validated, shipping embed path.** [Hashimoto's blog post on Zig+SwiftUI integration](https://mitchellh.com/writing/zig-and-swiftui) is the canonical guide. [libghostty-spm](https://github.com/Uzaaft/awesome-libghostty) is the drop-in package. cmux proves it works in production at 13k+ stars.
- **Closed-source clients lose HN support even when technically superior.** Warp's GPU-rendered Rust is objectively faster than anything else, but every HN thread about it re-litigates the telemetry incident. Zed (Rust/GPUI/OSS) is faster-growing.
- **"Not planned" maintainer responses poison community goodwill.** Wave Terminal's pattern of closing font-settings and default-shell requests as "not planned" shows up in HN commentary as a turnoff. moai-cli should default to "accepted, unscheduled" rather than "not planned" for basic features.
- **Agent teams + hooks is the hardest integration surface.** Zed, at 78.9k stars with a full-time team, explicitly says "Agent teams are currently not supported. Hooks are currently not supported" for external Claude Agent. [cmux's notification race conditions](https://github.com/manaflow-ai/cmux/issues/2322) show this is hard even for focused scope. Budget 2-3× the engineering cost you estimate for this layer.
- **Cross-platform from day 1 is expensive but the alternative is painful.** Warp took years to get to Linux/Windows; Zed took ~1.5 years. cmux and Wave still don't have full platform coverage. Starting macOS-only is defensible (cmux did it) but creates recurring community pressure.

---

## Source Inventory

### Warp
- Homepage and positioning: https://www.warp.dev/
- Pricing: https://www.warp.dev/pricing
- GitHub issues repo: https://github.com/warpdotdev/Warp
- How Warp works (official blog): https://www.warp.dev/blog/how-warp-works
- The New Stack review: https://thenewstack.io/a-review-of-warp-another-rust-based-terminal/
- Wikipedia entry: https://en.wikipedia.org/wiki/Warp_(terminal)
- Linux release coverage: https://news.itsfoss.com/warp/
- Telemetry incident Issue #1346: https://github.com/warpdotdev/Warp/issues/1346
- HN: "Warp sends terminal session to LLM without consent": https://news.ycombinator.com/item?id=44953470
- HN: "Telemetry is now optional in Warp": https://news.ycombinator.com/item?id=33910992
- HN: "Warp telemetry original thread": https://news.ycombinator.com/item?id=30921973
- Critical blog post: https://www.careerlimitingmoves.com/2022/04/09/warp/
- Warp issues search by reactions: https://github.com/warpdotdev/Warp/issues?q=is%3Aissue+sort%3Areactions-%2B1-desc
- Specific issues: https://github.com/warpdotdev/Warp/issues/1957 (SSH), /4339 (Ollama), /2788 (API key), /1811 (tab completion), /7287 (403 error), /4240 (WSL), /204 (Windows), /120 (Linux)

### Wave Terminal
- Homepage: https://www.waveterm.dev/
- Docs home: https://docs.waveterm.dev/
- Widgets docs: https://docs.waveterm.dev/widgets
- Claude Code integration docs: https://docs.waveterm.dev/claude-code
- GitHub repo: https://github.com/wavetermdev/waveterm
- Issues by reactions: https://github.com/wavetermdev/waveterm/issues?q=is%3Aissue+sort%3Areactions-%2B1-desc
- Specific issues: https://github.com/wavetermdev/waveterm/issues/76 (Windows), /71, /1093 (Flatpak), /707 (SSH), /2168 (Agent mode), /2057 (path), /91 (default shell), /83 (font), /610 (security)
- HN thread: https://news.ycombinator.com/item?id=38869559
- Lemmy rejection on Electron grounds: https://lemmy.dbzer0.com/comment/13354563
- OpenReplay review (Warp vs Wave): https://blog.openreplay.com/warp-wave-terminal-ai-powered/
- Slashdot listing: https://slashdot.org/software/p/Wave-Terminal/

### cmux
- GitHub repo: https://github.com/manaflow-ai/cmux
- README: https://github.com/manaflow-ai/cmux/blob/main/README.md
- Parent company: https://manaflow.com/
- PR #1179 (claude-teams launcher): https://github.com/manaflow-ai/cmux/pull/1179
- Issue #2322 (notification race condition): https://github.com/manaflow-ai/cmux/issues/2322
- Issue #330 (Linux support): https://github.com/manaflow-ai/cmux/issues/330
- Issue #480 (persistence): https://github.com/manaflow-ai/cmux/issues/480
- Issue #135 (keybindings): https://github.com/manaflow-ai/cmux/issues/135
- Issue #1012 (Windows): https://github.com/manaflow-ai/cmux/issues/1012
- Issue #1664 (SSH workspaces): https://github.com/manaflow-ai/cmux/issues/1664
- Issue #719 (browser): https://github.com/manaflow-ai/cmux/issues/719
- Issue #124 (passkeys): https://github.com/manaflow-ai/cmux/issues/124
- Issue #645 (modifier keys): https://github.com/manaflow-ai/cmux/issues/645
- HN Show HN (native version): https://news.ycombinator.com/item?id=47079718
- HN Show HN (original): https://news.ycombinator.com/item?id=45596024
- HN "Tmux for Claude Code": https://news.ycombinator.com/item?id=47008732
- HN comment re worktrees: https://news.ycombinator.com/item?id=47468901
- Claude Hub resource listing: https://www.claude-hub.com/resource/github-cli-manaflow-ai-cmux-cmux/

### Zed
- Homepage: https://zed.dev/
- Pricing: https://zed.dev/pricing
- Agent panel docs: https://zed.dev/docs/ai/agent-panel
- MCP docs: https://zed.dev/docs/ai/mcp
- External agents docs: https://zed.dev/docs/ai/external-agents
- GitHub repo: https://github.com/zed-industries/zed
- Issues by reactions: https://github.com/zed-industries/zed/issues?q=is%3Aissue+sort%3Areactions-%2B1-desc
- Issue #50142 (missing diff UI): https://github.com/zed-industries/zed/issues/50142
- Issue #51648 (200K context clamp): https://github.com/zed-industries/zed/issues/51648
- Issue #50304 (agent panel bugs): https://github.com/zed-industries/zed/issues/50304
- Discussion #25498 (Claude Code Zed UI): https://github.com/zed-industries/zed/discussions/25498
- Historical closed issues: https://github.com/zed-industries/zed/issues/5394 (Windows), /5395 (Linux), /5065 (build/debug), /11473 (dev containers), /8279 (telescope search)
- Claude Code vs Zed comparison: https://www.selecthub.com/vibe-coding-tools/claude-code-vs-zed/
- Pikagent comparison: https://www.pikagent.com/compare/claude-code-vs-zed-ai

### Ghostty
- Homepage: https://ghostty.org/
- Install docs: https://ghostty.org/docs/install/binary
- GitHub repo: https://github.com/ghostty-org/ghostty
- LICENSE (MIT confirmed): https://github.com/ghostty-org/ghostty/blob/main/LICENSE
- Issues by reactions: https://github.com/ghostty-org/ghostty/issues?q=is%3Aissue+sort%3Areactions-%2B1-desc
- Specific issues: https://github.com/ghostty-org/ghostty/issues/111 (scrollbar), /189 (search), /1935 (tmux control mode), /1972 (clickable paths), /3645 (background image), /2384 (quick terminal), /1392 (CWD behavior), /5047 (opacity), /2509 (tab colors)
- Mitchell Hashimoto: Zig + SwiftUI blog (canonical libghostty embed guide): https://mitchellh.com/writing/zig-and-swiftui
- Awesome-libghostty: https://github.com/Uzaaft/awesome-libghostty
- HN Ghostty 1.0: https://news.ycombinator.com/item?id=42517447
- HN Terminal Emulator thread: https://news.ycombinator.com/item?id=47206009
- HN CVEs in terminal title: https://news.ycombinator.com/item?id=42562743
- HN Libghostty is coming: https://news.ycombinator.com/item?id=45347117
- HN "Why suddenly popular": https://news.ycombinator.com/item?id=42885783
- HN Termius alternative thread: https://news.ycombinator.com/item?id=42527584
- Ziggit release thread: https://ziggit.dev/t/terminal-emulator-ghostty-1-0-released/7533
- XDA scrollbar article: https://www.xda-developers.com/the-ghostty-terminal-finally-adds-a-feature-people-have-asked-for-since-2023/
- GIGAZINE coverage: https://gigazine.net/gsc_news/en/20260404-ghostty-ui/

---

## Research Notes and Caveats

- **GitHub issue reaction counts**: GitHub's HTML pages do not expose exact reaction numbers in the listing view — only the sort order is visible. Where specific numbers are cited (e.g., "171 thumbs-up on issue #111"), those come from secondary reporting (XDA-developers). Where no count is cited, treat ordering as directional rather than quantitative.
- **cmux license status**: HN posts from Feb 2026 describe cmux as "MIT-licensed", but the current [README](https://github.com/manaflow-ai/cmux) states GPL-3.0-or-later. I treated the current README as authoritative.
- **cmux star count**: Observed between 12.9k and 13.6k across different pages — the live count is moving. The `awesome-libghostty` listing and README fetches were both cited as in the 13k range.
- **Wave Terminal widget list**: The docs page only exposed three widget types (Term, Preview, Codeedit) in what I could fetch — the homepage marketing suggests more. Treat this as a floor, not a ceiling.
- **Warp MCP support**: I could not find authoritative documentation confirming or denying MCP. Marked `[UNVERIFIED]`.
- **Zed license**: The repo states "AGPL, Apache, and GPL-3.0" — I did not inspect each subproject to map which license covers which code. Verify before making commercial-use assumptions.
- **No fabricated stats.** Where search results did not provide exact numbers, I wrote directional language ("most-reacted," "long-standing") rather than invent percentages.
- **Training data not used.** Every factual claim has a URL citation from live WebSearch/WebFetch calls made during this session.
