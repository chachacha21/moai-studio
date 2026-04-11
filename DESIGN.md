# moai-terminal 설계 문서 (v2)

> **목표**: Claude Code · Codex 위에서 **moai-adk (Go 단일 바이너리, Claude Code 전용 ADK)** 를 다중 에이전트로 동시에 운용할 수 있는 macOS 네이티브 에이전틱 IDE-쉘.
> **참고 오픈소스**:
> - [manaflow-ai/cmux](https://github.com/manaflow-ai/cmux) — 네이티브 터미널 셸 패턴
> - [modu-ai/moai-adk](https://github.com/modu-ai/moai-adk) — 통합 대상 ADK
>
> **작성일**: 2026-04-11 (v2 — v1 의 moai-adk 가정 오류 전면 수정 + Code Viewer 추가)

---

## 0. v1 대비 주요 변경 사항

v1 에서 내가 가정으로 적었던 내용 중 **사실과 다른 것** 을 바로잡는다.

| 항목 | v1 가정 (오류) | v2 사실 (근거) |
|---|---|---|
| 슬래시 커맨드 | `/alfred:1-spec`, `/alfred:2-build`, `/alfred:3-sync` | **`/moai plan`, `/moai run`, `/moai sync`** (+ `fix/loop/review/coverage/e2e/clean/mx/codemaps/feedback/project`, 그리고 `/agency`) |
| TAG | `@TAG SPEC-XXX:CODE-YY` | **`@MX:ANCHOR|WARN|NOTE|TODO`** — fan_in ≥ 3 등 임계값 기반 자동 부여 |
| TRUST 약어 | Traceability / Readability / Unified / Security / Tested | **Tested / Readable / Unified / Secured / Trackable** — LSP 품질 게이트와 결합 |
| 구현 언어 | TypeScript/Python 추정 | **Go 1.26+, 단일 바이너리, Cobra + Bubbletea** |
| 런타임 의존성 | Node/Python 등 | **없음** — `curl ... install.sh`, Windows 는 `install.ps1`. tmux 는 CG 모드에서만 |
| IDE 범위 | Claude Code / Codex / Cursor 전방위 | **Claude Code 전용.** Codex/Cursor 는 비지원 (Hook·Agent Teams API 가 Claude Code 독점) |
| 에이전트 | Alfred 중심 | **Alfred 없음.** 26개 전문 에이전트 (Manager 8 / Expert 8 / Builder 3 / Evaluator 1 / Agency 6) |
| 설정 | 단일 `moai.config.*` | **`.moai/config/sections/*.yaml`** 섹션 기반 (quality, workflow, language, llm, harness, mx, git-convention, security, constitution …) |
| Hook | 임의 wrapper | **Hook Protocol v2.10.1, 27개 이벤트** (SessionStart, PreToolUse, PostToolUse, TeammateIdle, TaskCompleted, SessionEnd …) |
| Trace | 별도 포맷 필요 | **이미 존재** — `.moai/logs/task-metrics.jsonl` |
| 병렬 모드 | 수동 worktree | moai-adk 자체가 **Agent Teams 모드** 에서 쓰기 팀원을 `isolation: "worktree"` 로 강제 |
| 멀티 LLM | 설계에서 누락 | **CG 모드 (Claude Leader + GLM Workers, tmux 기반)** 가 1급 기능 |
| **Code Viewer** | **누락** | **이번 v2 에서 1급 Surface 로 추가** |

---

## 1. 참고 저장소 분석 요약

### 1.1 cmux (manaflow-ai/cmux)

macOS 네이티브 터미널 워크스페이스. 우리가 계승할 베이스.

- **셸**: SwiftUI + AppKit. Electron/Tauri 미사용.
- **터미널 엔진**: **Ghostty** (`libghostty.xcframework`, Metal GPU 가속, PTY 내장).
- **레이아웃**: 자체 서브모듈 **Bonsplit**.
- **IPC**: `Sources/TerminalController.swift` (~16K LoC) — Unix domain socket 위의 JSON-RPC v2 + v1 텍스트.
- **계층**: `NSWindow → Workspace → Pane → Surface(Terminal|Browser)`.
- **권한**: 포그라운드 surface 만 위험 명령 가능한 **focus-intent** 모델.
- **원격**: Go 데몬 `daemon/remote/` 이 SSH reverse forward + HMAC challenge-response + 0600 relay 토큰으로 `localhost:포트` 투명 라우팅.
- **저장**: JSON 파일(`~/.cmux/`, `SessionPersistence.swift`). **SQLite 미사용**.
- **빌드/배포**: Xcode + create-dmg + notarytool + Sparkle appcast + 안정/Nightly 2채널.
- **없는 것**: 파일 탐색기, 마크다운 뷰어, 이미지 뷰어, **코드 뷰어**, 칸반, 에이전트 트레이스 뷰어.

### 1.2 moai-adk (modu-ai/moai-adk, Go Edition v2.7.x)

**제품 정의 (README.ko.md, CLAUDE.md 근거)**

> "바이브 코딩의 목적은 빠른 생산성이 아니라 코드 품질이다."
> Claude Code 위에서 동작하는 하네스 엔지니어링 ADK. 26개 전문 에이전트 + 47개 스킬이 TDD/DDD 기반으로 고품질 코드를 자동 생성한다. 18개 언어의 LSP/린터/테스트를 자동 선택한다.

**설치/실행**
- `curl -fsSL https://raw.githubusercontent.com/modu-ai/moai-adk/main/install.sh | bash` → `/usr/local/bin/moai`
- Windows: `irm ... install.ps1 | iex` (PowerShell 7.x+)
- Source: Go 1.26+, `make build`
- **의존성 없음**. Git 필수. tmux 는 **CG 모드에서만** 필수.
- Claude Code 가 필수 (`/moai ...` 슬래시 또는 `moai hook <event>` CLI).

**슬래시 커맨드 (실제 명세)**

| 카테고리 | 커맨드 | 역할 |
|---|---|---|
| 핵심 워크플로우 | `/moai plan`(=spec), `/moai run`(=impl), `/moai sync`(=docs,pr) | EARS SPEC → DDD/TDD 구현 → 문서/PR |
| 품질·테스트 | `/moai fix`, `/moai loop`(최대 100회), `/moai review`, `/moai coverage`, `/moai e2e`, `/moai clean` | LSP 자동 수정, 커버리지, Claude-in-Chrome/Playwright E2E, 데드 코드 제거 |
| 문서·코드베이스 | `/moai project`(=init), `/moai mx`, `/moai codemaps`, `/moai feedback` | 프로젝트 문서, @MX 태그 스캔, 아키텍처 코드맵, 피드백 수집 |
| 자율 | `/moai`(단독) | 완전 자율 plan→run→sync |
| Agency | `/agency "설명"` | 웹사이트 자율 제작 (인터뷰→빌드→테스트→학습) |

**워크플로우**

```
사용자 자연어
    ↓
/moai plan "기능 설명"
    ↓  (manager-spec 에이전트, EARS 형식)
.moai/specs/SPEC-{DOMAIN}-{NNN}/{spec.md, plan.md, research.md, acceptance.md}
    ↓
/moai run SPEC-AUTH-001
    ↓  (커버리지 ≥10% → TDD: RED→GREEN→REFACTOR,  <10% → DDD: ANALYZE→PRESERVE→IMPROVE)
구현 + LSP 품질 게이트 (max_errors=0, max_type_errors=0, max_lint_errors=0)
    ↓
/moai sync SPEC-AUTH-001
    ↓  (README/CHANGELOG 갱신, codemaps 업데이트, gh pr create)
```

**자동 모드 선택**: `domains ≥ 3 OR files ≥ 10 OR complexity_score ≥ 7` → **Agent Teams 모드** (병렬), 그 외 → Sub-Agent 순차. `--team / --solo` 로 강제 가능.

**TAG = @MX (MoAI eXplicit) 4종**

| 태그 | 트리거 | 한도 |
|---|---|---|
| `@MX:ANCHOR` | fan_in ≥ 3 인 함수/타입 | 파일당 ≤3 |
| `@MX:WARN` | goroutine, 복잡도 ≥ 15, 분기 ≥ 8 | 파일당 ≤5 |
| `@MX:NOTE` | 매직 상수, 누락된 godoc, 비즈니스 규칙 | 무제한 |
| `@MX:TODO` | 테스트 미작성, 미완성 구현 | 무제한 |

설정은 `.moai/config/sections/mx.yaml` 의 `thresholds`, `limits`, `exclude`.

**TRUST 5**

| | 의미 | LSP 게이트 |
|---|---|---|
| **T**ested | 85%+ 커버리지, 특성 테스트, 유닛 테스트 통과 | `lsp_type_errors==0` |
| **R**eadable | 명명 일관성, 린트 0 | `lsp_lint_errors==0` |
| **U**nified | 포맷팅·임포트 순서·구조 일관성 | `lsp_warnings < threshold` |
| **S**ecured | OWASP, 입력 검증, 보안 경고 0 | `lsp_security_warnings==0` |
| **T**rackable | Conventional commit, 이슈 참조, 구조화 로그 | Hook 메트릭 기록 |

**Ralph Engine**: LSP + AST-grep + 린터를 병렬 실행해 Level 1~4 로 에러 분류, 자동 수정 (`/moai fix`, `/moai loop`).

**디렉토리 규약**
```
.moai/
├── config/sections/
│   ├── quality.yaml          # dev_mode, coverage, LSP gates
│   ├── workflow.yaml         # team.enabled, complexity_thresholds
│   ├── language.yaml         # conversation/comments/commits 언어
│   ├── constitution.yaml     # 허용 언어/프레임워크, 금지 패턴
│   ├── llm.yaml              # policy: high|medium|low
│   ├── harness.yaml          # 검증 깊이
│   ├── mx.yaml               # @MX 임계값/제외
│   ├── git-convention.yaml
│   └── security.yaml
├── config/evaluator-profiles/*.md  # 4차원 채점 프로필
├── specs/SPEC-{DOMAIN}-{NNN}/
│   ├── spec.md               # EARS 형식 요구사항
│   ├── plan.md
│   ├── research.md
│   └── acceptance.md
├── project/
│   ├── product.md / structure.md / tech.md
│   └── codemaps/architecture.md, api-design.md, database.md
├── scripts/
├── memory/                   # 중단 작업 재개용 스냅샷
└── logs/task-metrics.jsonl   # JSON Lines 메트릭
```

**Hook Protocol v2.10.1 (27개 이벤트)**

| 이벤트 | 시점 | 용도 |
|---|---|---|
| `SessionStart` | Claude Code 세션 시작 | 환경 초기화, CLAUDE.md 로드 |
| `PreToolUse` | 도구 호출 직전 | 권한/입력 검증 |
| `PostToolUse` | 도구 호출 직후 | 메트릭 기록, @MX 관리, LSP 진단 |
| `PostToolUseFailure` | 실패 시 | 에러 분류, 재시도 |
| `TeammateIdle` | 팀원 대기 | LSP 품질 게이트 검증 |
| `TaskCompleted` | 작업 완료 | SPEC 문서/완료 마커 검증 |
| `SessionEnd` | 세션 종료 | 메모 저장, 정리 |
| …총 27개 |

Hook 타입 4종: **command / prompt / agent / http**. 설정: `.moai/hooks.yaml`.

**Task 메트릭 (parsable)** — `.moai/logs/task-metrics.jsonl`
```json
{"timestamp":"2026-04-11T10:30:45Z","session_id":"sess-abc","task_id":"task-xyz",
 "agent_type":"manager-tdd","operation":"RED","input_tokens":4500,"output_tokens":2800,
 "total_tokens":7300,"duration_ms":3200,"tool_calls":5,
 "tools_used":["Read","Write","Bash","Grep"],"status":"success","spec_id":"SPEC-AUTH-001"}
```

**26개 에이전트**: Manager 8 (spec/ddd/tdd/docs/quality/project/strategy/git), Expert 8 (backend/frontend/security/devops/performance/debug/testing/refactoring), Builder 3 (agent/skill/plugin), Evaluator 1 (active), Agency 6 (planner/copywriter/designer/builder/evaluator/learner).

**모델 정책**: High (Max $200, Opus 16/Sonnet 5/Haiku 3), Medium (Max $100, Opus 3/Sonnet 17/Haiku 4), Low (Plus $20, Opus 0/Sonnet 13/Haiku 11).

**CG 모드** (핵심): `moai glm <key>` → `moai cg` → Claude Leader + GLM Workers (tmux 환경 격리). 비용 60~70% 절감, 품질/비용 균형 권장 모드.

**Codex 현실**: moai-adk 는 **Claude Code 전용** 이다. Hook 과 Agent Teams API 가 Claude Code 독점이라 Codex 에서는 moai-adk 풀 파이프라인이 동작하지 않는다. 다만 moai-adk 가 만들어 둔 **산출물(SPEC 문서, TRUST 게이트, @MX 태그, task-metrics.jsonl)** 은 언어/IDE 중립이므로 Codex 는 "읽기 + 수동 편집 + 일반 쉘 실행" 수준까지는 된다. moai-terminal 은 이 비대칭을 명시적으로 반영해야 한다.

---

## 2. moai-terminal 제품 정의 (재정의)

### 2.1 한 줄 정의

> **moai-terminal**: macOS 네이티브 네이티브 에이전틱 IDE-쉘. **Claude Code 기반 moai-adk 다중 세션의 1급 호스트**이며, Codex 는 동일 창에서 "일반 코딩 에이전트" 로 병치 운용된다. 파일/코드/마크다운/이미지/브라우저/터미널/에이전트-런/칸반 을 한 화면에서 검수한다.

### 2.2 핵심 요구사항 (사용자 명시 + v2 추가)

1. 파일 탐색기
2. GPU 가속 터미널 (멀티 PTY)
3. **코드 뷰어 (v2 신규, 1급 Surface)**
4. 마크다운 뷰어 (라이브, EARS SPEC 특화)
5. 내장 브라우저 (DevTools)
6. 이미지 뷰어 (diff 모드 포함)
7. 에이전트 작업 뷰어 (Hook 이벤트 + task-metrics 시각화)
8. 에이전트 칸반 보드 (SPEC 카드 ↔ worktree ↔ `/moai run` 자동 연동)
9. 다중 멀티 세션 (16+ 에이전트 동시 협업)
10. moai-adk 의 모든 `/moai *` 커맨드를 **GUI 1-클릭** 으로 호출

### 2.3 비기능 요구사항

| 항목 | 목표치 |
|---|---|
| 콜드 스타트 (M1) | < 0.8s |
| 활성 메모리 (8 PTY + 2 WebView + 4 Code surface) | < 700 MB |
| 터미널 스크롤 | 60 fps @ 4K |
| 동시 에이전트 세션 | 16+ 안정 |
| 세션 복원 | 재시작 후 < 2s |
| IPC 응답 (p95) | < 30 ms |
| task-metrics.jsonl tail 지연 | < 500 ms |
| 크래시 격리 | 에이전트 1개 크래시가 나머지에 전파되지 않음 |

---

## 3. 아키텍처

### 3.1 전체 구조

```
┌──────────────────────────────────────────────────────────────────┐
│                    moai-terminal.app (macOS)                     │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                SwiftUI / AppKit Shell                      │  │
│  │   Sidebar · Tabs · Splits · Command Bar · Inspector        │  │
│  └─────────────┬──────────────────────────────┬───────────────┘  │
│                │                              │                  │
│  ┌─────────────▼──────────┐       ┌───────────▼──────────────┐  │
│  │   Surface Plugins      │       │   moai-core (Rust)        │  │
│  │  ────────────────────  │◄─────►│  ───────────────────────  │  │
│  │  • Terminal (Ghostty)  │  UDS  │  • Session Orchestrator   │  │
│  │  • Code Viewer ★NEW    │ JSON- │  • Agent Supervisor       │  │
│  │    (Tree-sitter +      │ RPC v2│    (OTP-style, one_for_one)│ │
│  │     ast-grep + LSP)    │       │  • PTY Pool               │  │
│  │  • Markdown            │       │  • Worktree Manager       │  │
│  │    (cmark-gfm + KaTeX  │       │  • SQLite (WAL)           │  │
│  │     + Mermaid)         │       │  • Event Bus (in-proc)    │  │
│  │  • Image Viewer        │       │  • File Watcher (FSEvents)│  │
│  │  • File Explorer       │       │  • Git Service (git2)     │  │
│  │  • Browser (WKWebView) │       │  • moai-adk Bridge        │  │
│  │  • Agent Run Viewer    │       │    (hook tail + jsonl     │  │
│  │  • Kanban Board        │       │     ingest + slash exec)  │  │
│  └────────────────────────┘       └───────────┬──────────────┘  │
│                                                │                  │
└────────────────────────────────────────────────┼──────────────────┘
                                                 │
                   ┌─────────────────────────────┴─────────────────┐
                   │        External processes (PTY children)       │
                   │ ─────────────────────────────────────────────── │
                   │ • claude  (Claude Code CLI, 메인 에이전트 호스트)│
                   │ • moai    (moai-adk Go 바이너리, hook/slash 실행)│
                   │ • codex   (Codex CLI, 병치 세션)                │
                   │ • tmux    (CG 모드 전용, Claude+GLM 분할)       │
                   │ • zsh     (일반 쉘)                             │
                   │ • cmuxd-remote (옵션, SSH 원격)                │
                   └─────────────────────────────────────────────────┘
```

### 3.2 추상화 계층 (5단)

```
Window
 └── Project          ← git 루트 + .moai/ 설정 감지. 없으면 `/moai project` 제안
      └── Workspace   ← 1 worktree = 1 에이전트 호스트
           ├── agent_host: claude_code | codex | shell | tmux_cg
           ├── binds: SPEC-{DOMAIN}-{NNN}  (옵션)
           └── Pane    ← Bonsplit-like 분할
                └── Surface  (Terminal | CodeViewer | Markdown | Image |
                              Browser | FileTree | AgentRun | Kanban)
```

- **Project** 계층 (cmux 엔 없음): `.moai/config/sections/` 존재 여부로 moai-adk 프로젝트임을 감지. 없으면 워크스페이스 생성 시 자동 `/moai project` 안내.
- **Workspace = git worktree**: moai-adk 의 Agent Teams 가 쓰기 팀원을 `isolation:"worktree"` 로 강제하는 정책과 1:1 로 매핑. moai-terminal 은 이 worktree 의 **라이프사이클 오너**.
- **agent_host** 필드로 워크스페이스가 "Claude Code + moai-adk 풀파이프" 인지, "Codex 순수 세션" 인지 명시.

### 3.3 멀티 에이전트 모델 — 이것이 핵심

**원칙**

1. **1 Agent = 1 Worktree = 1 Top-level PTY tree.** moai-adk Agent Teams 의 worktree 격리와 동일 정책.
2. **Supervisor 가 라이프사이클 소유 (Erlang OTP one_for_one)**. Rust `AgentSupervisor` 가 자식을 Tokio task 로 감싼다. PTY 의 master fd 는 **Rust 코어 소유** (UI crash 와 무관하게 생존).
3. **단방향 이벤트 버스**. PTY stdout → Rust reader → in-proc EventBus → (a) SQLite append-only, (b) UI 구독자, (c) moai-adk Bridge. UI 는 PTY 를 직접 read 하지 않는다 → 멀티 구독 가능.
4. **권한 = focus-intent + scoped token**. cmux focus-intent 계승. 외부 스크립트는 워크스페이스 ID 에 묶인 scoped token 으로만 호출. 교차 워크스페이스 조작은 명시적 권한 상승.
5. **moai-adk 에 손대지 않고 얹는다**. moai-terminal 은 moai-adk 의 구현을 재현하지 않는다. `moai` 바이너리를 호출하고, `.moai/logs/task-metrics.jsonl` 을 tail 하며, `.moai/hooks.yaml` 의 http hook 으로 이벤트를 전달받을 뿐이다.

**PTY Pool**
- Rust `portable-pty` 로 master 를 만들고, slave fd 를 child (claude/codex/zsh/tmux) 에 넘긴다.
- Ghostty Surface 는 "attach 모드" — 렌더만 담당. PTY 재사용 시 기존 히스토리를 SQLite 에서 replay.

**Worktree 자동화 플로우** (칸반 Doing 으로 드래그 시):
```
1. git worktree add .moai/worktrees/spec-042 -b agent/spec-042 origin/main
2. SQLite workspaces insert (status='starting')
3. PTY Pool 에 child spawn: claude  (cwd=worktree)
4. Pre-input: /moai run SPEC-042  (자동 입력)
5. SessionStart hook 수신 → status='running'
6. PostToolUse hook 스트림 → Agent Run Viewer 실시간 업데이트
7. TaskCompleted hook 또는 Ralph Engine 0-error 확정 → status='review'
8. 카드가 Review 레인으로 자동 이동
```

**백프레셔**: ring buffer (워크스페이스당 4MB) 꽉 차면 UI 구독을 중단하고 SQLite 에만 기록. UI 에는 "Output paused" 배너.

### 3.4 IPC

**외부 제어**: `~/.moai-terminal/sock/control.sock` (0600, JSON-RPC v2 only)
**이벤트 fan-out**: `~/.moai-terminal/sock/events.sock` (line-delimited JSON, 구독 전용)

주요 메서드:
| 네임스페이스 | 메서드 |
|---|---|
| `project.*` | list, open, detect_moai_adk, ensure_config |
| `workspace.*` | list, create(agent_host,spec_id?), attach, archive, merge |
| `pane.*` | split, focus, close, layout.get/set |
| `surface.*` | create(kind), focus, close, send_input, capture |
| `agent.*` | start, stop, restart, status, send_prompt |
| `moai.*` | **plan, run, sync, fix, loop, review, coverage, e2e, clean, mx, codemaps, project, feedback, agency** — 모두 `moai` 바이너리 래핑 |
| `hook.*` | subscribe, tail, last(type) |
| `metrics.*` | tail_task_metrics, query(spec_id?, agent_type?, since?) |
| `kanban.*` | board.get, card.create/move/update, label.* |
| `file.*` | tree, watch, read, write, reveal |
| `code.*` | open(path,line?), symbols(path), find_refs, go_to_definition, diff(path,rev_a,rev_b) |
| `git.*` | status, diff, branch, worktree.add/remove/list |
| `markdown.*` | render(path), watch(path), render_ears(spec_id) |

이 IPC 가 있기 때문에 **moai-adk 슬래시 커맨드 → moai-terminal GUI 액션** 양방향 브리지가 가능하다.

### 3.5 moai-adk Bridge (1급 통합 모듈)

`moai-core` 안의 `moai-adk-bridge` crate 가 전담.

1. **감지**: 프로젝트 루트에 `.moai/config/sections/` 가 있으면 moai-adk 프로젝트로 인식. `moai --version` 으로 버전 픽업.
2. **설정 파싱**: `quality.yaml`, `workflow.yaml`, `language.yaml`, `llm.yaml`, `mx.yaml` 등을 Rust 구조체로 파싱해 UI 에 공급.
3. **슬래시 실행**: `moai.run` IPC → `moai plan|run|sync ...` 를 해당 워크스페이스 PTY 에 주입. 단독 CLI 가 필요한 경우 (`moai hook ...`) 는 별도 서브프로세스.
4. **Hook http sink 등록**: 프로젝트 초기화 시 `.moai/hooks.yaml` 에 `type: http` 훅을 **옵트인** 으로 추가 → `POST http://127.0.0.1:<port>/hooks` 로 27개 이벤트 수신.
   ```yaml
   hooks:
     - event: PostToolUse
       type: http
       url: http://127.0.0.1:${MOAI_TERMINAL_PORT}/hooks
       headers:
         X-MoAI-Terminal-Token: ${MOAI_TERMINAL_TOKEN}
   ```
5. **task-metrics tail**: `.moai/logs/task-metrics.jsonl` 을 `inotify`/`FSEvents` 로 tail → EventBus → Agent Run Viewer + Kanban card 상태 갱신.
6. **@MX 스캐너 피드**: `/moai mx --dry` 결과(JSON) 를 Code Viewer 사이드바의 "ANCHOR / WARN / NOTE / TODO" 트리로 표시.
7. **LSP 게이트 대시보드**: `.moai/config/sections/quality.yaml` 의 `lsp_quality_gates.run.max_errors` 등 임계값을 불러와, 실시간 lint/type 에러 카운트 오버레이. 초과 시 워크스페이스 상단에 붉은 배너.
8. **CG 모드 지원**: workspace 생성 시 `agent_host=tmux_cg` 옵션 → `tmux new -s moai-cg-<ws>` 안에서 `moai cg` 실행. Pane 분할을 tmux 가 관리하지만, 우리는 tmux control mode (`-CC`) 로 구조를 받아 UI 와 동기화.
9. **Codex 병치**: Codex 워크스페이스는 moai-adk Hook/Agent Teams API 를 못 쓰므로, 대신 (a) `.moai/specs/` 읽기 전용 surface, (b) `git diff` 리뷰 surface, (c) TRUST 게이트 수동 실행 버튼 만 제공. UI 에서 "moai-adk Limited" 배지로 명시.

### 3.6 데이터 모델 (SQLite WAL)

```sql
CREATE TABLE projects (
  id INTEGER PRIMARY KEY, root TEXT UNIQUE, name TEXT,
  is_moai_adk INT, moai_version TEXT, opened_at INT
);

CREATE TABLE workspaces (
  id INTEGER PRIMARY KEY, project_id INT, name TEXT, branch TEXT,
  worktree_path TEXT, agent_host TEXT,   -- claude_code|codex|shell|tmux_cg
  spec_id TEXT NULL,                      -- e.g. SPEC-AUTH-001
  status TEXT,                            -- starting|running|waiting|review|error|archived
  created_at INT, last_active_at INT
);

CREATE TABLE panes (id INTEGER PK, workspace_id INT, parent_id INT, split TEXT, ratio REAL);

CREATE TABLE surfaces (
  id INTEGER PK, pane_id INT,
  kind TEXT,   -- terminal|code|markdown|image|browser|filetree|agent_run|kanban
  state_json TEXT
);

-- moai-adk task-metrics.jsonl 미러
CREATE TABLE task_metrics (
  id INTEGER PK, workspace_id INT, ts INT,
  session_id TEXT, task_id TEXT, agent_type TEXT, operation TEXT,
  input_tokens INT, output_tokens INT, total_tokens INT, duration_ms INT,
  tool_calls INT, tools_used TEXT, status TEXT, spec_id TEXT
);
CREATE INDEX task_metrics_ws_ts ON task_metrics(workspace_id, ts);
CREATE INDEX task_metrics_spec ON task_metrics(spec_id);

-- Hook 이벤트 스트림
CREATE TABLE hook_events (
  id INTEGER PK, workspace_id INT, ts INT,
  event TEXT,            -- SessionStart|PreToolUse|PostToolUse|...27종
  payload TEXT           -- JSON
);
CREATE INDEX hook_events_ws_ts ON hook_events(workspace_id, ts);

-- 에이전트 PTY 로그 (append-only, 14일 TTL)
CREATE TABLE agent_events (
  id INTEGER PK, workspace_id INT, ts INT, level TEXT,
  channel TEXT, payload TEXT
);

-- SPEC 카탈로그 (파일에서 파싱해 미러)
CREATE TABLE specs (
  id TEXT PRIMARY KEY,   -- SPEC-AUTH-001
  project_id INT, title TEXT, ears_md TEXT, plan_md TEXT,
  status TEXT,           -- draft|running|review|done
  updated_at INT
);

-- @MX 태그 인덱스 (/moai mx --dry 결과 미러)
CREATE TABLE mx_tags (
  id INTEGER PK, project_id INT, path TEXT, line INT,
  kind TEXT,             -- ANCHOR|WARN|NOTE|TODO
  reason TEXT
);

-- 칸반
CREATE TABLE kanban_boards (id INTEGER PK, project_id INT, name TEXT);
CREATE TABLE kanban_cards (
  id INTEGER PK, board_id INT, lane TEXT,   -- backlog|todo|doing|review|done|blocked
  title TEXT, body_md TEXT,
  workspace_id INT NULL, spec_id TEXT NULL, assignee TEXT NULL,
  created_at INT, updated_at INT
);

CREATE TABLE notifications (id INTEGER PK, ts INT, kind TEXT, ref TEXT, body TEXT, read INT);
```

`task_metrics` 와 `hook_events` 는 append-only + 30일 TTL. `journal_mode=WAL`, `synchronous=NORMAL`, batch insert 로 16 에이전트 × 50 events/s 견딤.

---

## 4. Surface 별 설계

### 4.1 파일 탐색기
- FSEvents + `notify` crate + EventBus.
- git2 로 worktree 별 M/A/D/? 색상.
- Reveal in Finder / Open in Code Viewer / Send path to focused agent / Diff against main / **"Create SPEC from selection"** (선택 파일 목록을 EARS 초안 생성용으로 `/moai plan` 에 첨부) 컨텍스트 액션.
- 드래그 드롭: 외부 파일 → 현재 workspace worktree 복사 + 에이전트 컨텍스트 자동 첨부.

### 4.2 Terminal (Ghostty)
- libghostty.xcframework 정적 링크. PTY master 소유는 Rust.
- Sixel / iTerm2 inline image 지원.
- **moai-adk 데코레이션**: Hook 이벤트와 동일 타임라인에 있는 PTY 출력 라인의 좌측 거터에 `●plan`, `●run`, `●sync`, `●fix`, `●loop` 등 아이콘. Hook JSON 의 `operation` 필드로 매핑.
- 스크롤백 검색은 SQLite FTS5 (agent_events 풀텍스트) 로 100k 라인도 < 50ms.
- **Command palette → Slash injection**: Cmd+K 에서 `/moai run SPEC-AUTH-001` 선택 시 포커스 워크스페이스의 claude PTY 에 자동 입력 + 엔터.

### 4.3 Code Viewer ★ v2 신규
cmux 에 없는, v1 에서 누락했던, 가장 중요한 Surface 중 하나.

**목표**: 에이전트가 자동으로 고치고 있는 코드를 사람이 **읽고/리뷰하고/되돌리기** 위한 1급 뷰어. 편집기 전체가 아니라 **리뷰+점프+@MX 가시화** 에 최적화.

**구성 요소**
- **렌더러**: SwiftUI `TextEditor` 가 아닌 `NSTextView` 서브클래스 + Tree-sitter 기반 하이라이트 (`tree-sitter` crate, Rust 측에서 파스 → Swift 로 attributed range 전송). 18개 언어 모두 커버.
- **LSP 클라이언트**: `moai-core` 가 project-local LSP (gopls, rust-analyzer, pyright, tsserver, …) 를 풀링. 파일 열기 → 진단 마커 + hover + go-to-definition + find-references.
- **@MX 거터**: Rust 가 `/moai mx --dry --json` 결과를 SQLite `mx_tags` 에 캐시. Code Viewer 는 파일을 열 때마다 해당 path 의 row 를 가져와 좌측 거터에 ★(ANCHOR)/⚠(WARN)/ℹ(NOTE)/☐(TODO) 아이콘. 클릭하면 우측 inspector 에 reason + 관련 SPEC 이 표시.
- **LSP quality gate 오버레이**: 파일 상단에 "errors: 0 / type_errors: 0 / lint: 2 ⚠" 같은 스트립. quality.yaml 의 임계값과 비교해 초과시 붉은색.
- **Tri-pane diff 모드**:
  ```
  [left: HEAD of main]  [center: working tree]  [right: agent's pending change]
  ```
  에이전트가 쓰기 중인 파일은 우측 pane 이 실시간으로 업데이트 (filesystem watch). Accept/Revert 버튼.
- **SPEC 링크**: 파일 내 `@MX:ANCHOR` 주석이 SPEC-ID 를 언급하면 자동으로 해당 SPEC markdown surface 로 점프 가능. 반대로 SPEC 의 acceptance 항목에서 "관련 파일" 링크 클릭 시 Code Viewer 가 뜬다.
- **Time travel**: `git log -p` 를 SQLite 에 인덱싱해 특정 파일의 과거 리비전을 슬라이더로 스크럽. 각 시점의 task-metric(토큰/모델/소요) 를 하단에 바 차트로.
- **편집은 최소**: 사용자가 직접 타이핑할 수는 있지만, 기본 컨셉은 "에이전트가 쓰고 사람이 읽는다". 직접 편집 시 즉시 git stash-like 스냅샷을 만들어 에이전트가 덮어쓰더라도 복구 가능.
- **보안**: read-only 기본. 편집 모드는 Pane focus-intent 필요 (cmux 패턴 계승).

Code Viewer 는 Terminal 과 함께 **워크스페이스의 필수 2대 Surface**. 칸반 카드를 Doing 으로 보내면 자동으로 Terminal(좌) + Code Viewer(중) + Agent Run Viewer(우) 3-pane 레이아웃이 세팅된다.

### 4.4 마크다운 뷰어
- `cmark-gfm` (C FFI) 또는 `comrak` (pure Rust). WebView 에 렌더.
- 확장: KaTeX (수식), Mermaid (다이어그램), Shiki (하이라이트).
- 라이브: FSEvents 200ms debounce.
- **EARS 특화 모드**: `.moai/specs/SPEC-*/spec.md` 열리면 Given/When/Then 블록을 카드로 렌더, acceptance 체크리스트는 인터랙티브 체크박스 (체크하면 SQLite `specs` 업데이트 + 파일 재쓰기).
- **2-up**: 좌측 SPEC, 우측 관련 파일 diff (`git diff` vs main).

### 4.5 Browser (WKWebView)
- cmux `BrowserWindowPortal.swift` 패턴.
- `setInspectable(true)` → Safari Web Inspector 로 DevTools.
- Port 스캐너: 에이전트가 띄운 dev 서버(예: 3000, 5173, 8080) 자동 감지 → 사이드바 "Listening ports" 에 나열, 클릭으로 Browser surface 생성.
- 원격: cmux Go relay (`daemon/remote`) 포팅해 SSH 호스트의 `localhost:포트` 투명 라우팅.
- **moai-adk `/moai e2e` 연동**: Playwright/Claude-in-Chrome 테스트 실행 시 결과 비디오/스크린샷을 Image Viewer 로 자동 오픈.

### 4.6 Image Viewer
- Core Image + Metal.
- `artifacts/` 폴더 자동 watch.
- **Diff 모드**: 두 PNG 픽셀 diff + SSIM 점수. `/moai e2e` 회귀 테스트 실패 시 자동 diff 표시.
- EXIF / 메타 사이드패널.

### 4.7 Agent Run Viewer
**데이터 소스**: (a) `task_metrics` 테이블, (b) `hook_events` 테이블, (c) `agent_events` 테이블.

**레이아웃**
- 좌측: 워크스페이스 내 세션/태스크 타임라인. 각 row = 1 task. 색상은 `agent_type`.
- 우측: 선택한 task 의 step-by-step 트레이스
  - `SessionStart` / `PreToolUse` / `PostToolUse` / `TaskCompleted` hook event 카드
  - 각 카드는 `tool_calls`, `tools_used`, 입출력 토큰, duration_ms, SPEC-ID 배지
  - Tool 호출 인자/결과는 접기/펼치기 (agent_events 의 stdout 캡처)
  - 상단에 누적 토큰/예상 비용 (model policy.yaml 기준)
- 하단 액션: **Replay from here** (이후 prompt 만 다른 워크스페이스에서 재실행), **Open failing file** (LSP error 좌표로 Code Viewer 점프), **Revert commits by this run**.

task-metrics.jsonl 이 이미 이 정보를 담고 있으므로 우리는 만들 필요 없이 **파싱하고 시각화** 만 한다. v1 에서 내가 자체 trace 포맷을 만들려 한 것은 삭제.

### 4.8 Kanban Board
- 레인: `Backlog / To-Do / Doing / Review / Done / Blocked`. 커스터마이즈.
- 카드 필드: title, body_md, spec_id, assignee(agent_host), labels, linked files.
- **Doing 자동화**:
  1. git worktree add
  2. Workspace row insert (agent_host = 카드 assignee, 기본 `claude_code`)
  3. PTY spawn → `claude` child
  4. `/moai run SPEC-XXX` 자동 입력 (카드에 spec_id 가 있는 경우)
  5. 3-pane 레이아웃 자동 구성 (Terminal + Code Viewer + Agent Run)
- **Review 자동화**:
  1. `git diff main..HEAD` → Markdown surface
  2. `.moai/scripts/check.sh` 또는 `/moai review` 실행
  3. TRUST 5 점수 + LSP gate 결과를 카드에 배지로
- **Done 자동화**: `gh pr create` 옵션 + worktree archive.
- **Backlog 생성**: `/moai plan` 결과를 훅으로 받아 자동으로 카드 생성 (spec_id 가 채워진 상태로).
- v1 범위: 로컬 SQLite. v2: Notion/Linear 미러.

### 4.9 명령 팔레트 (Cmd+K)
- 모든 IPC 메서드, 파일, SPEC, 카드, 심볼 (Tree-sitter) 퍼지 검색.
- **moai-adk 섹션** 1급: 14개 슬래시 커맨드 + 카드 컨텍스트 액션.
- "Run /moai coverage on focused workspace", "Open SPEC-AUTH-001", "Spawn codex workspace" 같은 동사형 항목.

---

## 5. 기술 스택 (확정)

### 5.1 Shell
- **SwiftUI + AppKit** (macOS 14+)
- **Ghostty libghostty.xcframework** (터미널 렌더/PTY)
- **Tree-sitter + tree-sitter-highlight** (Code Viewer 파싱)
- **Bonsplit-like splitter** (자체 또는 cmux fork)
- **Sparkle** (자동 업데이트)
- (옵션) Sentry-Cocoa, PostHog

### 5.2 Core (`moai-core`, Rust)
- `tokio` (런타임)
- `portable-pty` (PTY)
- `notify` (FSEvents)
- `git2` (libgit2)
- `rusqlite` + `r2d2`
- `serde` + `serde_json`
- `jsonrpsee` (IPC)
- `tracing` + `tracing-subscriber`
- `comrak` (cmark-gfm)
- `tree-sitter` + language grammars
- `ast-grep-core` (선택적)
- `tower-lsp` 클라이언트 모드 (LSP pool)
- `hyper` (hook http sink)
- `swift-bridge` (Swift↔Rust FFI)
- `clap` (sidecar CLI 모드)

### 5.3 번들
- Xcode project 1개
- `cargo xcframework` 로 Rust 코어 → universal .xcframework (arm64 + x86_64)
- GitHub Actions: macOS 14 arm64 러너, `matrix: [debug, release]`
- Release: create-dmg + notarytool + Sparkle appcast + Nightly 채널 (cmux 패턴)

### 5.4 디렉토리 (단일 레포)

```
moai-terminal/
├── apps/macos/
│   ├── moaiTerminal.xcodeproj
│   ├── Sources/
│   │   ├── App/               # @main, AppDelegate
│   │   ├── Shell/             # Sidebar, Tabs, Splits, CommandBar
│   │   ├── Surfaces/
│   │   │   ├── Terminal/      # Ghostty attach view
│   │   │   ├── CodeViewer/    # NSTextView + TS highlight + @MX gutter  ★ v2
│   │   │   ├── Markdown/
│   │   │   ├── Image/
│   │   │   ├── Browser/       # WKWebView
│   │   │   ├── FileTree/
│   │   │   ├── AgentRun/
│   │   │   └── Kanban/
│   │   ├── Bridge/            # swift-bridge 생성물 + 래퍼
│   │   └── Theme/
│   └── Resources/
├── core/                      # Rust workspace
│   ├── Cargo.toml
│   └── crates/
│       ├── moai-core/
│       ├── moai-session/
│       ├── moai-pty/
│       ├── moai-ipc/
│       ├── moai-store/        # SQLite 마이그레이션
│       ├── moai-git/
│       ├── moai-fs/
│       ├── moai-lsp/          # LSP 풀
│       ├── moai-codeview/     # tree-sitter 래퍼
│       ├── moai-kanban/
│       ├── moai-events/
│       ├── moai-adk-bridge/   # hook http sink, jsonl tail, slash exec, config parser  ★ v2
│       └── moai-cli/          # `moait` CLI (UDS 클라이언트)
├── vendor/
│   ├── ghostty/               # submodule
│   └── tree-sitter-grammars/  # submodule 모음
├── skills/                    # Claude Code skills
│   ├── moai-terminal-control/
│   ├── moai-terminal-kanban/
│   └── moai-terminal-browser/
├── scripts/
│   ├── reload.sh              # cmux 패턴 차용
│   ├── build-xcframework.sh
│   └── install-hooks.sh       # 프로젝트의 .moai/hooks.yaml 에 http sink 삽입
├── tests/
│   ├── ipc/                   # pytest JSON-RPC 회귀
│   ├── ui/                    # XCUITest
│   └── stress/                # 16-agent 시나리오
├── docs/
└── .github/workflows/{ci,release,nightly}.yml
```

---

## 6. moai-adk 통합 상세

### 6.1 통합 지점 매트릭스

| moai-adk 자산 | moai-terminal 활용 |
|---|---|
| `moai` 바이너리 | PTY 로 spawn, IPC `moai.*` 로 래핑 |
| 14개 슬래시 커맨드 | Cmd+K 1급, 칸반 카드 액션, 사이드바 버튼 |
| 27개 Hook 이벤트 | `.moai/hooks.yaml` 의 http sink 로 수신 → `hook_events` 테이블 |
| `.moai/logs/task-metrics.jsonl` | FSEvents tail → `task_metrics` 테이블 → Agent Run Viewer |
| `.moai/specs/SPEC-*/` | Markdown surface (EARS 모드) + Kanban 카드 자동 동기화 |
| `.moai/config/sections/*.yaml` | Rust 파서 → GUI 편집기 (스키마 검증) |
| `.moai/project/codemaps/*.md` | Markdown surface, 프로젝트 탭의 "아키텍처" 섹션 |
| `@MX:*` 주석 | `/moai mx --dry --json` → `mx_tags` → Code Viewer 거터 |
| TRUST 5 / LSP gate | quality.yaml 임계값 오버레이, Review 레인 자동 검증 |
| Agent Teams worktree 격리 | Workspace = git worktree 모델로 1:1 매핑 |
| 26개 전문 에이전트 | 건드리지 않음. 그대로 Claude Code 가 호출 |
| `/agency` 워크플로우 | 별도 "Agency" 프로젝트 모드 — Browser + Code Viewer + Markdown 3-pane 프리셋 |
| CG 모드 | workspace agent_host=`tmux_cg`, tmux -CC control mode 로 UI 동기화 |

### 6.2 훅 http sink 프로토콜

```
POST /hooks
X-MoAI-Terminal-Token: <scoped token>
Content-Type: application/json

{
  "event": "PostToolUse",
  "session_id": "sess-abc",
  "task_id": "task-xyz",
  "workspace_hint": "/path/to/worktree",
  "payload": { /* moai-adk hook payload 원형 */ }
}
```

Rust 측 `hyper` 서버가 127.0.0.1 바인드, token 검증, `hook_events` 에 insert, EventBus broadcast.

### 6.3 사용자 여정 — "이슈 → PR" E2E

```
1) 사용자가 GitHub 이슈 #142 를 Cmd+K 에서 "create spec from issue" 로 소환
2) moai-terminal 이 `gh issue view 142 --json` 결과를 프롬프트 템플릿에 주입 →
   포커스된 워크스페이스에 `/moai plan "<이슈 요약>"` 자동 입력
3) moai-adk 가 manager-spec 에이전트로 EARS SPEC 생성 → `.moai/specs/SPEC-AUTH-012/`
4) moai-terminal FS watcher 가 새 SPEC 디렉토리 감지 → 자동으로 칸반 Backlog 에 카드 생성 (spec_id=SPEC-AUTH-012)
5) 사용자가 카드를 Doing 으로 드래그
   → git worktree add, 새 Workspace, Claude Code PTY spawn, `/moai run SPEC-AUTH-012` 주입
   → 3-pane (Terminal | Code Viewer | Agent Run) 자동 세팅
6) 에이전트가 RED → GREEN → REFACTOR 를 진행. 각 PostToolUse hook 이 Agent Run Viewer 에 실시간 업데이트
7) TRUST 5 LSP 게이트 통과 + coverage ≥ 85% → Ralph Engine 0 에러 확정
   → 카드 자동으로 Review 레인 이동, `/moai review` 자동 실행, 결과를 카드 description 에 append
8) 사용자가 Code Viewer 에서 diff 리뷰 + @MX:ANCHOR 가 제대로 붙었는지 확인
9) 카드를 Done 으로 드래그 → `/moai sync SPEC-AUTH-012` → `gh pr create` → worktree archive
```

이 전체가 **창을 바꾸지 않고** 한 screen 안에서 일어나야 한다는 게 moai-terminal 의 핵심 가치.

### 6.4 Codex 병치 시나리오

Codex 는 moai-adk 풀파이프를 못 쓰기 때문에 moai-terminal 은 다음과 같이 지원한다.

- Workspace 생성 시 `agent_host=codex` 선택 가능.
- 해당 워크스페이스는 자동으로 `codex` CLI PTY + Code Viewer + FileTree 프리셋.
- 사이드바에 "moai-adk Limited" 배지 + 안내 툴팁: "Codex 세션에서는 Hook/Agent Teams 가 비활성입니다. SPEC 문서 읽기와 TRUST 수동 검증만 가능합니다."
- 단, `.moai/specs/` 마크다운과 `/moai review`, `/moai coverage` 같은 **읽기성/검증성** 커맨드는 별도 일회성 프로세스로 실행해 Code Viewer / Markdown surface 에 결과 표시는 가능.
- 한 프로젝트 안에서 Claude Code 워크스페이스와 Codex 워크스페이스가 **동시 공존** 하고, 서로 다른 worktree 에서 서로 다른 SPEC 을 진행할 수 있다.

---

## 7. 성능·안정성·보안

### 7.1 성능
- 숨겨진 Ghostty surface 는 `isHiddenOrHasHiddenAncestor` 체크로 렌더 폐기.
- 출력 fan-out: PTY → Rust reader → ring buffer (4MB/ws) + SQLite batch insert (100 rows 또는 100ms).
- SQLite: WAL, synchronous=NORMAL, 5분 또는 4MB 커밋 시 truncate checkpoint.
- 명령 팔레트: FTS5 인덱스, 100k 항목 < 50ms.
- task-metrics tail: inotify/FSEvents 200ms debounce, 1000 row/s 처리.
- Code Viewer: tree-sitter 증분 파싱, 1MB 파일 < 100ms 초기 파싱.
- LSP 풀: 프로젝트당 언어별 1 인스턴스, idle 5분 후 shutdown.

### 7.2 안정성 (OTP supervision tree)

```
RootSupervisor
 ├── IpcServer
 ├── HookHttpSink            ★ v2 신규
 ├── Store(SQLite)
 ├── EventBus
 ├── LspPool                 ★ v2 신규 (Code Viewer 지원)
 ├── ProjectSupervisor (1:N)
 │    └── WorkspaceSupervisor (1:N)
 │         ├── PtyTask           (자식: claude/codex/zsh/tmux)
 │         ├── MetricsTailTask   (task-metrics.jsonl)
 │         ├── FileWatcherTask
 │         └── GitWatcherTask
 └── KanbanService
```

- one_for_one: 자식 panic 시 부모만 재시작.
- Crash dump: `~/.moai-terminal/crash/` + Sentry 옵트인.
- Atomic config: `tmpfile+rename` 또는 SQLite tx.
- 재시작 복구: `workspaces.status='running'` 행에 대해 PTY 재attach 시도, 실패 시 `crashed`.

### 7.3 보안
- 소켓 0600, `~/.moai-terminal/` 0700.
- IPC 메서드 화이트리스트 + scoped token.
- 위험 메서드 (`moai.run`, `agent.send_input`, `surface.create(terminal)`, `git.*` write) 는 focus-intent 필요.
- Hook http sink: 127.0.0.1 바인드 + `X-MoAI-Terminal-Token` 검증.
- WebView: `nonPersistentDataStore` 기본, 사이트 격리, mixed content 차단.
- 자동 업데이트: EdDSA 서명 + https-only Sparkle appcast.
- `.moai/config/sections/security.yaml` 의 `forbidden_keywords` 를 IPC 인자에도 적용 (token/secret 유출 방지).

---

## 8. 테스트 전략

| 레벨 | 도구 | 대상 |
|---|---|---|
| Unit (Rust) | `cargo test` | 도메인 로직, 마이그레이션, hook 파서, metrics jsonl 파서 |
| Integration | Python pytest (cmux 패턴) | UDS JSON-RPC 회귀 + hook http sink |
| UI snapshot | XCUITest + swift-snapshot-testing | Sidebar, Code Viewer 거터, Kanban, Agent Run Viewer |
| Stress | 자체 harness | 16 workspace × 30분, claude 모킹 PTY child, task-metrics flood |
| E2E | Robot + AppleScript | "이슈 → plan → run → sync → PR" 전체 플로우 |
| moai-adk 호환성 | `moai --version` matrix | v2.7.x 지원 선언, nightly CI 에서 최신 태그 검증 |

---

## 9. 마일스톤

| 단계 | 기간 | 산출물 |
|---|---|---|
| **M0 Spike** | 2주 | Swift 셸 + 빈 Rust xcframework, Ghostty 단일 surface, swift-bridge hello, moai 바이너리 spawn 확인 |
| **M1 Core 세션** | 4주 | Workspace/Pane/Surface 모델, PTY pool, SQLite, IPC v1, Sidebar, Terminal |
| **M2 Viewers 1차** | 3주 | FileTree, Markdown, Image, Browser |
| **M3 Code Viewer** ★ | 4주 | Tree-sitter, LSP pool, @MX 거터, tri-pane diff, time-travel |
| **M4 moai-adk Bridge** | 4주 | Config parser, hook http sink, jsonl tail, Agent Run Viewer, 14 슬래시 래퍼 |
| **M5 Kanban + Worktree 자동화** | 3주 | Kanban 보드, Doing/Review/Done 자동화, EARS 마크다운 모드 |
| **M6 안정화/배포** | 3주 | Sparkle, 서명/공증, Nightly, 16-agent 스트레스 통과, DMG |
| **M7 (옵션)** | 4주 | SSH 원격(cmux daemon/remote 포팅), Notion/Linear 미러, `/agency` 프리셋 최적화 |

총 23~27주. 2~3인 팀 시 5~6개월.

---

## 10. cmux 대비 / moai-adk 대비 포지셔닝

| 축 | cmux | moai-adk | moai-terminal |
|---|---|---|---|
| 레이어 | 터미널 셸 | 개발 방법론 엔진 (Go CLI) | **두 층을 얹은 IDE-쉘** |
| 범위 | 멀티 PTY + 브라우저 | SPEC/TRUST/TAG/Hook/Agent Teams | 둘을 한 macOS 앱으로 |
| 대상 IDE | (자체) | Claude Code 전용 | **Claude Code 1급 + Codex 병치** |
| 코드 뷰 | 없음 | 없음 | **Code Viewer 1급 (@MX 거터)** |
| 칸반 | 없음 | 없음 | **SPEC ↔ Worktree ↔ 자동화 1급** |
| 트레이스 | 알림 링 | task-metrics.jsonl | **jsonl 을 소비한 Agent Run Viewer** |
| 저장 | JSON | YAML + jsonl | **SQLite WAL 미러 + 풀텍스트 검색** |
| 격리 | shell cwd | Agent Teams worktree | **worktree = workspace 일치** |
| 권한 | focus-intent | Hook PreToolUse | **focus-intent + scoped token 2중** |
| OS | macOS | macOS/Linux/Windows | **macOS (M0~M7)**, 코어는 cross-platform 가능 구조 |

---

## 11. 열린 결정 사항

1. **Ghostty 라이선스/빌드 수용** — zig build + xcframework 번들 운영 부담.
2. **Bonsplit** — fork vs 자체 구현 (라이선스 확인 필요).
3. **moai-adk 의 `hooks.yaml` 자동 수정 동의** — 사용자 프로젝트 파일을 우리가 편집하는 것에 대한 동의 플로우.
4. **moai-terminal 과 moai-adk 의 조직적 관계** — moai-adk 의 공식 GUI 로 포지셔닝할지, 독립 제품으로 갈지.
5. **원격(SSH) 우선순위** — M7 에 두는 게 맞는지 (cmux 수준의 원격 투명성은 만만치 않음).
6. **Windows/Linux 로드맵** — Rust 코어를 분리했으므로 Tauri 셸을 추가해 3-OS 지원할지.
7. **라이선스** — OSS(Apache-2.0?) vs freemium. cmux 는 상업 배포, moai-adk 는 MIT(확인 필요).
8. **테스트 더블** — moai-adk CI 에 moai-terminal IPC 테스트 훅을 넣을 수 있는지.

---

## 12. 다음 액션

1. 11장의 8개 결정에 답변 주시면, M0 상세 작업 분해 + Xcode/Cargo 스캐폴딩까지 만들겠습니다.
2. Ghostty `zig build -Demit-xcframework=true` 검증을 M0 의 첫 관문으로 권장합니다.
3. 실제 moai-adk 설치된 샘플 프로젝트 1개를 주시면, hook http sink → `hook_events` → Agent Run Viewer 까지의 최소 스파이크를 가장 먼저 증명하겠습니다. 이 경로가 뚫리면 나머지는 구현 노동만 남습니다.
