# B2 — Hook Events + Tool/Permission System 분석

> **Date**: 2026-04-11
> **Source**: `/Users/goos/moai/claude-code-map/src/`
> **Method**: 정적 분석, 파일:라인 인용
> **Agent**: general-purpose, B2 stream

---

## Executive Finding

moai-adk 가 사용하는 "27 hook 이벤트" 는 정확히 일치 (검증 완료). 그러나:

1. **Hook 은 단순 이벤트 알림이 아닙니다** — `hookSpecificOutput.updatedInput` 으로 **tool 입력을 런타임에 rewrite** 할 수 있고, `updatedMCPToolOutput` 으로 **MCP 응답을 rewrite** 할 수 있고, `additionalContext` 로 **컨텍스트를 주입** 할 수 있습니다.
2. **Hook 타입은 4가지**: `command` (shell), `prompt` (LLM 평가), `agent` (sub-agent verifier), `http` (POST URL). **moai-adk 는 command 만 사용 중.**
3. **SDK `initialize.hooks`** 를 통해 moai-cli 가 **in-process callback** 으로 hook 을 등록할 수 있습니다 → shell wrapper .sh 파일 전체 제거 가능.
4. **`async: true` + `asyncRewake: true`** 가 TeammateIdle blocking 메커니즘의 핵심 — moai-adk 의 기존 구현과 정확히 일치.

---

## 1. Full Hook Event Catalog (27 events)

정규 목록: `src/entrypoints/sdk/coreSchemas.ts:355-383` 의 `HOOK_EVENTS` — **정확히 27개**.

`BaseHookInputSchema` (`coreSchemas.ts:387-411`) 는 모든 이벤트에 mix-in:
```ts
session_id: string
transcript_path: string
cwd: string
permission_mode?: string
agent_id?: string          // subagent 에서만 존재
agent_type?: string        // --agent 세션 + subagent
```

### 27 이벤트 테이블

스키마 위치는 모두 `coreSchemas.ts`, dispatch site 는 `utils/hooks.ts` 또는 `services/tools/toolExecution.ts`.

| # | Event | Schema (file:line) | Payload | matcher 대상 |
|---|---|---|---|---|
| 1 | `PreToolUse` | coreSchemas.ts:414-423 | `tool_name, tool_input, tool_use_id` | `tool_name` |
| 2 | `PostToolUse` | coreSchemas.ts:436-446 | `tool_name, tool_input, tool_response, tool_use_id` | `tool_name` |
| 3 | `PostToolUseFailure` | coreSchemas.ts:448-459 | `tool_name, tool_input, tool_use_id, error, is_interrupt?` | `tool_name` |
| 4 | `Notification` | coreSchemas.ts:473-482 | `message, title?, notification_type` | `notification_type` |
| 5 | `UserPromptSubmit` | (hooks.ts:100 import) | `prompt: string` | none |
| 6 | `SessionStart` | coreSchemas.ts:493-502 | `source: 'startup'|'resume'|'clear'|'compact', agent_type?, model?` | `source` |
| 7 | `SessionEnd` | coreSchemas.ts:758-765 | `reason: 'clear'|'resume'|'logout'|'prompt_input_exit'|'other'|'bypass_permissions_disabled'` | `reason` |
| 8 | `Stop` | coreSchemas.ts:513-527 | `stop_hook_active, last_assistant_message?` | none |
| 9 | `StopFailure` | coreSchemas.ts:529-538 | `error, error_details?, last_assistant_message?` | `error` |
| 10 | `SubagentStart` | coreSchemas.ts:540-548 | `agent_id, agent_type` | `agent_type` |
| 11 | `SubagentStop` | coreSchemas.ts:550-567 | `stop_hook_active, agent_id, agent_transcript_path, agent_type, last_assistant_message?` | `agent_type` |
| 12 | `PreCompact` | coreSchemas.ts:569-577 | `trigger: 'manual'|'auto', custom_instructions: string|null` | `trigger` |
| 13 | `PostCompact` | coreSchemas.ts:579-589 | `trigger, compact_summary: string` | `trigger` |
| 14 | `PermissionRequest` | coreSchemas.ts:425-434 | `tool_name, tool_input, permission_suggestions?` | `tool_name` |
| 15 | `PermissionDenied` | coreSchemas.ts:461-471 | `tool_name, tool_input, tool_use_id, reason: string` | `tool_name` |
| 16 | `Setup` | coreSchemas.ts:504-511 | `trigger: 'init'|'maintenance'` | `trigger` |
| 17 | `TeammateIdle` | coreSchemas.ts:591-599 | `teammate_name, team_name` | none |
| 18 | `TaskCreated` | coreSchemas.ts:601-612 | `task_id, task_subject, task_description?, teammate_name?, team_name?` | none |
| 19 | `TaskCompleted` | coreSchemas.ts:614-625 | `task_id, task_subject, task_description?, teammate_name?, team_name?` | none |
| 20 | `Elicitation` | coreSchemas.ts:627-643 | `mcp_server_name, message, mode?, url?, elicitation_id?, requested_schema?` | `mcp_server_name` |
| 21 | `ElicitationResult` | coreSchemas.ts:645-660 | `mcp_server_name, elicitation_id?, mode?, action, content?` | `mcp_server_name` |
| 22 | `ConfigChange` | coreSchemas.ts:670-678 | `source: 'user_settings'|'project_settings'|'local_settings'|'policy_settings'|'skills', file_path?` | `source` |
| 23 | `InstructionsLoaded` | coreSchemas.ts:695-707 | `file_path, memory_type, load_reason, globs?, trigger_file_path?, parent_file_path?` | `load_reason` |
| 24 | `WorktreeCreate` | coreSchemas.ts:709-716 | `name: string` | none |
| 25 | `WorktreeRemove` | coreSchemas.ts:718-725 | `worktree_path: string` | none |
| 26 | `CwdChanged` | coreSchemas.ts:727-735 | `old_cwd, new_cwd` | none |
| 27 | `FileChanged` | coreSchemas.ts:737-745 | `file_path, event: 'change'|'add'|'unlink'` | `basename(file_path)` |

## 2. Matcher Syntax (`utils/hooks.ts:1346-1381`)

```ts
function matchesPattern(matchQuery, matcher) {
  if (!matcher || matcher === '*') return true
  if (/^[a-zA-Z0-9_|]+$/.test(matcher)) {
    if (matcher.includes('|')) {
      return matcher.split('|')
        .map(normalizeLegacyToolName)
        .includes(matchQuery)
    }
    return matchQuery === normalizeLegacyToolName(matcher)
  }
  // 그 외는 RegExp
  return new RegExp(matcher).test(matchQuery)
}
```

핵심 사실:
- `""` 또는 `"*"` 면 전부 매치
- `[a-zA-Z0-9_|]` 만 포함하면 **정확한 문자열 매치** (파이프는 OR, regex 아님)
- 그 외 문자 (특히 `^`, `$`, `.`, `[`) 포함 시 **JavaScript RegExp** 로 컴파일
- **AND 연산자 없음** — OR 만
- Tool 레거시 이름 fallback 있음 (`getLegacyToolNames`)
- FileChanged 의 matcher 는 **basename 에 대한 regex** — `"*.ts"` 는 glob 이 아니라 regex 로 파싱되므로 **작동 안 함** (`\.ts$` 필요)

## 3. Exit Code Contract (`utils/hooks.ts:2616-2696`)

| Exit | Outcome | Effect |
|---|---|---|
| `0` | success | stdout → `hook_success` attachment |
| `2` | **blocking** | PreToolUse → deny tool / Stop → re-wake / SessionStart → block turn / TeammateIdle → feedback to teammate. stderr → feedback 메시지 |
| 기타 | non_blocking_error | 경고 표시, 실행은 계속 |

**JSON output override** (stdout 이 `{` 로 시작할 때):
```json
{
  "continue": false,
  "suppressOutput": true,
  "stopReason": "explanation",
  "decision": "block" | "approve",
  "reason": "...",
  "systemMessage": "warning shown to user not model",
  "hookSpecificOutput": { /* 이벤트별 discriminated union */ }
}
```

**`asyncRewake: true`** 은 백그라운드 실행 + exit 2 시 모델 mid-query re-wake. **moai-adk 의 TeammateIdle blocking 메커니즘의 실체.**

**타임아웃:**
- 기본: `TOOL_HOOK_EXECUTION_TIMEOUT_MS = 10분`
- SessionEnd 는 특별: `SESSION_END_HOOK_TIMEOUT_MS_DEFAULT = 1500ms` (1.5초!)
- 환경변수 `CLAUDE_CODE_SESSIONEND_HOOKS_TIMEOUT_MS` 로 override

## 4. Hook Output Contract — `hookSpecificOutput` (types/hooks.ts:72-162)

**이것이 가장 강력한 부분.** Discriminated union 이 이벤트별로 다른 override 기능 제공:

| Event | 반환 가능 필드 | 효과 |
|---|---|---|
| `PreToolUse` | `permissionDecision: 'allow'|'deny'|'ask'`, `permissionDecisionReason`, `updatedInput`, `additionalContext` | **Tool 입력을 통째로 rewrite** + permission 결정 |
| `UserPromptSubmit` | `additionalContext` | 사용자 프롬프트 뒤에 컨텍스트 주입 |
| `SessionStart` | `additionalContext`, `initialUserMessage`, `watchPaths: string[]` | 시스템 텍스트 주입 + **초기 사용자 메시지 자동 제출** + **FileChanged watcher 등록** |
| `PostToolUse` | `additionalContext`, `updatedMCPToolOutput` | **MCP tool 응답을 모델이 보기 전에 rewrite** |
| `PermissionDenied` | `retry: boolean` | Claude 가 재시도할지 제어 |
| `PermissionRequest` | `decision: {behavior:'allow', updatedInput?, updatedPermissions?} \| {behavior:'deny', message?, interrupt?}` | 완전한 permission 결정 + **새 permission rule 영구화** |
| `Elicitation` | `action: 'accept'|'decline'|'cancel'`, `content` | MCP elicitation prompt 다이얼로그 없이 자동 응답 |
| `CwdChanged` | `watchPaths` | CWD 변경 시 FileChanged watcher 재등록 |
| `FileChanged` | `watchPaths` | 동일 |

## 5. Tool Permission Decision Tree

**2개 레이어:**
- `hasPermissionsToUseToolInner` (`utils/permissions/permissions.ts:1158-1319`) — 정책 엔진
- `useCanUseTool` (`hooks/useCanUseTool.tsx`) — 오케스트레이션 wrapper

**실제 결정 흐름:**

1. **PreToolUse hooks 먼저** (toolExecution.ts:800) — `permissionDecision` + `updatedInput` 반환 가능
2. **`hasPermissionsToUseToolInner`**:
   - 1a. Tool 의 deny rule → `deny`
   - 1b. Tool 의 ask rule → `ask`
   - 1c-d. `tool.checkPermissions(input, context)` — tool-specific rules (Bash 의 command classifier 등)
   - 1e. `tool.requiresUserInteraction()` + ask → 즉시 반환
   - 1f. Content-specific ask rules (bypass 모드 무시)
   - 1g. Safety checks (`.git/`, `.claude/`, `.vscode/`) — bypass-immune
   - 2a. `bypassPermissions` 모드면 강제 allow
   - 2b. `toolAlwaysAllowedRule` → allow
   - 3. `passthrough` → `ask` + suggestions
3. **`useCanUseTool` 래핑**:
   - Coordinator mode → `handleCoordinatorPermission` 먼저
   - Swarm worker → `handleSwarmWorkerPermission`
   - Bash + classifier → `peekSpeculativeClassifierCheck` (2초 유예)
   - 최종 → `handleInteractivePermission` (dialog 또는 bridge/channel callbacks)
4. **PermissionRequest hooks** — `ask` 분기 내부에서 실행, `decision.updatedPermissions` 로 새 rule persist 가능

**핵심 인사이트:** `PreToolUse` 와 `PermissionRequest` 는 **둘 다** tool 을 deny 하는 surface. 하지만 `PreToolUse` 는 무조건 실행되고, `PermissionRequest` 는 permission state 가 `ask` 일 때만 실행. **UI 가 커스텀 승인을 표시할 올바른 위치는 PermissionRequest** (permission suggestions 도 함께 받음).

## 6. Tool Registry (`tools.ts:193-251`)

**닫힌 레지스트리**. `getAllBaseTools()` 는 하드코딩된 배열. Tool 들은 파일 상단에서 static import (`tools.ts:3-97`). **외부에서 runtime 에 새 tool 을 등록할 공식 API 는 없음.**

동적 부분:
- Feature flags (`feature('AGENT_TRIGGERS')`, `feature('KAIROS')`) 가 subset gate
- 환경 변수 (`USER_TYPE === 'ant'`, `ENABLE_LSP_TOOL`, `CLAUDE_CODE_SIMPLE`) 가 추가 tool 활성화
- **MCP 서버만이 runtime-registrable tool 소스**: `assembleToolPool` (tools.ts:345-367) 이 built-in tools + `mcpTools` 병합

**moai-cli 가 tool 을 추가하는 방법:**
1. **MCP 서버 스폰** → tools 가 `mcp__<server>__<tool>` 로 자동 노출
2. `mcp_set_servers` control request (controlSchemas.ts:384-391) 로 런타임에 MCP 서버 추가
3. Tool 자체는 등록 불가, **기존 tool 의 동작을 PreToolUse hook 의 `updatedInput` 으로 rewrite 는 가능**

## 7. Surprising / Undocumented Findings

moai-adk 가 사용하지 **않는** 것:

1. **`PermissionRequest` + `updatedPermissions`**: hook 이 `{decision: {behavior: 'allow', updatedPermissions: [...]}}` 로 응답하면 새 permission rule 이 디스크에 **영구 저장**. VS Code 의 "always allow" 버튼이 이 메커니즘.

2. **`PreToolUse.updatedInput` input rewriting**: hook 이 tool 입력을 통째로 교체 가능. 예: 모든 `rm` 을 trash 명령으로 감싸기, 모든 `git push` 에 confirm dialog 넣기.

3. **`PostToolUse.updatedMCPToolOutput`**: hook 이 MCP 응답을 모델이 보기 전에 sanitize/redact/enrich.

4. **`SessionStart.initialUserMessage` + `watchPaths`**: SessionStart 가 **초기 사용자 프롬프트를 합성**하고 **파일 watcher 를 등록** → FileChanged 이벤트 트리거.

5. **`InstructionsLoaded` 이벤트**: CLAUDE.md / skill / memory 파일이 로드될 때마다 fire, `memory_type` 과 `load_reason` 동봉. **어떤 파일이 현재 세션 컨텍스트에 있는지 실시간 추적 가능.**

6. **`ConfigChange`**: settings.json 변경 시 fire. **Live config hot-reload** 구현 가능.

7. **4가지 Hook 타입** (`schemas/hooks.ts:176-189`):
   - `command` — shell 명령 (moai-adk 현재 사용)
   - `prompt` — LLM 프롬프트로 평가 (default: Haiku)
   - `agent` — 서브 에이전트 verifier 스폰
   - `http` — URL POST (SSRF guard 포함)
   - plus runtime-only: `callback` (SDK host), `function` (internal)

8. **`async: true` + `asyncRewake: true`** (`schemas/hooks.ts:55-64`): 백그라운드 실행, exit 2 시 모델 mid-conversation wake. TeammateIdle blocking 의 핵심.

9. **`if` conditions** (`schemas/hooks.ts:19-27`): Permission rule syntax filter (`"Bash(git *)"`) — hook 스폰 전에 evaluated. moai-adk 의 `Bash|Edit|Write` 매처를 더 정밀하게 refine 가능.

10. **`shell` override** (`schemas/hooks.ts:36-41`): 각 hook 이 `bash`/`powershell` 등 선택 가능. Windows 네이티브 PowerShell hook 지원.

11. **`once: true`** (`schemas/hooks.ts:51-54`): 일회성 hook, 실행 후 자동 제거.

## 8. Can moai-cli Inject Synthetic Events?

**부분적으로 YES — SDK `initialize.hooks` 경로를 통해.**

### Mechanism A: SDK Host 모드 (권장)

moai-cli 가 `claude --output-format stream-json --input-format stream-json` 으로 Claude Code 스폰 → **SDK host** 역할. Startup 시 `control_request` 로 `hooks` 맵 전송:

```ts
{
  subtype: 'initialize',
  hooks: {
    PreToolUse: [{ matcher: 'Bash', hookCallbackIds: ['moai-bash-guard'] }],
    SessionStart: [{ matcher: '*', hookCallbackIds: ['moai-autoload'] }],
    // ... 27개 모두
  }
}
```

CLI 가 각 `callback_id` 를 `HookCallback` (structuredIO.ts:661-689) 에 wire. Hook 발생 시 Claude Code 가 `control_request` 를 moai-cli 로 역전송 (subtype: `hook_callback`) + 타입된 `HookInput` + `tool_use_id`. moai-cli 는 `HookJSONOutput` 으로 응답 (`hookSpecificOutput.permissionDecision: 'deny'`, `updatedInput: {...}` 등 포함).

**효과:**
- Shell wrapper script 전체 제거
- 타입 안전한 이벤트 구독
- Permission 결정 inline
- Tool 입력 rewrite on the fly
- Context 주입 (UserPromptSubmit/SessionStart 에)
- 27 이벤트 모두 관찰

### Mechanism B: UserPromptSubmit 사실상 fabrication

moai-cli 가 `SDKUserMessage` 를 stdin 에 전송하면 Claude Code 가 **실제 `UserPromptSubmit` hook 을 실행**. 이것은 "fake event" 가 아니라 **합법적인 user message injection**.

### Mechanism C: 직접 fabrication 불가

`SessionStart`, `TaskCompleted`, `FileChanged` 같은 이벤트는 Claude Code 내부 라이프사이클에서만 fire. SDK host 가 "이 이벤트를 fire 해라" 라는 control request 는 존재하지 않음.

**요약:** moai-cli 는 **27 이벤트 전부 관찰 + intercept + modify 가능**. `UserPromptSubmit` 은 간접 트리거 가능. 다른 라이프사이클 이벤트는 직접 fabrication 불가.

## 9. Brainstorm — moai-cli 설계 추가 제안

### 1. SDK Hook Callbacks 로 shell wrapper 완전 제거 (킬러 기능)
- moai-cli 가 Claude Code 를 SDK 모드로 spawn + `initialize.hooks` 로 27 callback 등록
- `.claude/hooks/moai/handle-*.sh` 파일 전체 폐기
- Go `moai hook <event>` subcommand 도 제거 (이제 Swift callback 이 직접 처리)
- 각 hook 당 bash subprocess spawn overhead 제거 (현재 10-40ms → <1ms)

### 2. Native Permission Dialog via PermissionRequest
- `PermissionRequest` callback 등록 → SwiftUI 네이티브 모달
- `updatedPermissions` 반환으로 "Always Allow" 버튼 구현
- Permission rule 이 moai-cli 에서 Claude Code settings.json 으로 persist

### 3. Bash Command Input Rewriter (Smart Wrap)
- `PreToolUse` + `matcher: "Bash"` + `updatedInput.command = wrapSafe(original)`
- 예: `rm` → `trash`, `git push` → confirm dialog, `cd` → worktree-aware
- 모든 Bash 명령에 투명 sandboxing

### 4. Live Settings Hot-reload via ConfigChange
- `ConfigChange` callback → moai-cli UI 가 자동 새로고침
- `.moai/config/sections/*.yaml` 변경 시 agent tree, model policy 즉시 반영
- 토스트 알림: "Config reloaded"

### 5. File Explorer via SessionStart.watchPaths + FileChanged
- SessionStart callback 이 `watchPaths: ['src/', '.moai/specs/']` 반환
- FileChanged 이벤트로 File Explorer 트리 실시간 업데이트
- **Claude 의 tool call 과 순서가 완벽히 동기화** (독립 watcher 대비 우위)

### 6. Instructions Loaded Graph
- `InstructionsLoaded` 구독 → 현재 세션에 로드된 CLAUDE.md/skill/memory 트리 시각화
- 각 파일의 `memory_type`, `load_reason`, parent relationship 표시
- "Why is this file in context?" 디버깅 surface

### 7. Worktree Lifecycle Tracker
- `WorktreeCreate` / `WorktreeRemove` 구독 → 활성 worktree 패널
- 클릭 시 file explorer root 전환

### 8. Cost & Telemetry Sidebar
- `PostToolUse` callback 이 매 tool call 관찰 → 사이드바 토큰/비용/duration 갱신
- `PreCompact`/`PostCompact` → compaction indicator
- 모델별 비용 chart

### 9. Eliminate moai Go Hook Binary (Performance)
- 현재: 27 hook 각각이 bash + moai Go 바이너리 spawn (~10-40ms)
- 새로운: Swift 함수 직접 호출 (<1ms)
- 하루 수천 hook call 에서 수십 초 절약

### 10. MCP Elicitation Auto-responder
- `Elicitation` callback → SwiftUI 네이티브 form
- 또는 알려진 safe 패턴에 대해 auto-respond
- Claude Code TUI 의 elicitation dialog 대체

### 11. Hook 타입 확장 — prompt / agent / http 활용
- 현재 moai-adk 는 `command` 만 사용
- 예: `TeammateIdle` 을 `agent` hook 으로 → verifier sub-agent 가 품질 검증
- 예: 외부 CI 트리거는 `http` hook 으로 (POST 가 자동)

### 12. `once: true` 일회성 setup hook
- moai-cli onboarding 완료 시 트리거되는 `Setup` hook
- 최초 실행 시 project 초기화, 이후 자동 제거

---

## 10. Facts I Could Not Verify

1. **SDK `initialize.hooks` 의 `registerHookCallbacks` 와의 정확한 wiring 경로** — `print.ts:4448` 에서 호출되는 것은 확인했으나 `initialize` control request 에서 직접 flow 는 trace 못 함. 100 라인 정도 더 읽으면 확인 가능.
2. **settings.json hook layering 정확한 precedence** (`policySettings` > `user` > `project` > `local`)
3. **MCP tool matcher 동작** — `mcp__server__tool` 전체 이름과 매치 되는지 `tool` 만인지
4. **`updatedInput` 이 `tool.inputSchema.parse` 재검증을 거치는지**
5. **Feature flags 가 빌드 타임 constants 인지 runtime 인지**
6. **`StatusLine`, `FileSuggestion` 이벤트의 정확한 schema** — HOOK_EVENTS 에 없지만 unions 에 등장
7. **병렬 vs 순차 hook 실행** — `matchingHooks.map(async function*...)` 이 실제 병렬인지
8. **SDK callback hook 의 async behavior** — `HookCallback` 타입에 `async` flag 없음
9. **SDK init 의 per-matcher timeout 이 실제 registration 시 HookCallback.timeout 에 feed 되는지**
10. **`once: true` 가 plugin hook 에서도 작동하는지**

---

## Source Inventory

- `src/entrypoints/sdk/coreSchemas.ts` (HOOK_EVENTS + 27 schemas)
- `src/types/hooks.ts` (sync/async output schemas, HookCallback)
- `src/schemas/hooks.ts` (HookCommand union: command/prompt/agent/http)
- `src/utils/hooks.ts` (5022 lines — matcher, exit codes, executeHooks, dispatchers)
- `src/services/tools/toolExecution.ts` (runPreToolUseHooks)
- `src/services/tools/toolHooks.ts` (runPostToolUseHooks)
- `src/hooks/useCanUseTool.tsx` (permission orchestration)
- `src/utils/permissions/permissions.ts` (9-step decision tree)
- `src/tools.ts` (closed tool registry)
- `src/Tool.ts` (Tool type)
- `src/cli/structuredIO.ts` (SDK hook_callback wiring, createHookCallback)
- `src/entrypoints/sdk/controlSchemas.ts` (initialize + hook_callback schemas)
- `src/bootstrap/state.ts:1419` (registerHookCallbacks global registry)
