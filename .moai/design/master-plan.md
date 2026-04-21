# MoAI Studio Master Plan (2026-04-21, v3.2 Final)

**9 가지 핵심 결정 통합**:
1. 아키텍처: **GPUI + libghostty-vt** (Zed 스택, 순수 네이티브 Rust, WebView 없음)
2. 플랫폼: **macOS / Linux / Windows** (Windows 는 GPUI GA 대기 후 동시 출시)
3. UX 품질: macOS 수준 네이티브 UI/UX 를 3 플랫폼 모두에 제공
4. 브랜드: **"MoAI Studio"** 유지 + moai-adk 번들 플러그인
5. 디자인: Markdown source of truth + Claude Design (Pencil 선택적)
6. VT 라이브러리: **libghostty-vt alpha 즉시 사용 + pinned commit**
7. 플러그인 런타임: **인프로세스 Rust 정적 플러그인** (feature flag)
8. MVP 스코프: **v0.1 전체 범위** (Tier A+B+C+D + moai-adk)
9. 빌드 툴체인: cargo + **Zig 0.15.x** (libghostty 빌드 요구)

---

## 1. 제품 정체성

**MoAI Studio v3 = cross-platform native terminal multiplexer (Rust+GPUI) + optional moai-adk plugin**

두 유저 세그먼트:
- **범용 유저**: cmux/Wave/Tabby 대체 (moai-adk 비활성)
- **moai-adk 유저**: 위 + SPEC/Plan/Run/Sync + TRUST 5 + @MX + Hook + Mission Control

기술 차별화:
- **Tauri/Electron 아님** — WebView 없이 순수 네이티브 GPU
- **libghostty-vt 기반** — Ghostty 와 동일 battle-tested VT 파서
- **GPUI 기반** — Zed 와 동일 Rust GPU UI 프레임워크
- **Rust-only** — WebView/JS 없음

---

## 2. 기술 스택

```
MoAI Studio v0.1
│
├── Language
│   ├── Rust (stable, MSRV 1.82+)
│   └── Zig (0.15.x, libghostty 빌드 전용)
│
├── UI Framework
│   └── GPUI (Zed 엔진, @workspace dependency)
│       ├── macOS: Metal (120Hz ProMotion)
│       ├── Linux: Vulkan/OpenGL
│       └── Windows: WIP → GA 대기 (2026-Q3~Q4 추정)
│
├── Terminal
│   ├── libghostty-vt (VT state/parser, pinned commit)
│   │   └── via libghostty-rs FFI bindings
│   ├── portable-pty (PTY spawn, cross-platform)
│   └── custom render layer (GPUI + libghostty RenderState API)
│
├── Persistence
│   ├── rusqlite (기존 moai-core 재사용)
│   └── directories (~/.moai/studio/workspaces.json)
│
├── Async / IPC
│   ├── tokio
│   ├── Unix domain socket (macOS/Linux)
│   └── Named pipe (Windows)
│
├── Hook Server
│   └── axum + WebSocket (기존 moai-hook-http 재사용/확장)
│
├── Logging
│   └── tracing + tracing-subscriber
│
└── Plugins (인프로세스 Rust)
    ├── moai-adk (번들, cargo feature "moai-adk-plugin")
    ├── markdown-viewer
    ├── code-viewer (tree-sitter + LSP broker)
    ├── browser (WebView crate or wry)
    ├── image-viewer
    └── json-csv-viewer
```

**레퍼런스 구현**:
- [gpui-ghostty](https://xuanwo.io/2026/01-gpui-ghostty/) — GPUI + libghostty-vt 통합 선례
- [Zed Terminal](https://github.com/zed-industries/zed) — GPUI 기반 터미널
- [taskers](https://github.com/search?q=taskers+libghostty) — agent-first terminal workspace
- [ghostling_rs](https://github.com/Uzaaft/libghostty-rs) — libghostty-rs 레퍼런스

---

## 3. 디렉토리 구조 (Rust workspace)

```
moai-studio/
├── Cargo.toml                    # Rust workspace
├── crates/
│   ├── moai-studio-app/          # 메인 바이너리 (GPUI 엔트리)
│   ├── moai-studio-ui/            # GPUI 컴포넌트 (Sidebar, Toolbar, StatusBar, Panes)
│   ├── moai-studio-terminal/      # libghostty-vt + render 레이어
│   ├── moai-studio-surfaces/      # Terminal/Markdown/Code/Browser/Image/JSON/Mermaid/FileTree
│   ├── moai-studio-workspace/     # Multi-project workspace state
│   ├── moai-studio-plugin-api/    # Plugin trait 정의
│   ├── moai-studio-plugin-moai-adk/  # moai-adk 번들 플러그인
│   ├── moai-studio-smart-links/   # OSC 8 + regex 링크 파서
│   ├── moai-studio-socket/        # Unix socket/named pipe IPC
│   └── moai-core/                 # 기존 moai-core 재사용 (289 tests)
│
├── .moai/                        # SPEC, design, project docs
├── .claude/                      # Claude Code config
├── .github/workflows/            # CI matrix
├── archive/swift-legacy/         # 기존 Swift 코드 reference 보존
├── installer/
│   ├── macos/                    # .dmg + codesign + notarize
│   ├── linux/                    # .AppImage / .deb / .rpm
│   └── windows/                  # .msi (GPUI GA 후)
└── docs/                         # 사용자 문서
```

---

## 4. 8-Phase 로드맵 (GPUI 기반)

### Phase 0 — 준비

- 기존 Swift 코드 `archive/swift-legacy/` 이동
- `moai-core` crates 를 `crates/moai-core` 로 이동 (289 tests 검증)
- Cargo workspace 재구성
- GPUI + libghostty-rs 실험 스파이크
- Zig 0.15.x CI 툴체인 검증 (macOS+Linux)

**Exit**: Rust workspace 컴파일, GPUI 샘플 윈도우, libghostty 샘플 터미널

### Phase 1 — 스캐폴드 + 디자인 시스템 (SPEC-V3-001)

- `moai-studio-app` 바이너리: GPUI 윈도우 + 메뉴 바 스켈레톤
- `moai-studio-ui`: 디자인 시스템 기반 컴포넌트 라이브러리
- `moai-studio-workspace`: 첫 워크스페이스 + SQLite persistence
- Empty State CTA
- macOS+Linux CI 성공

**Exit**: 빈 GPUI 윈도우 + 사이드바 + 메뉴 바 + Empty State

### Phase 2 — Terminal Core (Tier A, SPEC-V3-002)

- libghostty-vt 통합 (pinned commit + Zig 체인)
- portable-pty + shell 실행 (zsh/bash/fish/pwsh/nu/cmd)
- tmux 호환성 (mouse, bracketed paste, alt screen, 256+24bit color)
- GPUI Terminal 컴포넌트 + 멀티 pane + 탭 UI
- Unix socket + named pipe 서버 기초

**Exit**: 3 플랫폼에서 tmux 세션 detach/reattach 정상

### Phase 3 — Smart Link Handling (Tier B, SPEC-V3-003)

- OSC 8 + regex (파일/URL)
- Custom parsers: SPEC-ID, @MX, Mermaid
- Link action dispatch
- Hover preview popup

**Exit**: `src/foo.rs:42` 클릭 → Code Viewer

### Phase 4 — Surfaces (Tier C, SPEC-V3-005)

- Markdown: pulldown-cmark + KaTeX + Mermaid
- Code Viewer: tree-sitter + LSP broker (moai-lsp 우선)
- Browser: wry / WKWebView
- Image, JSON/CSV, Mermaid, File Tree

**Exit**: 모든 Surface pane 에 로드 가능

### Phase 5 — Multi-Project Workspace + Navigation (Tier D+F, SPEC-V3-004+006)

- Workspaces JSON + 사이드바 스위처 + 전환 persistence
- 글로벌 검색
- Command Palette (⌘/Ctrl+K) nested + @/# mention
- 네이티브 메뉴/툴바/상태 바

**Exit**: 3 프로젝트 + 전환 + 글로벌 검색 + Palette

### Phase 6 — Plugin System + moai-adk Plugin (SPEC-V3-013)

- Plugin API trait (Surface / Command / LinkParser / Sidebar / StatusBar)
- 인프로세스 static plugin loader (cargo feature)
- moai-adk plugin:
  - Link parsers: SPEC-ID, @MX
  - Surfaces: SPEC card, TRUST 5, Agent Run Viewer, Kanban, Memory
  - Hook listener (27 이벤트)
- Onboarding 분기

**Exit**: moai-adk toggle + 양쪽 상태 안정

### Phase 7 — Windows 포팅 + E2E (GPUI Windows GA 대기 후)

- GPUI Windows 활성화
- ConPTY 검증 (portable-pty 이미 지원)
- Named pipe 검증
- Windows CI 추가
- Windows E2E 시나리오

**Exit**: Windows 에서 macOS/Linux 등가 기능 + 10 E2E 통과

### Phase 8 — Polish + Distribution

- 3 플랫폼 서명 + 배포 (DMG/MSI/AppImage + deb/rpm)
- 자동 업데이트 (Sparkle-style 또는 cargo-dist)
- Landing + README + Demo video
- Plugin marketplace stub

**Exit**: Clean VM → 5분 내 첫 SPEC

---

## 5. 의존성 그래프

```
Phase 0 (준비)
  │
  └► Phase 1 (스캐폴드)
       │
       └► Phase 2 (Terminal Core) ─┐
                                    ├► Phase 3 (Smart Links)
                                    ├► Phase 4 (Surfaces) ──┐
                                    │                        ▼
                                    └► Phase 5 (Workspace+Nav)
                                              │
                                              ▼
                                    Phase 6 (Plugin + moai-adk)
                                              │
                                              ▼
                                    Phase 7 (Windows)  ← GPUI Windows GA
                                              │
                                              ▼
                                    Phase 8 (Polish/Dist)
```

병렬 가능: Phase 3 + 4 + 5 (Phase 2 이후)

---

## 6. 위험 등록부

| ID | 리스크 | 영향 | 완화 |
|----|--------|------|------|
| R-1 | GPUI Windows GA 지연 | 고 | macOS+Linux 먼저, Zed 동향 모니터링 |
| R-2 | libghostty-vt alpha API 변경 | 중 | pinned commit + 얇은 wrapper 레이어 |
| R-3 | Zig 툴체인 CI 복잡도 | 중 | GitHub Actions Zig 설치 표준화 |
| R-4 | GPUI 문서/사용자 부족 | 중 | Zed 코드 참조, gpui-ghostty 선례 |
| R-5 | WebView-less plugin UI 제약 | 중 | Plugin API = GPUI view factory |
| R-6 | Monaco 없음 → Code Viewer 재작성 | 중 | tree-sitter + LSP 로 시작, 점진 강화 |
| R-7 | 기존 Swift 투자 손실 | 저 | archive/swift-legacy/ reference 보존 |
| R-8 | v0.1 풀 스코프 일정 | 고 | Alpha/Beta 사전 출시 옵션 |
| R-9 | GPUI 라이선스 | 저 | Zed = Apache/GPLv3 dual, GPUI 별도 확인 |

---

## 7. 성공 기준 (DoD v3.2)

- ✅ 3 플랫폼 네이티브 빌드
- ✅ GPUI + libghostty-vt 기반 60+ FPS (macOS 120Hz ProMotion)
- ✅ 25 기능 전체 + moai-adk 토글
- ✅ tmux + 6 쉘 정상
- ✅ OSC 8 + custom parsers
- ✅ Multi-project workspace
- ✅ Rust core 289+ tests 유지
- ✅ E2E 30+ × 3 플랫폼
- ✅ 서명 (Developer ID/Authenticode/AppImage)
- ✅ 자동 업데이트
- ✅ 배포 산출물
- ✅ 문서 + Demo

---

## 8. 즉시 행동 (Phase 0)

1. spec.md 최종 확정 (이 master-plan 의 구현 상세화)
2. SPEC-V3-001 작성 → Phase 0+1 범위 EARS
3. Swift 코드 archive 이동: `app/` → `archive/swift-legacy/`
4. Cargo workspace 재구성: `core/crates/` → `crates/`
5. GPUI + libghostty 스파이크
6. CI matrix 업데이트: macOS + Linux

---

버전: 3.2.0 · 2026-04-21 · 최종 9 결정 반영
