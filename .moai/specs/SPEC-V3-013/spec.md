---
id: SPEC-V3-013
version: 1.0.0
status: draft
created_at: 2026-04-26
updated_at: 2026-04-26
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-002, SPEC-V3-003, SPEC-V3-004]
parallel_with: []
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-2, ui, settings, persistence, design-system]
revision: v1.0.0 (initial draft, Settings Surface)
---

# SPEC-V3-013: Settings Surface — SettingsModal (880×640) + 6 sections + UserSettings 영속화 + 런타임 적용

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-26 | 초안 작성. RG-V13-1 ~ RG-V13-7, AC-V13-1 ~ AC-V13-12, MS-1/MS-2/MS-3 정의. IMPLEMENTATION-NOTES.md v1.1 §13.7 D 항목 (P0) carry. Round 2 시안 (`moai-revisions.jsx`) 정합. SPEC-V3-001 ~ V3-004 의 design module + RootView 인프라 활용. v0.1.0 단계는 Appearance + Keyboard full, 4 sub-panes (Editor/Terminal/Agent/Advanced) 는 skeleton + 1 setting 각 한정 (over-engineering 회피). USER-DECISION 게이트 없음 — 외부 차단 없음. |

---

## 1. 개요

### 1.1 목적

moai-studio 사용자가 Cmd+, (macOS) / Ctrl+, (Linux/Windows) 또는 메뉴를 통해 환경설정을 변경하고, 변경이 즉시/재기동 시 반영되며, platform-appropriate 위치에 영속화되는 SettingsModal 을 정의한다.

본 SPEC 의 산출은:

- 880×640 px 의 SettingsModal 이 backdrop scrim 위에 mount.
- 200px sidebar (6 section list) + 680px main pane.
- 6 sections: Appearance / Keyboard / Editor / Terminal / Agent / Advanced.
- AppearancePane: theme(dark/light/system) / density(compact/comfortable) / accent(teal/blue/violet/cyan) / font_size(12~18px).
- KeyboardPane: binding 테이블 + edit dialog + 충돌 검출.
- 4 sub-panes (Editor/Terminal/Agent/Advanced): skeleton + 최소 1개 setting 각 (v0.1.0 범위).
- UserSettings struct 의 JSON 영속화 (`dirs::config_dir()/moai-studio/settings.json`, atomic write, fail-soft load, schema_version `moai-studio/settings-v1`).
- theme/accent/density/font_size 변경 → design::runtime::ActiveTheme global update → cx.notify() 전체 re-render.
- keybinding 변경 → RootView keymap rebuild + 즉시 적용.

### 1.2 IMPLEMENTATION-NOTES.md §13.7 / Round 2 시안 정합

`.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md` v1.1 §13.7 (D 항목, P0) 가 6 sections 과 각 section 의 setting 후보를 명시했다. 본 SPEC 은 v0.1.0 단계에 그 범위를 다음과 같이 한정한다:

- Appearance: §13.7 의 4 setting (theme/density/accent/font_size) 모두 구현.
- Keyboard: §13.7 의 binding 테이블 + edit 모두 구현.
- Editor/Terminal/Agent/Advanced: §13.7 의 4~5 setting 후보 중 1개씩만 (skeleton). v0.2.0+ SPEC 에서 확장.

`.moai/design/from-claude-design/project/moai-revisions.jsx` 의 `SettingsModal` / `AppearancePane` / `KeyboardPane` 컴포넌트 prototype 을 GPUI 시각 구현의 reference 로 사용 (880×640, 200/680 split, 색상/typography).

### 1.3 근거 문서

- `.moai/specs/SPEC-V3-013/research.md` — 코드베이스 분석, 영속화 전략, UX 패턴, 위험.
- `.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md` v1.1 §13.7 / §13.9 / §14 D 항목.
- `.moai/design/from-claude-design/project/moai-revisions.jsx` — SettingsModal 시안.
- `.moai/design/tokens.json` v2.0.0 — canonical tokens (모두의AI 청록 #144a46/#22938a).
- `crates/moai-studio-ui/src/design/{tokens,layout,typography}.rs` — 기존 design module.
- `crates/moai-studio-workspace/src/persistence.rs` — 영속화 패턴 reference (atomic write, fail-soft load, schema_version).

---

## 2. 배경 및 동기

본 섹션의 상세는 `.moai/specs/SPEC-V3-013/research.md` §1 참조. 최소 맥락만 요약한다.

- moai-studio 는 dark theme 고정, 영속화 가능한 사용자 환경설정 부재. Cmd+P palette 외 사용자 customization 진입점 없음.
- IMPLEMENTATION-NOTES.md §14 D 항목 (P0) 이 Settings modal 을 후속 P0 작업으로 명시.
- design::tokens 가 이미 모두의AI 청록 + accent 4종 + dark/light 정의 보유 — runtime override 만 추가하면 즉시 활용 가능.
- workspace persistence.rs 가 atomic write + fail-soft load 패턴 보유 — 동일 패턴 차용으로 새로운 패턴 도입 비용 0.
- v0.1.0 release (SPEC-V3-011) 의 사용자 가치 완결을 위한 final UI surface — 환경설정 부재는 release-blocker 수준은 아니지만 modern IDE 의 minimum bar.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. 사용자가 Cmd+, (macOS) / Ctrl+, (Linux/Windows) 단축키 또는 메뉴 (`Settings`) 로 SettingsModal 을 mount 한다.
- G2. SettingsModal 이 880×640 px 컨테이너로 backdrop scrim 위 중앙 정렬 표시된다.
- G3. Sidebar (200px) 가 6 section row (Appearance/Keyboard/Editor/Terminal/Agent/Advanced) 를 표시하며 선택 시 main pane 이 swap.
- G4. AppearancePane 이 theme RadioGroup, density ToggleGroup, accent ColorSwatch (4 colors), font_size Slider (12~18px) 4 controls 를 모두 구현한다.
- G5. KeyboardPane 이 default + custom binding 테이블 + edit dialog + 충돌 검출 로직을 구현한다.
- G6. Editor/Terminal/Agent/Advanced 4 panes 가 section title + description + 최소 1개 setting control 을 구현한다 (skeleton).
- G7. UserSettings struct 가 JSON 으로 `dirs::config_dir()/moai-studio/settings.json` 에 저장된다.
- G8. 변경이 200ms debounce 후 atomic write (tempfile + rename) 로 영속화된다.
- G9. 앱 시작 시 UserSettings 가 자동 load 되며 손상 시 .bak.{timestamp} 백업 + Default 반환 + warn 로그.
- G10. theme/accent/density/font_size 변경 시 design::runtime::ActiveTheme global 이 update 되고 RootView 가 cx.notify() 로 전체 re-render.
- G11. keybinding 변경 시 RootView keymap rebuild + 즉시 적용 (재기동 불필요).
- G12. SettingsModal dismiss — Esc / scrim click / X 버튼 모두 지원.
- G13. 단위 테스트 coverage 85%+ 가능 구조 (UserSettings serde / SettingsViewState 변경 / KeyboardPane conflict detection).
- G14. Local 5 quality gates 통과 가능: cargo test PASS, clippy 0 warning, fmt PASS, bench 회귀 없음, cargo check --release PASS.

### 3.2 비목표 (Non-Goals)

- N1. **4 sub-panes (Editor/Terminal/Agent/Advanced) 의 full implementation** — v0.1.0 단계 비목표. skeleton + 1 setting 각만. v0.2.0+ 별 SPEC.
- N2. **사용자 정의 theme** — `~/.config/moai-studio/themes/*.json` import. v0.3.0+ 별 SPEC.
- N3. **multi-stroke keybinding** (`Cmd+K Cmd+S` 등) — v0.1.0 single-stroke 만. v0.3.0+ 별 SPEC.
- N4. **cloud sync** (iCloud / GitHub gist) — v0.2.0+ 별 SPEC.
- N5. **multi-window settings sync** — 단일 instance 가정. v0.2.0+ 별 SPEC.
- N6. **settings export / import 파일** — JSON file 직접 편집은 가능하나 UI export/import 버튼 없음. v0.2.0+ 별 SPEC.
- N7. **command palette 에서 setting 검색** — v0.1.0 단계 SettingsModal 내 검색 box 도 비목표. v0.2.0+.
- N8. **i18n 번역** — section title / control label 모두 한국어 (code_comments=ko). v0.3.0+ 다국어 별 SPEC.
- N9. **SettingsModal 의 keyboard navigation** (Tab / Arrow) — v0.1.0 단계는 mouse + click 만. accessibility 별 SPEC 으로 후속.
- N10. **theme runtime switch 시 모든 컴포넌트의 const 참조 일괄 마이그레이션** — MS-3 단계 점진 적용. v0.1.0 단계는 부분 unthemed 잔존 수용 (R-V13-3).
- N11. **moai-studio 의 design::tokens 정의 변경** — design::runtime::ActiveTheme 만 신설. tokens.rs 의 const 값은 그대로.
- N12. **font family 변경** — Pretendard 9-weight 고정. v0.3.0+ 별 SPEC.

---

## 4. 사용자 스토리

- **US-V13-1**: 사용자가 Cmd+, (macOS) 를 누르면 SettingsModal 이 880×640 으로 backdrop scrim 위에 mount 된다.
- **US-V13-2**: 사용자가 sidebar 의 "Appearance" row 를 보면 default 로 선택된 상태이며 main pane 이 AppearancePane 을 표시한다.
- **US-V13-3**: 사용자가 sidebar 의 다른 section 을 클릭하면 main pane 이 해당 PaneEntity 로 즉시 swap.
- **US-V13-4**: 사용자가 AppearancePane 의 theme RadioGroup 에서 "light" 를 클릭하면 앱 전체가 light theme 으로 즉시 전환된다.
- **US-V13-5**: 사용자가 AppearancePane 의 accent 4 color swatch 중 "violet" 을 클릭하면 모든 accent 색상 (focus border, button hover, selected row) 이 즉시 violet 으로 전환된다.
- **US-V13-6**: 사용자가 AppearancePane 의 font_size Slider 를 14 → 16 으로 드래그하면 코드 에디터와 UI 텍스트 모두 즉시 16px 로 reflow.
- **US-V13-7**: 사용자가 AppearancePane 의 density ToggleGroup 에서 "compact" 를 선택하면 패널 padding / row height 가 0.85x 로 축소.
- **US-V13-8**: 사용자가 KeyboardPane 의 binding 테이블에서 "command_palette" row 의 "Edit" 버튼을 클릭하면 sub-dialog 가 mount 되어 새 shortcut 을 캡처할 수 있다.
- **US-V13-9**: 사용자가 새 shortcut 을 입력하면 충돌 검사 후 충돌 없을 시 save, 충돌 있을 시 inline error 와 함께 reject.
- **US-V13-10**: 사용자가 EditorPane 을 클릭하면 section title "Editor", description, tab_size NumericInput 1개 control 을 표시.
- **US-V13-11**: 사용자가 SettingsModal 의 X 버튼 또는 Esc 를 누르면 modal 이 dismount 되고 backdrop 도 제거된다.
- **US-V13-12**: 사용자가 어떤 setting 을 변경하면 200ms 후 settings.json 이 atomic 으로 저장되며 사용자는 별도 "Save" 버튼 없이 자동 영속화된다.
- **US-V13-13**: 사용자가 앱을 재기동하면 모든 변경된 setting (theme/accent/density/font_size/keybindings) 이 자동 복원된다.
- **US-V13-14**: 사용자가 settings.json 을 외부에서 수동 편집하여 손상시킨 경우 다음 시작 시 .bak.{timestamp} 로 백업되고 Default 로 시작된다 (warn 로그 남김, 앱은 정상 시작).

---

## 5. 기능 요구사항 (EARS)

### RG-V13-1 — SettingsModal mount / dismiss / 컨테이너 레이아웃

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-V13-001 | Event-Driven | 사용자가 Cmd+, (macOS) 또는 Ctrl+, (Linux/Windows) 를 누르면, 시스템은 SettingsModal Entity 를 RootView 에 mount 한다. | When the user presses Cmd+, (macOS) or Ctrl+, (Linux/Windows), the system **shall** mount the SettingsModal Entity in RootView. |
| REQ-V13-002 | Ubiquitous | SettingsModal 컨테이너는 880×640 px 크기로 backdrop scrim 위 화면 중앙에 정렬된다. | The SettingsModal container **shall** be 880×640 px, centered on screen above a backdrop scrim. |
| REQ-V13-003 | Ubiquitous | SettingsModal 은 200 px sidebar (좌측) + 680 px main pane (우측) 의 horizontal split 으로 구성된다. | The SettingsModal **shall** consist of a 200 px sidebar (left) and 680 px main pane (right) in horizontal split. |
| REQ-V13-004 | Event-Driven | 사용자가 Esc 키, scrim 클릭, 또는 우상단 X 버튼을 누르면, 시스템은 SettingsModal 을 dismount 하고 backdrop 도 제거한다. | When the user presses Esc, clicks the scrim, or clicks the top-right X button, the system **shall** dismount the SettingsModal and remove the backdrop. |
| REQ-V13-005 | State-Driven | SettingsModal 이 mount 된 동안, 시스템은 z-index 30 으로 다른 modal/palette 보다 위에 표시한다. | While SettingsModal is mounted, the system **shall** display it at z-index 30, above other modals and palettes. |
| REQ-V13-006 | Unwanted | 시스템은 SettingsModal 이 이미 mount 된 상태에서 Cmd+, 가 다시 눌려도 추가 mount 하지 않는다 (이미 열려있으면 무시 또는 focus). | The system **shall not** re-mount the SettingsModal if it is already mounted; the keypress is ignored or refocuses the modal. |

### RG-V13-2 — Sidebar 6 sections + 선택 상태

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-V13-010 | Ubiquitous | Sidebar 는 6 section row 를 정해진 순서로 렌더한다: Appearance, Keyboard, Editor, Terminal, Agent, Advanced. | The sidebar **shall** render 6 section rows in fixed order: Appearance, Keyboard, Editor, Terminal, Agent, Advanced. |
| REQ-V13-011 | Ubiquitous | 각 section row 는 36 px 높이로, 좌측 16 px padding + icon (16 px) + 8 px gap + label (Pretendard 14 px) 로 구성된다. | Each section row **shall** be 36 px tall with 16 px left padding + 16 px icon + 8 px gap + 14 px Pretendard label. |
| REQ-V13-012 | Ubiquitous | SettingsModal 의 default 선택 section 은 Appearance 이며 mount 시 항상 Appearance 가 활성화된 상태로 표시된다. | The default selected section **shall** be Appearance; on mount, Appearance is always active. |
| REQ-V13-013 | Event-Driven | 사용자가 sidebar 의 row 를 클릭하면, 시스템은 SettingsViewState.selected_section 을 update 하고 main pane 을 해당 PaneEntity 로 swap 한다. | When the user clicks a sidebar row, the system **shall** update SettingsViewState.selected_section and swap the main pane to the corresponding PaneEntity. |
| REQ-V13-014 | State-Driven | 선택된 section row 동안, 시스템은 row 배경을 accent.soft (또는 dark theme 의 SOFT_APPROX) 로, label 색을 accent.base 로 표시한다. | While a section row is selected, the system **shall** render its background as accent.soft and its label as accent.base. |

### RG-V13-3 — AppearancePane (full implementation)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-V13-020 | Ubiquitous | AppearancePane 은 4 control 을 표시한다: theme RadioGroup, density ToggleGroup, accent ColorSwatch (4 colors), font_size Slider. | The AppearancePane **shall** display 4 controls: theme RadioGroup, density ToggleGroup, accent ColorSwatch (4 colors), font_size Slider. |
| REQ-V13-021 | Event-Driven | 사용자가 theme RadioGroup 의 옵션 (dark / light / system) 을 선택하면, 시스템은 UserSettings.appearance.theme 을 update 하고 ActiveTheme global 을 갱신한 뒤 cx.notify() 로 RootView 를 re-render 한다. | When the user selects a theme RadioGroup option, the system **shall** update UserSettings.appearance.theme, refresh the ActiveTheme global, and trigger cx.notify() on RootView. |
| REQ-V13-022 | Event-Driven | 사용자가 density ToggleGroup (compact / comfortable) 을 변경하면, 시스템은 UserSettings.appearance.density 를 update 하고 ActiveTheme.density 를 갱신한 뒤 layout::SPACING_MULTIPLIER 를 0.85 또는 1.0 으로 적용한다. | When the user toggles density, the system **shall** update UserSettings.appearance.density and apply spacing multiplier 0.85 (compact) or 1.0 (comfortable). |
| REQ-V13-023 | Event-Driven | 사용자가 accent 4 color swatch (teal / blue / violet / cyan) 중 하나를 클릭하면, 시스템은 UserSettings.appearance.accent 를 update 하고 ActiveTheme.accent 를 해당 enum 으로 갱신한다. | When the user clicks an accent swatch, the system **shall** update UserSettings.appearance.accent and refresh ActiveTheme.accent. |
| REQ-V13-024 | Event-Driven | 사용자가 font_size Slider 를 변경하면 (range 12~18 px, step 1), 시스템은 UserSettings.appearance.font_size_px 를 update 하고 ActiveTheme.font_size_px 를 갱신하여 코드 에디터와 UI 텍스트가 reflow 한다. | When the user adjusts the font_size Slider (12~18 px, step 1), the system **shall** update UserSettings.appearance.font_size_px and reflow code/UI text. |
| REQ-V13-025 | Unwanted | 시스템은 font_size Slider 의 값이 12 미만 또는 18 초과인 경우 적용하지 않는다 (UI 가 range 강제). | The system **shall not** accept font_size values outside the 12~18 range; the UI enforces the range. |

### RG-V13-4 — KeyboardPane (full implementation)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-V13-030 | Ubiquitous | KeyboardPane 은 default + custom 모든 keybinding 을 테이블 형태로 표시한다. column: action / current shortcut / Edit button. | The KeyboardPane **shall** render all default and custom keybindings in a table with columns: action, current shortcut, Edit button. |
| REQ-V13-031 | Event-Driven | 사용자가 binding row 의 "Edit" 버튼을 클릭하면, 시스템은 sub-dialog (400×200 px) 를 mount 하여 새 shortcut 캡처를 시작한다. | When the user clicks the Edit button on a binding row, the system **shall** mount a 400×200 px sub-dialog to capture a new shortcut. |
| REQ-V13-032 | Event-Driven | 사용자가 sub-dialog 에서 새 shortcut 을 입력하고 Save 를 누르면, 시스템은 충돌 검사 (conflict_check) 를 수행한다. | When the user enters a new shortcut and clicks Save, the system **shall** perform a conflict check. |
| REQ-V13-033 | Event-Driven | 충돌 검사 통과 시, 시스템은 UserSettings.keyboard.bindings 를 update 하고 RootView 의 keymap 을 rebuild 하여 즉시 적용한다. | When the conflict check passes, the system **shall** update UserSettings.keyboard.bindings, rebuild the RootView keymap, and apply immediately. |
| REQ-V13-034 | If-Unwanted | 충돌 검사 실패 (같은 shortcut 이 다른 action 에 binding 되어 있음) 시, 시스템은 sub-dialog 안에 inline error 를 표시하고 변경을 reject 한다. | If the conflict check fails (shortcut already bound to another action), then the system **shall** display an inline error and reject the change. |
| REQ-V13-035 | Event-Driven | 사용자가 sub-dialog 의 Cancel 버튼 또는 Esc 를 누르면, 시스템은 sub-dialog 를 dismount 하며 변경을 적용하지 않는다. | When the user presses Cancel or Esc in the sub-dialog, the system **shall** dismount the sub-dialog without applying changes. |
| REQ-V13-036 | Unwanted | 시스템은 single-stroke keybinding 만 허용한다. multi-stroke (`Cmd+K Cmd+S`) 는 N3 로 v0.1.0 단계 차단. | The system **shall** accept single-stroke keybindings only; multi-stroke combinations are out of scope (N3). |

### RG-V13-5 — 4 sub-panes (Editor/Terminal/Agent/Advanced, skeleton + 1 setting)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-V13-040 | Ubiquitous | EditorPane 은 section title "Editor" + description + tab_size NumericInput (range 2~8, default 4) 1개 control 을 표시한다. | The EditorPane **shall** display section title "Editor", description, and a tab_size NumericInput (range 2~8, default 4). |
| REQ-V13-041 | Ubiquitous | TerminalPane 은 section title "Terminal" + description + scrollback_lines NumericInput (range 1000~100000, default 10000) 1개 control 을 표시한다. | The TerminalPane **shall** display section title "Terminal", description, and a scrollback_lines NumericInput (range 1000~100000, default 10000). |
| REQ-V13-042 | Ubiquitous | AgentPane 은 section title "Agent" + description + auto_approve Toggle (default false) 1개 control 을 표시한다. | The AgentPane **shall** display section title "Agent", description, and an auto_approve Toggle (default false). |
| REQ-V13-043 | Ubiquitous | AdvancedPane 은 section title "Advanced" + description + experimental_flags placeholder (read-only Vec<String> 표시, default 빈 list) 를 표시한다. | The AdvancedPane **shall** display section title "Advanced", description, and an experimental_flags placeholder (read-only Vec<String>, default empty). |
| REQ-V13-044 | Event-Driven | 4 sub-panes 의 control 변경 시, 시스템은 UserSettings 의 해당 field 를 update 하고 영속화 debounce 를 트리거한다. | When a control in any sub-pane changes, the system **shall** update the corresponding UserSettings field and trigger the persistence debounce. |
| REQ-V13-045 | Unwanted | 4 sub-panes 의 setting 은 UI 표시 + UserSettings 영속화까지만 한정한다. consumer 모듈 (viewer / terminal / agent) 의 동작 변경은 v0.1.0 단계 비목표 (N1). | The system **shall not** wire sub-pane settings to consumer modules (viewer/terminal/agent) in v0.1.0; UI and persistence only (N1). |

### RG-V13-6 — UserSettings 영속화 (JSON / atomic write / fail-soft load)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-V13-050 | Ubiquitous | UserSettings struct 는 schema_version `moai-studio/settings-v1` 을 포함하며 serde 로 JSON 직렬화된다. | The UserSettings struct **shall** include schema_version `moai-studio/settings-v1` and serialize to JSON via serde. |
| REQ-V13-051 | Ubiquitous | UserSettings 영속화 경로는 `dirs::config_dir()/moai-studio/settings.json` 이다. dirs::config_dir() 가 None 반환 시 fallback 으로 `std::env::temp_dir()/moai-studio/settings.json` 사용 + warn 로그. | The settings file path **shall** be `dirs::config_dir()/moai-studio/settings.json`; if config_dir() returns None, fallback to `temp_dir()/moai-studio/settings.json` with a warn log. |
| REQ-V13-052 | Event-Driven | 사용자가 setting 을 변경한 후 200 ms 가 경과하면 (debounce), 시스템은 UserSettings 를 atomic write (tempfile + rename) 로 settings.json 에 저장한다. | 200 ms after a setting change (debounce), the system **shall** atomically write UserSettings to settings.json (tempfile + rename). |
| REQ-V13-053 | Event-Driven | 사용자가 SettingsModal 을 dismiss 하면, 시스템은 debounce 대기를 cancel 하고 dirty 상태일 때 즉시 flush 한다. | When the user dismisses the SettingsModal, the system **shall** cancel the debounce and flush immediately if dirty. |
| REQ-V13-054 | Event-Driven | 앱 시작 시, 시스템은 settings.json 을 읽어 UserSettings 를 deserialize 하고 ActiveTheme global + RootView keymap 을 초기화한다. | On app startup, the system **shall** read settings.json, deserialize into UserSettings, and initialize the ActiveTheme global and RootView keymap. |
| REQ-V13-055 | If-Unwanted | settings.json 이 손상되어 deserialize 가 실패하면, 시스템은 파일을 `settings.json.bak.{utc_timestamp}` 로 rename 백업하고 Default::default() 를 반환하며 tracing::warn! 로그를 남긴다. | If settings.json is corrupted and deserialize fails, then the system **shall** rename it to `settings.json.bak.{utc_timestamp}`, return Default::default(), and log a tracing::warn!. |
| REQ-V13-056 | If-Unwanted | settings.json 의 schema_version 이 `moai-studio/settings-v1` 과 다르면, 시스템은 v0.1.0 단계에서 마이그레이션 함수를 호출한다 (현재 미구현 — 동일 경로로 백업 + Default 반환). | If schema_version mismatch occurs, then the system **shall** invoke the migration function (in v0.1.0, identical to corruption path: backup + Default). |
| REQ-V13-057 | Unwanted | 시스템은 settings.json 의 부분 쓰기 (truncated file) 산출을 만들지 않는다 — atomic write (tempfile + rename) 로 전체 또는 nothing 보장. | The system **shall not** produce a partially-written settings.json; atomic write (tempfile + rename) ensures all-or-nothing. |

### RG-V13-7 — design::runtime::ActiveTheme + 런타임 적용

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-V13-060 | Ubiquitous | 시스템은 신규 모듈 `crates/moai-studio-ui/src/design/runtime.rs` 에 ActiveTheme struct + ThemeMode/AccentColor/Density enum 을 정의한다. | The system **shall** define ActiveTheme struct + ThemeMode/AccentColor/Density enums in the new module `crates/moai-studio-ui/src/design/runtime.rs`. |
| REQ-V13-061 | Ubiquitous | RootView 는 mount 시 UserSettings 로부터 ActiveTheme 을 derive 하여 cx 의 global state 로 set 한다 (`cx.set_global(active_theme)`). | RootView **shall**, on mount, derive ActiveTheme from UserSettings and set it as cx global state via cx.set_global(active_theme). |
| REQ-V13-062 | Event-Driven | UserSettings.appearance 가 변경되면, 시스템은 ActiveTheme global 을 update 하고 RootView 에 cx.notify() 를 발생시켜 전체 re-render 한다. | When UserSettings.appearance changes, the system **shall** update the ActiveTheme global and emit cx.notify() on RootView for full re-render. |
| REQ-V13-063 | Ubiquitous | ActiveTheme 은 다음 method 를 노출한다: `bg_app() -> u32`, `bg_panel() -> u32`, `fg_primary() -> u32`, `accent_color() -> u32`, `font_size_px() -> f32`, `spacing_multiplier() -> f32`. | The ActiveTheme **shall** expose methods: bg_app(), bg_panel(), fg_primary(), accent_color(), font_size_px(), spacing_multiplier(). |
| REQ-V13-064 | Ubiquitous | UserSettings.keyboard.bindings 가 변경되면, 시스템은 RootView 의 keymap 을 rebuild 하여 즉시 적용 (재기동 불필요) 한다. | When UserSettings.keyboard.bindings changes, the system **shall** rebuild the RootView keymap and apply immediately (no restart required). |
| REQ-V13-065 | Unwanted | 시스템은 design::tokens 의 const 정의를 변경하지 않는다 — ActiveTheme 은 const 값을 dispatch 하는 wrapper 일 뿐. | The system **shall not** modify const definitions in design::tokens; ActiveTheme only dispatches existing const values. |
| REQ-V13-066 | State-Driven | 일부 컴포넌트가 design::tokens 의 const 를 직접 참조하는 동안 (마이그레이션 미완), 시스템은 그 컴포넌트가 dark theme 색상으로 잔존함을 수용한다 (R-V13-3 — v0.1.0 단계 partial unthemed). | While some components still directly reference design::tokens consts (migration incomplete), the system **shall** accept that those components remain dark-themed (R-V13-3 — v0.1.0 partial unthemed). |

---

## 6. Acceptance Criteria

| AC ID | 검증 시나리오 | 통과 조건 | 검증 수단 | RG 매핑 |
|------|--------------|----------|----------|---------|
| AC-V13-1 | SettingsModal 이 Cmd+, 단축키로 mount | RootView 의 settings_modal 필드가 Some(Entity), 컨테이너 size = 880×640, backdrop scrim 존재 | unit (mock keypress) + render assertion | RG-V13-1 |
| AC-V13-2 | Sidebar 가 6 section row 를 표시하며 default 선택은 Appearance | sidebar children count = 6, selected_section = Appearance, AppearancePane 이 main pane 에 mount | unit (Entity inspect) | RG-V13-2 |
| AC-V13-3 | sidebar 의 다른 section 클릭 시 main pane swap | KeyboardPane row 클릭 → selected_section = Keyboard, main pane child = KeyboardPane Entity | unit (event simulation) | RG-V13-2 |
| AC-V13-4 | AppearancePane 의 theme RadioGroup 변경 시 ActiveTheme update | "light" 클릭 → ActiveTheme.theme = ThemeMode::Light, RootView cx.notify() 발생 | unit (state assertion) | RG-V13-3, RG-V13-7 |
| AC-V13-5 | accent ColorSwatch 클릭 시 ActiveTheme.accent 변경 | "violet" 클릭 → ActiveTheme.accent = AccentColor::Violet, accent_color() = 0x6a4cc7 | unit (method call assertion) | RG-V13-3, RG-V13-7 |
| AC-V13-6 | font_size Slider 12~18 range 강제 | 외부 set 시도 (11 또는 19) → UI reject, ActiveTheme.font_size_px 변경 안됨 | unit (boundary test) | RG-V13-3 |
| AC-V13-7 | KeyboardPane 의 binding 테이블이 default + custom 모든 binding 표시 | 테이블 row count = 모든 default binding count + 모든 custom binding count | unit (Entity inspect) | RG-V13-4 |
| AC-V13-8 | binding edit dialog 의 충돌 검사 — pass / fail 양 케이스 | 새 shortcut 이 unused → save 성공, 기존 사용 중 → inline error + reject | unit (conflict_check 함수 + dialog state) | RG-V13-4 |
| AC-V13-9 | 4 sub-panes (Editor/Terminal/Agent/Advanced) 가 각 1개 control 을 렌더 | EditorPane: tab_size NumericInput / TerminalPane: scrollback NumericInput / AgentPane: auto_approve Toggle / AdvancedPane: experimental_flags placeholder | unit (각 PaneEntity render) | RG-V13-5 |
| AC-V13-10 | UserSettings 변경 → 200 ms debounce → atomic write | 변경 후 250 ms wait → settings.json 존재 + 변경 내용 반영 + tempfile 잔존 없음 | integration (tokio test + tempdir + 파일 확인) | RG-V13-6 |
| AC-V13-11 | 앱 재기동 시 settings.json 자동 load 및 변경 setting 복원 | 변경 → save → 모달 close → fresh load_or_default(path) 호출 → ActiveTheme 이 변경 상태와 일치 | integration (full lifecycle test) | RG-V13-6, RG-V13-7 |
| AC-V13-12 | settings.json 손상 시 .bak.{ts} 백업 + Default 반환 + warn 로그 | 손상 JSON 파일 미리 작성 → load_or_default 호출 → settings.json 존재 안함 + .bak.{ts} 존재 + 반환값 = Default + tracing::warn 발생 | unit (tempdir + tracing-subscriber capture) | RG-V13-6 |

---

## 7. 비기능 요구사항

| 항목 | 요구 |
|------|------|
| 모달 mount latency | Cmd+, 입력 → 모달 visible 까지 16 ms 이내 (60 fps frame budget) |
| theme switch latency | RadioGroup click → 전체 re-render 완료까지 16 ms 이내 (single frame) |
| font_size reflow latency | Slider drag 1 px → 다음 frame 에 반영 (16 ms 이내) |
| settings.json save latency | debounce 200 ms 후 disk write 완료 30 ms 이내 |
| settings.json load latency | 앱 시작 시 50 ms 이내 |
| settings.json 크기 | < 4 KB (default), < 16 KB (full custom binding) |
| 단위 테스트 coverage | 85%+ (UserSettings serde + SettingsViewState 변경 + KeyboardPane conflict_check + load_or_default fail-soft) |
| Local 5 quality gates | cargo test PASS / clippy 0 warning / fmt PASS / bench 회귀 없음 / cargo check --release PASS |
| Rust toolchain | workspace `rust-toolchain` (현행 1.92+) |
| GPUI version | 0.1 (workspace pinned) |
| code_comments 언어 | `ko` (`.moai/config/sections/language.yaml`) |
| OS support | macOS 14+ / Ubuntu 22.04+ / Windows 10/11 (모두 dirs crate 정합) |

---

## 8. 의존성 / 통합 인터페이스

### 8.1 선행 SPEC

- **SPEC-V3-001 (Foundation)**: workspace 구조, design module 기존.
- **SPEC-V3-002 (Panes)**: pane container 패턴 (SettingsModal 의 main pane swap 패턴 carry).
- **SPEC-V3-003 (Tabs)**: TabContainer 영속화 패턴 (persistence.rs reference).
- **SPEC-V3-004 (Render)**: RootView 의 cx.notify() / theme bootstrap 인프라.

### 8.2 병행 가능 SPEC

- 없음. 본 SPEC 은 RootView + design module 을 변경하므로 그 모듈을 동시 변경하는 다른 SPEC 과 병행 시 머지 충돌 위험.

### 8.3 외부 의존 (신규)

`crates/moai-studio-ui/Cargo.toml` 에 추가:

```toml
[dependencies]
dirs = "5"             # config_dir() — cross-platform settings 위치
serde_json = { workspace = true }   # 이미 보유
tempfile = { workspace = true }     # 이미 워크스페이스에 사용 (verify in plan.md)
chrono = { workspace = true }       # 백업 timestamp (verify in plan.md)
serde = { workspace = true, features = ["derive"] }   # 이미 보유
thiserror = { workspace = true }    # 이미 보유
tracing = { workspace = true }      # 이미 보유
```

verify (plan.md 의 MS-1 task): `Cargo.toml` workspace common 에서 `tempfile`, `chrono` 가 이미 가용한지 확인. 미가용 시 추가.

### 8.4 외부 차단 — 없음

본 SPEC 은 외부 차단 없이 진입 가능 (research §10).

---

## 9. 마일스톤 (priority-based, 시간 추정 없음)

### MS-1 (Priority: High) — SettingsModal Entity + AppearancePane

산출:
- `crates/moai-studio-ui/src/settings/mod.rs` — re-export
- `crates/moai-studio-ui/src/settings/settings_modal.rs` — SettingsModal Entity (880×640 컨테이너 + sidebar + main pane swap)
- `crates/moai-studio-ui/src/settings/settings_state.rs` — SettingsViewState (selected_section enum + 4 PaneEntity slots)
- `crates/moai-studio-ui/src/settings/panes/mod.rs`
- `crates/moai-studio-ui/src/settings/panes/appearance.rs` — AppearancePane (4 controls: theme RadioGroup, density ToggleGroup, accent ColorSwatch, font_size Slider). 변경은 in-memory state 만 (영속화는 MS-3).
- `crates/moai-studio-ui/src/lib.rs` — Cmd+, keybinding 등록 + RootView.settings_modal: Option<Entity<SettingsModal>>
- 단위 테스트:
  - SettingsModal mount/dismount 상태 전이 (Esc / scrim click / X 버튼)
  - sidebar 6 row 렌더 + selected_section 변경 시 main pane swap
  - AppearancePane 의 4 control state 변경 (theme/density/accent/font_size)
  - font_size 12~18 range 강제 (11 / 19 reject)
- AC-V13-1, AC-V13-2, AC-V13-3, AC-V13-4 (in-memory only — ActiveTheme global 미구현 단계), AC-V13-5 (in-memory), AC-V13-6 통과
- ActiveTheme global / 영속화는 MS-3 단계.

### MS-2 (Priority: High) — KeyboardPane + 4 sub-panes (skeleton)

산출:
- `crates/moai-studio-ui/src/settings/panes/keyboard.rs` — KeyboardPane (binding 테이블 + edit sub-dialog + conflict_check)
- `crates/moai-studio-ui/src/settings/panes/editor.rs` — EditorPane (skeleton + tab_size NumericInput)
- `crates/moai-studio-ui/src/settings/panes/terminal.rs` — TerminalPane (skeleton + scrollback_lines NumericInput)
- `crates/moai-studio-ui/src/settings/panes/agent.rs` — AgentPane (skeleton + auto_approve Toggle)
- `crates/moai-studio-ui/src/settings/panes/advanced.rs` — AdvancedPane (skeleton + experimental_flags placeholder)
- 단위 테스트:
  - KeyboardPane: binding 테이블 렌더 (default count) + edit dialog mount/dismount + conflict_check pass/fail
  - 4 sub-panes 각 control 의 in-memory state 변경
- AC-V13-7, AC-V13-8, AC-V13-9 통과
- 영속화는 MS-3 단계.

### MS-3 (Priority: High) — UserSettings 영속화 + design::runtime::ActiveTheme + 적용

산출:
- `crates/moai-studio-ui/src/settings/user_settings.rs`
  - UserSettings struct + AppearanceSettings + KeyboardSettings + EditorSettings + TerminalSettings + AgentSettings + AdvancedSettings
  - serde Serialize/Deserialize + schema_version `moai-studio/settings-v1`
  - load_or_default(path: &Path) -> UserSettings — fail-soft + .bak.{ts} 백업
  - save_atomic(path: &Path, settings: &UserSettings) -> Result<(), PersistError> — tempfile + rename
- `crates/moai-studio-ui/src/design/runtime.rs`
  - ActiveTheme struct + ThemeMode/AccentColor/Density enum
  - bg_app() / bg_panel() / fg_primary() / accent_color() / font_size_px() / spacing_multiplier() method
- `crates/moai-studio-ui/src/lib.rs`
  - 앱 시작 시 load_or_default 호출 → ActiveTheme global set + RootView keymap rebuild
  - SettingsModal 변경 emit → UserSettings update → 200ms debounce → save_atomic → ActiveTheme global update → cx.notify()
  - SettingsModal dismiss 시 dirty flush
- 통합 테스트:
  - tempdir 내 settings.json save → 재read → 동일 struct 복원
  - 손상 JSON → .bak.{ts} 백업 + Default 반환 + warn 로그 capture
  - debounce 200ms 검증 (tokio test + Timer)
  - dismiss 시 즉시 flush 검증
  - 변경 → ActiveTheme global update → cx.notify() 트리거 검증
- AC-V13-10, AC-V13-11, AC-V13-12 통과
- partial unthemed 잔존 수용 (R-V13-3, REQ-V13-066 — 모든 컴포넌트의 const → cx.global() 마이그레이션은 후속 task).

---

## 10. 위험 (Risk Register)

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R-V13-1 | dirs::config_dir() None 반환 (rare CI / sandbox env) | 영속화 무효 | REQ-V13-051 의 fallback (temp_dir + warn) |
| R-V13-2 | tempfile rename 실패 (cross-device) | save 실패 | persistence.rs 패턴 carry — Result 반환 + UI error toast |
| R-V13-3 | theme runtime switch 시 일부 컴포넌트가 const 직접 참조 | 부분 unthemed UI | REQ-V13-066 로 명시 수용. 후속 SPEC 으로 점진 마이그레이션 |
| R-V13-4 | font_size 변경 시 전체 layout reflow 비용 | lag 위험 | bench (criterion) + 16ms budget 검증. cx.refresh() 일괄 호출. |
| R-V13-5 | keybinding 충돌 검출 누락 | shortcut 회수 시 다른 action 잠재 작동 | KeyboardPane 의 conflict_check 단위 테스트 + default binding 일괄 conflict scan |
| R-V13-6 | UserSettings schema 향후 변경 | 사용자 file backward-compat 깨짐 | schema_version 검증 + migration hook (v0.1.0 미구현 — backup + Default) |
| R-V13-7 | 200ms debounce 동안 앱 crash → 변경 손실 | 사용자 짜증 | 손실 0~200ms 수용. dismiss 시 flush 추가. |
| R-V13-8 | settings.json 손상 (외부 수동 편집) | 앱 시작 실패 | REQ-V13-055 fail-soft (.bak + Default + warn) |
| R-V13-9 | accent change 시 syntax highlight 색상과 충돌 | 가독성 저하 | accent 영역은 ide_accent 만. syntax 색상 격리 (current 도) |
| R-V13-10 | Sidebar/main pane 의 width hard-code (200/680) → 880 contain 시 sub-pixel | 작은 시각 잘림 | layout::SETTINGS_MODAL_* 상수로 빼고 design::layout.rs 에 추가 |
| R-V13-11 | tempfile crate 가 workspace common 에 미가용 | 의존 누락 | MS-1 첫 task 로 verify. 미가용 시 추가. |
| R-V13-12 | dirs v5 vs 다른 crate 호환 | 빌드 fail | workspace common 추가 시 cargo tree 검증 |

---

## 11. 외부 인터페이스 (불변 약속)

본 SPEC 은 다음 인터페이스를 fix 한다. 후속 SPEC 이 본 SPEC 의 산출물을 consume 할 때 신뢰할 수 있다:

```rust
// crates/moai-studio-ui/src/settings/user_settings.rs

pub const SCHEMA_VERSION: &str = "moai-studio/settings-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSettings {
    pub schema_version: String,
    pub appearance: AppearanceSettings,
    pub keyboard: KeyboardSettings,
    pub editor: EditorSettings,
    pub terminal: TerminalSettings,
    pub agent: AgentSettings,
    pub advanced: AdvancedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceSettings {
    pub theme: ThemeMode,           // Dark / Light / System
    pub density: Density,           // Compact / Comfortable
    pub accent: AccentColor,        // Teal / Blue / Violet / Cyan
    pub font_size_px: u8,           // 12~18
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyboardSettings {
    pub bindings: Vec<KeyBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyBinding {
    pub action: String,             // e.g., "command_palette"
    pub shortcut: String,            // e.g., "cmd-shift-p"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorSettings { pub tab_size: u8 }                        // 2~8

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalSettings { pub scrollback_lines: u32 }              // 1000~100000

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSettings { pub auto_approve: bool }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedSettings { pub experimental_flags: Vec<String> }

impl Default for UserSettings { /* all sections at sensible defaults */ }

pub fn load_or_default(path: &Path) -> UserSettings;
pub fn save_atomic(path: &Path, settings: &UserSettings) -> Result<(), PersistError>;
pub fn settings_path() -> PathBuf;   // dirs::config_dir() + fallback
```

```rust
// crates/moai-studio-ui/src/design/runtime.rs

pub struct ActiveTheme {
    pub theme: ThemeMode,
    pub accent: AccentColor,
    pub density: Density,
    pub font_size_px: f32,
}

impl ActiveTheme {
    pub fn from_settings(s: &AppearanceSettings) -> Self;
    pub fn bg_app(&self) -> u32;
    pub fn bg_panel(&self) -> u32;
    pub fn fg_primary(&self) -> u32;
    pub fn accent_color(&self) -> u32;
    pub fn font_size_px(&self) -> f32;
    pub fn spacing_multiplier(&self) -> f32;
}
```

후속 SPEC 이 변경할 수 없는 부분: SCHEMA_VERSION 문자열, UserSettings top-level 필드명 (appearance/keyboard/editor/terminal/agent/advanced), ThemeMode/AccentColor/Density enum variant 이름. 신규 setting 추가는 가능 (semver minor — 새 field default 값 + serde default attribute).

---

## 12. 추적성

### 12.1 IMPLEMENTATION-NOTES.md ↔ 본 SPEC

| IMPLEMENTATION-NOTES.md 섹션 | 본 SPEC 매핑 |
|------------------------------|---------------|
| §13.7 Settings/Preferences Modal — 6 sections | RG-V13-3, RG-V13-4, RG-V13-5 |
| §13.7 — Action: 신규 `crates/moai-studio-ui/src/settings/` 모듈 | §9 MS-1 산출 (settings/ 디렉터리 신설) |
| §13.7 — Action: UserSettings struct + SettingsModal Entity | §9 MS-3 산출 (UserSettings) + MS-1 산출 (SettingsModal) |
| §13.9 Backdrop Scrim — modal/palette 공용 | RG-V13-1 의 backdrop scrim (별 SPEC 가 공용 Scrim Entity 신설하기 전까지는 본 SPEC 내 inline) |
| §14 D 항목 (P0) | 본 SPEC 의 priority: High 정합 |

### 12.2 Round 2 시안 ↔ 본 SPEC

| moai-revisions.jsx 컴포넌트 | 본 SPEC 매핑 |
|----------------------------|---------------|
| SettingsModal | settings_modal.rs (RG-V13-1) |
| AppearancePane | panes/appearance.rs (RG-V13-3) |
| KeyboardPane | panes/keyboard.rs (RG-V13-4) |

GPUI 시각 구현 (color/typography/spacing) 은 시안 정합 + design::tokens 의 const 값 사용.

### 12.3 design::tokens v2.0.0 carry

본 SPEC 은 design::tokens 의 다음 const 값을 직접 활용 (변경 없음):

- `theme::dark::background::PANEL` (#0e1513) — modal panel 배경
- `theme::dark::background::ELEVATED` (#182320) — sidebar 배경
- `theme::dark::border::DEFAULT_APPROX` — section row 테두리
- `theme::dark::accent::SOFT_APPROX` — 선택된 section row 배경
- `ide_accent::TEAL/BLUE/VIOLET/CYAN` — accent ColorSwatch 4 옵션
- `mx_tag::*` — KeyboardPane 의 액션 카테고리 표시 (ANCHOR/WARN/NOTE/TODO 색상 재사용 가능, optional)

---

## 13. 용어 정의

| 용어 | 정의 |
|------|------|
| SettingsModal | 880×640 px modal 컨테이너. 사용자 환경설정 진입점. backdrop scrim 위 mount. |
| sidebar | SettingsModal 좌측 200 px. 6 section row list. |
| main pane | SettingsModal 우측 680 px. 선택된 section 의 PaneEntity 가 mount 되는 영역. |
| AppearancePane | 6 sections 중 첫 번째. theme/density/accent/font_size 4 controls 보유. |
| KeyboardPane | 6 sections 중 두 번째. binding 테이블 + edit dialog 보유. |
| ActiveTheme | runtime mutable theme state. UserSettings.appearance 로부터 derive. cx.global() 로 접근. |
| UserSettings | 사용자 환경설정 root struct. JSON 직렬화. settings.json 영속화. |
| SettingsViewState | SettingsModal 의 transient UI state (selected_section, sub-dialog open 상태 등). 영속화 안 함. |
| atomic write | tempfile 에 쓰고 → final path 로 rename. 부분 쓰기 시 기존 파일 보존. |
| fail-soft load | 손상된 파일 시 panic 하지 않고 backup + Default 반환. |
| schema_version | UserSettings JSON 의 top-level 필드. v0.1.0 = `moai-studio/settings-v1`. 향후 마이그레이션 trigger. |
| debounce 200ms | 사용자 변경 후 200ms 무변경 시 save 트리거. 연속 변경 시 disk I/O 최소화. |
| backdrop scrim | modal 뒤 darken/blur 레이어. 현재는 본 SPEC 내 inline (별 SPEC 의 공용 Scrim Entity 가 신설되면 그쪽으로 이관). |

---

## 14. 변경 이력 정책

본 spec.md 는 추가 revision 누적 시 `## 15. Sprint Contract Revisions` section 을 신설하고 `### 15.1 / 15.2 / ...` 로 누적한다 (SPEC-V3-003 / SPEC-V3-011 §15.x 패턴 따름). RG-V13-* 의 self-application — 본 SPEC 자신이 향후 사용자 환경설정 변경 검증 fixture 가 된다 (settings.json 의 schema_version 진화 시 backward-compat 검증).

---

작성 종료. 본 spec.md 는 research.md (배경 분석) 와 함께 SPEC-V3-013 implement 진입의 입력이다. implement 는 별도 feature 브랜치 (`feature/SPEC-V3-013-settings`) 에서 시작한다. plan.md 는 implement 진입 시점에 생성한다 (CLAUDE.local.md §6.1 carry — feature 브랜치에서 RUN phase 진입 시 manager-ddd/tdd 가 plan.md 를 작성).
