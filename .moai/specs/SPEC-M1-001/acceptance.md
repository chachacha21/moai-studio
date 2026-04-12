# SPEC-M1-001: Acceptance Criteria

---

## 1. GhosttyKit Terminal Surface (RG-M1-2)

### AC-1.1: GhosttyKit xcframework 빌드 성공

**Given** Metal Toolchain 이 설치된 macOS 환경에서
**When** `scripts/build-ghostty-xcframework.sh` 를 실행하면
**Then** `app/Frameworks/GhosttyKit.xcframework` 가 생성되고 Xcode 에서 링크 가능해야 한다

### AC-1.2: Metal Toolchain 미설치 환경 처리

**Given** Metal Toolchain 이 설치되지 않은 환경에서
**When** `scripts/build-ghostty-xcframework.sh` 를 실행하면
**Then** Metal Toolchain 설치 안내 메시지를 출력하고 exit code 1 로 종료해야 한다

### AC-1.3: GhosttyKit 터미널 렌더링

**Given** GhosttyKit xcframework 가 앱에 링크된 상태에서
**When** 워크스페이스의 콘텐츠 영역이 활성화되면
**Then** zsh shell 이 Metal 렌더링으로 표시되어야 한다

### AC-1.4: GhosttyKit 초기화 실패 처리

**Given** GhosttyKit 초기화가 불가능한 환경에서 (Metal 미지원 등)
**When** 터미널 surface 생성을 시도하면
**Then** "Terminal unavailable" 메시지와 재시도 버튼이 표시되어야 한다

---

## 2. swift-bridge FFI 전환 (RG-M1-3)

### AC-2.1: swift-bridge FFI 기본 호출

**Given** swift-bridge 로 마이그레이션된 `moai-ffi` crate 에서
**When** Swift 측에서 `RustCore.new()` 를 호출하면
**Then** Rust `RustCore` 인스턴스가 생성되고 `version()` 이 유효한 버전 문자열을 반환해야 한다

### AC-2.2: swift-bridge workspace CRUD FFI

**Given** `RustCore` 인스턴스가 초기화된 상태에서
**When** Swift 측에서 `create_workspace(name: "test", project_path: "/tmp/test")` 를 호출하면
**Then** 유효한 workspace ID 문자열이 반환되어야 한다

**Given** 워크스페이스가 존재하는 상태에서
**When** Swift 측에서 `list_workspaces()` 를 호출하면
**Then** 생성된 워크스페이스가 포함된 목록이 반환되어야 한다

**Given** 워크스페이스가 존재하는 상태에서
**When** Swift 측에서 `delete_workspace(id)` 를 호출하면
**Then** `true` 가 반환되고 이후 `list_workspaces()` 에서 해당 워크스페이스가 제외되어야 한다

### AC-2.3: FFI 성능

**Given** swift-bridge FFI 가 구성된 상태에서
**When** FFI 함수를 1000회 반복 호출하면
**Then** 평균 호출 시간이 1ms 미만이어야 한다

### AC-2.4: 기존 테스트 회귀 없음

**Given** C ABI 에서 swift-bridge 로 마이그레이션된 상태에서
**When** `cargo test --workspace` 를 실행하면
**Then** 기존 103개 이상의 테스트가 모두 통과해야 한다

---

## 3. Workspace Lifecycle (RG-M1-4)

### AC-3.1: 워크스페이스 생성 전체 플로우

**Given** 앱이 실행되고 프로젝트가 로드된 상태에서
**When** 사용자가 "New Workspace" 를 요청하면
**Then** 다음 순서로 처리되어야 한다:
  1. `moai-store` 에 workspace 레코드 삽입 (상태: Created)
  2. `moai-git` 로 git worktree 생성
  3. `moai-fs` 로 워크스페이스 경로 감시 시작
  4. `moai-claude-host` 로 Claude subprocess spawn (상태: Starting -> Running)
  5. 전체 과정이 3초 이내에 완료

### AC-3.2: 워크스페이스 삭제

**Given** `Running` 상태의 워크스페이스가 존재할 때
**When** 사용자가 해당 워크스페이스 삭제를 요청하면
**Then** Claude subprocess 종료, git worktree 정리, store 레코드 삭제, 파일 감시 중단이 모두 수행되어야 한다

### AC-3.3: 앱 재시작 후 워크스페이스 복원

**Given** 2개의 워크스페이스가 존재하는 상태에서 앱을 종료한 후
**When** 앱을 다시 실행하면
**Then** 사이드바에 2개의 워크스페이스가 표시되어야 한다 (Claude subprocess 는 선택 시 lazy spawn)

### AC-3.4: 워크스페이스 상태 전환

**Given** 워크스페이스가 `Created` 상태에서
**When** 시작이 요청되면
**Then** `Starting` -> `Running` 순서로 상태가 전환되어야 한다

**Given** 워크스페이스가 `Running` 상태에서
**When** Claude subprocess 가 비정상 종료하면
**Then** 상태가 `Error` 로 전환되고 사이드바에 오류 아이콘이 표시되어야 한다

### AC-3.5: 다중 워크스페이스 동시 운영

**Given** 4개의 워크스페이스가 모두 `Running` 상태일 때
**When** 각 워크스페이스에서 Claude 와 독립적으로 통신하면
**Then** 메시지 전송/수신이 서로 간섭 없이 동작해야 한다

---

## 4. Claude Subprocess Full Integration (RG-M1-5)

### AC-4.1: 메시지 전송 및 응답 수신

**Given** 워크스페이스가 `Running` 상태이고 Claude subprocess 가 동작 중일 때
**When** 사용자가 "Hello" 메시지를 전송하면
**Then** assistant 응답이 stream-json 을 통해 실시간으로 수신되어야 한다

### AC-4.2: MCP tool round-trip

**Given** Claude subprocess 가 `--mcp-config` 로 MoAI MCP 서버에 연결된 상태에서
**When** Claude 에 `mcp__moai__echo` 를 호출하는 프롬프트를 전달하면
**Then** MCP 도구 호출 -> 서버 처리 -> 결과 반환이 50ms 이내에 완료되어야 한다

### AC-4.3: Hook event 수신

**Given** Claude subprocess 가 plugin hook 으로 구성된 상태에서
**When** Claude 가 PreToolUse 이벤트를 발생시키면
**Then** `moai-hook-http` 엔드포인트에서 HTTP POST 를 수신하고 EventBus 에 발행해야 한다

### AC-4.4: 워크스페이스 간 subprocess 격리

**Given** 2개의 워크스페이스가 각각 독립 Claude subprocess 를 실행 중일 때
**When** 워크스페이스 A 에 메시지를 전송하면
**Then** 워크스페이스 B 의 Claude subprocess 에는 영향이 없어야 한다

---

## 5. Sidebar + Content Layout (RG-M1-6)

### AC-5.1: 메인 윈도우 레이아웃

**Given** 앱이 처음 실행될 때
**When** 메인 윈도우가 표시되면
**Then** 사이드바 (왼쪽, 기본 250px) + 콘텐츠 영역 (오른쪽) 의 2-pane 레이아웃이어야 한다

### AC-5.2: 워크스페이스 목록 표시

**Given** 3개의 워크스페이스가 존재할 때 (Running 2개, Error 1개)
**When** 사이드바를 확인하면
**Then** 3개 항목이 표시되고, Running 은 녹색 원, Error 는 빨간 원 아이콘이 표시되어야 한다

### AC-5.3: 워크스페이스 전환

**Given** 워크스페이스 A 의 터미널이 콘텐츠 영역에 표시된 상태에서
**When** 사이드바에서 워크스페이스 B 를 선택하면
**Then** 100ms 이내에 콘텐츠 영역이 워크스페이스 B 의 터미널로 전환되어야 한다

### AC-5.4: 새 워크스페이스 생성 UI

**Given** 사이드바 하단의 "+" 버튼이 표시된 상태에서
**When** "+" 버튼을 클릭하면
**Then** 워크스페이스 이름 입력 필드가 나타나야 한다

### AC-5.5: 컨텍스트 메뉴

**Given** 사이드바에 워크스페이스 항목이 표시된 상태에서
**When** 항목을 우클릭하면
**Then** "Rename", "Restart Claude", "Delete" 옵션이 포함된 컨텍스트 메뉴가 표시되어야 한다

### AC-5.6: 빈 상태 환영 메시지

**Given** 워크스페이스가 하나도 없는 상태에서
**When** 콘텐츠 영역을 확인하면
**Then** 환영 메시지와 "Create Workspace" 버튼이 표시되어야 한다

### AC-5.7: 사이드바 너비 조정

**Given** 사이드바가 표시된 상태에서
**When** 사이드바 경계를 드래그하면
**Then** 너비가 200px ~ 400px 범위 내에서 조정되어야 한다

### AC-5.8: 윈도우 상태 복원

**Given** 사이드바 너비를 300px 로 조정하고 앱을 종료한 후
**When** 앱을 다시 실행하면
**Then** 사이드바 너비가 300px 로 복원되어야 한다

---

## 6. Plugin Auto-Installation (RG-M1-7)

### AC-6.1: 최초 실행 시 플러그인 설치

**Given** `~/.claude/plugins/moai-studio@local/` 이 존재하지 않을 때
**When** 앱이 처음 실행되면
**Then** 번들 내 plugin 이 해당 경로에 복사되어야 한다

### AC-6.2: 플러그인 무결성 검증

**Given** 플러그인이 설치된 상태에서
**When** `moai-plugin-installer` 가 무결성 검증을 수행하면
**Then** `plugin.json` 과 `hooks/hooks.json` 이 유효한 JSON 이고 필수 필드가 존재해야 한다

### AC-6.3: 플러그인 업데이트

**Given** 설치된 플러그인 버전이 "0.1.0" 이고 번들 내 버전이 "0.2.0" 일 때
**When** 앱이 실행되면
**Then** 플러그인이 "0.2.0" 으로 업데이트되어야 한다

### AC-6.4: 쓰기 권한 오류 처리

**Given** `~/.claude/plugins/` 에 쓰기 권한이 없을 때
**When** 플러그인 설치를 시도하면
**Then** 오류가 로그에 기록되고 사용자에게 수동 설치 안내가 표시되어야 한다

---

## 7. E2E 통합 시나리오

### AC-7.1: Full Working Shell 시나리오

**Given** 앱이 설치되고 `ANTHROPIC_API_KEY` 가 설정된 환경에서
**When** 다음 시나리오를 순서대로 실행하면:
  1. 앱 실행
  2. "+" 버튼으로 "workspace-1" 생성
  3. 터미널에 zsh shell 표시 확인
  4. Claude 에 메시지 전송
  5. Assistant 응답 실시간 표시 확인
  6. "+" 버튼으로 "workspace-2" 생성
  7. 사이드바에서 workspace-2 선택 -> 콘텐츠 전환
  8. 사이드바에서 workspace-1 우클릭 -> "Delete"
  9. 앱 종료 후 재실행
**Then** workspace-2 만 사이드바에 표시되어야 한다

### AC-7.2: 4-워크스페이스 안정성

**Given** 4개의 워크스페이스가 모두 Running 상태일 때
**When** 10분 이상 동시 운영하면
**Then** 메모리 사용량 400MB 이하, 크래시 0회, 각 워크스페이스 독립 동작 확인

---

## 8. Quality Gate Criteria

### Definition of Done

- [ ] 모든 RG (RG-M1-1 ~ RG-M1-7) 에 대한 acceptance criteria 통과
- [ ] `cargo test --workspace` 전체 통과 (기존 103+ 테스트 + M1 신규 테스트)
- [ ] `cargo check --workspace` 0 errors, 0 warnings
- [ ] Xcode 빌드 0 errors
- [ ] swift-bridge FFI call overhead < 1ms
- [ ] E2E 시나리오 (AC-7.1) 통과
- [ ] 4-워크스페이스 안정성 테스트 (AC-7.2) 통과
- [ ] M1 completion report 작성

### Edge Cases

| 시나리오 | 기대 동작 |
|----------|-----------|
| 동일 이름 워크스페이스 생성 시도 | 자동 suffix 추가 ("workspace-1 (2)") 또는 오류 메시지 |
| Claude subprocess spawn 실패 (API key 미설정) | workspace 상태 `Error`, 사이드바 오류 아이콘, 설정 안내 |
| git worktree 생성 실패 (디스크 공간 부족) | workspace 생성 중단, 오류 메시지 표시, store 정리 |
| 앱 강제 종료 (kill -9) 후 재시작 | store 에서 workspace 복원, orphan worktree 정리 |
| 네트워크 없는 환경에서 실행 | Claude subprocess 실패를 graceful 하게 처리, 오프라인 메시지 표시 |
| 사이드바 최소 너비 이하로 드래그 | 200px 에서 멈춤 (최소 제한) |
| 5번째 워크스페이스 생성 시도 | M1 에서는 4개 제한, 제한 안내 메시지 표시 |
