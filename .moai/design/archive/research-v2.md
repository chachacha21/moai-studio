# Agentic Coding IDE UI/UX Research Synthesis (2026-04)

본 문서는 MoAI Studio 재설계를 위한 2026년 기준 agentic coding IDE 베스트 프랙티스 조사 결과를 종합한다.

## 1. 조사 대상

| 카테고리 | 제품 | 핵심 참조 |
|----------|------|-----------|
| IDE | Cursor 3 / Composer 2 | Agent-centric layout, Mission Control, Visual Editor |
| IDE | Windsurf (Cognition 인수) | Cascade, Flow, 1.5B 모델 |
| IDE | Zed | Agent Panel, ACP (Agent Client Protocol) |
| CLI | Claude Code 2026 | Skills, Hooks, MCP, 플러그인, /loop |
| Launcher | Raycast | ⌘-Space 대체, nested palette |
| Design | Linear / Arc / Notion | ⌘K modal, fuzzy search |
| Observability | Braintrust / LangSmith / Sentry | span tree, OpenTelemetry gen_ai |

---

## 2. 핵심 UI 패턴 (2026)

### P-1. 에이전트가 1급 객체 (First-class Agent)
- Cursor 3: 사이드바에서 Agent 를 "작업 단위" 로 취급 (conversation 이 아닌 run 단위 관리)
- Mission Control 그리드: macOS Exposé 스타일로 parallel 에이전트 실행 현황 일람
- 각 에이전트는 isolated worktree + 독립 plan + 고유 diff
- **MoAI 시사점**: MoAI 의 SPEC 단위 worktree + manager/expert agents 구조와 동형 → Mission Control 추가 필요

### P-2. Plan-Then-Execute + Rules/Hooks 스캐폴딩
- Cursor: Plan Mode → Rules → Commands → Hooks → Agent 순차 파이프라인
- Claude Code: `/plan`, `.claude/rules/`, `.claude/skills/`, `.claude/hooks/` 계층
- **예측 가능성 > 속도**: 유저가 Plan 에 개입할 수 있어야 신뢰
- **MoAI 시사점**: 이미 `/moai plan → run → sync` 파이프라인과 `.claude/rules/` 계층 존재. UI 에서 Plan 단계 가시화 필요

### P-3. 멀티파일 원자 diff + hunk-level 리뷰
- Cursor Composer: 한 지시 → 여러 파일 동시 편집 → 미리보기 diff → hunk 단위 accept/reject/edit
- 계층: **Plan → Diff Preview → Hunk Accept → Inline Edit → Stage/Commit/PR**
- Undo/Redo 로 전체 멀티파일 변경 한 번에 되돌리기
- **MoAI 시사점**: Frame 04 (Code Viewer Deep Dive tri-pane) 에 hunk-level accept/reject 추가

### P-4. Split-brain 계획 엔진
- Windsurf Cascade: 백그라운드 planning 에이전트 + 전경 execution 모델
- 긴 작업 중 plan 은 지속 갱신, execution 은 단기 액션에 집중
- 대화 중간에 Todo 리스트 자동 편집
- **MoAI 시사점**: `/moai run` 의 Agent Teams + manager-strategy 와 동형. UI 에서 Plan vs Execute 분리 표시

### P-5. 컨텍스트 파이프라인 + Flow Awareness
Windsurf 모델:
```
Load Rules (.windsurfrules 전역 + 프로젝트)
→ Load Memories
→ Read Open Files (활성 파일 가중치 ↑)
→ Codebase Retrieval
→ Read Recent Actions (에디터 활동 추적)
→ Assemble Prompt
```
- 유저가 수동으로 설명 안 해도 자동으로 최신 컨텍스트 주입
- **MoAI 시사점**: MoAI 는 CLAUDE.md + .moai/project/ + auto-memory 로 Rules/Memory 커버. Flow Awareness (편집 활동 추적) 는 미구현 → 추가 기회

### P-6. @/# 멘션 기반 컨텍스트 첨부 (VS Code 모델)
- `@file`, `@folder`, `@symbol` - 파일/폴더/심볼 첨부
- `#tool` - 도구 명시 호출
- 이미지 drag-drop 으로 비전 첨부
- URL 첨부 → 웹페이지 내용 가져오기
- **MoAI 시사점**: Command Palette 에 @/# 멘션 picker 확장 필요

### P-7. 메시지 큐잉 (Windsurf/Zed 공통)
- 에이전트 실행 중에도 사용자가 다음 프롬프트 타이핑 가능
- Enter → 큐에 추가, 턴 경계에서 실행
- 이중 Enter → 즉시 실행
- **MoAI 시사점**: 에이전트 실행 중 UI 블로킹 방지 (Command Palette 상시 활성)

### P-8. Follow the Agent (Zed)
- 크로스헤어 아이콘 토글 → 에이전트가 열거나 편집하는 파일로 자동 이동
- Cmd 누르면서 전송 → 자동 follow 모드
- **MoAI 시사점**: Agent Run Viewer + Code Viewer 연동 가능 (pane 자동 전환)

### P-9. Thread 모드 구분 (Zed)
- **Agent Thread**: 풀 agentic (도구 호출, 파일 편집, 명령 실행)
- **Text Thread**: 순수 대화 (도구 없음, 큰 폰트, 읽기 편함)
- Thread 제목 자동 생성 + 사용자 편집 가능
- **MoAI 시사점**: 지금은 터미널 서라지만 Agent Thread 뷰 신규 추가 가치 큼

### P-10. Span Tree 기반 관측 (Braintrust/LangSmith/Sentry)
- 에이전트 실행을 계층적 span tree 로 시각화
- 각 span: duration, LLM duration, TTFT, tokens (input/output/cached/reasoning), cost, tool 호출, 에러
- SSE 스트리밍: `agent.start`, `text.delta`, `tool.call.start`, `tool.call.result`, `run.finish`
- 3-pane 대시보드: Run list → Span tree → Span details
- OpenTelemetry `gen_ai` 시맨틱 컨벤션이 표준으로 수렴
- **MoAI 시사점**: Frame 05 (Agent Run Viewer) 이미 유사 구조 보유. 스탬/메트릭 세분화 필요

### P-11. 커맨드 팔레트 세부 패턴 (Raycast/Linear)
- **⌘K modal overlay**: 중앙 오버레이 + 백드롭 (업계 표준)
- **Fuzzy + keyword alias**: 각 명령에 alias 등록
- **Favorites + 1-9 hotkey**: 자주 쓰는 명령 우선
- **Shortcut 표시**: 결과 행에 단축키 inline
- **Nested palette**: 항목 선택 → 하위 팔레트 (GitHub 조직/리포 선택 등)
- **검색 데이터**: 명령 뿐 아니라 유저 데이터 (파일, SPEC, 태그) 도 결과
- **MoAI 시사점**: Frame 08 구조 기반 + nested 확장 + @/# 멘션 결합

### P-12. First-Run / Empty State (Notion 모델)
- 빈 화면 금지 → CTA 버튼 (예: "첫 워크스페이스 만들기")
- 샘플 데이터 1-click dismiss + "Sample" 라벨 명시
- "Aha moment" 를 최소 마찰로 도달
- Day 1 목표: 첫 커밋 완료
- 점진적 공개 (100 페이지 docs 대신 단계적 노출)
- **MoAI 시사점**: 현재 구현의 "빈 창" 이 직접 위반 → 긴급 개선 필요
- Frame 11 (Onboarding) 은 이미 존재하나 첫 실행 이후 재사용 안 되는 경로

### P-13. 메모리 레이어 이중 구조
- **단기 (세션 내)**: 컨텍스트 윈도우, 열린 파일, 최근 액션
- **장기 (세션 간)**: Rules 파일 (CLAUDE.md/AGENTS.md) + Memory 서비스 + 세션 스캐닝
- 세션 시작 시 자동 주입 (ContextPool 모델)
- Compaction 이후 재주입: 최근 편집 파일 + 활성 스킬 + pending 작업
- 보안: 주입된 내용은 "데이터" 로 표기 (지시로 해석 금지, 프롬프트 인젝션 방지)
- **MoAI 시사점**: MoAI 이미 CLAUDE.md + auto-memory (`~/.claude/projects/.../memory/`) 보유. UI 에서 Memory Viewer 제공 가치 있음

### P-14. 컨텍스트 버스트 (Context Burst)
- 베이스라인 프롬프트는 작게 유지 (1-2K)
- 결정적 액션 직전에 고밀도 컨텍스트 주입 (실패한 테스트 + 해당 함수 + 호출자 + 스택)
- **MoAI 시사점**: 에이전트 UI 에 "Context Burst" 인디케이터 (이번 턴에 주입된 추가 컨텍스트 표시) 가치

### P-15. macOS 네이티브 관례
- **메뉴 바**: File/Edit/View/Window/Help 외에 앱 전용 메뉴 추가 → 발견성 ↑
- **툴바**: `toolbar(content:)` SwiftUI API 로 빠른 행동
- **단축키 규범**: Cmd+N (New), Cmd+W (Close), Cmd+, (Settings), Cmd+K (Search)
- **MenuBarExtra** (옵션): 상주형 상태 바 아이콘 (에이전트 실행 상태 표시용)
- **MoAI 시사점**: 현재 메뉴 바 공백 → 심각한 네이티브 관례 위반

---

## 3. 반-패턴 (Anti-Patterns) — 피해야 할 것

1. **빈 첫 화면**: CTA 없이 빈 창 → 유저 이탈 (MoAI 현재 상태)
2. **메뉴 바 공백**: macOS 관례 위반 → 단축키 발견 불가
3. **단일 무한 채팅**: Cursor 가 명시적으로 피함. 에이전트를 작업 단위로 분리
4. **100 페이지 docs 덤프**: 점진적 공개 원칙 위반
5. **침묵 실패**: 에러 없이 아무 반응 없음 → 유저 혼란 (현재 Cmd+N 버그)
6. **수동 컨텍스트 재설명**: Flow Awareness 부재 → 반복 피로
7. **Plan 스킵**: 유저가 Plan 단계를 볼 수 없으면 신뢰 ↓

---

## 4. MoAI Studio 특화 시사점

### 강점 (이미 우수한 것)
- SPEC-First DDD 철학 (Cursor/Windsurf 의 "Plan Mode" 와 상위 호환)
- Agent Teams + 역할 프로파일 (manager/expert) → Mission Control 과 친화적
- Worktree isolation (Cursor 의 agent-per-worktree 와 동형)
- Hook 이벤트 27개 (OpenTelemetry gen_ai 초과)
- Rust core + Swift UI 분리 (확장성 기반)

### 차별화 기회
- **SPEC-driven Agent Run**: 일반 agentic IDE 는 자연어 → 코드. MoAI 는 SPEC → TDD → 코드. 이 체계를 UI 로 가시화
- **MX Tag 통합**: @MX:NOTE / @MX:WARN / @MX:ANCHOR / @MX:TODO 를 거터에 표시하면 Cursor Bugbot 과 차별화된 "자가 주석" 시스템
- **TRUST 5 Quality Gate 시각화**: 테스트/가독성/통일성/보안/추적 5축 점수 대시보드 가능
- **Plan → Run → Sync 3단계 리프레시**: 일반 툴은 단일 agent loop. MoAI 는 명확한 단계 → 실패 지점 localization ↑

### 약점 (우선 해결)
1. 빈 첫 화면 (Frame 11 Onboarding 은 존재하지만 접근 경로 불명)
2. 메뉴 바 공백 (Cmd+N 데드 링크 포함)
3. 툴바 공백
4. Project Path 입력을 TextField 로만 받음 (NSOpenPanel 누락)
5. CLI 감지 / 환경 상태 표시 없음 (Frame 11 의 "ENVIRONMENT DETECTED" 카드를 메인으로 끌어오기)

---

## 5. 참고 문헌 (Sources)

### Cursor / Composer
- [Cursor 2.0 and Composer: how a multi-agent rethink surprised AI coding](https://www.cometapi.com/cursor-2-0-what-changed-and-why-it-matters/)
- [Meet the new Cursor](https://cursor.com/blog/cursor-3)
- [Cursor AI Review 2026](https://prismic.io/blog/cursor-ai)

### Claude Code
- [Claude Code overview](https://code.claude.com/docs/en/overview)
- [Claude Code Cheat Sheet 2026](https://www.view-page-source.com/claude-code-cheat-sheet/)
- [awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code)

### Windsurf Cascade
- [Cascade | Windsurf](https://windsurf.com/cascade)
- [Windsurf Cascade docs](https://docs.windsurf.com/windsurf/cascade/cascade)
- [Agentic IDE Comparison: Cursor vs Windsurf vs Antigravity](https://www.codecademy.com/article/agentic-ide-comparison-cursor-vs-windsurf-vs-antigravity)

### Zed
- [Zed Agent Panel](https://zed.dev/docs/ai/agent-panel)
- [Zed Text Threads](https://zed.dev/docs/ai/text-threads)
- [AI Agent System | Zed DeepWiki](https://deepwiki.com/zed-industries/zed/8-ai-agent-system)

### Command Palette
- [Designing a Command Palette | Destiner](https://destiner.io/blog/post/designing-a-command-palette/)
- [Command Palette Interfaces | Philip Davis](https://philipcdavis.com/writing/command-palette-interfaces)
- [Raycast in 2026](https://dev.to/dharanidharan_d_tech/raycast-in-2026-the-mac-launcher-that-replaced-4-apps-in-my-dev-workflow-3pka)

### Agent Run Viewer / Observability
- [AI observability tools 2026 | Braintrust](https://www.braintrust.dev/articles/best-ai-observability-tools-2026)
- [AI Agent Observability Guide | Atlan](https://atlan.com/know/ai-agent-observability/)
- [AI Agent Observability | Sentry](https://blog.sentry.io/ai-agent-observability-developers-guide-to-agent-monitoring/)

### Onboarding / Empty State
- [Developer Onboarding Guide 2026 | River](https://rivereditor.com/blogs/write-developer-onboarding-guide-30-days)
- [User Onboarding Best Practices 2026 | Formbricks](https://formbricks.com/blog/user-onboarding-best-practices)

### Context / Memory
- [Why Every AI Coding Assistant Needs a Memory Layer](https://towardsdatascience.com/why-every-ai-coding-assistant-needs-a-memory-layer/)
- [Manage context for AI | VS Code](https://code.visualstudio.com/docs/copilot/chat/copilot-chat-context)
- [ContextPool](https://www.producthunt.com/products/contextpool)

### macOS Native
- [Building and customizing the menu bar with SwiftUI](https://developer.apple.com/documentation/SwiftUI/Building-and-customizing-the-menu-bar-with-SwiftUI)
- [SwiftUI Toolbars](https://developer.apple.com/documentation/swiftui/toolbars)
- [macOS Menu Bar App with SwiftUI 2026](https://www.hendoi.in/blog/macos-menu-bar-utility-app-swiftui-startups-2026)

---

버전: 1.0.0 · 2026-04-17
