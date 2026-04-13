# SPEC-M2-001 Acceptance Criteria

---
spec_id: SPEC-M2-001
version: 1.0.0
status: draft
created: 2026-04-13
---

## 1. Given/When/Then 시나리오

### RG-M2-1: Pane Splitting

**AC-1.1: 수평 분할**
- Given: 단일 pane 에 Terminal surface 가 표시된 상태
- When: 사용자가 Cmd+\ 를 누른다
- Then: 현재 pane 이 수평으로 분할되어 좌우 두 개의 pane 이 생성된다. 좌측에 기존 surface, 우측에 EmptyState 가 표시된다. ratio 는 0.5 (균등 분할).

**AC-1.2: 수직 분할**
- Given: 단일 pane 이 활성 상태
- When: 사용자가 Cmd+Shift+\ 를 누른다
- Then: 현재 pane 이 수직으로 분할되어 상하 두 개의 pane 이 생성된다.

**AC-1.3: 리사이즈**
- Given: 수평 분할된 두 개의 pane
- When: 사용자가 분할 경계를 드래그한다
- Then: ratio 가 실시간으로 변경된다. 각 pane 은 200pt 미만으로 축소되지 않는다.

**AC-1.4: Pane 닫기**
- Given: 수평 분할된 두 개의 pane
- When: 사용자가 우측 pane 을 닫는다 (Cmd+Shift+W)
- Then: 우측 pane 이 제거되고, 좌측 pane 이 전체 영역을 차지한다.

**AC-1.5: 마지막 Pane 보호**
- Given: 단일 pane 만 존재하는 상태
- When: 사용자가 pane 닫기를 시도한다
- Then: 닫기가 무시된다. pane 은 유지된다.

**AC-1.6: 레이아웃 영속**
- Given: 4개 pane 으로 분할된 복잡한 레이아웃 (2단계 binary tree)
- When: 앱을 종료하고 재시작한다
- Then: 동일한 pane tree 구조, ratio, 각 pane 의 surface 가 복원된다.

**AC-1.7: DB 영속성**
- Given: panes 테이블에 workspace_id=1 의 tree 가 저장된 상태
- When: `moai-store` 에서 `list_panes_by_workspace(1)` 호출
- Then: parent_id, split, ratio 가 올바른 tree 구조를 반환한다.

---

### RG-M2-2: Tab UI

**AC-2.1: 새 탭 생성**
- Given: pane 에 1개의 탭 (Terminal) 이 있는 상태
- When: 사용자가 + 버튼 또는 Cmd+T 를 누른다
- Then: 새 탭이 추가되고 EmptyState 가 표시된다. tab_order 가 기존 탭 다음으로 설정된다.

**AC-2.2: 탭 닫기**
- Given: pane 에 3개의 탭이 있는 상태
- When: 사용자가 2번째 탭의 X 버튼을 클릭한다
- Then: 2번째 탭이 닫히고, 인접 탭이 활성화된다.

**AC-2.3: 마지막 탭 닫기**
- Given: pane 에 1개의 탭만 존재하고, 형제 pane 이 있는 상태
- When: 사용자가 마지막 탭의 X 또는 Cmd+W 를 누른다
- Then: 해당 pane 이 제거된다 (RG-M2-1 규칙 적용).

**AC-2.4: 탭 드래그 재배치**
- Given: pane 에 3개의 탭 [A, B, C] 가 있는 상태
- When: 사용자가 탭 C 를 A 앞으로 드래그한다
- Then: 탭 순서가 [C, A, B] 로 변경된다. tab_order 가 DB 에 반영된다.

**AC-2.5: 활성 탭 표시**
- Given: pane 에 3개의 탭이 있는 상태
- When: 사용자가 2번째 탭을 클릭한다
- Then: 2번째 탭이 시각적으로 활성 표시 (밑줄 또는 배경색) 되고, 해당 surface 가 표시된다.

**AC-2.6: Tab 영속**
- Given: pane 에 3개 탭 [Terminal, FileTree, Markdown] 이 있고 2번째 탭이 활성
- When: 앱을 종료하고 재시작한다
- Then: 동일한 3개 탭이 같은 순서로 복원되고, 2번째 탭이 활성화된다.

---

### RG-M2-3: Command Palette

**AC-3.1: 열기/닫기**
- Given: 앱이 활성 상태
- When: 사용자가 Cmd+K 를 누른다
- Then: 화면 중앙 상단에 Command Palette 오버레이가 표시된다. Escape 로 닫힌다.

**AC-3.2: Fuzzy search**
- Given: Command Palette 가 열린 상태
- When: 사용자가 "mkdn" 을 입력한다
- Then: "Open Markdown Surface" 명령이 결과 목록에 표시된다 (fuzzy matching).

**AC-3.3: Slash injection**
- Given: Command Palette 가 열린 상태, 워크스페이스에 Claude subprocess 가 실행 중
- When: 사용자가 `/moai plan "auth system"` 을 선택한다
- Then: 해당 텍스트가 Rust core slash injection 경로로 전달되어 Claude subprocess 에 SDKUserMessage 로 도달한다.

**AC-3.4: 키보드 네비게이션**
- Given: Command Palette 에 5개 결과가 표시된 상태
- When: 사용자가 Arrow Down 2회 -> Enter 를 누른다
- Then: 3번째 명령이 실행된다.

**AC-3.5: Surface 열기**
- Given: Command Palette 에서 "Open FileTree" 를 선택
- When: 명령이 실행된다
- Then: 활성 pane 에 새 탭으로 FileTree surface 가 열린다.

---

### RG-M2-4: FileTree Surface

**AC-4.1: 트리 렌더링**
- Given: 워크스페이스 루트에 10개 파일 + 2개 디렉토리 (하위 5개 파일) 가 있는 상태
- When: FileTree surface 가 활성화된다
- Then: 루트 레벨에 10개 파일 + 2개 디렉토리가 표시된다. 디렉토리는 collapse 상태.

**AC-4.2: Expand/Collapse**
- Given: FileTree 에 collapse 된 디렉토리가 표시된 상태
- When: 사용자가 디렉토리를 클릭한다
- Then: 하위 파일 목록이 펼쳐진다. 다시 클릭하면 접힌다.

**AC-4.3: Git status 색상**
- Given: git 저장소에서 `file.rs` 가 modified, `new.rs` 가 untracked 상태
- When: FileTree 에 해당 파일이 표시된다
- Then: `file.rs` 는 노란색, `new.rs` 는 회색으로 표시된다.

**AC-4.4: 파일 열기 (확장자별)**
- Given: FileTree 에 `README.md` 와 `logo.png` 가 표시된 상태
- When: 사용자가 `README.md` 를 더블클릭한다
- Then: 활성 pane 에 새 탭으로 Markdown surface 가 열리고 내용이 렌더링된다.

**AC-4.5: 실시간 갱신**
- Given: FileTree surface 가 활성 상태
- When: 외부에서 (터미널 등) 새 파일 `test.txt` 를 생성한다
- Then: 300ms 이내에 FileTree 에 `test.txt` 가 나타난다.

---

### RG-M2-5: Markdown Surface

**AC-5.1: 기본 렌더링**
- Given: `# Title\n\nParagraph text\n\n- item 1\n- item 2` 내용의 .md 파일
- When: Markdown surface 에서 열린다
- Then: H1 제목, 단락, 불릿 리스트가 올바르게 렌더링된다.

**AC-5.2: EARS SPEC 포매팅**
- Given: EARS 형식의 spec.md 파일 (requirement ID, Given/When/Then 블록 포함)
- When: Markdown surface 에서 열린다
- Then: requirement ID 가 강조 표시되고, Given/When/Then 블록이 시각적으로 구분된다.

**AC-5.3: KaTeX 수식**
- Given: `$$E = mc^2$$` 블록이 포함된 .md 파일
- When: Markdown surface 에서 열린다
- Then: LaTeX 수식이 올바르게 렌더링된다 (WKWebView 임베드).

**AC-5.4: Mermaid 다이어그램**
- Given: ` ```mermaid\ngraph TD\nA-->B\n``` ` 블록이 포함된 .md 파일
- When: Markdown surface 에서 열린다
- Then: 플로우차트 다이어그램이 올바르게 렌더링된다.

**AC-5.5: 자동 리로드**
- Given: Markdown surface 에 파일이 열린 상태
- When: 외부에서 해당 파일을 수정한다
- Then: 1초 이내에 변경된 내용으로 리로드된다.

**AC-5.6: 다크/라이트 테마**
- Given: Markdown surface 가 활성 상태
- When: macOS 시스템 설정에서 다크 모드로 전환한다
- Then: Markdown 렌더링이 다크 테마로 변경된다 (배경/전경색 반전).

---

### RG-M2-6: Image Surface

**AC-6.1: 이미지 표시**
- Given: 3MB PNG 파일
- When: Image surface 에서 열린다
- Then: 500ms 이내에 "Fit to Window" 모드로 이미지가 표시된다.

**AC-6.2: Zoom/Pan**
- Given: Image surface 에 이미지가 표시된 상태
- When: 사용자가 Cmd++ 로 줌인하고 드래그로 패닝한다
- Then: 이미지가 확대되고, 패닝으로 보이지 않는 영역을 탐색할 수 있다.

**AC-6.3: Diff 모드 + SSIM**
- Given: Image surface 에 원본 이미지가 열린 상태
- When: 사용자가 diff 모드를 활성화하고 비교 이미지를 선택한다
- Then: 두 이미지가 side-by-side 로 표시되고, SSIM 점수가 0~1 범위로 표시된다.

**AC-6.4: 다양한 포맷**
- Given: JPEG, GIF, SVG, WebP 파일
- When: 각각 Image surface 에서 열린다
- Then: 모든 포맷이 정상적으로 표시된다.

---

### RG-M2-7: Browser Surface

**AC-7.1: URL 네비게이션**
- Given: Browser surface 가 열린 상태
- When: 사용자가 URL bar 에 `https://example.com` 을 입력하고 Enter
- Then: 해당 웹 페이지가 WKWebView 에 로드된다.

**AC-7.2: Dev server auto-detect**
- Given: localhost:3000 에서 React dev 서버가 실행 중
- When: Browser surface 가 활성화된다
- Then: 자동으로 `http://localhost:3000` 을 감지하여 URL bar 에 표시하고 로드한다.

**AC-7.3: 네비게이션 컨트롤**
- Given: Browser surface 에서 2개 페이지를 순차 방문한 상태
- When: 사용자가 뒤로 버튼을 클릭한다
- Then: 이전 페이지로 이동한다. 앞으로 버튼으로 다시 돌아올 수 있다.

**AC-7.4: 외부 링크**
- Given: Browser surface 에 `https://internal.dev` 가 로드된 상태
- When: 사용자가 `https://github.com` 외부 도메인 링크를 클릭한다
- Then: 시스템 기본 브라우저에서 해당 URL 이 열린다.

---

### RG-M2-8: CI/CD Pipeline

**AC-8.1: Rust CI**
- Given: Rust 소스 코드가 변경된 PR
- When: PR 이 생성 또는 push 된다
- Then: GitHub Actions 에서 `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` 가 순차 실행되어 모두 통과한다.

**AC-8.2: Swift CI**
- Given: Swift 소스 코드가 변경된 PR
- When: PR 이 생성 또는 push 된다
- Then: GitHub Actions 에서 `xcodebuild build-for-testing`, `xcodebuild test` 가 실행되어 통과한다.

**AC-8.3: 캐싱**
- Given: 이전 CI 실행에서 GhosttyKit xcframework 가 빌드된 상태
- When: GhosttyKit 관련 파일 변경 없이 새 CI 가 실행된다
- Then: 캐시에서 xcframework 를 복원하여 빌드 시간을 절약한다.

**AC-8.4: CI 매트릭스**
- Given: CI 워크플로우가 실행된다
- When: 매트릭스 설정을 확인한다
- Then: macOS 14+ runner 에서 Xcode 15+ 로 테스트가 실행된다.

---

### RG-M2-9: M1 Carry-over

**AC-9.1: C-1 UITest 서명**
- Given: CI 환경이 구성된 상태
- When: UITest 를 포함한 CI 가 실행된다
- Then: `E2EWorkingShellTests` 가 서명 오류 없이 실행된다.

**AC-9.2: C-2 Claude CLI E2E**
- Given: Claude CLI 가 설치되고 API key 가 설정된 환경
- When: `scripts/validate-claude-e2e.sh` 를 실행한다
- Then: MoAI Studio -> Rust -> Claude subprocess -> 응답 수신 파이프라인이 성공한다.

**AC-9.3: C-3 Stress Test**
- Given: 앱이 실행 중이고 4개 워크스페이스가 활성 상태
- When: 10분간 stress test 를 실행한다
- Then: RSS 가 400MB 미만이고 deadlock 이 발생하지 않는다.

**AC-9.4: C-6 Token Rotation**
- Given: moai-hook-http 에 만료된 auth token 이 설정된 상태
- When: hook HTTP 요청이 발생한다
- Then: 토큰이 자동으로 갱신되고 요청이 성공한다.

**AC-9.5: C-8 force_paused API**
- Given: 워크스페이스가 running 상태
- When: `force_paused()` public API 를 호출한다
- Then: 상태가 paused 로 전환되고, 재개 가능하다.

---

## 2. Edge Cases

| 시나리오 | 기대 동작 |
|----------|-----------|
| 4단계 깊이 pane split (16개 leaf) | 렌더링 정상, 각 pane 최소 200pt 보장 (화면 크기 부족 시 스크롤 없이 분할 거부) |
| 100개 탭 열기 | 탭 bar 가 가로 스크롤로 대응, 메모리 leak 없음 |
| 10,000 파일 디렉토리 FileTree | 500ms 이내 초기 로드 (lazy loading), 스크롤 60fps |
| 빈 디렉토리 FileTree | "(빈 디렉토리)" 메시지 표시 |
| 잘못된 Markdown 구문 | 원본 텍스트 그대로 표시 (crash 없음) |
| 깨진 이미지 파일 | 에러 아이콘 + "이미지를 표시할 수 없습니다" 메시지 |
| localhost 포트 없음 (Browser) | "실행 중인 dev 서버를 찾을 수 없습니다" 메시지 표시 |
| Command Palette 에서 존재하지 않는 명령 검색 | "결과 없음" 메시지 표시 |
| 앱 비정상 종료 후 재시작 | 마지막 저장된 레이아웃 복원 시도, 실패 시 기본 레이아웃 (단일 pane + Terminal) |
| DB V3 마이그레이션 실패 | 기존 V2 상태 유지, 에러 로그 기록, 사용자 알림 |
| WKWebView 로딩 타임아웃 (30s) | 타임아웃 메시지 표시, 재시도 버튼 제공 |
| 동시에 같은 파일 Markdown + FileTree 에서 열기 | 둘 다 정상 표시, 파일 변경 시 둘 다 갱신 |

---

## 3. Quality Gate (Definition of Done)

### 코드 품질
- [ ] `cargo check --workspace`: 0 errors, 0 warnings
- [ ] `cargo clippy --workspace -- -D warnings`: 0 errors, 0 warnings
- [ ] `cargo fmt --all -- --check`: clean
- [ ] `cargo test --workspace`: 모든 테스트 통과 (M1 186 + M2 신규)
- [ ] Xcode build: 0 errors, 0 warnings (deprecation 경고 허용)
- [ ] SwiftUI Preview: 주요 뷰 preview 동작

### 테스트 커버리지
- [ ] Rust: 신규 코드 85%+ 커버리지
- [ ] Swift: 신규 뷰모델/로직 코드 70%+ 커버리지
- [ ] E2E: pane split -> tab -> surface -> command palette 전체 흐름 1건 이상

### NFR 달성
- [ ] Pane split 반응 <100ms
- [ ] Tab 전환 <50ms
- [ ] Command Palette 열기 <200ms
- [ ] FileTree 초기 로드 <500ms (1000 파일)
- [ ] Markdown 렌더링 <1s (100KB)
- [ ] Image 로드 <500ms (10MB)
- [ ] 메모리 RSS <600MB (8 pane, 8 tab)
- [ ] 앱 재시작 후 레이아웃 100% 복원

### TRUST 5
- [ ] Tested: 위 커버리지 달성
- [ ] Readable: clippy + fmt clean, 한국어 코드 주석
- [ ] Unified: SurfaceProtocol 일관 적용
- [ ] Secured: WKWebView sandbox, 외부 URL 격리
- [ ] Trackable: conventional commits, T-xxx 참조

### @MX Tags
- [ ] 신규 public API: @MX:ANCHOR 또는 @MX:NOTE 부착
- [ ] fan_in >= 3 함수: @MX:ANCHOR 필수
- [ ] 복잡도 >= 15: @MX:WARN + @MX:REASON 필수
- [ ] TODO 0건 (GREEN 완수)

### CI/CD
- [ ] GitHub Actions Rust CI 통과
- [ ] GitHub Actions Swift CI 통과
- [ ] GhosttyKit 캐싱 동작

### M1 Carry-over
- [ ] C-1: UITest CI 실행 확인
- [ ] C-2: Claude CLI E2E 검증 (수동 또는 스크립트)
- [ ] C-3: 4-ws 10min stress RSS <400MB
- [ ] C-4: Metal 60fps 벤치마크 측정
- [ ] C-5: Vectorizable workaround 제거 (조건부)
- [ ] C-6: Token rotation 구현
- [ ] C-7: FFI <1ms XCTest 통과
- [ ] C-8: force_paused public API 승격
