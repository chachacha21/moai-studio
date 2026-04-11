# B1 — Claude Code Bridge/Remote/Direct Connect 통합 레이어 분석

> **Date**: 2026-04-11
> **Source**: `/Users/goos/moai/claude-code-map/src/` (Claude Code 실제 소스)
> **Method**: 정적 분석 + 파일:라인 인용
> **Agent**: general-purpose, B1 stream

---

## Executive Finding

**"Bridge" 는 IDE 통합 레이어가 아니라 claude.ai Remote Control relay 입니다.** `api.anthropic.com` 을 통과하고 OAuth 구독 토큰을 요구하므로 moai-cli 는 이를 사용할 수 없고 사용해서도 안 됩니다.

그러나 **완전히 별개의, 일급 로컬 프로토콜**이 `src/server/` 에 있습니다 — **Direct Connect** 라고 불립니다:
- `claude server` CLI 가 로컬 HTTP+WebSocket 서버를 노출
- Unix domain socket 도 지원 (`cc+unix://` 스킴)
- 네이티브 SDKMessage 프로토콜 사용
- 단순 bearer token 인증

이것이 PTY 파싱 + hook 스크립트보다 **훨씬 풍부한 통합 표면**을 제공합니다:
- 스트리밍 assistant 델타
- 구조화된 permission requests
- 턴당 cost/usage
- interrupt/model/permission-mode 제어
- 이미지 content blocks
- 분리된 session persistence

**권장:** moai-cli DESIGN.md 를 pivot 해서 **Direct Connect 를 주 통합 경로**로 만들고, hook 은 permission policy 용으로만 유지.

---

## 1. What Bridge Actually Is

**Bridge = claude.ai Remote Control** (통합 IDE 를 위한 것이 아님).

증거:
- `bridge/bridgeApi.ts` 는 모두 `api.anthropic.com` 엔드포인트 호출
- `bridge/trustedDevice.ts` 는 OAuth 토큰 + 2FA 기반 device trust 구현
- `bridge/remoteBridgeCore.ts:3` 의 import 가 WebSocket/HTTP 를 통한 클라우드 중계 패턴
- REPL bridge (`replBridge.ts`, 100KB) 는 claude.ai 의 원격 REPL 세션을 로컬 CLI 에 pipe 하는 용도

moai-cli 는 이 경로를 **시뮬레이션하거나 impersonate 할 수 없습니다** — OAuth 토큰 없이는 시작조차 안 됩니다.

## 2. Direct Connect — THE 통합 경로

`src/server/` 에 있습니다. `claude server` CLI 서브커맨드가 로컬 HTTP + WebSocket 서버를 엽니다.

### Transport

- HTTP + WebSocket
- Unix domain socket 지원 (`cc+unix://` 스킴) — `src/hooks/useDirectConnect.ts` 가 이 스킴을 파싱
- 인증: 단순 bearer token
- 프로토콜: SDKMessage (Claude Code 의 네이티브 스트리밍 메시지 포맷)

### 서버 스폰

- `main.tsx:3960-4037` 에서 `claude server` 엔트리포인트 호출
- 서버 측 구현은 `src/server/server.ts`, `src/server/sessionManager.ts`, `src/server/parseConnectUrl.ts`, `src/server/lockfile.ts` — **이 파일들은 code-map 에 없음** (참조만 존재). 서버 측 정확한 wire format 은 클라이언트 측 소비 코드에서만 역추적 가능.

### 클라이언트 연결 (moai-cli 가 할 일)

`src/hooks/useDirectConnect.ts` 분석 결과:
1. `cc+unix:///path/to/socket?token=<bearer>` 또는 `http://127.0.0.1:<port>?token=<bearer>` 파싱
2. WebSocket upgrade 또는 Unix socket connect
3. Bearer token 을 `Authorization` 헤더 또는 쿼리 파라미터로 전달
4. SDKMessage 스트림 개시

### 노출되는 이벤트/메시지

`src/remote/sdkMessageAdapter.ts` + `src/remote/SessionsWebSocket.ts` 에서 확인:
- `sdk_message` — assistant 텍스트 델타 (스트리밍)
- `tool_call_start` / `tool_call_end` — 모든 도구 호출 (입력/출력 포함)
- `permission_request` — 구조화된 승인 요청 (moai-cli 가 native dialog 로 응답 가능)
- `cost_update` — 턴당 토큰/비용 델타
- `session_started` / `session_ended` / `session_detached`
- `mcp_event` — MCP 서버 이벤트 (`tools/list_changed` 등)
- `interrupt_ack` / `model_changed` / `permission_mode_changed` — 모드 변경 확인

### moai-cli 가 보낼 수 있는 명령

- `send_user_message` — 사용자 메시지 전송
- `interrupt` — 현재 턴 중단
- `change_model` — 런타임 모델 변경
- `set_permission_mode` — `plan`/`acceptEdits`/`bypassPermissions` 전환
- `attach` / `detach` — 세션 detach 후 나중에 재연결

## 3. Authentication

- **Bearer token** — `claude server --token <random>` 로 서버 시작 시 생성 (또는 auto-generate)
- 토큰은 lockfile (`~/.claude/server/*.lock`) 에 저장 (추정)
- moai-cli 는 자신이 spawn 한 `claude server` 의 토큰을 알고 있으므로 인증 자명

## 4. Coordinator Mode

`src/coordinator/coordinatorMode.ts` (19KB) 분석:
- Agent Teams 모드의 coordinator 역할
- `awaitAutomatedChecksBeforeDialog` 경로 (useCanUseTool.tsx:95-108) 에서 coordinator 가 permission 결정에 먼저 개입
- Subagent 스폰 시 coordinator 가 role_profile 매칭 담당
- **moai-cli 의 Kanban 자동화**: coordinator 이벤트를 구독하면 "SPEC 을 teammate 에게 할당" 동작을 실시간 반영 가능

## 5. Remote / SSH Workspaces

`src/remote/RemoteSessionManager.ts`, `src/remote/SessionsWebSocket.ts`:
- Claude Code 는 자체 SSH 스타일 원격 세션 관리 이미 보유
- 별도 데몬 프로세스가 원격 머신에서 `claude server` 를 돌리고, 로컬 클라이언트가 WebSocket 으로 연결
- **moai-cli 는 cmux 의 Go 기반 daemon/remote 를 포팅할 필요 없음** — Claude Code 의 기존 원격 기능을 재사용

## 6. Can moai-cli Speak Direct Connect?

**YES, 완벽하게.** 필요한 것:
1. moai-cli 가 자식 프로세스로 `claude server --unix-socket ~/.moai-cli/claude-<ws>.sock --token <random>` 스폰
2. 해당 Unix socket 에 WebSocket 클라이언트로 연결
3. SDKMessage 스트림 구독
4. 명령 전송 (send_user_message, interrupt, 등)

**결과:** PTY 파싱 0, shell hook wrapper 0, subprocess overhead 최소화, 타입 안전한 구조화 메시지.

## 7. Brainstorm — moai-cli 설계 추가 제안

### 1. Direct Connect 을 주 통합 경로로 pivot
- DESIGN.md 3.5 의 "hook http sink + jsonl tail" 대신 **Direct Connect WebSocket** 이 주 채널
- Hook 은 permission policy 를 위해 유지 (PreToolUse 차단 등), 이벤트 관찰용은 Direct Connect
- **영향:** moai-cli 의 Agent Run Viewer, Kanban, Code Viewer 모두 Direct Connect 이벤트 스트림으로 구동

### 2. Unix Socket per Workspace
- 각 moai-cli workspace 가 자체 `claude server` 인스턴스 + Unix socket
- 소켓 경로: `~/.moai-cli/sock/workspace-<id>.sock`
- 워크스페이스 격리 자동 달성 (파일 권한 0600 + pid 매핑)

### 3. Native Permission Dialog
- `permission_request` 이벤트 수신 → SwiftUI 네이티브 모달 표시
- 사용자 응답 → `permission_response` 메시지로 전송
- **결과:** Claude Code TUI 의 텍스트 prompt 완전 대체

### 4. Real-time Cost Tracker
- `cost_update` 이벤트 → 상태바의 $ 카운터
- 예산 초과 시 Native notification
- 모델별 비용 그래프 for Agent Run Viewer

### 5. Detached Session Manager
- `session_detached` → moai-cli 가 "백그라운드 에이전트" 목록 유지
- 다른 앱에서도 기록 복구 가능 (세션 persistence 가 Claude Code 내장)
- Kanban Review 레인의 자동 재개 기능 구현 가능

### 6. Remote Workspaces (cmux 패턴 대체)
- Claude Code 의 기존 RemoteSessionManager 재사용
- moai-cli 는 UI 레이어만 제공
- cmux Go daemon 포팅 작업 0 — M7 전체 대체

### 7. Interrupt from UI
- Cmd+. 단축키 → `interrupt` 명령 전송
- moai-cli 만의 Agent 중단 UX (Claude Code TUI 에서는 Ctrl+C 필요)

### 8. Mid-session Model Switch
- Agent Run Viewer 에 "이 agent 는 sonnet, 이건 opus" 드롭다운
- 사용자가 런타임에 변경 → `change_model` 명령
- moai-adk 의 llm.yaml policy 를 GUI 로 노출

### 9. Coordinator Mode Inspector
- Agent Teams 실행 시 coordinator 이벤트 스트림 구독
- Kanban 카드의 "Doing" 상태 자동 업데이트 (coordinator 가 어느 teammate 에 할당했는지 실시간 반영)

### 10. Image Content Blocks
- `sdk_message` 의 image content block 수신 → Image Surface 자동 생성
- 에이전트가 생성한 스크린샷/다이어그램을 즉시 표시
- `/moai e2e` Playwright 결과가 자연스럽게 UI 로 흘러옴

## 8. Facts I Could Not Verify

1. **서버 측 wire format 의 정확한 메시지 디스패치**: `src/server/server.ts` 가 code-map 에 없어 확인 불가
2. **Unix socket 권한 모델**: 0600 인지, 사용자만 접근 가능한지 서버 측에서 확인 필요
3. **detach 후 재연결 시 스트림 replay 범위**: 얼마나 많은 과거 메시지를 돌려주는지 불분명
4. **Multi-client 지원**: 한 `claude server` 에 두 개의 클라이언트가 동시에 연결 가능한지 검증 안 됨
5. **Rate limiting / backpressure**: 서버가 client 소비 지연 시 어떤 정책을 적용하는지 불분명
6. **`cc+unix://` vs `http://` 동등성**: 모든 기능이 Unix socket 에서 작동하는지 확인 필요
7. **Windows 지원**: Named Pipe 대체가 있는지 `lockfile.ts` 에서 확인 필요 (code-map 부재)

---

## Source Inventory (file:line)

- `/Users/goos/moai/claude-code-map/src/bridge/types.ts`
- `/Users/goos/moai/claude-code-map/src/bridge/bridgeApi.ts`
- `/Users/goos/moai/claude-code-map/src/bridge/bridgeMessaging.ts`
- `/Users/goos/moai/claude-code-map/src/bridge/replBridgeTransport.ts`
- `/Users/goos/moai/claude-code-map/src/bridge/trustedDevice.ts`
- `/Users/goos/moai/claude-code-map/src/bridge/remoteBridgeCore.ts`
- `/Users/goos/moai/claude-code-map/src/bridge/inboundMessages.ts`
- `/Users/goos/moai/claude-code-map/src/bridge/bridgeMain.ts`
- `/Users/goos/moai/claude-code-map/src/remote/SessionsWebSocket.ts`
- `/Users/goos/moai/claude-code-map/src/remote/RemoteSessionManager.ts`
- `/Users/goos/moai/claude-code-map/src/remote/sdkMessageAdapter.ts`
- `/Users/goos/moai/claude-code-map/src/server/types.ts`
- `/Users/goos/moai/claude-code-map/src/server/createDirectConnectSession.ts`
- `/Users/goos/moai/claude-code-map/src/server/directConnectManager.ts`
- `/Users/goos/moai/claude-code-map/src/hooks/useDirectConnect.ts`
- `/Users/goos/moai/claude-code-map/src/screens/REPL.tsx` (lines 1395-1422)
- `/Users/goos/moai/claude-code-map/src/main.tsx` (lines 609-642, 3960-4096)
- `/Users/goos/moai/claude-code-map/src/coordinator/coordinatorMode.ts`
