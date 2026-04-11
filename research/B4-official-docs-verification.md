# B4 — Claude Agent SDK + Claude Code 공식 문서 재검증

> **Date**: 2026-04-11
> **Method**: docs.anthropic.com / code.claude.com 공식 문서 직접 fetch
> **Method**: Agent (general-purpose), WebSearch + WebFetch only, 소스 코드 참조 금지
> **Purpose**: B1/B2/B3 에서 소스 코드 분석으로 도출한 결론을 공식 문서로 교차 검증

---

## Executive Summary

1. **"Claude Code SDK" 는 공식적으로 "Claude Agent SDK" 로 리네임** 되었음. 정규 도메인은 `code.claude.com`. `platform.claude.com` 은 307 리디렉트.
2. **Agent SDK 는 Python + TypeScript 2개 언어만** 공식 지원. Swift/Rust/Go/Java/Kotlin **없음**. Agent SDK 조차 내부에서 `claude` CLI 를 subprocess 로 spawn 함.
3. **Rust/Swift 에서 Claude Code 를 "임베드" 하는 공식 경로는 없음.** 유일한 공식 경로 = `claude -p --output-format stream-json --bare` subprocess.
4. **공식 IDE 통합 패턴**: IDE 가 `127.0.0.1` 로컬 MCP 서버 + `~/.claude/ide/{port}.lock` 을 열고 Claude Code CLI 가 자동 연결. VS Code / JetBrains 가 이미 이 패턴 사용.
5. **Hook 이벤트 카탈로그**: SDK docs = 18개, Plugin reference = 25개, 우리가 소스에서 찾은 것 = 27개. 18~25 는 공식, 나머지는 내부/실험적.
6. **브랜딩 제약**: "Claude Code" 는 제품명/UI/ASCII art 에 사용 금지. "Claude Agent" / "Claude" 는 허용.
7. **claude.ai 로그인은 제3자 제품에서 금지.** `ANTHROPIC_API_KEY` 또는 Bedrock/Vertex/Foundry.

---

## Part 1: Claude Agent SDK

### 정의 (overview 페이지 인용)

> "Build production AI agents with Claude Code as a library. Build AI agents that autonomously read files, run commands, search the web, edit code, and more. The Agent SDK gives you the same tools, agent loop, and context management that power Claude Code, programmable in **Python and TypeScript**."

### 언어 지원

| 언어 | 패키지 | 설치 | 근거 |
|---|---|---|---|
| Python | `claude-agent-sdk` | `pip install claude-agent-sdk` | [overview](https://code.claude.com/docs/en/agent-sdk/overview) |
| TypeScript | `@anthropic-ai/claude-agent-sdk` | `npm install @anthropic-ai/claude-agent-sdk` | [overview](https://code.claude.com/docs/en/agent-sdk/overview) |

**Swift / Rust / Go / Java / Kotlin / C# 바인딩 없음.** `code.claude.com/docs/llms.txt` 전체 인덱스에서 확인. SDK 레퍼런스 페이지 = `python` 과 `typescript` 2개뿐.

### Agent SDK 는 `claude` CLI 를 대체하지 않는다

공식 문서 인용 ([hosting](https://code.claude.com/docs/en/agent-sdk/hosting)):

> "Node.js (required by the bundled Claude Code CLI that the SDK spawns; both SDK packages include it, so no separate install is needed)."

Agent SDK 의 TypeScript Options 타입에는 다음 필드가 존재:
- `pathToClaudeCodeExecutable: string` — 어떤 `claude` 바이너리를 spawn 할지
- `executable: 'bun' | 'deno' | 'node'` — JS 런타임 선택
- `executableArgs: string[]`
- `spawnClaudeCodeProcess: (options) => SpawnedProcess` — 커스텀 spawn 로직

**결론**: Agent SDK = `claude` CLI 의 타입 안전 래퍼. CLI 를 spawn 하지 않는 경로는 없음.

### moai-cli 에 주는 의미

Rust/Swift host 에서 Claude Code 를 사용하려면 3가지 옵션:

1. **Python/Node embed**: Swift/Rust 안에 Python 또는 Node 런타임 embed → `claude_agent_sdk.query()` 호출. 런타임 의존성 재도입. **비추**.
2. **`claude -p --output-format stream-json` 직접 spawn**: Agent SDK 가 내부에서 하는 것을 직접 함. stdin/stdout 의 JSON 프로토콜이 실제 interop 경계. **권장**.
3. **자체 Agent SDK 구현**: Rust/Swift 에서 같은 JSON 프로토콜을 speak. 다만 wire 포맷 일부는 [UNVERIFIED 공식 미문서화]. Reverse engineering 부담.

### Agent SDK 의 Hook 이벤트 카탈로그 (공식)

TypeScript SDK hooks 페이지 [hooks](https://code.claude.com/docs/en/agent-sdk/hooks) 에서 **18개 이벤트** 확인:

`PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `UserPromptSubmit`, `Stop`, `SubagentStart`, `SubagentStop`, `PreCompact`, `PermissionRequest`, `SessionStart`, `SessionEnd`, `Notification`, `Setup`, `TeammateIdle`, `TaskCompleted`, `ConfigChange`, `WorktreeCreate`, `WorktreeRemove`

Plugin reference 페이지 [plugins-reference](https://code.claude.com/docs/en/plugins-reference) 에서 **25개 이벤트** 확인 (SDK 18개 + 다음):

`PermissionDenied`, `TaskCreated`, `StopFailure`, `InstructionsLoaded`, `CwdChanged`, `FileChanged`, `PostCompact`, `Elicitation`, `ElicitationResult`

B2 에서 소스 코드로 도출한 27개 중 2개 (`StatusLine`, `FileSuggestion`) 는 공식 어디에도 없음 — 내부 전용.

### Hook Output Contract (공식 인용)

Hooks 페이지 인용:

> "Your callback returns an object with two categories of fields: Top-level fields control the conversation: `systemMessage` injects a message into the conversation visible to the model, and `continue` (`continue_` in Python) determines whether the agent keeps running after this hook. **`hookSpecificOutput` controls the current operation.** The fields inside depend on the hook event type. For `PreToolUse` hooks, this is where you set `permissionDecision` (`"allow"`, `"deny"`, or `"ask"`), `permissionDecisionReason`, and **`updatedInput`**. For `PostToolUse` hooks, you can set `additionalContext` to append information to the tool result."

**확인 완료**:
- ✅ `hookSpecificOutput` 존재
- ✅ `permissionDecision: 'allow' | 'deny' | 'ask'`
- ✅ `permissionDecisionReason`
- ✅ `PreToolUse.updatedInput` (tool 입력 rewrite)
- ✅ `PostToolUse.additionalContext`

**미확인 (공식 미문서화, 주의 필요)**:
- ⚠️ `updatedPermissions` — 소스에만 존재, 공식 페이지 미언급
- ⚠️ `PostToolUse.updatedMCPToolOutput` — 소스에만 존재
- ⚠️ `SessionStart.initialUserMessage` + `watchPaths` — 공식 미문서화

### Permission API (공식 인용)

[user-input](https://code.claude.com/docs/en/agent-sdk/user-input) 에서 `canUseTool` 시그니처:

```typescript
canUseTool: async (toolName, input, { signal, suggestions }) => {
  return { behavior: "allow", updatedInput: input };
  // or
  return { behavior: "deny", message: "User denied" };
}
```

6 permission modes: `default`, `dontAsk`, `acceptEdits`, `bypassPermissions`, `plan`, `auto`.

Evaluation order ([permissions](https://code.claude.com/docs/en/agent-sdk/permissions)):
1. Hooks 먼저
2. Deny rules (`bypassPermissions` 에서도 유지)
3. Permission mode
4. Allow rules
5. `canUseTool` callback

---

## Part 2: Claude Code Headless / Stream-JSON

### 공식 용어 변경

[headless](https://code.claude.com/docs/en/headless) 에서 인용:

> "The CLI was previously called 'headless mode.' The -p flag and all CLI options work the same way."

현재 공식 용어: **"Run Claude Code programmatically"** 또는 **"Agent SDK via the CLI"**. "SDK mode" 는 공식 용어 아님. "Stream-json output format" 이 프로토콜 이름.

### 공식 Startup Command

```bash
# 권장 (곧 기본값이 될 flag)
claude --bare -p "Summarize this file" --allowedTools "Read"

# Stream-JSON 출력
claude -p "Write a poem" --output-format stream-json --verbose --include-partial-messages
```

공식 인용:

> "`--bare` is the recommended mode for scripted and SDK calls, and will become the default for `-p` in a future release."

`--bare` 가 비활성화하는 것들:
- 암묵적 hooks 자동 로드
- Skills 자동 로드
- Plugins 자동 로드
- MCP 서버 자동 로드
- Auto memory / CLAUDE.md 자동 로드
- OAuth / keychain 읽기

Bare 모드에서 context 주입하는 flag:
- `--append-system-prompt <text>` / `--append-system-prompt-file <path>`
- `--settings <file-or-json>` — settings.json 로드
- `--mcp-config <file-or-json>` — MCP 서버 등록
- `--agents <json>` — 커스텀 에이전트 정의
- `--plugin-dir <path>` — 플러그인 디렉토리
- `--allowedTools "Read,Edit,Bash"` — 허용 tool 목록
- `--resume <session_id>` / `--continue` — 세션 재개
- `--output-format {text|json|stream-json}`
- `--json-schema <schema>` — 구조화 출력

### Plugin Manifest Schema (공식)

[plugins-reference](https://code.claude.com/docs/en/plugins-reference) 에서 공식 `.claude-plugin/plugin.json` 스키마:

```json
{
  "name": "plugin-name",            // 필수
  "version": "1.2.0",
  "description": "...",
  "author": { "name", "email", "url" },
  "homepage": "...",
  "repository": "...",
  "license": "MIT",
  "keywords": [],
  "skills": "./custom/skills/",
  "commands": ["./custom/commands/special.md"],
  "agents": "./custom/agents/",
  "hooks": "./config/hooks.json",
  "mcpServers": "./mcp-config.json",
  "outputStyles": "./styles/",
  "lspServers": "./.lsp.json",       // ★ 공식 first-class
  "userConfig": { /* user prompts at install */ },
  "channels": [ /* Slack/Telegram-style message channels */ ]
}
```

환경 변수:
- `${CLAUDE_PLUGIN_ROOT}` — 플러그인 설치 디렉토리
- `${CLAUDE_PLUGIN_DATA}` — `~/.claude/plugins/data/{id}/` 영구 저장소

---

## Part 3: Claude Code Embedding — 공식 권장 패턴

### VS Code Extension 패턴 (공식)

[vs-code](https://code.claude.com/docs/en/vs-code) 에서 공식 인용:

> "The extension includes the CLI (command-line interface), which you can access from VS Code's integrated terminal for advanced features."

그리고 가장 중요한 부분:

> "When the extension is active, it runs a **local MCP server** that the CLI connects to automatically. This is how the CLI opens diffs in VS Code's native diff viewer, reads your current selection for @-mentions, and — when you're working in a Jupyter notebook — asks VS Code to execute cells. The server is named `ide` and is hidden from `/mcp` because there's nothing to configure."
>
> "The server binds to `127.0.0.1` on a random high port and is not reachable from other machines. Each extension activation generates a fresh random auth token that the CLI must present to connect. The token is written to a lock file under `~/.claude/ide/` with `0600` permissions in a `0700` directory."

### JetBrains Plugin 패턴

[jetbrains](https://code.claude.com/docs/en/jetbrains) 에서:

> "Run `claude` from your IDE's integrated terminal, and all integration features will be active."

JetBrains 플러그인 = 터미널 wrapper + 같은 `ide` MCP 서버 기법.

### 정리: 공식 IDE 통합 패턴

```
IDE 프로세스 (VS Code / JetBrains / moai-cli)
  │
  ├── 기동 시:
  │    ├── 127.0.0.1 에 로컬 MCP 서버 바인드 (random high port)
  │    ├── random auth token 생성
  │    └── ~/.claude/ide/{port}.lock 드롭 (0600, 0700)
  │
  ├── Claude Code CLI 스폰:
  │    └── claude -p --bare --mcp-config ... 등
  │
  └── Claude 가 MCP 로 IDE 에게 역방향 콜:
       ├── mcp__ide__getDiagnostics()
       ├── mcp__ide__openDiff(...)
       ├── mcp__ide__executeCode(...)
       └── mcp__ide__readSelection()
```

**이것이 Anthropic 이 공식적으로 지원하는 모든 IDE 통합의 패턴입니다.** moai-cli 가 VS Code / JetBrains 처럼 동작하면 됩니다.

### VS Code 의 URI Handler

```
vscode://anthropic.claude-code/open?prompt=...&session=...
```

외부 도구가 URL 하나로 VS Code 에서 Claude 세션을 열 수 있음. moai-cli 도 유사하게 `moai-cli://open?...` URI 핸들러를 등록 가능.

---

## Part 4: moai-cli v4 에 주는 영향 (7가지 변경)

### 4.1 Subprocess 회피는 포기 — 공식 경로는 오직 subprocess

Agent SDK 조차 `claude` CLI 를 spawn 합니다. Rust/Swift 에서 no-subprocess 경로는 **공식적으로 존재하지 않습니다**. v3 에서 "SDK 임베드" 라고 표현한 것을 **정정해야 합니다** — moai-cli 는 Claude Code 를 **subprocess 로 관리하는 호스트** 입니다.

변경 표현:
- v3: "Claude Code 를 SDK 라이브러리로 임베드"
- v4: **"Claude Code 를 subprocess 로 spawn 하고 stream-json 프로토콜로 제어"**

### 4.2 `--bare` 를 기본값으로 사용

Anthropic 이 명시적으로 권장:
- `--bare` + `--settings` 로 context 명시적 로드
- CLAUDE.md 자동 로드 금지 (deterministic 동작 보장)
- 향후 `-p` 의 기본이 될 예정 → 지금부터 사용

### 4.3 IDE Server Pattern 을 PRIMARY 경로로

v3 의 "IDE Lockfile Impersonation = fallback" → v4 의 "IDE Server Pattern = PRIMARY". 이것이 공식 문서화된 유일한 IDE 통합 경로입니다.

아키텍처:
1. moai-cli 가 기동 시 `127.0.0.1:<random>` 로컬 MCP 서버 바인드
2. `~/.claude/ide/<port>.lock` 드롭 (0600, `workspaceFolders`, `authToken`, `ideName: "MoAI"`, `transport: "ws"`)
3. moai-cli 가 `claude` subprocess spawn 시 자동 연결됨
4. Claude 가 `mcp__moai__*` tool 을 호출해 moai-cli UI 조작

**이 패턴의 장점:**
- 완전 공식 지원
- Rust/Swift 언어 무관 (HTTP + token 만 구현하면 됨)
- 외부에서 실행된 Claude 세션도 같은 cwd 에 있으면 자동 연결
- MCP 프로토콜은 공개 표준, wire 형식 안정

### 4.4 Hook 브리징은 shell command + `http` 타입 사용

In-process hook callback 은 Python/TS SDK 의 특권. Rust/Swift host 는 **shell command hooks** 또는 **`http` hook type** 을 써야 합니다.

권장 구성:
1. Plugin manifest 에서 27 이벤트에 대한 hook 등록
2. Hook 타입은 **`http`** — POST to `http://127.0.0.1:<moai_port>/hooks/<event>`
3. moai-cli 의 로컬 MCP 서버 (또는 같은 프로세스 내 별도 HTTP handler) 가 수신
4. `http` hook 의 `X-Auth-Token` 헤더로 인증 (IDE lockfile 의 token 재사용)
5. Response 로 `hookSpecificOutput` 반환 가능

이것이 shell wrapper script 를 피하면서도 언어 무관한 hook 통합 방법입니다.

### 4.5 LSP 는 `.lsp.json` plugin feature 로 무료 획득

[plugins-reference](https://code.claude.com/docs/en/plugins-reference) 에서 `.lsp.json` 이 plugin manifest 의 공식 first-class 필드:

```json
{
  "lspServers": "./lsp-servers.json"
}
```

`.lsp.json` 형식:
```json
{
  "gopls": { "command": "gopls", "args": ["serve"], "filetypes": ["go"] },
  "rust-analyzer": { "command": "rust-analyzer", "filetypes": ["rust"] }
}
```

**효과**: Claude 가 자동으로 `mcp__ide__getDiagnostics` 를 통해 LSP 진단을 받아옵니다. moai-cli 의 Code Viewer 는 별도 LSP 클라이언트 구현 없이 **같은 MCP tool 을 구독** 하여 진단을 표시할 수 있습니다.

v3 의 "SwiftLSPClient 또는 Rust tower-lsp 자체 구현" → v4 의 **"moai-cli-plugin 의 `.lsp.json` 에 선언만, 구현은 Claude Code 에 위임"**.

### 4.6 브랜딩 제약

공식 인용:

> "Unless previously approved, Anthropic does not allow third party developers to offer claude.ai login or rate limits for their products, including agents built on the Claude Agent SDK."

그리고:

> "Third parties may use 'Claude Agent' or 'Claude' in their products, but MUST NOT use 'Claude Code' or 'Claude Code Agent' names, or Claude Code-branded ASCII art."

**moai-cli 제품명 및 UI 에 적용:**
- ✅ OK: "moai-cli", "MoAI Agent IDE", "moai + Claude"
- ❌ NO: "moai Claude Code GUI", "Claude Code for MoAI", Claude Code ASCII 로고 차용
- ✅ OK: "Powered by Claude" 배지
- ❌ NO: "Official Claude Code extension" 주장

**인증**: moai-cli 는 `ANTHROPIC_API_KEY` / Bedrock / Vertex / Foundry 만 사용. Claude.ai OAuth 구현 금지.

### 4.7 `claude server` / `cc+unix://` 는 공식 미문서화 → 사용 금지

v3 에서 "Direct Connect 를 M2 이후 업그레이드" 로 적은 것을 **철회합니다**. `claude server` CLI, Direct Connect, Unix socket, `cc+unix://` 스킴은 공식 문서 어디에도 없습니다. 내부/실험적 기능으로 간주하고 의존하지 않습니다.

대신: **stdin/stdout stream-json 을 영구 주 transport** 로 사용. IDE Server Pattern (127.0.0.1 MCP) 이 "다중 클라이언트 / detached session" 같은 고급 사용 사례를 커버.

---

## Part 5: New Risks

| 리스크 | 완화 |
|---|---|
| `--bare` 가 향후 `-p` 기본값이 됨 | 명시적으로 `--bare` 를 항상 패스 |
| `spawnClaudeCodeProcess` 존재 = wire format 불안정 신호 | CLI version 범위 테스트 + pin |
| Managed Agents (Anthropic hosted 2026-04 beta) | 지켜보기. Subprocess 경로가 deprioritize 되면 재평가 |
| 브랜딩 제약 위반 → 법무 리스크 | 제품명/UI 에서 "Claude Code" 완전 배제 |
| claude.ai OAuth 금지 | API key 경로만 구현 |
| 공식 hook 목록 18~25 vs 소스 27 | 18개 핵심만 사용, 나머지는 best-effort |
| `--input-format stream-json` 공식 미문서화 | User input 은 표준 stdin 프로토콜로 fallback 가능 유지 |

---

## Part 6: New Opportunities

1. **TypeScript V2 preview API** — moai-cli 가 Node 브리지를 쓰면 훨씬 단순한 session 모델 사용 가능
2. **`AskUserQuestion.previewFormat: "html"`** — Claude 가 HTML preview 를 넣어 moai-cli 가 네이티브 렌더링. 독창적 UX.
3. **Plugin `channels`** — Slack/Telegram 같은 메시지 채널을 plugin 에서 선언 가능. moai-cli 의 teammate 알림에 활용.
4. **`${CLAUDE_PLUGIN_DATA}`** — 플러그인 업데이트에도 유지되는 공식 데이터 디렉토리. moai-cli 상태 저장 위치.
5. **`dontAsk` permission mode** — bypass 보다 깔끔. 허용 목록에 없으면 자동 거절 (CI 친화적).
6. **Structured outputs** (`--output-format json --json-schema`) — SPEC 생성 결과를 스키마 검증 JSON 으로.

---

## Source Inventory

### Claude Agent SDK (canonical: code.claude.com)
- https://code.claude.com/docs/en/agent-sdk/overview
- https://code.claude.com/docs/en/agent-sdk/typescript
- https://code.claude.com/docs/en/agent-sdk/python
- https://code.claude.com/docs/en/agent-sdk/typescript-v2-preview
- https://code.claude.com/docs/en/agent-sdk/hooks
- https://code.claude.com/docs/en/agent-sdk/permissions
- https://code.claude.com/docs/en/agent-sdk/user-input
- https://code.claude.com/docs/en/agent-sdk/mcp
- https://code.claude.com/docs/en/agent-sdk/plugins
- https://code.claude.com/docs/en/agent-sdk/streaming-vs-single-mode
- https://code.claude.com/docs/en/agent-sdk/hosting

### Claude Code CLI
- https://code.claude.com/docs/en/headless
- https://code.claude.com/docs/en/plugins-reference
- https://code.claude.com/docs/en/setup

### IDE Integrations
- https://code.claude.com/docs/en/vs-code
- https://code.claude.com/docs/en/jetbrains

### Index
- https://code.claude.com/docs/llms.txt (공식 전체 문서 인덱스)
