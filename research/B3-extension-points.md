# B3 — Claude Code Extension Points 분석

> **Date**: 2026-04-11
> **Source**: `/Users/goos/moai/claude-code-map/src/`
> **Method**: 정적 분석, 파일:라인 인용
> **Agent**: general-purpose, B3 stream

---

## Executive Finding

**Plugin manifest (`plugin.json`) 가 Claude Code 의 진짜 확장 표면.** 한 개의 manifest 가 제공 가능한 것:

- Hooks (27 이벤트 전부)
- Commands (슬래시 커맨드)
- Agents (서브에이전트)
- Skills
- Output styles
- **MCP servers (in-process `type: 'sdk'` 포함)**
- LSP servers
- Whitelisted settings

**그리고 무료 namespace 획득**: `/moai-cli:*` 슬래시 커맨드, `moai-cli:*` agent 이름.

**핵심 발견 3가지:**
1. **`type: 'sdk'` MCP 서버** = `InProcessTransport` 로 Claude Code 와 같은 프로세스에서 실행 → subprocess 0, 소켓 0
2. **IDE lockfile impersonation** = `~/.claude/ide/<port>.lock` 에 moai-cli 가 자기 자신을 IDE 로 등록 → 모든 Claude Code 인스턴스가 자동으로 WebSocket MCP 로 연결
3. **Memory 는 평범한 markdown** = `~/.claude/projects/<sanitized-git-root>/memory/` 에 `MEMORY.md` + topic 파일들. Parser 불필요, moai-cli 가 직접 read/write 가능

---

## 1. Plugin System

### Manifest Schema (`utils/plugins/schemas.ts:884-898`)

```ts
PluginManifestSchema = z.object({
  ...metadata,              // name, version, description, author, etc
  ...hooks,                 // HooksSettings | path to hooks.json
  ...commands,              // string | string[] | Record<name, CommandMetadata>
  ...agents,                // path(s) to .md
  ...skills,                // directory path(s)
  ...outputStyles,          // dir/file path(s)
  ...channels,              // MCP-backed message channels
  ...mcpServers,            // inline | .mcp.json | MCPB file
  ...lspServers,            // inline | .lsp.json
  ...settings,              // allowlist (currently only `agent`)
  ...userConfig,            // user-prompted values with type/title/sensitive
})
```

**Unknown top-level 필드는 silently strip** (schemas.ts:876-883).

### PluginComponent Union (`types/plugin.ts:72-77`)

```ts
type PluginComponent =
  | 'commands'
  | 'agents'
  | 'skills'
  | 'hooks'
  | 'output-styles'
```

`LoadedPlugin` (`types/plugin.ts:48-70`) 에 추가로:
- `mcpServers: Record<string, McpServerConfig>`
- `lspServers`
- merged `settings`

### Registration Lifecycle

**2-tier:**

1. **Built-in plugins** (`src/plugins/builtinPlugins.ts:28-32`):
   ```ts
   registerBuiltinPlugin(definition: BuiltinPluginDefinition)
   ```
   `src/plugins/bundled/index.ts:20` 의 `initBuiltinPlugins()` — **현재 비어있음**. 스캐폴딩만 존재, Anthropic 은 built-in plugin 을 아직 제공 안 함.

2. **Marketplace plugins** (`/plugin` 으로 설치):
   - Sources (`schemas.ts:906-1044`): GitHub, git, npm, file, directory, URL, settings-inline
   - Loader: `loadAllPluginsCacheOnly()` → `enabled`/`disabled`/`errors` 로 분리
   - Auto-update: `performStartupChecks.tsx`, `pluginAutoupdate.ts`, `reconciler.ts`

### Namespacing — 무료!

**Plugin commands**: `{pluginName}:{namespace}:{commandBaseName}` (`loadPluginCommands.ts:77-95`)

**Plugin agents**: `{pluginName}:{namespace}:{baseName}` (`loadPluginAgents.ts:88-90`)

→ moai-cli plugin 을 `"name": "moai-cli"` 로 선언하면:
- `/moai-cli:kanban`, `/moai-cli:surface`, `/moai-cli:memory` 자동
- agent 이름 `moai-cli:kanban-manager`, `moai-cli:workspace-opener` 자동

### Security Boundaries

- Plugin agent `.md` frontmatter 는 **`permissionMode`, `hooks`, `mcpServers` 설정 불가** (`loadPluginAgents.ts:161-168` 에서 silently strip). Manifest level 에서만 가능 (install-time consent).
- Settings 화이트리스트: `PluginManifestSettingsSchema` (schemas.ts:857-867) 현재 `agent` 키만 허용.
- Sensitive `userConfig` 값 → macOS Keychain / `.credentials.json` (schemas.ts:608-616). Non-sensitive → `settings.json`.
- `pluginBlocklist.ts`, `pluginPolicy.ts`, `strictKnownMarketplaces` 로 enterprise 제한.

### 판결: Hybrid 전략 — Plugin + Native Shell

**moai-cli 는 둘 다 동시에 존재해야 합니다:**

1. **Plugin 형태** (`moai-cli-plugin/` 서브디렉토리 in moai-cli repo)
   - `.claude-plugin/plugin.json` with `name: "moai-cli"`
   - 무료 namespace (`/moai-cli:*`)
   - Install-time trust prompt (`userConfig` 필드)
   - Hooks + MCP + skills + agents + output-style 한 설치로
   - Auto-update via marketplace
   - Settings 는 `~/.claude/settings.json` 에 persist

2. **Native Shell 형태** (기존 DESIGN.md)
   - SwiftUI + AppKit + libghostty 로 UI 소유
   - `claude --output-format stream-json` 으로 Claude Code 스폰
   - Shell 내부에서 자체 plugin 을 자동 설치/연결

Plugin 은 **capability 제공자**, Shell 은 **UI 소유자**. 한 레포 안에 둘 다 거주.

---

## 2. Skill System

### Frontmatter Schema (`skills/loadSkillsDir.ts:185-265`)

| 필드 | 의미 |
|---|---|
| `name` / `description` | 모델 + 사용자 display |
| `when_to_use` | 모델에게 보이는 트리거 조건 |
| `paths` | CSV glob — 모델이 매칭 파일을 touch 한 후에만 skill 등장 |
| `argument-hint` / `arguments` | `/skill <args>` 도움말 + named arg substitution |
| `allowed-tools` | skill 실행 중 허용된 tool 들 (CSV) |
| `model` | `inherit` \| 명시적 모델 |
| `disable-model-invocation` | 모델에서 숨김, user-only |
| `user-invocable` | user typeahead 에서 숨김 |
| `hooks` | session-scoped hooks 등록 |
| `context` | `inline` (대화에 확장) \| `fork` (서브 에이전트로) |
| `agent` | `context: fork` 시 사용할 agent type |
| `effort` | Budget 조절 |
| `shell` | `!` command injection 용 shell |
| `version` | skill 버전 |

### Loading (`skills/loadSkillsDir.ts:407-480`)

- **디렉토리 형식만** 지원: `skill-name/SKILL.md` (`:424-427`)
- Sources: `projectSettings` (`.claude/skills/`), `userSettings` (`~/.claude/skills/`), `policySettings`, `plugin`, `bundled`, `mcp`
- 토큰 추정 for visibility: `estimateSkillFrontmatterTokens` (`:100-105`) — frontmatter 만 로드, body 는 invocation 시점에

### Bundled Skills (`skills/bundledSkills.ts:15-41`)

Plugin 에 programmatic skill 을 임베드 가능:
```ts
BundledSkillDefinition {
  ...schema fields,
  files: Record<string, string>  // invocation 시 first-time extract
}
```

File extraction 은 `O_EXCL|O_NOFOLLOW` + 0o600/0o700 + per-process nonce dir (`:169-194`).

### moai-adk 의 기존 스킬과의 관계

moai-adk 는 이미 `.claude/skills/moai/...` + `.claude/skills/agency/...` 에 ~60 스킬 보유. Project-scoped.

**moai-cli 권장:**
- 기존 스킬 그대로 유지 (moai-adk 가 로드)
- moai-cli plugin 은 **UI 전용 스킬** 소수 번들:
  - `moai-cli:open-workspace`
  - `moai-cli:focus-kanban`
  - `moai-cli:split-surface`
- 프로젝트 무관하게 항상 사용 가능

---

## 3. MCP Integration

### Configuration Scopes (`services/mcp/config.ts:908-989`)

| Scope | Source |
|---|---|
| `project` | `.mcp.json` walked up to FS root |
| `user` | `~/.claude.json` 의 `mcpServers` |
| `local` | `getCurrentProjectConfig().mcpServers` |
| `enterprise` / `managed` | Managed settings file |
| `dynamic` | Runtime-added via `/mcp` or `onChangeDynamicMcpConfig` |
| `claudeai` | `McpClaudeAIProxyServerConfigSchema` 의 proxy servers |
| Plugin-provided | Plugin manifest 의 `mcpServers`, scope flag `pluginSource` |

### Transport Types (`services/mcp/types.ts:23-26`)

```ts
z.enum(['stdio', 'sse', 'sse-ide', 'http', 'ws', 'sdk'])
```

Plus 내부: `ws-ide`, `claudeai-proxy`.

### 🔥 In-Process MCP — THE 킬러 기능

`services/mcp/InProcessTransport.ts:1-64`:

```ts
export function createLinkedTransportPair(): [Transport, Transport] {
  const a = new InProcessTransport()
  const b = new InProcessTransport()
  a._setPeer(b)
  b._setPeer(a)
  return [a, b]
}
```

Linked-pair transport 가 MCP 서버와 클라이언트를 **같은 프로세스**에서 실행 — **subprocess 0, 소켓 0**.

`McpSdkServerConfigSchema` (`types.ts:108-113`): `{type: 'sdk', name: string}`

**결과: plugin 이 Claude Code 프로세스 내부에서 실행되는 MCP 서버를 등록 가능.** 외부 프로세스 스폰 필요 없음. 메모리 공유.

### IDE Lockfile Discovery (`utils/ide.ts:298-393`)

Claude Code 는 `~/.claude/ide/*.lock` 을 scan. 각 lockfile JSON:

```json
{
  "workspaceFolders": ["/path/to/project"],
  "pid": 12345,
  "ideName": "VSCode",
  "transport": "ws",
  "runningInWindows": false,
  "authToken": "..."
}
```

파일 이름: `{port}.lock`. Workspace folder 가 cwd 를 포함하면 Claude Code 가 자동으로 `sse-ide` 또는 `ws-ide` transport 로 연결 (`client.ts:678-734`). Stale lockfile 은 startup 에서 sweep (`ide.ts:522-576`).

**🔥 moai-cli 는 IDE 를 impersonate 할 수 있습니다** — `~/.claude/ide/<moaiPort>.lock` 을 drop 하면 Claude Code 가 자동으로 moai-cli 의 MCP 서버에 연결하고 그 tools 를 모델에게 노출.

### moai-cli 가 노출할 UI Tools (제안)

- `Workspace.open({ path })` — focused workspace 전환
- `Kanban.createCard({ title, column, assignee })`
- `Kanban.moveCard({ id, to })`
- `AgentRun.focus({ agentId })`
- `Surface.reveal({ surface: 'memory' | 'kanban' | 'logs' })`
- `Notification.post({ title, body, level })` — 네이티브 macOS notification
- `File.revealInFinder({ path })`
- `Panel.splitHorizontal({ left, right })`
- `Terminal.spawn({ cmd, cwd, title })`

### 두 가지 배치 방법

1. **In-process via plugin + SDK transport**
   - Plugin manifest: `mcpServers: { "moai-cli-ui": { "type": "sdk", "name": "moai-cli-ui" } }`
   - Plugin 이 SDK control transport 를 통해 서버 등록
   - Zero subprocess, zero auth, shared memory
   
2. **Out-of-process via IDE lockfile**
   - moai-cli daemon 이 `~/.claude/ide/<port>.lock` drop
   - `transport: "ws"`, `ideName: "MoAI"`
   - 모든 Claude Code 세션 (CLI, VSCode, JetBrains) 자동 연결
   - **Plugin 설치 없이도 작동** (외부 실행된 Claude Code 도 catch)

---

## 4. Slash Commands

### 4가지 Command 타입 (`types/command.ts:17-206`)

1. **`PromptCommand`** (`type: 'prompt'`) — 템플릿 user message 로 확장. Skill 과 대부분 user command 가 이 타입.
2. **`LocalCommand`** (`type: 'local'`) — text 반환 또는 compaction 트리거. Built-in 만.
3. **`LocalJSXCommand`** (`type: 'local-jsx'`) — **lazy-loaded React component** 가 Ink TUI 내부에 렌더. `/plugin`, `/mcp`, `/agents`, `/ide` 가 이 타입.
4. **MCP commands** (`isMcp: true`) — MCP 서버가 제공하는 prompt 에서 auto-generate.

예시 (`commands/plugin/plugin.tsx:4-6`):
```tsx
export async function call(onDone, _context, args?) {
  return <PluginSettings onComplete={onDone} args={args} />
}
```

### LocalJSXCommandContext 기능 (`types/command.ts:80-98`)

JSX command 가 접근 가능:
- `setMessages(updater)` — 대화 이력 직접 mutate
- `onChangeAPIKey()` — API key flow 트리거
- **`onChangeDynamicMcpConfig(config)`** — **런타임에 MCP 서버 추가**
- `onInstallIDEExtension(ide)`
- `resume(sessionId, log, entrypoint)` — 다른 세션으로 점프
- 전체 `ToolUseContext` + `getAppState()` / `setAppState()` / `canUseTool`

`LocalJSXCommandOnDone` (`:117-126`) 는 `nextInput`, `submitNextInput`, `metaMessages` + display 모드 `'skip' | 'system' | 'user'`.

### Dialog / Modal 가능?

**YES, native.** `/plugin`, `/ide`, `/mcp`, `/agents` 는 모두 full-screen Ink UI 를 렌더 — select menu, dialog, form 모두 가능.

### Plugin Command 형식 (3가지) (`schemas.ts:429-452`)

1. Single path string
2. Path array
3. Object mapping with metadata:
   ```json
   {
     "about": {
       "source": "./README.md",
       "description": "...",
       "argumentHint": "...",
       "model": "...",
       "allowedTools": [...]
     }
   }
   ```

---

## 5. Output Styles

### Schema (`constants/outputStyles.ts:11-28`)

```ts
OutputStyleConfig = {
  name: string
  description: string
  prompt: string                    // 시스템 프롬프트 replace/augment
  source: SettingSource | 'built-in' | 'plugin'
  keepCodingInstructions?: boolean  // 기본 coding rules 유지 여부
  forceForPlugin?: boolean          // plugin 활성화 시 자동 적용
}
```

### Loader (`outputStyles/loadOutputStylesDir.ts:26-92`)

- Walks `.claude/output-styles/*.md` (project), `~/.claude/output-styles/*.md` (user), plugin dirs
- 각 markdown = 한 style. Frontmatter: name/description/keepCodingInstructions/forceForPlugin. Body: prompt.

### 판결

**System prompt 텍스트만 커스터마이즈 가능**. UI theme, message formatting, renderer 커스터마이즈 불가.

moai-adk 는 이미 `moai` output style 사용 중. **과투자 금지**. Agent Run Viewer 는 output style 로 구현하지 말고, **hook + MCP tools** 조합을 쓸 것.

**유일한 사용법:**
- Plugin 에 `forceForPlugin: true` 인 output style 제공
- Prompt body 에 "You are running inside moai-cli. Use `moai-cli-ui:*` MCP tools to drive the UI" 한 줄 추가
- 모델이 UI tool 들을 proactively 사용하도록 보장

---

## 6. Memory / Context Persistence

### Storage Layout (`memdir/paths.ts:85-259`)

**3단계 path resolution:**

1. **Env var override** (`CLAUDE_COWORK_MEMORY_PATH_OVERRIDE`) — absolute path, path-traversal 검증
2. **Settings override** (`autoMemoryDirectory` in user/policy/local settings, **NOT** projectSettings for security)
3. **Default**: `<memoryBase>/projects/<sanitized-git-root>/memory/` where `memoryBase = CLAUDE_CODE_REMOTE_MEMORY_DIR ?? ~/.claude`

**Canonical git root** 사용 → worktree 들이 같은 repo 의 memory 를 공유.

### 파일 구조

- `MEMORY.md` — top-level entrypoint, **200 라인 AND 25,000 바이트 제한** (`memdir.ts:35-38`), 초과 시 warning + truncate
- Topic files — arbitrary `.md` under same dir
- `logs/YYYY/MM/YYYY-MM-DD.md` — Kairos/assistant mode 일일 작업 로그
- Team memory (`feature('TEAMMEM')`) — `memdir/teamMemPaths.ts`

### Schema

`memdir/memoryTypes.ts` (22KB, 미상세 읽음) 에 `MEMORY_FRONTMATTER_EXAMPLE`, `TYPES_SECTION_INDIVIDUAL` 등 정의. Memory 는 **plain markdown with structured frontmatter** — 데이터베이스 아님.

### 판결

**moai-cli 가 이 파일들을 직접 read/write 할 수 있습니다.** Path 는 deterministic:

```
~/.claude/projects/<git-root-sanitized>/memory/MEMORY.md
~/.claude/projects/<git-root-sanitized>/memory/<topic>.md
~/.claude/projects/<git-root-sanitized>/memory/logs/2026/04/2026-04-11.md
```

**moai-cli 는 Memory Surface 를 ship 해야 합니다** — Claude Code 가 쓰는 같은 파일을 렌더. Parser 불필요, 그냥 markdown preview + edit.

moai-cli 는 이미 이 패턴을 사용 중: `/Users/goos/.claude/projects/-Users-goos-MoAI-moai-adk-go/memory/MEMORY.md`.

---

## 7. Subagents + Coordinator

### Plugin Agent Definition (`utils/plugins/loadPluginAgents.ts:65-229`)

Frontmatter 필드:
- `name` → `{pluginName}:{namespace}:{baseName}` 로 transform
- `description` / `when-to-use`
- `tools` (CSV)
- `skills` (CSV)
- `color` (AgentColorName)
- `model` (`inherit` | 명시적)
- `background: true` — non-blocking 실행
- `memory` (`none` | scope value) — 설정 시 `Read`/`Write`/`Edit` tool 자동 주입
- `isolation: "worktree"` — isolated git worktree 에 스폰
- `effort`
- `maxTurns`
- `disallowedTools`

**Plugin agents 는** `permissionMode`, `hooks`, `mcpServers` **설정 불가** (`:161-168` 에서 strip). Security 조치.

User-authored `.claude/agents/*.md` 는 모든 필드 가능 (plugin loader 와 parallel 한 project loader).

### `/agents` Command (`commands/agents/agents.tsx:6-11`)

`<AgentsMenu tools={...} onExit={onDone} />` 를 렌더. 단순 list 표시, 실제 spawn 은 `Agent`/`Task` tool 을 통해.

### moai-adk 의 26 에이전트와의 관계

moai-adk 는 이미 `.claude/agents/moai/*.md` 에 8 managers + 8 experts + 3 builders + 1 evaluator + 6 agency = 26 에이전트 ship 중. Project-scoped, user-authored (모든 frontmatter 필드 가능).

**moai-cli 권장:**
- Agent registry 재구현 금지
- **`SubagentStart`/`SubagentStop`/`PreToolUse`/`PostToolUse` hook 을 구독** 해서 실시간 Agent Run Viewer 구축
- 트리: Session → Agents → Tool calls → Outputs
- 각 agent = status/progress/model/tool count 포함 expandable card
- 클릭 시 full transcript

---

## 8. Brainstorm — moai-cli 설계 추가 제안

### Idea 1 — Hybrid 전략: Plugin + Native Shell

**Extension point**: Plugin manifest (`schemas.ts:884-898`)

**DESIGN.md 추가:**
- `moai-cli-plugin/` sub-directory in moai-cli repo
- `.claude-plugin/plugin.json` with `name: "moai-cli"`, `mcpServers: { "moai-cli-ui": { "type": "sdk", "name": "moai-cli-ui" } }`
- Commands: `/moai-cli:surface`, `/moai-cli:kanban`, `/moai-cli:memory`, `/moai-cli:agents`
- Install: `moai-cli install-plugin` → `~/.claude/plugins/moai-cli@local/` + enable in `~/.claude/settings.json`

**Effort**: Medium (packaging work)

### Idea 2 — In-process MCP Server for UI Tools

**Extension point**: `InProcessTransport.ts:1-64` + `McpSdkServerConfigSchema`

**DESIGN.md 추가**: `moai-cli-ui` MCP 서버 (SDK transport via in-process). Tools:
- `Workspace.open`, `Kanban.createCard`, `Kanban.moveCard`
- `AgentRun.focus`, `Surface.reveal`, `Notification.post`
- `Panel.splitHorizontal`, `Terminal.spawn`, `File.revealInFinder`

IPC channel (Unix socket 또는 stdio-pipe) 로 shell process 에 dispatch.

**Effort**: High — IPC 프로토콜 정의 + 각 tool 의 shell-side 구현

### Idea 3 — IDE Lockfile Impersonation

**Extension point**: `utils/ide.ts:298-393` + `sse-ide`/`ws-ide` transports

**DESIGN.md 추가**: moai-cli daemon 이 lockfile drop:
```json
{
  "workspaceFolders": ["<moai-cli workspace>"],
  "pid": <daemon-pid>,
  "ideName": "MoAI",
  "transport": "ws",
  "authToken": "<random>"
}
```

**효과**: 플러그인 설치 없이도 외부 실행된 Claude Code 인스턴스가 자동 연결.

**Effort**: Medium — WebSocket MCP 서버 + lockfile management

### Idea 4 — Local-JSX Slash Commands as Escape Hatches

**Extension point**: `types/command.ts:144-152` (`LocalJSXCommand`)

**DESIGN.md 추가**: Shell 미실행 시 fallback 으로 Claude Code TUI 안에 Ink React UI 렌더:
- `/moai-cli:kanban` → `<KanbanBoard />`
- `/moai-cli:memory` → `<MemoryViewer />`
- `/moai-cli:agents` → `<AgentRunViewer />`
- `/moai-cli:config` → `<ConfigPanel />`

Shell 이 실행 중이면 대신 IPC message 로 shell 에 redirect 후 "opened in moai-cli" 한 줄 표시.

**Caveat**: Plugin 이 `.tsx` JSX command 를 ship 가능한지 **미확인** (B3 report Section 9.2 참조). 만약 불가하면 이 아이디어는 moai-cli 가 Claude Code fork 또는 다른 injection 필요.

### Idea 5 — Memory Viewer Surface

**Extension point**: `memdir/paths.ts:223-235` — deterministic paths

**DESIGN.md 추가**: MemorySurface:
- `<memoryBase>/projects/<git-root>/memory/` 를 fs-events 로 watch
- `MEMORY.md` 를 index 로 렌더, 참조된 topic file 의 inline preview
- 일일 log scroll view
- Edit 버튼 → moai-cli editor pane 열림
- 25KB / 200 라인 cap 에 대한 progress bar + warning

**Effort**: Low — plain markdown, deterministic paths, fs-events, 기존 renderer

### Idea 6 — Hook Event Bus → Agent Run Viewer

**Extension point**: 27 hook 이벤트

**DESIGN.md 추가**:
- moai-cli daemon 이 loopback port listen
- moai-cli plugin 이 추가 hook handler 를 ship: 모든 `PreToolUse`, `PostToolUse`, `SubagentStart`, `SubagentStop`, `Notification`, `Stop` 을 `http://127.0.0.1:<port>/hook-event` 로 POST
- Daemon 이 live timeline tree 구축: Session → Agents → Tools
- AgentRunViewer surface 가 tree 렌더 (expand/collapse + filter)

**Effort**: Low-medium — handler 는 20 라인 shell script, daemon 은 WebSocket/SSE 서버 + tree reducer

### Idea 7 — userConfig 로 First-run Setup

**Extension point**: `schemas.ts:587-653` — plugin userConfig (string/number/boolean/directory/file, sensitive 값은 keychain)

**DESIGN.md 추가**: moai-cli plugin userConfig:
- `moai_cli_daemon_port` (number, default 7428)
- `moai_cli_auth_token` (string, sensitive: true → keychain)
- `moai_cli_workspace_dir` (directory)
- `moai_cli_enable_notifications` (boolean)

Claude Code 가 install 시 `PluginOptionsDialog` 로 prompt. In-process MCP 서버가 `${user_config.moai_cli_auth_token}` 로 접근.

**Effort**: Very low — manifest 선언만

### Idea 8 — Skill `paths` Gating for Context-Aware Surfaces

**Extension point**: `loadSkillsDir.ts:159-178` — paths 매칭 파일 touch 후에만 skill 등장

**DESIGN.md 추가**: Path-aware skills:
- `skills/moai-cli-git-ops/SKILL.md` with `paths: "**/.git/HEAD"` → git 연산 후 자동 등장, GitSurface 열림
- `skills/moai-cli-spec-focus/SKILL.md` with `paths: ".moai/specs/**"` → SPEC 파일 touch 시 등장, SPEC viewer + adjacent code preview
- `skills/moai-cli-db-console/SKILL.md` with `paths: "**/*.sql,**/schema.prisma"` → DB Console surface

**Effort**: Low — 각 skill 은 짧은 markdown

### Idea 9 — Plugin Output Style as Session Marker

**Extension point**: `forceForPlugin: true`

**DESIGN.md 추가**: `.claude/output-styles/moai-cli.md` with `force-for-plugin: true`, `keep-coding-instructions: true`. Prompt body:
> "You are running inside moai-cli. You can use the `moai-cli-ui:*` MCP tools to drive the UI."

모델이 UI tools 를 proactively 사용하도록 보장.

**Effort**: Trivial — 단일 markdown file

### Idea 10 — Settings Redirect via Local-JSX

**Extension point**: `types/command.ts:89-92` — `onChangeDynamicMcpConfig`

**DESIGN.md 추가**: `/moai-cli:connect` (10 라인 JSX command):
1. Local port 에서 moai-cli daemon 감지
2. `onChangeDynamicMcpConfig({ "moai-cli-ui": { type: "ws", url: "ws://127.0.0.1:7428/mcp", ... } })`
3. 연결된 tools 로 confirmation 렌더
4. Claude Code restart 또는 settings.json 편집 불필요

**효과**: Plugin install 없이 기존 Claude Code 세션에 moai-cli UI tools "one-command attach"

**Caveat**: Idea 4 와 같은 플러그인 contribution 문제 — `.tsx` plugin command 가능 여부 미확인.

---

## 9. Facts I Could Not Verify

1. **SDK control transport 의 정확한 semantics** — `InProcessTransport` 와 `McpSdkServerConfigSchema` 의 실제 wiring 은 `services/mcp/SdkControlTransport.ts` 에 있으나 full read 안 됨
2. **`local-jsx` commands 가 plugin contribution 으로 가능한지** — Plugin command 는 markdown (`loadPluginCommands.ts`) 에서 로드되므로 `.tsx` 파일 ship 경로가 불명확. Built-in `/plugin`, `/agents`, `/mcp` 는 `src/commands.ts` (hardcoded) 에 있음
3. **Plugin-manifest-level hooks vs skill frontmatter hooks session scoping** — `loadPluginHooks.ts` 미독
4. **LSP plugin integration** — `LspServerConfigSchema` 존재하나 `lspPluginIntegration.ts` 미독
5. **Memory schema 상세** — `memdir/memoryTypes.ts` (22KB) 전체 미독
6. **Output style runtime swapping** — `clearOutputStyleCaches()` 존재하나 active session 중 invoke 시 re-render 여부 불명
7. **27 hook events 수** — B2 stream 이 담당. 내가 확인한 것은 `registerFrontmatterHooks.ts:1` 에서 `HOOK_EVENTS` import
8. **`commands/mcp/addCommand.ts` 동작** — 10KB file 미독, `/mcp add` runtime MCP 등록 구현

---

## Source Inventory

- `src/utils/plugins/schemas.ts` (plugin manifest schema)
- `src/types/plugin.ts` (LoadedPlugin, PluginComponent types)
- `src/plugins/builtinPlugins.ts`, `src/plugins/bundled/index.ts`
- `src/utils/plugins/loadPluginCommands.ts`, `loadPluginAgents.ts`, `loadPluginHooks.ts`, `loadPluginSkills.ts`
- `src/utils/plugins/performStartupChecks.tsx`, `pluginAutoupdate.ts`, `reconciler.ts`
- `src/skills/loadSkillsDir.ts`, `bundledSkills.ts`
- `src/services/mcp/config.ts`, `types.ts`, `InProcessTransport.ts`, `SdkControlTransport.ts`
- `src/utils/ide.ts` (lockfile scan)
- `src/types/command.ts` (LocalJSXCommand definitions)
- `src/commands/plugin/plugin.tsx`, `commands/agents/agents.tsx`
- `src/outputStyles/loadOutputStylesDir.ts`, `constants/outputStyles.ts`
- `src/memdir/paths.ts`, `memdir/memdir.ts`
