# SPEC-M1-001: Implementation Plan

---

## 1. 구현 전략

M1 은 M0 carry-over 4건 해소를 선행한 후, 다중 워크스페이스 관리와 UI shell 을 구축하는 bottom-up 전략을 따른다. Rust core 계층을 먼저 완성하고, swift-bridge FFI 를 통해 Swift UI 에 노출한다.

### 계층별 구현 순서

```
Layer 1: M0 carry-over 해소 (RG-M1-2, RG-M1-3)
  ↓
Layer 2: Rust core 다중 워크스페이스 (RG-M1-4)
  ↓
Layer 3: Claude subprocess 전체 통합 (RG-M1-5)
  ↓
Layer 4: SwiftUI Shell + Sidebar (RG-M1-1, RG-M1-6)
  ↓
Layer 5: Plugin auto-installation (RG-M1-7)
  ↓
Layer 6: E2E 통합 검증
```

---

## 2. 마일스톤

### MS-1: M0 Carry-Over 해소 (Priority High)

**목표**: GhosttyKit xcframework 빌드 성공 + swift-bridge FFI 전환

**작업**:
- Metal Toolchain 설치 및 `zig build -Demit-xcframework=true` 실행
- GhosttyKit.xcframework 를 `app/Frameworks/` 에 배치
- C ABI (`#[no_mangle] extern "C"`) 를 `#[swift_bridge::bridge]` 로 마이그레이션
- `core/crates/moai-ffi/build.rs` 에 swift-bridge 코드 생성 설정
- 기존 103개 테스트 회귀 없음 확인

**검증**:
- `GhosttyKit.xcframework` 빌드 성공
- `swift-bridge` FFI 경유 Rust 함수 호출 성공 (Swift 측)
- `cargo test` 전체 통과

---

### MS-2: Workspace Lifecycle (Priority High)

**목표**: `RootSupervisor` 기반 다중 워크스페이스 생성/삭제/복원

**작업**:
- `moai-supervisor`: `RootSupervisor` 에 multi-workspace orchestration 추가
  - `WorkspaceSupervisor` 생성/종료 lifecycle
  - workspace 상태 머신: Created -> Starting -> Running -> Paused -> Error -> Deleted
- `moai-store`: workspace CRUD 구현
  - `workspaces` 테이블 스키마 확장 (name, project_path, worktree_path, status, spec_id 등)
  - migration v2 추가
- `moai-git`: workspace 별 git worktree 생성/삭제
- `moai-fs`: workspace 경로별 파일 감시 시작/중단
- 앱 재시작 시 store 에서 workspace 목록 복원

**검증**:
- workspace 생성 -> store 기록 -> 삭제 -> store 정리 round-trip 테스트
- 앱 재시작 후 workspace 목록 복원 테스트
- workspace 상태 전환 테스트 (6가지 상태)

---

### MS-3: Claude Subprocess Full Stack (Priority High)

**목표**: UI -> FFI -> Claude subprocess -> stream-json -> EventBus -> UI 전체 파이프라인

**작업**:
- `moai-claude-host`: workspace 별 독립 subprocess spawn
- `moai-stream-json`: 13개 SDKMessage 실시간 디코딩 + EventBus 발행
- `moai-ide-server`: workspace 별 MCP 서버 인스턴스 (포트 분리)
- `moai-hook-http`: workspace 별 hook 엔드포인트
- EventBus -> swift-bridge FFI -> Swift `@Observable` ViewModel 파이프라인

**검증**:
- 메시지 전송 -> assistant 응답 수신 round-trip
- MCP tool 호출 round-trip (`mcp__moai__echo`)
- hook event 수신 + EventBus 발행 확인
- 2개 이상 workspace 동시 Claude subprocess 운영

---

### MS-4: SwiftUI Shell + Sidebar (Priority High)

**목표**: 사이드바 워크스페이스 목록 + 콘텐츠 영역 터미널 surface

**작업**:
- Xcode 프로젝트 구성 (또는 xcodegen spec)
- `app/Sources/App/`: @main, AppDelegate, WindowGroup
- `app/Sources/Shell/Sidebar/`: 워크스페이스 목록 뷰
  - 상태 아이콘 (Starting/Running/Error/Paused)
  - "+" 버튼 (새 워크스페이스)
  - 우클릭 컨텍스트 메뉴
- `app/Sources/Shell/Content/`: 콘텐츠 영역
  - GhosttyKit 터미널 surface 래핑
  - 빈 상태 환영 메시지
- `app/Sources/Bridge/`: swift-bridge 생성 코드 래퍼
- `NavigationSplitView` 기반 레이아웃
- 윈도우 크기/사이드바 너비 UserDefaults 저장/복원

**검증**:
- 앱 실행 -> 메인 윈도우 표시
- 사이드바 워크스페이스 목록 렌더링
- 워크스페이스 선택 -> 콘텐츠 전환
- GhosttyKit 터미널 zsh 렌더링

---

### MS-5: Plugin Auto-Installation (Priority Medium)

**목표**: 첫 실행 시 plugin 자동 설치 + 버전 관리

**작업**:
- `moai-plugin-installer`: 설치 로직 확장
  - 번들 내 plugin -> `~/.claude/plugins/moai-studio@local/` 복사
  - 버전 비교 (plugin.json version 필드)
  - 무결성 검증 (JSON 파싱, 필수 필드)
- plugin 디렉토리 구조 검증: `.claude-plugin/plugin.json`, `hooks/hooks.json`, `mcp-config.json`
- 쓰기 권한 오류 처리

**검증**:
- 첫 실행 -> plugin 설치 확인
- 버전 업그레이드 시 재설치 확인
- 무결성 검증 실패 시 오류 처리

---

### MS-6: E2E 통합 검증 + M1 Go/No-Go (Priority High)

**목표**: 전체 시나리오 검증 및 M1 완료 보고서

**작업**:
- E2E 테스트 시나리오 작성 및 실행:
  1. 앱 실행 -> 메인 윈도우 표시
  2. "New Workspace" -> Claude subprocess spawn
  3. 터미널 surface zsh 렌더링
  4. 사용자 메시지 전송 -> assistant 응답 실시간 표시
  5. 두 번째 워크스페이스 생성 -> 전환
  6. 첫 번째 워크스페이스 삭제
  7. 앱 재시작 -> 남은 워크스페이스 복원
- M1 completion report 작성

**검증**:
- 전체 E2E 시나리오 통과
- 4개 동시 워크스페이스 안정 동작
- 비기능 요구사항 달성 확인

---

## 3. 기술 접근

### swift-bridge FFI 패턴

```rust
// core/crates/moai-ffi/src/lib.rs
#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type RustCore;

        #[swift_bridge(init)]
        fn new() -> RustCore;
        fn version(&self) -> String;
        fn create_workspace(&self, name: String, project_path: String) -> String;
        fn delete_workspace(&self, workspace_id: String) -> bool;
        fn list_workspaces(&self) -> Vec<WorkspaceInfo>;
        fn send_user_message(&self, workspace_id: String, message: String);
    }

    #[swift_bridge(swift_repr = "struct")]
    struct WorkspaceInfo {
        id: String,
        name: String,
        status: String,
    }
}
```

### Workspace 상태 머신

```
Created ──(start)──> Starting ──(ready)──> Running
                         │                    │
                         │                    ├──(pause)──> Paused ──(resume)──> Running
                         │                    │
                         └──(fail)──> Error ──(restart)──> Starting
                                        │
                                        └──(delete)──> Deleted

Running ──(delete)──> Deleted
Paused ──(delete)──> Deleted
```

### EventBus 아키텍처

```
Claude subprocess (stdout)
  → moai-stream-json (SDKMessage 디코딩)
    → EventBus (tokio broadcast)
      → moai-ide-server (MCP tool 호출 처리)
      → moai-hook-http (hook event 저장)
      → swift-bridge callback (UI 업데이트)
```

---

## 4. 리스크 대응

| 리스크 | 대응 전략 |
|--------|-----------|
| Metal Toolchain 빌드 실패 | MS-1 에서 우선 해소. 실패 시 NSTextView fallback 터미널 (Ghostty 없이 M1 진행 가능) |
| swift-bridge async 불안정 | sync FFI + DispatchQueue.global().async 패턴으로 우회. async 는 M2 에서 재시도 |
| 다중 subprocess 메모리 | M1 은 4개 제한. 비활성 workspace 는 subprocess 종료 (lazy restart) |

---

## 5. 선행 조건 체크리스트

- [x] SPEC-M0-001 conditional GO
- [x] 103 tests 통과, 12 crates 동작
- [ ] Metal Toolchain 설치 (MS-1 에서 해소)
- [ ] Xcode 프로젝트 생성 (MS-4 에서 해소)
- [ ] swift-bridge CLI 설치 (MS-1 에서 해소)
