# SPEC-V3-013 Research — Settings Surface

작성: MoAI (manager-spec, 2026-04-26)
브랜치: `develop` @ 9f90188 (PR #21 merged)
선행: SPEC-V3-001 ~ V3-010 (foundation/panes/tabs/render/explorer/markdown/terminal/spec-mgmt/agent-dashboard 모두 사용)
범위: moai-studio 사용자 환경설정 모달 (Appearance/Keyboard/Editor/Terminal/Agent/Advanced 6 sections) + UserSettings 영속화 + 런타임 적용 (theme/accent/density/font_size).

---

## 1. 동기

### 1.1 사용자 가치

moai-studio 는 다음 사용자 환경설정 요구를 누적해 왔다:

- 다크/라이트/system theme — 현재는 dark 기본 고정. 사용자 선호 반영 불가.
- accent color — tokens.json `color.ide_accent` 에 4종 정의 (teal/blue/violet/cyan). 런타임 선택 불가.
- density — compact / comfortable. 패널 padding/row height 조정. 사용자별 시각 선호.
- font size — Pretendard 9-weight 시스템 + 사용자가 12~18 px 범위 조정.
- keyboard binding — Cmd+P/Cmd+Shift+P 등 default 만 존재. 충돌 시 사용자 재정의 불가.
- 영속화 — 위 모든 선택이 재기동 시 복원 필요.

본 SPEC 이 완료되면:

- 사용자가 Cmd+, (또는 메뉴) 로 SettingsModal 을 열어 위 6 sections 의 각 setting 을 변경한다.
- 변경은 즉시 visual 적용 (theme/accent/density/font_size) 또는 다음 세션부터 적용 (keybinding) 된다.
- 변경된 설정이 platform-appropriate 위치 (`~/.config/moai-studio/settings.json` 또는 macOS `~/Library/Application Support/moai-studio/settings.json` 등 dirs crate 결정) 에 영속화된다.
- 재기동 시 자동 복원.

### 1.2 IMPLEMENTATION-NOTES.md §13.7 carry

`.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md` v1.1 §13.7 가 Settings/Preferences Modal 의 6 sections 을 명시했다 (D 항목 P0):

| Section | 내용 |
|---------|------|
| Appearance | theme (dark/light/system) / density / accent (teal/blue/violet/cyan) / font size |
| Keyboard | binding 테이블 + edit |
| Editor | tab size / word wrap / minimap / format on save |
| Terminal | shell / font / scrollback / opacity |
| Agent | model / cost limit / auto-approve / hook config |
| Advanced | LSP servers / tree-sitter languages / experimental flags |

본 SPEC 의 v0.1.0 단계는 **Appearance + Keyboard 가 full implementation**, 나머지 4 sections 은 **skeleton + 최소 1개 setting 각** 으로 한정한다 (over-engineering 회피, MS 분할).

### 1.3 Round 2 시안 정합

`.moai/design/from-claude-design/project/moai-revisions.jsx` 가 `SettingsModal` / `AppearancePane` / `KeyboardPane` 컴포넌트 prototype 을 포함. 본 SPEC 의 GPUI 구현은 시각적 정렬을 그 시안에 맞춘다 (880×640, 200px sidebar, 680px main pane).

---

## 2. 코드베이스 분석 — 현재 가용한 building block

### 2.1 워크스페이스 구조

`crates/moai-studio-ui/src/`:
- `lib.rs` (36 KB) — RootView, theme bootstrap, keybinding registration.
- `design/{tokens,layout,typography}.rs` — design 모듈 (이미 존재). `tokens.rs` 의 `theme::dark` / `theme::light` / `ide_accent` 가 그대로 활용 가능.
- `panes/`, `tabs/`, `terminal/`, `viewer/`, `explorer/`, `agent/` — 모두 themed.

`crates/moai-studio-workspace/src/persistence.rs` (14 KB) — pane layout JSON 영속화의 reference 구현. 본 SPEC 의 UserSettings 영속화는 동일 패턴 차용:
- tempfile → rename 의 atomic write
- 손상된 JSON 안전 실패 (panic 금지, Default 반환 + warn 로그)
- 스키마 버전 검증 (`moai-studio/settings-v1`)
- thiserror `PersistError` enum 동일 구조

### 2.2 신규 모듈 위치

`crates/moai-studio-ui/src/settings/` — 신규 디렉터리. 본 SPEC 단독 소유.

```
settings/
├── mod.rs                          # re-export (SettingsModal, UserSettings, ...)
├── settings_modal.rs               # SettingsModal Entity (880×640 컨테이너)
├── user_settings.rs                # UserSettings struct + serde + 영속화
├── settings_state.rs               # SettingsViewState (현재 선택 section 등 transient state)
└── panes/
    ├── mod.rs
    ├── appearance.rs               # AppearancePane (full impl)
    ├── keyboard.rs                 # KeyboardPane (full impl)
    ├── editor.rs                   # EditorPane (skeleton + 1 setting)
    ├── terminal.rs                 # TerminalPane (skeleton + 1 setting)
    ├── agent.rs                    # AgentPane (skeleton + 1 setting)
    └── advanced.rs                 # AdvancedPane (skeleton + 1 setting)
```

### 2.3 통합 지점 — RootView

`crates/moai-studio-ui/src/lib.rs` 의 RootView:
- 신규 keybinding `cmd-,` (macOS) / `ctrl-,` (Linux/Windows) → SettingsModal mount.
- RootView 가 SettingsModal Entity 를 보유 (Option<Entity<SettingsModal>>) — mount 시 Some, dismiss 시 None.
- backdrop scrim — IMPLEMENTATION-NOTES.md §13.9 의 공용 Scrim Entity 활용 (별 SPEC 또는 본 SPEC 내 inline).
- theme/accent 변경 시 RootView 의 `cx.notify()` 로 전체 re-render. design::tokens runtime override 메커니즘은 본 SPEC 이 신설.

---

## 3. 영속화 전략 검토

### 3.1 위치 — dirs crate 활용

`dirs::config_dir()` 반환값:
- macOS: `~/Library/Application Support/`
- Linux: `~/.config/`
- Windows: `%APPDATA%\Roaming\`

전체 경로: `{config_dir}/moai-studio/settings.json`

대안 검토:
- (a) `dirs::config_dir()` (권장) — XDG Base Directory 표준 + macOS Apple HIG 정합. cross-platform.
- (b) `~/.moai-studio/settings.json` — 단순. 그러나 dotfile 위치는 modern OS 의 권장 location 이 아니다.
- (c) workspace-local — `.moai/local-settings.json`. global setting 과 부적합 (사용자 단위가 아닌 workspace 단위).

→ (a) 채택. dirs v5 (workspace 호환) 추가 의존.

### 3.2 schema — JSON + serde

```json
{
  "schema_version": "moai-studio/settings-v1",
  "appearance": {
    "theme": "dark" | "light" | "system",
    "density": "compact" | "comfortable",
    "accent": "teal" | "blue" | "violet" | "cyan",
    "font_size_px": 14
  },
  "keyboard": {
    "bindings": [
      {"action": "command_palette", "shortcut": "cmd-shift-p"},
      {"action": "settings", "shortcut": "cmd-,"},
      ...
    ]
  },
  "editor": { "tab_size": 4 },
  "terminal": { "scrollback_lines": 10000 },
  "agent": { "auto_approve": false },
  "advanced": { "experimental_flags": [] }
}
```

`schema_version` 으로 향후 마이그레이션 지원 (persistence.rs 의 `SCHEMA_VERSION` 패턴 carry).

### 3.3 save 정책 — debounce 200ms

매 키스트로크/슬라이더 frame 마다 저장하면 disk I/O 폭증. tokio task spawn + debounce 200ms:

```rust
// 개념 — UserSettings 변경 emit 시
self.dirty = true;
let cx = cx.clone();
cx.spawn(|this, mut cx| async move {
    Timer::after(Duration::from_millis(200)).await;
    this.update(&mut cx, |this, cx| {
        if this.dirty {
            this.save_to_disk(cx);
            this.dirty = false;
        }
    });
});
```

대안 검토:
- (a) debounce 200ms (권장) — 일반적 IDE pattern (VS Code 100~300ms). I/O minimal.
- (b) 매 변경 즉시 — disk I/O 과다.
- (c) 모달 dismiss 시에만 — 사용자가 의도하지 않은 dismiss (panic / crash) 시 손실 위험.

→ (a) 채택.

### 3.4 load 정책 — startup + fail-soft

`UserSettings::load_or_default(path: &Path) -> UserSettings`:
- 파일 없음 → `Default::default()` 반환 (warn 없음 — first run normal).
- 파일 손상 (JSON parse fail) → backup `.bak.{timestamp}` 으로 rename → `Default::default()` 반환 + `tracing::warn!`.
- schema_version 불일치 → 마이그레이션 함수 호출 (v0.1.0 단계 없음, 향후 추가).

persistence.rs 의 `load_panes_or_default` 와 동일 정책.

### 3.5 atomic write

tempfile → rename — persistence.rs 패턴 그대로:

```rust
fn save_atomic(path: &Path, settings: &UserSettings) -> Result<(), PersistError> {
    let tmp = NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))?;
    serde_json::to_writer_pretty(&tmp, settings)?;
    tmp.persist(path)?;
    Ok(())
}
```

부분 쓰기 시 rename 미완 → 기존 파일 보존. 안전.

---

## 4. 적용 (apply) 메커니즘

### 4.1 theme/accent — design::tokens runtime override

현재 `design/tokens.rs` 는 모든 색상이 `pub const` (compile-time). runtime 변경 불가. 본 SPEC 이 추가:

```rust
// design/runtime.rs (신규)

pub struct ActiveTheme {
    pub theme: ThemeMode,           // dark / light / system
    pub accent: AccentColor,         // teal / blue / violet / cyan
    pub density: Density,            // compact / comfortable
    pub font_size_px: f32,
}

impl ActiveTheme {
    pub fn bg_app(&self) -> u32 { /* theme-resolved */ }
    pub fn bg_panel(&self) -> u32 { /* ... */ }
    pub fn accent_color(&self) -> u32 { /* enum match */ }
    // ... 기존 tokens flat-alias 의 instance method 버전
}
```

- RootView 가 `Entity<ActiveTheme>` 보유 + `cx.global::<ActiveTheme>()` provide.
- 모든 themed 컴포넌트가 `cx.global::<ActiveTheme>()` 로 색상 조회 (현재의 const 직접 참조 → method call 로 점진 마이그레이션, MS-3 단계).
- v0.1.0 단계 — Appearance change → ActiveTheme update → cx.notify() → 전체 re-render. 점진 마이그레이션 시 임시로 dark 만 유지하다 light 까지 확장.

### 4.2 density — layout module 의 spacing override

`design/layout.rs` 가 spacing 상수 보유. ActiveTheme.density 가 multiplier 결정 (compact = 0.85x, comfortable = 1.0x). 동일 globals 패턴.

### 4.3 font_size — typography module override

`design/typography.rs` 가 base 14px. ActiveTheme.font_size_px override.

### 4.4 keybinding — RootView 의 keymap rebuild

`UserSettings.keyboard.bindings` 변경 → RootView keymap rebuild → cx.notify().
- 변경 즉시 적용 (재기동 불필요).
- 충돌 검출 — 같은 shortcut 이 2 actions 에 binding 시 변경 reject + UI error toast.

### 4.5 나머지 4 sections 의 1개 setting 각

v0.1.0 단계 minimal:
- Editor.tab_size — `crates/moai-studio-ui/src/viewer/` 의 indent guides 가 consume.
- Terminal.scrollback_lines — `crates/moai-studio-terminal/` (또는 ui/terminal) 가 consume.
- Agent.auto_approve — `crates/moai-studio-ui/src/agent/` 의 approve flow 가 consume.
- Advanced.experimental_flags — `Vec<String>`, runtime 동작 없음 (v0.1.0 placeholder).

각 setting 의 consumer 모듈 변경은 본 SPEC 이 아닌 후속 SPEC. v0.1.0 단계는 **저장 + UI 반영** 까지만.

---

## 5. UX 패턴 — Settings Modal 시안

### 5.1 모달 컨테이너

- 880×640 px.
- backdrop scrim (rgba(8,12,11,0.55) dark / rgba(20,30,28,0.18) light).
- 중앙 정렬 (RootView 의 absolute inset 0 + flex center).
- z-index 30 (palette 보다 위).
- dismiss — Esc, scrim click, X 버튼.

### 5.2 sidebar (200px)

- background: theme.surface.
- 각 section row — 36px 높이, icon + label.
- 선택된 section 하이라이트 (accent.soft background + accent.base text).

### 5.3 main pane (680px)

- background: theme.panel.
- 상단 24px section title + 12px description.
- 각 setting row — 48px 높이 (label + control + hint).
- scroll — section 내용이 길면 vertical scroll.

### 5.4 controls

| Control | 사용 |
|---------|------|
| RadioGroup | theme (3 options) |
| ToggleGroup | density (2 options) |
| ColorSwatch | accent (4 options, 8x8px circle) |
| Slider | font_size (12~18px, step 1) |
| TableRow | keybinding (action / shortcut / edit button) |
| Toggle | agent.auto_approve, future Editor settings |
| TextInput | scrollback_lines (numeric) |

### 5.5 keybinding edit dialog

- 사용자가 row 의 "Edit" 버튼 클릭 → 작은 sub-dialog (400×200) mount.
- "새 단축키를 입력하세요" prompt + 키 캡처.
- Cancel / Save 버튼.
- 충돌 시 inline error.

---

## 6. 의존성 추가

`crates/moai-studio-ui/Cargo.toml`:

```toml
dirs = "5"             # config_dir()
serde_json = "1"       # workspace common (이미 보유)
tempfile = "3"         # atomic write (이미 워크스페이스 사용 가능성 — verify)
chrono = "0.4"         # backup timestamp (workspace common 가능)
```

verify: `Cargo.toml` workspace common 에서 `tempfile`, `chrono` 가 이미 가용한지 확인 — research 단계 미확인. plan 단계에서 명시.

---

## 7. 위험 분석

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R-V13-1 | dirs::config_dir() 가 None 반환 (rare CI / sandbox env) | 영속화 무효 | fallback to `std::env::temp_dir() + "moai-studio/settings.json"` + warn 로그 |
| R-V13-2 | tempfile rename 실패 (cross-device) | save 실패 | persistence.rs 와 동일 — Result 반환 + UI error toast |
| R-V13-3 | theme runtime switch 시 일부 컴포넌트가 const 직접 참조 (현행) → 변경 미적용 | 부분 unthemed UI | MS-3 마이그레이션 task — `cx.global::<ActiveTheme>()` 점진 적용. v0.1.0 단계는 이 위험을 수용 (비동기 마이그레이션). |
| R-V13-4 | font_size 변경 시 layout reflow 비용 | 큰 codebase 시 lag | font_size apply 시 GPUI 의 `cx.refresh()` 일괄 호출. metric 검증 (bench) — 16ms budget 내. |
| R-V13-5 | keybinding 충돌 검출 누락 | shortcut 회수 시 다른 action 이 잠재 작동 | KeyboardPane 의 valid_check 함수 단위 테스트 + 모든 default binding 의 일괄 conflict scan |
| R-V13-6 | UserSettings schema 향후 변경 | 사용자 file backward-compat 깨짐 | schema_version 검증 + 향후 마이그레이션 함수 hook |
| R-V13-7 | debounce 200ms 동안 앱 crash → 변경 손실 | 사용자 짜증 | crash 시 손실 0~200ms 범위 — 수용. 모달 dismiss 시 즉시 flush 추가 |
| R-V13-8 | settings.json 손상 (외부 수동 편집) | 앱 시작 실패 | fail-soft — backup .bak + Default 반환 + warn (R-V13 패턴) |
| R-V13-9 | accent change 시 syntax highlight color 와 충돌 | 가독성 저하 | accent 영역은 ide_accent (4 options) 만 변경. syntax 색상은 별 token 으로 격리 (현재도) |
| R-V13-10 | Sidebar / Main pane 의 width hard-code (200/680) → 실제 880 contain 시 sub-pixel | 작은 시각 잘림 | layout::SETTINGS_MODAL_* 상수로 빼고 design::layout.rs 에 추가 |

---

## 8. AC 후보 사전 정의

(spec.md 의 AC table 작성 시 차용)

- AC-V13-1: SettingsModal 이 Cmd+, 단축키 또는 메뉴에서 mount 되고 880×640 으로 렌더된다.
- AC-V13-2: Sidebar 가 6 section row 를 표시하며 Appearance 가 default 선택이다.
- AC-V13-3: 사용자가 sidebar 의 다른 section 클릭 시 main pane 이 해당 PaneEntity 로 swap.
- AC-V13-4: AppearancePane 의 theme RadioGroup 변경 시 ActiveTheme 가 update + RootView 가 cx.notify() 호출.
- AC-V13-5: AppearancePane 의 accent ColorSwatch 클릭 시 ActiveTheme.accent 변경 + cx.notify().
- AC-V13-6: AppearancePane 의 font_size Slider 변경 시 (12~18 range) ActiveTheme.font_size_px 변경.
- AC-V13-7: KeyboardPane 의 binding 테이블이 default + custom 모든 binding 을 표시.
- AC-V13-8: 사용자가 binding 을 edit → 새 shortcut 입력 → 충돌 없을 시 save, 충돌 시 inline error.
- AC-V13-9: Editor/Terminal/Agent/Advanced 4 panes 가 skeleton + 최소 1개 control 을 표시.
- AC-V13-10: UserSettings 변경 시 200ms debounce 후 settings.json 에 atomic write.
- AC-V13-11: 앱 재기동 시 settings.json 이 자동 load 되며 변경된 setting 이 복원된다.
- AC-V13-12: settings.json 손상 시 .bak.{timestamp} 백업 후 Default 시작 + warn 로그.

---

## 9. USER-DECISION 게이트 (없음)

본 SPEC 은 IMPLEMENTATION-NOTES.md §13.7 + Round 2 시안이 이미 결정한 6 sections + 적용 범위로 진입. USER-DECISION 게이트 없음.

만약 v0.2.0+ 단계에 다음을 도입한다면 별 SPEC 의 USER-DECISION 으로:
- crash recovery toast (Banner SPEC)
- multi-window settings sync (별 SPEC)
- cloud sync (별 SPEC)

---

## 10. 외부 차단 — 없음

본 SPEC 은 외부 의존 없이 진입 가능:
- design::tokens 이미 보유.
- workspace persistence 패턴 reference 보유.
- Round 2 시안 보유.
- dirs crate 추가만 필요 (workspace 호환).

---

## 11. 후속 SPEC 후보 (v0.2.0+)

- v0.2.0 SPEC: 4 sub-panes (Editor/Terminal/Agent/Advanced) full implementation (현행 skeleton + 1 setting → 5~10 settings 각).
- v0.2.0 SPEC: cloud sync (Apple iCloud / GitHub gist) — UserSettings serialize → remote.
- v0.3.0 SPEC: theme custom (사용자 정의 색 — `~/.config/moai-studio/themes/*.json`).
- v0.3.0 SPEC: keybinding 의 multi-stroke (`Ctrl+K Ctrl+S` like VS Code).

---

## 12. 결론

본 research 의 결정:

1. 신규 모듈 위치: `crates/moai-studio-ui/src/settings/` (mod.rs, settings_modal.rs, user_settings.rs, panes/{appearance,keyboard,editor,terminal,agent,advanced}.rs).
2. 영속화 위치: `dirs::config_dir() + "moai-studio/settings.json"` (cross-platform).
3. 영속화 패턴: persistence.rs 의 atomic write + fail-soft load + schema_version `moai-studio/settings-v1`.
4. 적용 메커니즘: design::runtime::ActiveTheme global state + cx.notify() re-render.
5. v0.1.0 범위: Appearance + Keyboard 가 full, 4 sections 은 skeleton + 1 setting 각.
6. 의존: dirs v5 추가, tempfile/chrono 워크스페이스 검증.
7. 마이그레이션 위험: R-V13-3 (theme runtime switch 시 const 직접 참조 컴포넌트) — MS-3 점진 적용으로 수용.

본 research 의 산출은 spec.md 의 EARS RG / AC / MS 의 입력이다.

---

작성 종료. 본 research 는 SPEC-V3-013 plan 의 입력이며, MS-1 / MS-2 / MS-3 분할은 spec.md §9 에서 정의한다.
