# moai-cli 설계 문서 (v3)

> **정체성**: moai-adk 의 공식 macOS 네이티브 IDE-쉘. Claude Code 를 호스트가 아닌 **SDK 라이브러리**로 임베드하여, moai-adk 의 멀티 에이전트 워크플로우를 한 화면에서 검수·조작한다.
>
> **작성일**: 2026-04-11 (v3 — v2 의 아키텍처 가정 오류 전면 수정 + Claude Code 소스코드 정적 분석 기반 재설계)
>
> **근거 자료** (동일 디렉토리):
> - `research/R1-native-ai-shells.md` — 경쟁사 리서치 (Warp/Wave/cmux/Zed/Ghostty)
> - `research/B1-bridge-direct-connect.md` — Claude Code Bridge/Remote/Direct Connect 레이어 분석
> - `research/B2-hook-events-tool-system.md` — 27 hook 이벤트 + tool permission 시스템 분석
> - `research/B3-extension-points.md` — Plugin/Skill/MCP/OutputStyle 확장 포인트 분석
>
> **이전 버전**: `DESIGN.md` (v2, 2026-04-11 오전) 는 참고용으로 유지. v3 는 v2 를 완전 대체.

---

## 0. v3 의 3가지 핵심 피벗

v2 와 비교해 3가지 근본적 전환이 있다. 각 피벗은 **Claude Code 소스코드 직접 분석**에서 도출되었다.

### 피벗 1: "호스트" 가 아니라 "임베드" 다

v2 는 Claude Code 를 PTY 로 spawn → 출력 파싱 → shell wrapper hook 으로 이벤트 수집하는 구조였다. 이것은 **외부 프로그램을 검사하는 관찰자** 모델이다.

v3 는 Claude Code 를 **SDK 라이브러리**로 임베드한다. moai-cli 는 SDK host 역할을 맡아서:
- Claude Code 를 `--output-format stream-json --input-format stream-json` 모드로 spawn
- `initialize` control request 로 27개 hook callback 을 **타입 안전하게** 등록
- `hook_callback` control request 로 이벤트를 **in-process Swift 함수**로 직접 수신
- tool 호출, permission 결정, context 주입을 **런타임에 rewrite**

근거: `src/cli/structuredIO.ts:661-689` (`createHookCallback`), `src/entrypoints/sdk/controlSchemas.ts:57-75` (initialize with hooks), `src/types/hooks.ts:50-166` (hookSpecificOutput union).

**결과**: `.claude/hooks/moai/handle-*.sh` 27개 shell wrapper + `moai hook <event>` Go subcommand **완전 제거 가능**. Bash subprocess spawn overhead (~10-40ms × 수천 회/일) → in-process Swift callback (<1ms).

### 피벗 2: Rust core 를 버리고 Pure Swift 로 간다

v2 는 `moai-core` Rust crate 12개 (`moai-session`, `moai-pty`, `moai-ipc`, `moai-store`, `moai-git`, `moai-fs`, `moai-lsp`, `moai-codeview`, `moai-kanban`, `moai-events`, `moai-adk-bridge`, `moai-cli`) + swift-bridge FFI + cargo xcframework 빌드 매트릭스를 가정했다.

이 구조의 전제는 "PTY 오너십", "커스텀 IPC", "크로스 플랫폼 코어" 였다. 모두 사라졌다:
- **PTY 오너십** → Claude Code SDK 가 stdin/stdout 파이프로 처리. moai-cli 는 PTY 를 소유하지 않는다. Terminal surface 만이 Ghostty PTY 를 사용하고, 그것도 `GhosttyKit.xcframework` 가 직접 Swift 와 대화한다 ([Mitchell Hashimoto's blog](https://mitchellh.com/writing/zig-and-swiftui)).
- **커스텀 IPC** → Claude Code SDK control protocol 이 이미 정의되어 있다 (`src/entrypoints/sdk/controlSchemas.ts`). 우리는 이걸 클라이언트로 구현할 뿐.
- **크로스 플랫폼 코어** → macOS 영구 단독 확정 (Round 3). Linux 옵션은 protocol-oriented SwiftUI 로 처리 (M8+ optional).

**Swift 네이티브 대체제 검증 완료:**

| Rust 원안 | Swift 대체제 | 성숙도 |
|---|---|---|
| `tree-sitter` + `swift-bridge` | **SwiftTreeSitter** | Production (CodeEdit, Tuist 사용) |
| `rusqlite` + `r2d2` | **GRDB.swift** | Production (Mastodon 앱들) |
| `git2` (libgit2) | **SwiftGit2** 또는 `git` CLI shell-out | SwiftGit2 안정, shell-out 간단 |
| `notify` (FSEvents) | **FSEventStream** (macOS 네이티브) | Apple SDK |
| `jsonrpsee` + `hyper` | **URLSession** + WebSocket | Apple SDK, async/await |
| `tower-lsp` 클라이언트 | **SwiftLSPClient** 또는 Claude Code hook events 재사용 | SwiftLSPClient 안정 |
| `tokio` runtime | **Swift structured concurrency** (async/await, actor) | Swift 5.5+ 네이티브 |
| `portable-pty` | **GhosttyKit.xcframework** 직접 호출 | Production (cmux) |
| `serde_json` | **Codable** | Apple SDK |

**결과:**
- Xcode 프로젝트 1개만 존재 (cargo workspace + xcframework 빌드 과정 소멸)
- M0 일정 1-2주 단축
- 빌드 매트릭스 단순화 (arm64 only, Rosetta x86_64 fallback 만)
- 디버깅 단일 언어 (Xcode 디버거 전체 커버)
- FFI 경계 0 (swift-bridge 유지보수 부담 0)

### 피벗 3: "Hybrid" 배포 — Plugin + Native Shell 한 저장소

v2 는 moai-cli 를 단일 macOS 앱으로 상정했다. v3 는 **같은 저장소에서 2가지 배포 아티팩트를 동시 생산**한다:

1. **moai-cli-plugin** (Claude Code Plugin 형식)
   - `.claude-plugin/plugin.json` manifest
   - 무료 namespace: `/moai-cli:*` 슬래시 커맨드, `moai-cli:*` 에이전트 이름
   - `type: 'sdk'` in-process MCP 서버 (UI 제어 tools 노출)
   - Hook 등록, skill 번들, output style, userConfig prompt
   - 설치 경로: `~/.claude/plugins/moai-cli@local/` + `~/.claude/settings.json` 의 `enabledPlugins`
   - **Shell 없이도 동작**: 외부에서 실행된 Claude Code 세션에서도 moai-cli 의 capability 를 사용할 수 있다
   - 근거: `src/utils/plugins/schemas.ts:884-898`, `src/services/mcp/types.ts:108-113` (sdk transport)

2. **moai-cli.app** (SwiftUI + AppKit macOS GUI)
   - 8 Surface 시스템 (Terminal, CodeViewer, Markdown, Image, Browser, FileTree, AgentRun, Kanban, **Memory ★v3 신규**)
   - Plugin 을 자동 설치/등록
   - 자기가 spawn 한 Claude Code 서브프로세스에 **IDE lockfile** (`~/.claude/ide/<port>.lock`) 을 drop → 외부 Claude 세션도 moai-cli 의 MCP 서버에 자동 연결
   - 근거: `src/utils/ide.ts:298-393`

**Plugin 은 capability 제공자, Shell 은 UI 소유자.** 두 아티팩트가 같은 Git 저장소 (`modu-ai/moai-adk` 모노레포 — Round 1) 안에서 공존한다.

---

## 1. v2 → v3 변경 요약

본 섹션은 형님이 v2 를 이미 읽었다는 전제에서 **무엇이 왜 바뀌었는지**만 기록한다. 상세 근거는 `research/` 폴더 참조.

### 1.1 사실 오류 정정

| v2 항목 | v2 의 잘못 | v3 의 진실 | 근거 |
|---|---|---|---|
| `.moai/hooks.yaml` | 존재 가정 | **존재하지 않음**. Hook 등록 위치는 `.claude/settings.json` 의 `hooks` 섹션 | 실제 settings.json 확인 |
| "Hook 4종: command/prompt/agent/http. 설정: `.moai/hooks.yaml`" | 맞음 (타입 4종) + 틀림 (yaml 위치) | 타입은 맞음. moai-adk 는 **command 만** 사용 중. 설정 위치는 `.claude/settings.json` | `src/schemas/hooks.ts:176-189` |
| "Bridge" | IDE 통합 프로토콜이라 추정 | **claude.ai Remote Control relay**. `api.anthropic.com` 통과. OAuth 구독 필요. moai-cli 는 사용 불가 | `src/bridge/trustedDevice.ts`, `src/bridge/bridgeApi.ts` |
| Hook 이벤트 = 단순 알림 | 알림 수집만 | Hook 은 **tool 제어 엔진**. `hookSpecificOutput` 으로 input rewrite, output rewrite, permission rule persist 가능 | `src/types/hooks.ts:50-166` |
| PTY 파싱 + http sink + jsonl tail (3중 경로) | 이 중 한두 개를 주 채널로 | **Claude Code SDK control protocol** 이 3가지를 모두 대체 | `src/cli/structuredIO.ts`, `src/entrypoints/sdk/controlSchemas.ts` |
| cmux 는 "계승할 베이스" | 참고용 | **정면 경쟁자**. 우리 설계의 80% 를 먼저 구현함 (GPL-3.0, 13k stars) | `research/R1-native-ai-shells.md` §3 |
| Rust `moai-core` | 필수 | **불필요**. Pure Swift 로 대체 가능 (위 표) | 피벗 2 참조 |
| 사용자 파일 `.moai/hooks.yaml` 패치 동의 플로우 (결정 #3) | 필요 | **불필요**. SDK callback 은 사용자 파일 0 수정 | `src/cli/structuredIO.ts:661-689` |
| SSH 원격 = cmux daemon/remote 포팅 | M7 신규 작업 | **불필요**. Claude Code 에 `RemoteSessionManager` 내장 | `src/remote/RemoteSessionManager.ts`, `src/remote/SessionsWebSocket.ts` |

### 1.2 소크라테스 인터뷰 확정 사항 (Round 1~4)

v2 작성 후 진행된 인터뷰에서 확정된 결정들 (v3 에 반영):

| # | 결정 | 확정값 |
|---|---|---|
| 1 | 포지셔닝 | moai-adk 공식 macOS GUI. 모노레포 (`modu-ai/moai-adk` 내부 `moai-cli/` 서브디렉토리, 독립 버전 라인). Onboarding 에서 moai-adk 자동 설치 제안. |
| 2 | 라이선스 | **MIT** (moai-adk 도 MIT 로 전환 예정) |
| 3 | OS 로드맵 | macOS 영구 단독. M8+ Linux 옵션만 (protocol-oriented SwiftUI 로 포팅 가능 구조 유지) |
| 4 | Ghostty 수용 | **수용**. M0 첫 관문 = `GhosttyKit.xcframework` 빌드 + 단일 PTY 표시 |
| 5 | Pane Splitter | **NSSplitView 자체 구현** (Bonsplit/SwiftUI HSplitView 거부). Binary tree + NSCoder 직렬화 + 자체 drag handle. 1-2주 M1 초반 흡수. |

### 1.3 아키텍처 피벗 (세 가지 핵심 — §0 참조)

1. Claude Code = embed SDK, not PTY-observed subprocess
2. Rust core 제거, Pure Swift
3. Hybrid: Plugin + Native Shell 동시 배포

### 1.4 신규 기능 (v3 추가)

- **Memory Surface** (§6.9) — `~/.claude/projects/<git-root>/memory/` 의 markdown 파일 렌더링
- **Instructions Loaded Graph** (§6.10) — `InstructionsLoaded` hook 구독으로 세션 컨텍스트 디버거
- **Bash Input Rewriter** (§5.4) — `PreToolUse.updatedInput` 으로 모든 Bash 명령을 안전 wrapper 에 감싸기
- **Native Permission Dialog** (§5.5) — `PermissionRequest` callback + SwiftUI 모달
- **In-process MCP UI Tools** (§5.6) — Claude 가 직접 moai-cli UI 를 조작 (`Kanban.moveCard`, `Surface.reveal`, `Notification.post`, `Terminal.spawn` …)
- **IDE Lockfile Impersonation** (§5.7) — `~/.claude/ide/<port>.lock` 로 외부 Claude 세션 auto-attach

### 1.5 삭제된 항목

- Rust `moai-core` 전체 12 crate
- `swift-bridge` FFI
- `cargo xcframework` 빌드 파이프라인
- `.moai/hooks.yaml` 자동 패치 로직
- 사용자 프로젝트 파일 수정 동의 플로우
- Bonsplit fork 옵션
- Windows 로드맵 (영구)
- SSH 원격 포팅 (Claude Code 내장 재사용)

---

## 2. 제품 정의

### 2.1 한 줄 정의

> **moai-cli**: macOS 네이티브 IDE-쉘. Claude Code 를 SDK 라이브러리로 임베드하여, moai-adk 의 26개 전문 에이전트 + 27개 hook 이벤트 + TRUST 5 품질 게이트 + @MX 태그 시스템을 **한 화면에서 검수·조작**한다.

### 2.2 차별화 한 줄

> cmux 는 터미널 멀티화 + 브라우저 surface 를 잘 한다. moai-cli 는 그 위에 **SPEC/TRUST/@MX/Kanban 워크플로우 시각화** 와 **Claude 가 직접 UI 를 조작할 수 있는 in-process MCP 레이어** 를 올린다.

### 2.3 누가 쓰는가

| 사용자 유형 | 사용 방식 |
|---|---|
| moai-adk 일반 사용자 | `moai-cli.app` 설치 → SPEC-driven 워크플로우를 GUI 로 운용. TUI 대비 2-3x 빠른 리뷰 속도 목표 |
| moai-adk 파워 유저 (Agent Teams 활용) | 16+ 에이전트 동시 운용을 Kanban + Agent Run Viewer 로 시각화 |
| 외부 Claude Code 사용자 | `moai-cli-plugin` 만 설치. shell 없이도 `/moai-cli:*` 커맨드 + In-process MCP 도구 사용 |
| 팀 리더 | Dashboard 에서 TRUST 5 게이지, task-metrics 실시간 모니터링 |

### 2.4 핵심 요구사항

v2 의 9개 요구사항 유지 + v3 추가:

1. 파일 탐색기 (FileTree surface)
2. GPU 가속 터미널 (Ghostty)
3. Code Viewer (SwiftTreeSitter + LSP 진단 overlay + @MX 거터 + tri-pane diff)
4. 마크다운 뷰어 (EARS SPEC 특화)
5. 내장 브라우저 (WKWebView + DevTools)
6. 이미지 뷰어 (diff 모드)
7. Agent Run Viewer (Direct Connect stream + SDK hook callbacks)
8. Kanban 보드 (SPEC ↔ worktree ↔ `/moai run` 자동 연동)
9. 다중 세션 (16+ 에이전트 동시)
10. `/moai *` 14 슬래시 커맨드 GUI 1-클릭 호출
11. **★ v3 신규**: Memory Viewer (Claude Code 의 `~/.claude/projects/<root>/memory/` 렌더링)
12. **★ v3 신규**: Instructions Loaded Graph (세션 컨텍스트 디버거)
13. **★ v3 신규**: Claude 가 직접 UI 를 조작 (In-process MCP 도구)
14. **★ v3 신규**: Native Permission Dialog (TUI text prompt 대체)

### 2.5 비기능 요구사항 (v2 계승 + 조정)

| 항목 | 목표 | v2 대비 |
|---|---|---|
| 콜드 스타트 (M1 MacBook) | < 0.6s | v2 의 0.8s 에서 tighten (Rust core 제거로 실제 가능) |
| 활성 메모리 (8 PTY + 2 WebView + 4 Code surface + Claude SDK host) | < 700 MB | v2 와 동일 |
| 터미널 스크롤 | 60 fps @ 4K | v2 와 동일 (Ghostty Metal) |
| 동시 에이전트 세션 | 16+ | v2 와 동일 |
| 세션 복원 | < 2s | v2 와 동일 |
| **Hook callback latency (moai-cli host 내)** | < 2ms | v3 신규 (shell wrapper 제거 효과 측정) |
| **Direct Connect 메시지 → UI 업데이트** | < 30ms | v3 신규 |
| 크래시 격리 | 1 에이전트 크래시가 나머지에 전파되지 않음 | v2 와 동일 |

---

## 3. 경쟁 지형 (cmux 중심)

### 3.1 cmux 가 이미 한 것 (우리는 반복하지 않는다)

[manaflow-ai/cmux](https://github.com/manaflow-ai/cmux) (13k+ stars, GPL-3.0) 는 이미 다음을 완성했다:

- Swift + AppKit + libghostty 네이티브 터미널 셸 ✅
- 세션별 vertical sidebar tabs (git branch, PR, ports metadata) ✅
- `cmux claude-teams` 런처: Claude Code 의 tmux-based teammate spawning 을 cmux 네이티브 split 으로 가로채는 shim ✅ ([PR #1179](https://github.com/manaflow-ai/cmux/pull/1179))
- 브라우저 surface (agent-browser 기반, SSH 라우팅 지원) ✅
- 20+ 브라우저 import ✅
- `cmux notify` CLI + OSC 9/99/777 시퀀스로 알림 ✅
- Native macOS 성능, GPU 렌더링 ✅

**교훈:** Swift + AppKit + libghostty 조합은 검증된 기술 스택이다. 우리도 이걸 쓴다. cmux 를 기술적으로 앞지르려 하지 않는다.

### 3.2 cmux 가 못 한 것 (우리의 moat)

cmux 의 공식 README 와 issue 트래커에서 확인된 누락 영역:

| 기능 | cmux | moai-cli 계획 |
|---|---|---|
| 파일 트리 사이드바 | ❌ | ✅ (§6.6) |
| 코드 뷰어 | ❌ | ✅ (§6.3) |
| 마크다운 뷰어 | ❌ | ✅ (§6.4) |
| 이미지 뷰어 | ❌ | ✅ (§6.7) |
| **Kanban / SPEC 보드** | ❌ | ✅ (§6.8) |
| **Agent Run Viewer** (token/cost 대시보드) | ❌ | ✅ (§6.9) |
| **Memory Viewer** | ❌ | ✅ (§6.11) |
| MCP 통합 | ❌ | ✅ (§5.6, in-process + IDE lockfile) |
| **hook 이벤트 UI 노출** | ⚠️ OSC sequences 만 | ✅ (27 events, typed payloads) |
| 라이브 프로세스 복원 | ❌ (의도적 미구현) | ✅ (Claude Code 의 detached session 활용, §5.3) |
| SPEC 파일 인식 | ❌ | ✅ (EARS 특화 markdown 모드, §6.4) |
| TRUST 5 게이지 | ❌ | ✅ (Dashboard, `design-exports/03-dashboard.png`) |
| @MX 태그 거터 | ❌ | ✅ (§6.3 Code Viewer) |

### 3.3 cmux 의 공식 pain points (우리가 피해야 할 것)

cmux issue 트래커 상위 이슈 ([research/R1 §3.3](./research/R1-native-ai-shells.md#3-cmux-top-user-pain-points)):

1. **[#330 Linux support](https://github.com/manaflow-ai/cmux/issues/330)** — macOS-only 가 최대 pain. moai-cli 는 같은 선택을 하므로 같은 community pressure 예상.
2. **[#480 Session persistence](https://github.com/manaflow-ai/cmux/issues/480)** — 라이브 프로세스 상태 복원 불가. moai-cli 는 Claude Code 의 detached session 으로 해결.
3. **[#2322 Claude notification race condition](https://github.com/manaflow-ai/cmux/issues/2322)** — 이중 delivery path (shim hooks + OSC) 경합. moai-cli 는 **단일 신뢰 채널 (SDK callback)** 로 예방.
4. **탭 재정렬 UX 불만** — notification recency 로 자동 재정렬되면서 키보드 단축키 깨짐. moai-cli 는 stable tab order 고정.

### 3.4 Wave Terminal 의 "badge rollup" 아이디어 채택

Wave Terminal 이 [Claude Code hooks + `wsh badge` 조합으로 다중 pane 탭 헤더에 최고 우선순위 상태를 표시](https://docs.waveterm.dev/claude-code)하는 기법은 훌륭하다. moai-cli 도 같은 개념을 Kanban 카드 + 탭 metadata 에 적용한다 (§6.8).

### 3.5 Zed 의 "ACP 한계" 에서 배우는 교훈

Zed 는 78.9k stars 에 풀타임 팀이 있는 Rust/GPUI 프로젝트인데도 Claude Code 통합에 P2 버그 3건 동시 발생:
- [#50142 Diff/리뷰 UI 미표시](https://github.com/zed-industries/zed/issues/50142)
- [#51648 컨텍스트 200K 클램프](https://github.com/zed-industries/zed/issues/51648)
- [#50304 Agent Panel 설정 무시](https://github.com/zed-industries/zed/issues/50304)

결정타: Zed 공식 문서에 ["Agent teams are currently not supported. **Hooks are currently not supported**"](https://zed.dev/docs/ai/external-agents) 명시.

**교훈:** Hook + Agent Teams 통합은 업계 최고 팀도 못 한 일이다. **moai-cli 가 여기서 잘하면 그 자체가 moat**. 우리가 Claude Code 소스 분석으로 확보한 SDK 지식 + 모노레포 이점 (moai-adk 를 직접 수정 가능) 이 Zed 대비 독보적 포지션을 만든다.

### 3.6 차별화 포지셔닝 (한 그림)

```
+-------------------------------------------------------------+
|                    Native macOS Shell 시장                   |
|                                                             |
|  cmux:         Terminal + Browser + 1급 claude-teams         |
|  Warp:         Terminal + Cloud Agents (유료, 폐쇄 소스)      |
|  Wave:         Widgets (Electron, 메모리 비판)                |
|  Zed:          Editor + ACP (Hook 미지원, Agent Teams 불가)   |
|  Ghostty:      Terminal only (AI 없음, libghostty 원 제공자)  |
|                                                             |
|  moai-cli:     Terminal + Code + Markdown + Image + Browser +|
|                FileTree + Kanban + AgentRun + Memory +       |
|                Claude Code SDK 1급 임베드 +                   |
|                27 hook 이벤트 + 26 에이전트 + TRUST +         |
|                SPEC/@MX 1급 + Native Permission Dialog +    |
|                In-process MCP (Claude 가 UI 조작)            |
+-------------------------------------------------------------+
```

---

## 4. 아키텍처 개요

### 4.1 5단 계층 (v2 유지)

```
Window
 └── Project          ← git 루트 + .moai/ 감지. 없으면 /moai project 제안
      └── Workspace   ← 1 Claude Code SDK 세션 = 1 git worktree = 1 에이전트 호스트
           ├── agent_host: claude_code_sdk | codex (read-only) | shell | tmux_cg
           ├── binds: SPEC-{DOMAIN}-{NNN} (옵션)
           └── Pane    ← NSSplitView binary tree
                └── Surface (Terminal | CodeViewer | Markdown | Image |
                              Browser | FileTree | AgentRun | Kanban |
                              Memory | InstructionsGraph)  ← v3: 10 surfaces
```

변경점:
- **Workspace = Claude Code SDK 세션**: v2 의 PTY child 모델 대신 SDK-hosted child process
- **Surface 10개**: v2 의 8개 + Memory + InstructionsGraph

### 4.2 전체 구조 다이어그램

```
┌───────────────────────────────────────────────────────────────────┐
│                      moai-cli.app (macOS, Swift)                  │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                 SwiftUI + AppKit Shell                      │  │
│  │  Sidebar · Tabs · NSSplitView · Command Palette · Inspector │  │
│  └──────┬──────────────────────────────────────────┬──────────┘   │
│         │                                          │              │
│  ┌──────▼────────────────┐           ┌─────────────▼───────────┐  │
│  │  Surface Plugins      │           │  Core Services (Swift)  │  │
│  │  ─────────────────    │◄─────────►│  ─────────────────────  │  │
│  │  • Terminal           │           │  • ClaudeCodeSDKHost    │  │
│  │    (GhosttyKit)       │           │    (stdin/stdout +      │  │
│  │  • CodeViewer         │           │     Direct Connect)     │  │
│  │    (SwiftTreeSitter   │           │  • HookCallbackRegistry │  │
│  │     + @MX gutter)     │           │    (27 events)          │  │
│  │  • Markdown           │           │  • InProcessMCPServer   │  │
│  │    (Down + KaTeX      │           │    (UI tools)           │  │
│  │     + Mermaid)        │           │  • WorkspaceSupervisor  │  │
│  │  • Image              │           │    (OTP-style)          │  │
│  │    (Core Image)       │           │  • GRDB.swift Store     │  │
│  │  • Browser            │           │    (SQLite WAL)         │  │
│  │    (WKWebView)        │           │  • FSEventStream        │  │
│  │  • FileTree           │           │  • GitService           │  │
│  │  • AgentRun (v3 재설계) │           │    (SwiftGit2 or CLI)  │  │
│  │  • Kanban             │           │  • IDE lockfile daemon  │  │
│  │  • Memory ★ v3         │           │  • UpdateManager        │  │
│  │  • InstructionsGraph  │           │    (Sparkle)            │  │
│  │    ★ v3                │           │                         │  │
│  └───────────────────────┘           └──────────────┬──────────┘  │
│                                                     │             │
└─────────────────────────────────────────────────────┼─────────────┘
                                                      │
                ┌─────────────────────────────────────┴─────────┐
                │   Managed child processes (SDK clients)        │
                │   ─────────────────────────────────────────── │
                │   • claude (stream-json SDK mode, per WS)     │
                │   • moai (Go CLI, worktree ops)               │
                │   • git  (CLI, read-only commands)            │
                │   • codex (read-only mode, optional)          │
                │   • zsh  (for Shell workspaces)               │
                │                                               │
                │   Terminal surfaces own their own Ghostty PTY │
                │   children (not SDK-managed).                 │
                └───────────────────────────────────────────────┘
```

### 4.3 멀티 에이전트 모델 (v3 재정의)

**원칙:**

1. **1 Workspace = 1 Claude Code SDK session = 1 git worktree.** 이것이 병렬 에이전트 격리의 기본 단위.
2. **WorkspaceSupervisor 가 라이프사이클 소유.** Swift actor 패턴으로 OTP one_for_one 유사 supervision. 각 WS 는 자식 `claude` 프로세스를 async Task 로 감싼다.
3. **SDK control protocol 이 단방향 이벤트 버스.** Claude Code → stdin/stdout → Swift HookCallbackRegistry → (a) GRDB.swift append-only log, (b) UI 구독자, (c) moai-adk Go CLI (worktree 작업 시).
4. **권한 = Direct Connect bearer token + in-process MCP 경계.** PermissionRequest hook 이 Swift 네이티브 다이얼로그로 라우팅되므로 tool 권한 = UI 포커스 상태와 동기.
5. **moai-adk 는 건드리지 않고 얹는다.** moai-cli 는 `moai` Go 바이너리를 subprocess 로 호출하고, Claude Code 는 SDK 로 임베드한다. moai-adk 자체 로직은 손대지 않는다.

**v2 와의 차이:**
- v2 의 "PTY master 소유 by Rust core" → v3 의 "SDK subprocess 소유 by Swift Task". PTY 는 Terminal surface 전용.
- v2 의 "ring buffer per workspace 4MB" → v3 의 "SDK 는 자체 스트리밍 backpressure 가짐, 우리는 Swift AsyncStream 으로 소비". 별도 ring buffer 불필요.
- v2 의 "hook http sink (127.0.0.1)" → v3 의 "SDK `hook_callback` control request in-process". 포트 바인딩 없음.

---

## 5. Claude Code SDK 통합 (v3 의 핵심)

이 섹션이 v3 의 모든 것이다. 나머지 섹션은 이 토대 위에서 구축된다.

### 5.1 SDK Host 역할

moai-cli 가 Claude Code 를 SDK 모드로 spawn:

```swift
// 의사 코드 (실제 구현은 Swift Process + Pipe)
let claude = Process()
claude.executableURL = URL(fileURLWithPath: "/usr/local/bin/claude")
claude.arguments = [
  "--output-format", "stream-json",
  "--input-format", "stream-json",
]
claude.currentDirectoryURL = workspace.worktreePath
claude.environment = ProcessInfo.processInfo.environment

let stdin = Pipe()
let stdout = Pipe()
let stderr = Pipe()
claude.standardInput = stdin
claude.standardOutput = stdout
claude.standardError = stderr

try claude.run()

// SDK control protocol 개시
let host = ClaudeCodeSDKHost(stdin: stdin.fileHandleForWriting,
                             stdout: stdout.fileHandleForReading)
try await host.initialize(hooks: registerAll27HookCallbacks())
```

### 5.2 Initialize Control Request

`src/entrypoints/sdk/controlSchemas.ts:57-75` 에 정의된 `initialize` control request 스키마에 따라:

```json
{
  "type": "control_request",
  "request_id": "init-001",
  "request": {
    "subtype": "initialize",
    "hooks": {
      "PreToolUse": [
        { "matcher": "Bash", "hookCallbackIds": ["moai-cli-bash-guard"], "timeout": 5000 },
        { "matcher": "Write|Edit", "hookCallbackIds": ["moai-cli-mx-scan"], "timeout": 10000 }
      ],
      "PostToolUse": [
        { "matcher": "Write|Edit", "hookCallbackIds": ["moai-cli-post-write"] }
      ],
      "SessionStart": [
        { "matcher": "*", "hookCallbackIds": ["moai-cli-autoload"] }
      ],
      "PermissionRequest": [
        { "matcher": "*", "hookCallbackIds": ["moai-cli-native-dialog"] }
      ],
      "UserPromptSubmit": [
        { "matcher": "*", "hookCallbackIds": ["moai-cli-context-inject"] }
      ],
      "SubagentStart": [
        { "matcher": "*", "hookCallbackIds": ["moai-cli-agent-start"] }
      ],
      "SubagentStop": [
        { "matcher": "*", "hookCallbackIds": ["moai-cli-agent-stop"] }
      ],
      "TeammateIdle": [
        { "matcher": "*", "hookCallbackIds": ["moai-cli-team-idle"] }
      ],
      "TaskCreated":    [{ "matcher": "*", "hookCallbackIds": ["moai-cli-task-created"] }],
      "TaskCompleted":  [{ "matcher": "*", "hookCallbackIds": ["moai-cli-task-completed"] }],
      "FileChanged":    [{ "matcher": "*", "hookCallbackIds": ["moai-cli-file-changed"] }],
      "CwdChanged":     [{ "matcher": "*", "hookCallbackIds": ["moai-cli-cwd-changed"] }],
      "WorktreeCreate": [{ "matcher": "*", "hookCallbackIds": ["moai-cli-worktree-create"] }],
      "WorktreeRemove": [{ "matcher": "*", "hookCallbackIds": ["moai-cli-worktree-remove"] }],
      "ConfigChange":   [{ "matcher": "*", "hookCallbackIds": ["moai-cli-config-reload"] }],
      "InstructionsLoaded": [{ "matcher": "*", "hookCallbackIds": ["moai-cli-instructions"] }],
      "Notification":   [{ "matcher": "*", "hookCallbackIds": ["moai-cli-notif"] }],
      "SessionEnd":     [{ "matcher": "*", "hookCallbackIds": ["moai-cli-session-end"] }],
      "PreCompact":     [{ "matcher": "*", "hookCallbackIds": ["moai-cli-pre-compact"] }],
      "PostCompact":    [{ "matcher": "*", "hookCallbackIds": ["moai-cli-post-compact"] }],
      "Setup":          [{ "matcher": "*", "hookCallbackIds": ["moai-cli-setup"] }],
      "PermissionDenied": [{ "matcher": "*", "hookCallbackIds": ["moai-cli-perm-denied"] }],
      "PostToolUseFailure": [{ "matcher": "*", "hookCallbackIds": ["moai-cli-tool-fail"] }],
      "Stop":           [{ "matcher": "*", "hookCallbackIds": ["moai-cli-stop"] }],
      "StopFailure":    [{ "matcher": "*", "hookCallbackIds": ["moai-cli-stop-fail"] }],
      "Elicitation":    [{ "matcher": "*", "hookCallbackIds": ["moai-cli-elicit"] }],
      "ElicitationResult": [{ "matcher": "*", "hookCallbackIds": ["moai-cli-elicit-result"] }]
    }
  }
}
```

27개 모두 등록. moai-cli 는 이벤트 소비자 겸 tool 제어 엔진이 된다.

### 5.3 Hook Callback 수신

Claude Code 가 hook 을 fire 할 때 `hook_callback` control request 를 역방향으로 보낸다 (`src/cli/structuredIO.ts:661-689`):

```json
{
  "type": "control_request",
  "request_id": "hc-00042",
  "request": {
    "subtype": "hook_callback",
    "callback_id": "moai-cli-bash-guard",
    "input": {
      "hook_event_name": "PreToolUse",
      "session_id": "sess-abc",
      "transcript_path": "/Users/.../.claude/sessions/sess-abc.jsonl",
      "cwd": "/Users/.../moai-adk-go",
      "permission_mode": "acceptEdits",
      "tool_name": "Bash",
      "tool_input": { "command": "rm -rf node_modules/", "description": "cleanup" },
      "tool_use_id": "toolu_001"
    },
    "tool_use_id": "toolu_001"
  }
}
```

moai-cli Swift 측 핸들러:

```swift
// 개념 코드
@HookCallback(id: "moai-cli-bash-guard", event: .preToolUse)
func guardBashCommand(_ input: PreToolUseInput) async -> HookJSONOutput {
  guard case .bash(let bashInput) = input.toolInput else { return .passthrough }

  // 1. 안전 wrapper 로 rewrite
  if let wrapped = BashSafeWrapper.wrap(bashInput.command) {
    return .init(
      hookSpecificOutput: .preToolUse(
        updatedInput: .bash(command: wrapped, description: bashInput.description)
      )
    )
  }

  // 2. 위험 패턴이면 UI 다이얼로그로 넘기기
  if BashDangerDetector.isDangerous(bashInput.command) {
    let decision = await NativeBashConfirmDialog.show(
      workspace: currentWorkspace,
      command: bashInput.command
    )
    return .init(
      hookSpecificOutput: .preToolUse(
        permissionDecision: decision.allowed ? .allow : .deny,
        permissionDecisionReason: decision.reason
      )
    )
  }

  return .passthrough
}
```

**타임아웃**: matcher 의 `timeout` 필드 (milliseconds) 가 Claude Code `src/utils/hooks.ts:2195` 에서 `commandTimeoutMs = hook.timeout * 1000` 으로 적용. SDK callback 도 동일 규칙 추정 (B2 §9 unverified).

### 5.4 Tool Input Rewriting (핵심 기능)

`PreToolUse` hook 의 `hookSpecificOutput.updatedInput` (`src/types/hooks.ts:76`) 으로 tool 입력을 런타임에 교체 가능하다.

**예시 1 — Bash safe wrapper**:
```swift
// 모든 "rm" → "trash" (macOS)
// 모든 "cd X && ..." → 워크트리 인식 버전
// 모든 "git push" → confirm dialog
func bashSafeWrap(_ cmd: String) -> String {
  if cmd.hasPrefix("rm ") {
    return cmd.replacingOccurrences(of: "rm ", with: "trash ")
  }
  if cmd.contains("git push") {
    return "moai-cli-confirm 'Confirm git push?' && \(cmd)"
  }
  return cmd
}
```

**예시 2 — File path normalization**:
```swift
// Write 도구의 상대 경로를 worktree 절대 경로로 확정
func normalizeWritePath(_ input: WriteInput) -> WriteInput {
  var mutated = input
  if !input.filePath.hasPrefix("/") {
    mutated.filePath = workspace.worktreeRoot.appendingPathComponent(input.filePath).path
  }
  return mutated
}
```

**예시 3 — MCP 응답 rewriting** (`PostToolUse.updatedMCPToolOutput`):
```swift
// 외부 MCP 서버 응답에서 민감 정보 redact
func redactMCPOutput(_ toolName: String, _ output: Any) -> Any {
  return Redactor.redactSecrets(output)
}
```

**주의**: updatedInput 이 `tool.inputSchema` 를 다시 통과하는지 여부는 B2 §9.4 에서 unverified. M0 spike 에서 검증 필수.

### 5.5 Native Permission Dialog

`PermissionRequest` hook (`src/types/hooks.ts:121-133`):

```swift
@HookCallback(id: "moai-cli-native-dialog", event: .permissionRequest)
func showNativeDialog(_ input: PermissionRequestInput) async -> HookJSONOutput {
  let decision = await withCheckedContinuation { continuation in
    DispatchQueue.main.async {
      let window = PermissionDialogWindow(
        toolName: input.toolName,
        toolInput: input.toolInput,
        suggestions: input.permissionSuggestions ?? []
      )
      window.onDecision = { result in
        continuation.resume(returning: result)
      }
      window.showAsSheet(from: activeWindow)
    }
  }

  switch decision {
  case .allowOnce:
    return .init(hookSpecificOutput: .permissionRequest(
      decision: .allow(updatedInput: nil, updatedPermissions: nil)
    ))
  case .alwaysAllow(let rule):
    // updatedPermissions 로 영구 저장
    return .init(hookSpecificOutput: .permissionRequest(
      decision: .allow(updatedInput: nil, updatedPermissions: [rule])
    ))
  case .deny(let reason):
    return .init(hookSpecificOutput: .permissionRequest(
      decision: .deny(message: reason, interrupt: false)
    ))
  case .interrupt:
    return .init(hookSpecificOutput: .permissionRequest(
      decision: .deny(message: "User cancelled", interrupt: true)
    ))
  }
}
```

**Claude Code TUI 의 text prompt 가 나타나지 않고, 대신 SwiftUI 네이티브 모달이 부모 윈도우의 sheet 로 뜬다.** "Always Allow" 버튼은 `updatedPermissions` 로 settings.json 에 영구 저장된다.

### 5.6 In-process MCP Server (UI 제어 tools)

`src/services/mcp/types.ts:108-113` 의 `McpSdkServerConfigSchema` 와 `src/services/mcp/InProcessTransport.ts:1-64` 를 사용한다.

moai-cli-plugin 의 `plugin.json`:
```json
{
  "name": "moai-cli",
  "version": "0.1.0",
  "description": "moai-adk official macOS GUI plugin",
  "mcpServers": {
    "moai-cli-ui": {
      "type": "sdk",
      "name": "moai-cli-ui"
    }
  },
  "hooks": { /* inline HooksSettings for additional plugin-level hooks */ },
  "commands": {
    "kanban":  { "source": "./commands/kanban.md",  "description": "Open Kanban board" },
    "memory":  { "source": "./commands/memory.md",  "description": "Open Memory viewer" },
    "surface": { "source": "./commands/surface.md", "description": "Reveal a surface" },
    "connect": { "source": "./commands/connect.md", "description": "Attach moai-cli daemon to current session" }
  },
  "userConfig": {
    "daemon_port":       { "type": "number", "title": "moai-cli daemon port", "default": 7428 },
    "auth_token":        { "type": "string", "title": "Daemon auth token", "sensitive": true },
    "workspace_dir":     { "type": "directory", "title": "Default workspace directory" },
    "enable_notifications": { "type": "boolean", "title": "Native notifications", "default": true }
  }
}
```

In-process MCP 서버가 노출할 tool 카탈로그:

| Tool 이름 | 효과 | 호출 예시 |
|---|---|---|
| `Workspace.open` | focused workspace 전환 | `{"path": "/Users/goos/moai/moai-adk-go"}` |
| `Workspace.create` | 새 workspace + git worktree + Claude Code 세션 생성 | `{"branch": "spec-042", "base": "main", "agent_host": "claude_code_sdk"}` |
| `Kanban.createCard` | Kanban 카드 생성 | `{"title": "...", "lane": "backlog", "spec_id": "SPEC-AUTH-042"}` |
| `Kanban.moveCard` | 카드 이동 | `{"id": 123, "to": "doing"}` |
| `AgentRun.focus` | Agent Run Viewer 특정 에이전트로 스크롤 | `{"agent_id": "builder-1"}` |
| `Surface.reveal` | 특정 surface 포커스 | `{"surface": "memory"}` |
| `Surface.open` | 새 surface 생성 | `{"pane_id": 5, "kind": "code_viewer", "path": "src/auth.go"}` |
| `Notification.post` | 네이티브 macOS 알림 | `{"title": "...", "body": "...", "level": "info"}` |
| `Panel.splitHorizontal` | NSSplitView 수평 분할 | `{"pane_id": 5, "ratio": 0.5}` |
| `Terminal.spawn` | 새 Ghostty PTY surface | `{"cmd": "bash", "cwd": ".", "title": "build"}` |
| `File.revealInFinder` | Finder 에 파일 하이라이트 | `{"path": "src/auth.go"}` |
| `File.openInEditor` | 기본 편집기에서 열기 | `{"path": "...", "line": 42}` |
| `Memory.revealEntry` | Memory Surface 의 특정 entry 로 스크롤 | `{"topic": "decisions/auth-v2"}` |
| `InstructionsGraph.highlight` | 특정 CLAUDE.md 파일을 그래프에서 강조 | `{"file_path": "CLAUDE.md", "reason": "session_start"}` |

**효과:** Claude 모델이 도구 호출만으로 moai-cli UI 를 조작한다. 예:

```
User: "이 SPEC 을 Doing 으로 옮겨줘"
Claude: Kanban.moveCard({id: 42, to: "doing"})
         → moai-cli UI 가 즉시 카드 이동 애니메이션 표시
         → Workspace.create({branch: "spec-042", ...})
         → Workspace.open({path: ".../worktrees/spec-042"})
         → AgentRun.focus({agent_id: new_session})
```

Claude 가 직접 "IDE 를 운전" 하는 것이 moai-cli 의 signature UX.

### 5.7 IDE Lockfile Impersonation

`src/utils/ide.ts:298-393` 이 `~/.claude/ide/*.lock` 을 스캔해 자동 연결하는 메커니즘을 활용한다.

moai-cli daemon 이 drop 하는 lockfile:
```json
{
  "workspaceFolders": ["/Users/goos/moai/moai-adk-go"],
  "pid": 7428,
  "ideName": "MoAI",
  "transport": "ws",
  "runningInWindows": false,
  "authToken": "<random 32-byte hex>"
}
```

파일 위치: `~/.claude/ide/7428.lock`

**효과:**
- moai-cli 없이 외부에서 `claude` 실행해도 cwd 가 workspaceFolder 안에 있으면 자동 연결
- WebSocket 으로 moai-cli 의 MCP 서버에 attach
- VS Code, Cursor, JetBrains, plain CLI 모두 catch
- Plugin 설치 없이도 moai-cli UI tools 사용 가능

**Stale lockfile sweep**: Claude Code 가 startup 에서 `ide.ts:522-576` 으로 pid 가 죽은 lockfile 자동 제거.

### 5.8 Direct Connect — Advanced Transport

기본 M0 에서는 **stdin/stdout SDK 프로토콜** 만 사용 (가장 안정적, 확실히 문서화됨).

M2 이후 **Direct Connect** (WebSocket + Unix socket, `src/server/`) 로 업그레이드:

| stdin/stdout SDK | Direct Connect |
|---|---|
| 한 Claude 프로세스 = 한 클라이언트 (moai-cli) | 한 Claude 프로세스 = 다중 클라이언트 가능 (unverified) |
| 세션 종료 시 프로세스도 종료 | **detached session** 지원 — Claude 프로세스는 살아있고 클라이언트만 detach/reattach |
| 프로세스 재시작 시 세션 잃음 | 프로세스 재시작 없이 세션 재개 |
| 단일 transport | HTTP + WebSocket + Unix socket (`cc+unix://` 스킴) |

moai-cli 가 Direct Connect 로 업그레이드하면:
- Kanban Review 레인의 "백그라운드 에이전트" 가 moai-cli 앱 종료 후에도 살아있다가 다시 켜면 결과 수집
- 다른 터미널에서 `claude --connect cc+unix:///Users/.../.moai-cli/sock/ws-42.sock` 으로 같은 세션에 attach 해 관찰 가능

**우려 사항** (`research/B1 §10` unverified 항목):
- 서버 측 `src/server/server.ts` 등이 code-map 에 없음. wire format 은 클라이언트 측 소비 코드에서만 역추적.
- M0 spike 에서 `claude server` 가 실제로 실행 가능한지 검증 필수.
- Fallback: 안정 stdin/stdout SDK 로 계속 운영.

### 5.9 Hook 타입 확장 — prompt/agent/http 활용

moai-adk 는 현재 `command` hook 타입만 사용 중이다 (`src/schemas/hooks.ts:176-189` 의 4가지 타입 중 1개). moai-cli 는 나머지 3가지를 활용할 여지가 있다:

| Hook 타입 | 용도 | 사용 예 |
|---|---|---|
| `command` | Shell 실행 | 기존 moai-adk 가 사용 |
| `prompt` | LLM prompt 평가 (default Haiku) | `TeammateIdle` 을 LLM verifier 로 — "이 팀원의 현재 작업이 완료 상태로 보이나?" |
| `agent` | Sub-agent verifier spawn | `PostToolUse` 에 evaluator-active 에이전트 붙여 TRUST 5 실시간 채점 |
| `http` | URL POST (SSRF guard 포함) | 외부 CI 시스템 트리거, Slack 알림 |

M4 에서 `prompt` / `agent` hook 도입 검토.

### 5.10 Task-metrics.jsonl 은 백업 소스

v2 에서 주 데이터 소스였던 `.moai/logs/task-metrics.jsonl` 은 v3 에서 **백업 소스**로 격하된다:

- **주 데이터**: SDK hook callback 이 실시간으로 in-process 전달 → GRDB.swift 에 즉시 저장
- **백업 데이터**: moai-adk 가 쓰는 jsonl 파일을 FSEventStream 으로 watch → moai-cli 앱이 꺼져 있을 때 발생한 이벤트도 다음 실행 시 복구

이 이중화로 "moai-cli 꺼진 상태에서 진행한 작업도 로그 유지" 를 보장한다.

---

## 6. Surfaces (10개)

v2 의 8개 + Memory + InstructionsGraph. 각 Surface 는 독립 구현 가능 (protocol-oriented).

공통 프로토콜 (Round 3 에서 확정한 확장성 계약):

```swift
protocol Surface: AnyObject {
  var kind: SurfaceKind { get }
  var pane: Pane? { get set }
  func handleFocus()
  func handleBlur()
  func encode(to coder: NSCoder)  // 세션 복원용
  static func decode(from coder: NSCoder) -> Self?
}

protocol SurfaceRenderer: View {
  associatedtype Surface: moai_cli.Surface
  init(surface: Surface)
}

protocol PaneLayout {
  func split(axis: Axis, ratio: CGFloat) -> (Pane, Pane)
  func collapse(pane: Pane)
  func encodeTree() -> Data
  func decodeTree(_ data: Data) throws
}
```

### 6.1 Terminal Surface (Ghostty)

- **엔진**: `GhosttyKit.xcframework` (Zig/C → Swift via `import GhosttyKit`)
- **소유권**: PTY master = Ghostty. moai-cli 는 attach view 만 렌더.
- **Sixel / iTerm2 inline image**: Ghostty 가 네이티브 지원
- **moai-adk 데코레이션**: Hook callback 이 PostToolUse 를 받을 때 해당 PTY 의 스크롤백에 좌측 거터 아이콘 주입 (`●plan`, `●run`, `●sync`, `●fix`)
- **Scrollback 검색**: GRDB.swift FTS5 (agent_events 풀텍스트)
- **Command palette → Slash injection**: Cmd+K 에서 `/moai run SPEC-AUTH-042` 선택 시 포커스 workspace 의 claude SDK session 에 `SDKUserMessage` 전송

### 6.2 Code Viewer Surface (v3 핵심 surface)

cmux 에 없는, v2 에서 신설한 1급 surface. v3 에서 tech stack 만 단순화.

- **렌더러**: `NSTextView` 서브클래스 + **SwiftTreeSitter** 하이라이트 (Rust FFI 없음)
- **LSP 진단**: 두 가지 경로
  - 경로 A: SwiftLSPClient 로 project-local LSP (gopls, rust-analyzer, …) 풀링 — 직접 진단
  - 경로 B: **Claude Code 의 LSP hook 이벤트** 구독 — moai-adk 가 이미 통합한 LSP 진단을 재사용 (`Notification` 이벤트 또는 `PostToolUse.additionalContext` 로 흐름)
  - v3 권장: 경로 B 우선. 경로 A 는 moai-adk 가 LSP 를 제공하지 않는 언어에 대해서만.
- **@MX 거터**: moai-adk 의 `/moai mx --dry --json` 결과 파싱 → GRDB.swift `mx_tags` 테이블 캐시. 거터 아이콘 ★(ANCHOR)/⚠(WARN)/ℹ(NOTE)/☐(TODO). 클릭 → inspector.
- **LSP gate overlay**: `.moai/config/sections/quality.yaml` 의 `lsp_quality_gates.run.max_errors` 등 읽어 상단 스트립 "errors: 0 / type_errors: 0 / lint: 2 ⚠" 표시. 초과 시 붉은 배너.
- **Tri-pane diff 모드**: `HEAD:main | working tree | agent pending`. 오른쪽 pane 은 `PostToolUse` 이벤트 수신 시 실시간 업데이트. Accept / Revert 버튼.
- **SPEC 링크**: `@MX:ANCHOR SPEC-AUTH-042` 주석 클릭 → Markdown surface 로 점프
- **Time travel**: `git log -p` 를 GRDB.swift 에 인덱싱 → 슬라이더 스크럽. 각 시점 task-metric (토큰/모델/소요) 를 bar chart.
- **Edit mode**: 기본 read-only. 편집 모드 진입 시 즉시 git stash-like 스냅샷 (에이전트 덮어쓰기 복구)

### 6.3 Markdown Surface (EARS 특화)

- **렌더러**: `Down` (Swift cmark wrapper) 또는 `swift-cmark` 직접 FFI
- **확장**: KaTeX (수식), Mermaid (다이어그램) — WKWebView 로 렌더
- **라이브**: FSEventStream 200ms debounce
- **EARS 특화 모드**: `.moai/specs/SPEC-*/spec.md` 열면 Given/When/Then 블록을 카드로 렌더. Acceptance 체크리스트는 인터랙티브 (체크 → GRDB.swift `specs` 업데이트 + 파일 재쓰기).
- **2-up 모드**: 좌측 SPEC, 우측 관련 파일 `git diff vs main`

### 6.4 Image Surface

- **렌더러**: Core Image + Metal
- **Artifacts watch**: `FSEventStream` 으로 `artifacts/` 폴더 자동 감지
- **Diff 모드**: 두 PNG 픽셀 diff + SSIM 점수 (Vision framework)
- **`/moai e2e` 연동**: Playwright 결과 스크린샷 자동 오픈

### 6.5 Browser Surface

- **엔진**: `WKWebView` (cmux `BrowserWindowPortal.swift` 패턴)
- **DevTools**: `setInspectable(true)` → Safari Web Inspector
- **Port 스캐너**: 에이전트가 띄운 dev 서버 (3000, 5173, 8080) 자동 감지 → 사이드바 "Listening ports" 리스트 → 클릭으로 Browser surface 생성
- **원격**: Claude Code 의 `RemoteSessionManager` 재사용 (cmux Go daemon 포팅 없음)
- **`/moai e2e` 연동**: Claude-in-Chrome 테스트 실행 결과를 이 surface 에 임베드

### 6.6 FileTree Surface

- **엔진**: `FSEventStream` + 네이티브 Swift 디렉토리 열거
- **Git status**: SwiftGit2 또는 `git status --porcelain` shell-out → 각 파일에 M/A/D/? 색상
- **컨텍스트 액션**:
  - Reveal in Finder
  - Open in Code Viewer
  - Send path to focused agent (SDK user message injection)
  - Diff against main
  - "Create SPEC from selection" — 선택 파일 목록을 EARS 초안 생성용으로 `/moai plan` 에 첨부
- **드래그 드롭**: 외부 파일 → workspace worktree 복사 + 에이전트 컨텍스트 자동 첨부
- **v3 추가**: `FileChanged` hook 이벤트를 구독해서 **Claude 가 만든 파일** 과 **사용자가 만든 파일** 을 색상으로 구분 표시

### 6.7 Agent Run Viewer (v3 재설계)

v2 에서는 task-metrics.jsonl 파싱. v3 에서는 **SDK hook callback stream 직접 구독** + jsonl 백업.

**데이터 소스**:
- Primary: 27 hook callback stream → GRDB.swift `hook_events` 테이블
- Secondary: `.moai/logs/task-metrics.jsonl` tail (moai-cli 꺼진 동안 유실 보완)

**레이아웃** (design-exports/05-agent-run.png 참고, v3 업그레이드):

- **좌측**: Workspace 내 세션/태스크 타임라인
  - 각 row = 1 task (SubagentStart ~ SubagentStop)
  - 색상 = `agent_type` (manager-spec, expert-backend, …)
  - Progress bar = `PostToolUse` 이벤트 카운트 / 예상
- **중앙**: 선택 task 의 step-by-step 트레이스
  - SessionStart / PreToolUse / PostToolUse / Notification / TaskCompleted 카드
  - 각 카드: `tool_calls`, `tools_used`, duration_ms, token 추정 (SDK 는 cost_update 이벤트 제공)
  - 펼치기: stdin/stdout 캡처, 인자, 결과
  - 상단: 누적 토큰 / 예상 비용 (`llm.yaml` model policy 기반)
- **우측**: **v3 신규 — Live agent state**
  - 현재 어떤 tool 이 실행 중인지 spinner
  - 현재 어떤 파일을 편집 중인지 (PreToolUse 의 tool_input)
  - "이 agent 를 interrupt" 버튼 (`interrupt` Direct Connect 명령)
  - "모델 변경" 드롭다운 (`change_model` 명령)
  - "Permission mode" 토글 (`set_permission_mode`)
- **하단 액션**:
  - **Replay from here**: 이후 prompt 만 다른 workspace 에서 재실행 (`SDKUserMessage` 로 부활)
  - **Open failing file**: LSP error 좌표로 Code Viewer 점프
  - **Revert commits by this run**: `git reset` 도우미

### 6.8 Kanban Board

- **레인**: Backlog / To-Do / Doing / Review / Done / Blocked (커스터마이즈 가능)
- **카드 필드**: title, body_md, spec_id, assignee (agent_host), labels, linked files
- **Doing 자동화 (v3 업그레이드)**:
  1. 드래그 감지 → `Kanban.moveCard` MCP tool 호출 (자기 자신)
  2. `Workspace.create` 호출 → git worktree add + Claude SDK session 생성
  3. `initialize.hooks` 로 27 callback 등록
  4. `SDKUserMessage` 로 `/moai run SPEC-AUTH-042` 전송 (**SDK user message = 합법적 UserPromptSubmit 트리거**)
  5. `Surface.reveal({surface: "agent_run"})` 로 3-pane 레이아웃 자동 구성
- **Review 자동화**:
  1. `git diff main..HEAD` → Markdown surface
  2. `/moai review` → SDK user message
  3. TRUST 5 점수 + LSP gate 결과를 카드 배지로 (hook events 에서 파싱)
- **Done 자동화**: `gh pr create` 옵션 + worktree archive
- **Backlog 생성**: `/moai plan` 결과를 `WorktreeCreate` / FSEventStream 으로 감지 → 자동 카드 생성 (spec_id 포함)
- **저장**: GRDB.swift `kanban_cards` 테이블 (WAL)

### 6.9 Memory Surface (★ v3 신규)

- **데이터**: `~/.claude/projects/<sanitized-git-root>/memory/` 의 markdown 파일 직접 렌더
  - `MEMORY.md` (index, 25KB/200 라인 cap)
  - 토픽 파일 (`*.md`)
  - `logs/YYYY/MM/YYYY-MM-DD.md`
- **렌더러**: Markdown surface 재사용 + 특화 UI
  - 좌측: `MEMORY.md` 를 index 로 트리 렌더
  - 우측: 선택한 토픽 파일 preview
  - 하단: 25KB / 200 라인 cap progress bar (80% 넘으면 주황, 95% 넘으면 빨강)
- **편집**: Edit 버튼 → Code Viewer 에서 해당 파일 열림. 저장 시 Claude Code 도 같은 파일을 즉시 반영.
- **ConfigChange hook**: `memory_type` 변화 감지 → UI 자동 refresh
- **근거**: `research/B3 §6 Memory / Context Persistence`

### 6.10 InstructionsGraph Surface (★ v3 신규)

`InstructionsLoaded` hook (`src/entrypoints/sdk/coreSchemas.ts:695-707`) 을 구독해 현재 세션에 어떤 CLAUDE.md / skill / memory 가 로드되었는지 실시간 트리 시각화.

- **노드**: 각 로드된 파일
  - `memory_type`: User / Project / Local / Managed (색상 구분)
  - `load_reason`: session_start / nested_traversal / path_glob_match / include / compact (아이콘 구분)
  - `globs`, `trigger_file_path`, `parent_file_path` → edge 로 렌더
- **클릭**: 해당 파일을 Markdown Surface 또는 Code Viewer 에서 열림
- **용도**: "왜 이 파일이 컨텍스트에 있는가?" 디버깅. 프롬프트 엔지니어링의 블랙박스를 열어봄.
- **독창성**: 어떤 경쟁사도 이 기능 없음. moai-cli 의 unique value.

### 6.11 명령 팔레트 (Cmd+K) [Surface 아님, Shell 기능]

- **소스**: 모든 MCP tool, slash 커맨드, 파일, SPEC, 카드, 심볼 (SwiftTreeSitter)
- **moai-adk 섹션** 1급: 14개 `/moai *` 슬래시 커맨드
- **동사형 항목**: "Run /moai coverage on focused workspace", "Open SPEC-AUTH-042", "Spawn Codex workspace (read-only)"
- **Slash injection**: 선택 시 `SDKUserMessage` 로 포커스 workspace 의 Claude session 에 전달

---

## 7. 데이터 모델 (GRDB.swift / SQLite WAL)

v2 의 SQLite 스키마를 Swift 친화적으로 refactor. GRDB.swift 의 `PersistableRecord` 와 migration 시스템 사용.

```swift
// GRDB migration 선언 예시 (의사 코드)
migrator.registerMigration("v1") { db in
  try db.create(table: "projects") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("root", .text).unique()
    t.column("name", .text)
    t.column("is_moai_adk", .boolean).defaults(to: false)
    t.column("moai_version", .text)
    t.column("opened_at", .datetime)
  }

  try db.create(table: "workspaces") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("project_id", .integer).references("projects")
    t.column("name", .text)
    t.column("branch", .text)
    t.column("worktree_path", .text)
    t.column("agent_host", .text)  // claude_code_sdk|codex|shell|tmux_cg
    t.column("spec_id", .text)     // SPEC-AUTH-042
    t.column("status", .text)      // starting|running|waiting|review|error|archived
    t.column("claude_session_id", .text)  // SDK session_id
    t.column("created_at", .datetime)
    t.column("last_active_at", .datetime)
  }

  try db.create(table: "panes") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("workspace_id", .integer).references("workspaces")
    t.column("parent_id", .integer)
    t.column("split", .text)       // horizontal|vertical|leaf
    t.column("ratio", .double)
  }

  try db.create(table: "surfaces") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("pane_id", .integer).references("panes")
    t.column("kind", .text)        // terminal|code|markdown|image|browser|filetree|agent_run|kanban|memory|instructions_graph
    t.column("state_json", .blob)  // Codable encoded state
  }

  // SDK hook event stream (v3 주 데이터 소스)
  try db.create(table: "hook_events") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("workspace_id", .integer).references("workspaces").indexed()
    t.column("ts", .datetime).indexed()
    t.column("event", .text)       // 27 종
    t.column("callback_id", .text)
    t.column("session_id", .text)
    t.column("agent_id", .text)
    t.column("tool_use_id", .text)
    t.column("matcher", .text)     // 어떤 matcher 와 매치되었는지
    t.column("payload", .blob)     // 타입 안전한 Codable 직렬화
    t.column("response_payload", .blob)  // moai-cli 가 보낸 응답 (updatedInput 등)
    t.column("duration_ms", .integer)
  }

  // cost_update 이벤트 전용 (Direct Connect 에서만 풍부하게 제공)
  try db.create(table: "cost_updates") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("workspace_id", .integer).references("workspaces").indexed()
    t.column("ts", .datetime)
    t.column("turn_number", .integer)
    t.column("model", .text)
    t.column("input_tokens", .integer)
    t.column("output_tokens", .integer)
    t.column("cache_read_tokens", .integer)
    t.column("cache_write_tokens", .integer)
    t.column("estimated_cost_usd", .double)
  }

  // Task-metrics.jsonl 백업 미러 (moai-cli 꺼진 동안 유실 복구용)
  try db.create(table: "task_metrics_mirror") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("workspace_id", .integer).references("workspaces")
    t.column("ts", .datetime).indexed()
    t.column("session_id", .text)
    t.column("task_id", .text)
    t.column("agent_type", .text)
    t.column("operation", .text)
    t.column("input_tokens", .integer)
    t.column("output_tokens", .integer)
    t.column("total_tokens", .integer)
    t.column("duration_ms", .integer)
    t.column("tool_calls", .integer)
    t.column("tools_used", .text)   // JSON array
    t.column("status", .text)
    t.column("spec_id", .text).indexed()
  }

  // SPEC 카탈로그 (파일 파싱 결과 미러)
  try db.create(table: "specs") { t in
    t.primaryKey("id", .text)       // SPEC-AUTH-042
    t.column("project_id", .integer).references("projects")
    t.column("title", .text)
    t.column("ears_md", .text)
    t.column("plan_md", .text)
    t.column("status", .text)       // draft|running|review|done
    t.column("updated_at", .datetime)
  }

  // @MX 태그 인덱스
  try db.create(table: "mx_tags") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("project_id", .integer).references("projects")
    t.column("path", .text).indexed()
    t.column("line", .integer)
    t.column("kind", .text)         // ANCHOR|WARN|NOTE|TODO
    t.column("reason", .text)
  }

  // Kanban
  try db.create(table: "kanban_boards") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("project_id", .integer).references("projects")
    t.column("name", .text)
  }
  try db.create(table: "kanban_cards") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("board_id", .integer).references("kanban_boards")
    t.column("lane", .text)
    t.column("title", .text)
    t.column("body_md", .text)
    t.column("workspace_id", .integer).references("workspaces")
    t.column("spec_id", .text)
    t.column("assignee", .text)
    t.column("created_at", .datetime)
    t.column("updated_at", .datetime)
  }

  // Notifications (native macOS 알림 중복 제거용)
  try db.create(table: "notifications") { t in
    t.autoIncrementedPrimaryKey("id")
    t.column("ts", .datetime)
    t.column("kind", .text)
    t.column("ref", .text)
    t.column("body", .text)
    t.column("read", .boolean)
  }
}
```

**TTL 정책:**
- `hook_events`: 30 일
- `cost_updates`: 90 일
- `task_metrics_mirror`: 30 일
- 나머지: 영구

**인덱스:**
- 모든 시계열 테이블에 `(workspace_id, ts)` composite
- `mx_tags` 에 `path`, `spec_id` 에 `spec_id`

---

## 8. 기술 스택 (확정)

### 8.1 Shell (Swift 네이티브)

| 컴포넌트 | 기술 | 비고 |
|---|---|---|
| UI 프레임워크 | SwiftUI + AppKit | macOS 14+ |
| 터미널 엔진 | **GhosttyKit.xcframework** (Zig 빌드) | cmux 와 동일 |
| 분할 시스템 | **NSSplitView** 자체 구현 (binary tree + NSCoder) | Round 4 확정. 1-2주 M1 흡수 |
| 코드 파싱 | **SwiftTreeSitter** (+ language grammars) | Rust FFI 없음 |
| Markdown | **Down** (cmark wrapper) 또는 `swift-cmark` 직접 | |
| LSP 클라이언트 | **SwiftLSPClient** (필요 시만) | Claude Code 의 LSP hook 우선 사용 |
| WebView | **WKWebView** | |
| DB | **GRDB.swift** (SQLite WAL) | |
| Git | **SwiftGit2** 또는 `git` CLI shell-out | |
| 자동 업데이트 | **Sparkle** | |
| File watching | **FSEventStream** (Swift wrapper) | |
| Crash reporting | **Sentry-Cocoa** (opt-in) | v2 와 동일 |
| Analytics | **PostHog** (opt-in) | v2 와 동일 |

**Rust 0, swift-bridge 0, cargo 0.**

### 8.2 Claude Code SDK Client (Swift 자체 구현)

별도 Swift package (`ClaudeCodeSDK`) 로 분리:

| 모듈 | 역할 |
|---|---|
| `SDKMessageCodec` | JSON-stream 인코더/디코더 |
| `ControlRequestRouter` | `initialize`, `can_use_tool`, `mcp_message`, `hook_callback`, `set_permission_mode`, `interrupt`, `mcp_set_servers` 등 |
| `HookCallbackRegistry` | 27 hook event 에 대한 callback 라우팅 |
| `SessionManager` | Claude process lifecycle, detached session 지원 (Direct Connect) |
| `InProcessMCPServer` | `moai-cli-ui` 서버 구현 |
| `DirectConnectClient` | WebSocket / Unix socket transport (M2 이후) |

이 package 는 장기적으로 **별도 오픈소스** 로 분리 가능. "Claude Code SDK for Swift" 자체가 커뮤니티에 가치 있는 자산.

### 8.3 moai-cli-plugin (Claude Code Plugin)

`moai-cli/plugin/` 디렉토리:
```
moai-cli/
├── plugin/
│   ├── .claude-plugin/
│   │   └── plugin.json          # 매니페스트
│   ├── hooks/
│   │   └── moai-cli-hooks.json  # HooksSettings inline
│   ├── commands/
│   │   ├── kanban.md
│   │   ├── memory.md
│   │   ├── surface.md
│   │   └── connect.md
│   ├── skills/
│   │   ├── moai-cli-open-workspace/
│   │   │   └── SKILL.md
│   │   └── moai-cli-focus-agent/
│   │       └── SKILL.md
│   ├── output-styles/
│   │   └── moai-cli.md          # forceForPlugin: true
│   └── mcp-server-sdk/           # SDK transport MCP 서버
│       └── (implementation linked from moai-cli.app)
```

### 8.4 디렉토리 (단일 저장소, `modu-ai/moai-adk` 내부)

```
moai-adk-go/                            # 기존 moai-adk Go CLI
├── cmd/moai/                            # Go entry
├── internal/                            # Go 내부
├── pkg/                                 # Go 공개 패키지
├── moai-cli/                            # v3 신규 루트
│   ├── DESIGN.v3.md                     # 이 문서
│   ├── research/                        # R1, B1, B2, B3
│   ├── design-exports/                  # 기존 PNG 목업
│   │
│   ├── app/                             # macOS app
│   │   ├── moai-cli.xcodeproj
│   │   ├── Sources/
│   │   │   ├── App/                     # @main, AppDelegate
│   │   │   ├── Shell/                   # Sidebar, Tabs, NSSplitView, CommandPalette
│   │   │   ├── Surfaces/
│   │   │   │   ├── Terminal/            # GhosttyKit wrapper
│   │   │   │   ├── CodeViewer/          # NSTextView + SwiftTreeSitter + @MX gutter
│   │   │   │   ├── Markdown/
│   │   │   │   ├── Image/
│   │   │   │   ├── Browser/             # WKWebView
│   │   │   │   ├── FileTree/
│   │   │   │   ├── AgentRun/
│   │   │   │   ├── Kanban/
│   │   │   │   ├── Memory/              # ★ v3 신규
│   │   │   │   └── InstructionsGraph/   # ★ v3 신규
│   │   │   ├── Core/
│   │   │   │   ├── ClaudeCodeSDKHost/
│   │   │   │   ├── HookCallbacks/       # 27 callback 핸들러
│   │   │   │   ├── InProcessMCP/        # UI tools MCP 서버
│   │   │   │   ├── Store/               # GRDB.swift
│   │   │   │   ├── Git/
│   │   │   │   ├── FS/
│   │   │   │   └── IDEImpersonator/     # lockfile drop
│   │   │   └── Theme/
│   │   └── Resources/
│   │
│   ├── sdk/                             # Swift Package: ClaudeCodeSDK
│   │   ├── Package.swift
│   │   └── Sources/
│   │
│   ├── plugin/                          # Claude Code plugin
│   │   ├── .claude-plugin/plugin.json
│   │   ├── hooks/
│   │   ├── commands/
│   │   ├── skills/
│   │   └── output-styles/
│   │
│   ├── vendor/
│   │   ├── ghostty/                     # submodule
│   │   └── tree-sitter-grammars/        # submodule
│   │
│   ├── scripts/
│   │   ├── build-xcframework.sh         # GhosttyKit 빌드
│   │   ├── install-plugin.sh            # ~/.claude/plugins/moai-cli@local/ 드롭
│   │   └── reload.sh                    # cmux 패턴 차용
│   │
│   ├── tests/
│   │   ├── unit/                        # Swift Testing
│   │   ├── ui/                          # XCUITest
│   │   ├── integration/                 # Mock Claude SDK 로 hook 라운드트립 검증
│   │   └── stress/                      # 16-workspace 시나리오
│   │
│   └── docs/
│
├── .github/workflows/
│   ├── ci-go.yml                        # 기존 moai-adk Go CI
│   ├── ci-moai-cli.yml                  # ★ v3 신규: Xcode 빌드 + Swift test
│   ├── release-cli.yml                  # ★ v3 신규: notarize + DMG + Sparkle appcast
│   └── ...
└── ...
```

**path filter** (모노레포 CI 최적화):
- `moai-cli/**` 변경 시만 `ci-moai-cli.yml` 트리거
- 그 외는 기존 Go CI 만
- 양쪽이 동시 바뀌는 PR 은 두 워크플로우 모두 실행

---

## 9. 마일스톤 (v3 조정)

v2 의 M0~M7 을 Rust core 제거로 단축.

| 단계 | v2 기간 | v3 기간 | v3 산출물 |
|---|---|---|---|
| **M0 Spike** | 2주 | **1주** | Xcode 프로젝트 + `GhosttyKit.xcframework` 빌드 + 단일 PTY 표시 + Claude Code spawn via SDK stream-json + SessionStart hook callback 1개 왕복 (proof of life) |
| **M1 Core Sessions** | 4주 | **3주** | Workspace/Pane/Surface 모델, NSSplitView 바이너리 트리, GRDB.swift 스키마 v1, IPC (Unix socket), Sidebar, Terminal surface, HookCallbackRegistry 전체 27 이벤트 wired |
| **M2 Viewers 1** | 3주 | **3주** | FileTree, Markdown, Image, Browser (v2 와 동일) |
| **M3 Code Viewer** ★ | 4주 | **3주** | SwiftTreeSitter, Claude Code LSP event 구독, @MX 거터, tri-pane diff, time-travel |
| **M4 Claude SDK 깊은 통합** ★ 신규 | — | **3주** | Plugin manifest + In-process MCP 서버 + UI tools (`Kanban.moveCard` 등) + IDE lockfile impersonation + Native permission dialog |
| **M5 Agent Run + Kanban + Memory** | 3주 | **3주** | Agent Run Viewer (SDK event stream), Kanban 자동화, Memory surface, InstructionsGraph, EARS markdown 모드 |
| **M6 Direct Connect + Detached Sessions** ★ 신규 | — | **2주** | Direct Connect transport 업그레이드, detached session 지원, 백그라운드 에이전트 |
| **M7 안정화/배포** | 3주 | **2주** | Sparkle, notarize, 16-agent stress, DMG, 네트워크 연결 관련 실측 튜닝 |
| **M8 (옵션)** | 4주 | — | Linux 포팅 (protocol-oriented SwiftUI → GTK) — 수요 기반 |

**총: v2 23-27주 → v3 20주** (약 3-7주 단축). Rust core 제거 + SDK 재사용 이득.

### M0 상세 (첫 1주)

**성공 기준**: macOS 창에서 단일 Ghostty terminal 이 뜨고, Claude Code 가 stream-json SDK 모드로 spawn 되고, SessionStart hook 이 Swift 함수를 호출해 로그를 찍는다.

**작업 분해:**

| 일 | 작업 |
|---|---|
| D1 | Xcode 프로젝트 생성 (SwiftUI + AppKit hybrid). `GhosttyKit.xcframework` 서브모듈 추가 + 빌드 검증 |
| D2 | `import GhosttyKit` + 단일 PTY attach view. `zsh` spawn 해서 렌더 확인 |
| D3 | `ClaudeCodeSDKHost` 초기 구현. `Process` 로 `claude --output-format stream-json --input-format stream-json` spawn. stdin/stdout 스트림 파서 |
| D4 | `ControlRequest` + `ControlResponse` codable 타입. `initialize` 요청 송신 + `initialize_response` 수신 |
| D5 | `HookCallbackRegistry` 스파이크. SessionStart callback 1개만 등록. Claude 가 callback 을 호출하면 Swift 함수에서 로그 찍기 |
| D6 | `SDKUserMessage` 송신 시도. "Hello Claude" 보내고 응답 스트림 읽기 |
| D7 | 검증: M0 성공 기준 충족 여부 확인. Go/No-Go 결정 |

**Go/No-Go 기준:**
- Ghostty xcframework 빌드 성공 ✅
- Claude Code SDK spawn 성공 ✅
- Initialize control request 왕복 성공 ✅
- SessionStart hook callback 왕복 성공 ✅
- SDKUserMessage 로 "Hello" 보내고 assistant 응답 델타 수신 성공 ✅

하나라도 실패 시 대안 검토 (PTY 모드 fallback, 외부 sidecar process 등).

---

## 10. 성능 / 안정성 / 보안

v2 의 §7 을 축약하고 v3 차이점만 명시.

### 10.1 성능

- **Hook callback latency**: shell wrapper 제거로 기존 10-40ms → <2ms 목표. 측정 도구: `hook_events` 테이블에 `duration_ms` 컬럼으로 P50/P95/P99 기록
- **Ghostty surface**: `isHiddenOrHasHiddenAncestor` 체크로 숨은 surface 렌더 폐기 (cmux 패턴)
- **SDK 이벤트 fan-out**: AsyncStream 으로 자연스러운 backpressure. Swift actor 로 경합 방지
- **GRDB.swift**: WAL, `synchronous=NORMAL`, batch insert (100 row 또는 100ms)
- **SwiftTreeSitter incremental parsing**: 1MB 파일 초기 파싱 < 100ms 목표
- **Command palette**: FTS5 (GRDB 내장) 100k 항목 < 50ms
- **task-metrics tail**: FSEventStream 200ms debounce

### 10.2 안정성 — Swift Actor Supervision

v2 의 "Erlang OTP one_for_one" 개념을 Swift actor 로 표현:

```swift
actor WorkspaceSupervisor {
  var state: WorkspaceState = .starting
  var claudeProcess: Process?
  var sdkHost: ClaudeCodeSDKHost?
  var hookRegistry: HookCallbackRegistry?
  var terminalPTY: GhosttyPTY?
  var metricsTailTask: Task<Void, Error>?
  var fileWatcherTask: Task<Void, Error>?
  var gitWatcherTask: Task<Void, Error>?

  func start() async throws { ... }
  func stop() async { ... }

  // 자식 Task 중 하나가 fail 하면 이 actor 만 재시작
  // 다른 workspace 는 영향 없음
}

actor RootSupervisor {
  var workspaces: [Workspace.ID: WorkspaceSupervisor] = [:]
  var ipcServer: IPCServer
  var store: Store
  var ideImpersonator: IDEImpersonator
  var updateManager: UpdateManager
}
```

**Crash dump**: `~/.moai-cli/crash/` + Sentry opt-in.

**재시작 복구**: `workspaces.status == 'running'` 행에 대해 Claude SDK process 재attach 시도. Direct Connect 으로 detached session 재연결. stdin/stdout SDK 의 경우 session_id 를 저장해 두었다가 `claude --resume <session_id>` 로 복구.

### 10.3 보안

- **IDE lockfile**: 0600 파일 권한 + auth_token 은 32-byte random hex
- **In-process MCP**: 같은 프로세스이므로 IPC 보안 문제 없음. 외부 소켓 0
- **Direct Connect Unix socket**: 0600, `~/.moai-cli/sock/` 0700
- **Bearer token**: macOS Keychain 에 저장 (SDK plugin userConfig 의 sensitive: true)
- **WebView**: `nonPersistentDataStore` 기본, 사이트 격리, mixed content 차단
- **자동 업데이트**: EdDSA 서명 + HTTPS Sparkle appcast
- **PreToolUse hook 기반 security scan**: `.moai/config/sections/security.yaml` 의 `forbidden_keywords` 를 Bash 명령에 적용 (moai-adk 가 이미 하는 것 재사용)
- **Permission persistence**: `PermissionRequest` 의 `updatedPermissions` 는 `~/.claude/settings.json` 에 쓰이므로 사용자 확인 필수

### 10.4 Privacy (중요)

R1 리서치에서 Warp 의 2022 텔레메트리 사건 + 2025 "session sent to LLM" 사건이 여전히 HN 여론을 지배한다는 사실을 확인했다.

**moai-cli 의 원칙:**
1. **기본 0 텔레메트리.** 설치 시점에 묻지도 않음.
2. **크래시 리포트는 명시적 opt-in.** Settings > Privacy 에서 체크해야 Sentry 활성화.
3. **Analytics opt-in.** PostHog 는 opt-in, 기본 off.
4. **네트워크 호출 감시**: NetworkFlow 아이콘을 상단 바에 표시. 사용자가 언제든 현재 outbound connection 목록을 볼 수 있음.
5. **오픈 소스 MIT**: 모든 코드 공개, 리뷰 가능.

---

## 11. 테스트 전략

| 레벨 | 도구 | 대상 |
|---|---|---|
| Unit (Swift) | Swift Testing + swift-snapshot-testing | 도메인 로직, GRDB migration, hook 파서, MCP 서버 |
| Integration | Custom harness with **Mock Claude Code SDK** | 27 hook callback round-trip, updatedInput/updatedOutput/updatedPermissions 동작 |
| UI snapshot | XCUITest + swift-snapshot-testing | Sidebar, Code Viewer 거터, Kanban, Agent Run Viewer |
| Stress | 자체 harness | 16 workspace × 30분, Mock Claude process, SDK event flood |
| E2E | Robot / AppleScript | "이슈 → plan → run → sync → PR" 전체 플로우 |
| Claude Code 호환성 | Claude Code version matrix | v2.2.x / v2.3.x 지원 선언, nightly CI 에서 최신 tag 검증 |

### 11.1 Mock Claude Code SDK

M0 에서 만들어야 할 핵심 인프라. 역할:
- 실제 `claude` 바이너리 없이 SDK control protocol 을 에뮬레이트
- 임의의 hook event 를 `hook_callback` control request 로 주입 가능
- 27 이벤트 전부 fixture 제공
- Permission dialog 라운드트립 검증

### 11.2 moai-adk 호환성 매트릭스

```yaml
# ci-moai-cli.yml 일부
matrix:
  claude-code-version: ["2.2.x", "2.3.x", "nightly"]
  moai-adk-version: ["2.10.x", "2.11.x"]
```

업스트림 breaking change 조기 감지.

---

## 12. 경쟁 포지셔닝 (최종)

| 축 | cmux | Warp | Wave | Zed | Ghostty | moai-cli |
|---|---|---|---|---|---|---|
| 터미널 엔진 | libghostty | 자체 Rust GPU | Chromium | GPUI | 자체 | **libghostty** |
| UI | Swift + AppKit | 자체 Rust UI | Electron | GPUI (자체) | AppKit / GTK | **SwiftUI + AppKit** |
| 라이선스 | GPL-3.0 | 폐쇄 | Apache-2.0 | GPL/AGPL/Apache | MIT | **MIT** |
| 가격 | 무료 | $0~$180/mo | 무료 | $0~$10/mo | 무료 | **무료** |
| Claude Code 통합 | teammate shim | cloud agents | badge rollup (hook) | ACP (hook 미지원) | 없음 | **SDK 1급 임베드 + 27 hook callback** |
| MCP 통합 | ❌ | ❌ (미문서) | ❌ | ✅ (편집기) | ❌ | **✅ in-process MCP + IDE lockfile** |
| Kanban / SPEC 보드 | ❌ | ❌ | ❌ | ❌ | ❌ | **✅ (worktree 자동화)** |
| Agent Run Viewer | ❌ | ❌ | ❌ | ❌ | ❌ | **✅ (27 이벤트 실시간)** |
| Memory Viewer | ❌ | ❌ | ❌ | ❌ | ❌ | **✅ (`~/.claude/projects/`)** |
| Permission dialog | TUI | TUI | TUI | 부분 | N/A | **✅ Native SwiftUI** |
| @MX 태그 | ❌ | ❌ | ❌ | ❌ | ❌ | **✅ (거터 + inspector)** |
| TRUST 5 게이지 | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Hook 이벤트 UI 노출 | OSC 만 | ❌ | badge | ❌ | ❌ | **✅ (27 종 전부)** |
| Tool input rewriting | ❌ | ❌ | ❌ | ❌ | ❌ | **✅ (PreToolUse.updatedInput)** |
| 라이브 프로세스 복원 | ❌ (의도적) | 부분 | ❌ | ❌ | N/A | **✅ (detached session)** |
| SSH 원격 | ✅ | ✅ | 부분 | ✅ | ❌ | **✅ (Claude Code 내장 재사용)** |
| OS 지원 | macOS | mac/linux/win | mac/linux | 3-OS | mac/linux | **macOS 영구** |
| GPU 렌더 | Metal | Metal/wgpu | Chromium | Metal/Vulkan | Metal/OpenGL | **Metal (Ghostty)** |

**moai-cli 의 moat (다른 5개 제품 모두 0):**
1. Kanban / SPEC 보드 + worktree 자동화
2. Agent Run Viewer (27 hook 이벤트 실시간)
3. Memory Viewer
4. @MX 태그 거터 + TRUST 5 게이지
5. Tool input rewriting via PreToolUse.updatedInput
6. Native permission dialog (SwiftUI 모달 + persist rules)
7. In-process MCP (Claude 가 UI 조작)

7개 중 6개가 경쟁사 전무. 나머지 1개 (Permission dialog) 는 Zed 가 부분 구현 — 우리가 완성도 높여서 차별화.

---

## 13. 남은 열린 결정 사항 (v2 의 8개 → v3 의 5개)

v2 의 8개 open decision 중 4개는 인터뷰 라운드에서 확정, 4개는 리서치가 답을 제공했다. 새로 떠오른 5개:

### O1. In-process MCP 와 moai-cli Shell 사이의 IPC 프로토콜

In-process MCP 서버가 UI tool 호출을 받으면 그것을 moai-cli Shell process (같은 프로세스이지만 다른 actor) 로 전달해야 한다.

- **옵션 A**: Swift actor 직접 호출 (같은 프로세스이므로 가능)
- **옵션 B**: NSXPCConnection (Apple 표준 cross-process IPC)
- **옵션 C**: UNIX socket on loopback

**권장**: A. 모든 것이 같은 프로세스 안에 있으므로 actor call 이 가장 간단. B/C 는 oversolving.

### O2. Plugin 의 `local-jsx` slash command 가능 여부

`research/B3 §9.2` 에서 unverified. Built-in `/plugin`, `/agents` 는 `src/commands.ts` 에 하드코딩되어 있고, plugin 이 `.tsx` 파일을 ship 할 수 있는 공식 경로는 확인 못 함.

**M0 검증 필요.** 만약 불가하면:
- **Fallback 1**: `prompt` 타입 slash command 로 대체 (`/moai-cli:kanban` → user message 로 변환되어 Claude 가 해석 후 MCP tool 호출)
- **Fallback 2**: Claude Code fork 운영 (최후 수단, 유지보수 부담 큼)

### O3. Direct Connect 서버 wire format 안정성

`research/B1 §10.1` 에서 unverified. `src/server/server.ts` 가 code-map 에 없어서 구현 세부를 못 봤다.

**M0 검증 필요.** `claude server` CLI 가 실제로 실행되고, Unix socket + WebSocket 이 예상대로 동작하는지 확인. 실패 시 stdin/stdout SDK 로 영구 운영 (M6 Direct Connect 업그레이드 취소).

### O4. updatedInput 재검증

`research/B2 §9.4` 에서 unverified. PreToolUse hook 의 `updatedInput` 이 tool.inputSchema 재검증을 거치는지 모름. 재검증 없으면 우리가 잘못된 데이터를 inject 하면 Claude Code 가 크래시할 수 있음.

**M0 검증 필요.** Mock Claude SDK 에서 의도적으로 잘못된 updatedInput 을 보내서 동작 확인.

### O5. 자동 업데이트 채널 구조

- **옵션 A**: Stable only (Sparkle 단일 appcast)
- **옵션 B**: Stable + Nightly (cmux 패턴)
- **옵션 C**: Stable + Beta + Nightly 3채널

**권장**: B. M7 까지는 A, 출시 후 Nightly 도입.

---

## 14. 다음 액션 (즉시 실행 가능)

### 14.1 리서치 기반 검증 스파이크 (M0 이전, ~3일)

형님이 DESIGN.v3 을 승인하시면 즉시 실행 가능한 검증 작업:

1. **스파이크 1 — `claude --output-format stream-json` 기본 동작 확인 (1일)**
   - bash 에서 직접 실행: `echo '{"type":"user_message","content":{"text":"Hello"}}' | claude --output-format stream-json --input-format stream-json`
   - 출력 스트림을 jq 로 파싱해 `SDKMessage` 포맷 실측
   - `initialize` control request 송신 수동 시뮬레이션

2. **스파이크 2 — `GhosttyKit.xcframework` 빌드 (1일)**
   - Ghostty submodule clone
   - `zig build -Demit-xcframework=true`
   - 성공 시 최소 Xcode 프로젝트에서 `import GhosttyKit` + 단일 PTY 표시

3. **스파이크 3 — `claude server` Direct Connect (1일)**
   - `claude server --help` 로 실제 CLI 존재 확인
   - Unix socket 또는 localhost 서버 기동 시도
   - WebSocket 연결 + 1회 메시지 왕복

세 스파이크 결과로 O2 / O3 / O4 에 대한 답을 얻음.

### 14.2 인터뷰 (남은 결정 사항 5개)

O1~O5 에 대해 소크라테스 인터뷰 형식으로 답변 수집. 주로 기술 검증 이후 결정.

### 14.3 M0 착수 (검증 완료 후, 1주)

§9 의 M0 상세 분해대로 진행.

### 14.4 커뮤니티 신호 확인

- `modu-ai/moai-adk` README 에 moai-cli 로드맵 섹션 추가 (초기 관심 수집)
- Twitter / HN "Show HN: moai-cli is coming" 예고 포스팅 (M4 완료 후)
- cmux 팀에 friendly outreach — 협업 여지 탐색 (GPL-3.0 vs MIT 라이선스 차이로 직접 코드 공유는 어려우나 아이디어 공유 가능)

---

## 15. 요약 — 한 페이지 Executive Summary

### 제품
macOS 네이티브 IDE-쉘. moai-adk 의 공식 GUI. MIT 라이선스.

### 차별화
cmux 가 터미널 + 브라우저를 잘 한다면, moai-cli 는 그 위에 **SPEC/TRUST/@MX/Kanban 워크플로우** 와 **Claude Code SDK 1급 임베드** 를 올린다.

### 3가지 핵심 피벗 (v2 → v3)
1. Claude Code 를 **SDK 라이브러리로 임베드** (PTY 관찰자 ❌)
2. **Rust core 제거, Pure Swift** (~3-7주 일정 단축)
3. **Hybrid 배포**: Plugin + Native Shell 한 저장소

### 7가지 moat (경쟁사 0)
1. Kanban + SPEC + worktree 자동화
2. Agent Run Viewer (27 hook 이벤트 실시간)
3. Memory Viewer (`~/.claude/projects/`)
4. @MX 태그 거터 + TRUST 5 게이지
5. Tool input rewriting
6. Native permission dialog + persist rules
7. In-process MCP (Claude 가 moai-cli UI 를 직접 조작)

### 기술 스택
- Swift + AppKit + SwiftUI (macOS 14+)
- GhosttyKit.xcframework
- SwiftTreeSitter + GRDB.swift + SwiftGit2 (+ Down for markdown)
- Swift 자체 구현 Claude Code SDK client
- 27 hook callback registry + In-process MCP server
- Sparkle 자동 업데이트

### 일정
- M0 Spike: 1주 (v2 의 2주에서 단축)
- M1~M7: 19주
- 총: **20주** (v2 의 23-27주에서 3-7주 단축)

### 첫 3일
1. stdin/stdout SDK 스트림 수동 검증 (1일)
2. GhosttyKit.xcframework 빌드 (1일)
3. `claude server` Direct Connect 존재 검증 (1일)

세 스파이크 통과 → M0 공식 착수.

---

**Version**: 3.0.0
**Status**: Draft for review
**Last Updated**: 2026-04-11
**Authors**: GOOS + Claude (Opus 4.6)
**Supersedes**: DESIGN.md (v2, 2026-04-11 morning)
**Referenced research**: research/R1, B1, B2, B3

**리뷰 요청:** 형님이 이 v3 을 승인하시면 §14.1 의 3가지 검증 스파이크로 즉시 착수. 검증 통과 시 M0 1주 스프린트 시작.
