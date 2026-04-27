# MoAI Studio 로컬 운영 지침

본 문서는 `moai-studio` 레포의 **프로젝트 로컬 운영 규칙**을 정의한다. `CLAUDE.md` (프레임워크 공통) 를 보완하며, **GitHub Flow** 전략을 명시한다.

> **2026-04-27 변경**: Enhanced GitHub Flow에서 GitHub Flow로 전환. develop 브랜치 폐지. feature 브랜치에서 직접 main으로 머지.

Scope: 본 레포 (`github.com/modu-ai/moai-studio`) 한정. MoAI-ADK 프레임워크 저장소 (`moai-adk-go`) 에는 적용되지 않는다.

---

## 1. Branch Model (GitHub Flow)

단순하고 직관적인 GitHub Flow를 사용한다.

```
main                          (stable, v0.1.0+ tagged releases 만 포함)
 ├── feature/SPEC-XXX-short-desc
 ├── feature/SPEC-V3-003-ms2  (예시)
 └── feat/v3-scaffold         (legacy — 점진 이관)
hotfix/v{x.y.z+1}-{slug}      (main 에서 분기, production 긴급 수정)
```

### 1.1 각 브랜치 수명/역할

| 브랜치 | 수명 | 분기 원 | 머지 대상 | 상태 |
|--------|------|---------|-----------|------|
| `main` | 영구 | — | — | 정식 릴리스만 포함. tag 부착. |
| `feature/SPEC-XXX-...` | 임시 | `main` | `main` | SPEC 단위 1개. 머지 후 삭제. |
| `hotfix/v{x.y.z+1}-{slug}` | 임시 | `main` | `main` | production 긴급. |

### 1.2 Hotfix 브랜치 명명 규칙 [HARD]

- 포맷: `hotfix/v{major}.{minor}.{patch+1}-{short-slug-kebab-case}`
- 예시: `hotfix/v0.1.1-pane-focus-crash`, `hotfix/v0.1.2-pty-fd-leak`
- `{short-slug-kebab-case}` 은 2~5 단어, 영문 소문자, 하이픈 구분.
- 머지 시 `main` 에만 반영.

### 1.3 Feature 브랜치 명명 규칙

- 포맷: `feature/SPEC-{area}-{nnn}-{short-slug}` 또는 SPEC 연계 없을 시 `feature/{short-slug}`
- 예시: `feature/SPEC-V3-003-ms2-tabcontainer`, `feature/doc-brand-refresh`
- 기존 `feat/v3-scaffold` 같은 legacy 이름은 유지 허용 (점진 이관).

---

## 2. Branch Protection Rules [HARD — GitHub Settings]

`modu-ai/moai-studio` 저장소의 branch protection rules. **2026-04-26 활성 완료** (gh api 로 적용, settings 동기화).

> **2026-04-27 변경**: develop 브랜치 폐지로 develop 관련 protection 규칙 삭제.

### 2.1 `main` 브랜치 (활성, 2026-04-26)

- [x] Require a pull request before merging
  - Required approvals: **1**
  - Dismiss stale approvals when new commits are pushed: **on**
- [x] Require status checks to pass — strict (브랜치 up-to-date 강제)
  - Required contexts (7): `fmt (macOS)`, `fmt (Linux)`, `clippy (macOS)`, `clippy (Linux)`, `test (macOS)`, `bench-smoke (macOS)`, `bench-smoke (Linux)`
  - **Excluded** (별개 이슈, 추후 SPEC 으로 fix 후 추가): `test (Linux)` (active_branch_returns_none_when_no_git), `tmux-test (macOS)` (file watcher flaky), `tmux-test (Linux)` (느린 cache 빌드)
- [x] Allow force pushes: **off**
- [x] Allow deletions: **off**
- [ ] Include administrators: **off** (긴급 hotfix 우회 허용 — v0.1.0 release 후 on 재검토)

### 2.2 Auto-merge 운영

`modu-ai/moai-studio` 의 repo 설정 (2026-04-26 활성):

- `allow_auto_merge: true` — PR 에서 auto-merge 토글 가능
- `delete_branch_on_merge: true` — 머지 후 feature 브랜치 자동 삭제
- `allow_squash_merge: true` (feature → main)
- `allow_merge_commit: true` (hotfix → main)
- `allow_rebase_merge: false` (사용 안 함)

**Auto-merge 사용 패턴:**

```bash
# PR 생성 직후 auto-merge 활성 (squash)
gh pr create --base main --title "..." --body "..."
gh pr merge --auto --squash <PR#>

# hotfix → main 은 merge commit
gh pr merge --auto --merge <PR#>
```

Auto-merge 동작:
- 모든 required status check (§2.1 의 7 contexts) PASS 시 자동 머지
- required approvals 미충족 시 대기 (main 의 경우)
- conflict 발생 시 auto-merge 자동 해제 — 수동 rebase/resolve 후 재활성

설정 완료:
- [x] main: 활성 2026-04-26
- [x] auto-merge / delete-branch-on-merge: 활성 2026-04-26

---

## 3. Label 체계 — 3축 분류

Issue 및 PR 은 다음 3개 축에서 **각 1개 이상** 라벨을 가진다. 관리 파일: `.github/labels.yml`.

### 3.1 Type (무엇인가) — prefix `type/`

| Label | 색상 | 설명 |
|-------|------|------|
| `type/feature` | 0e8a16 | 새 기능 추가 |
| `type/bug` | d73a4a | 결함 수정 |
| `type/refactor` | c5def5 | 동작 유지 리팩토링 |
| `type/docs` | 0075ca | 문서 변경 |
| `type/test` | fbca04 | 테스트 코드 |
| `type/chore` | eeeeee | 빌드/설정/보조 스크립트 |
| `type/security` | b60205 | 보안 관련 수정 |
| `type/perf` | ffa500 | 성능 개선 |

### 3.2 Priority (얼마나 급한가) — prefix `priority/`

| Label | 색상 | SLA 가이드 |
|-------|------|----------|
| `priority/p0-critical` | b60205 | production 차단, 즉시 대응 |
| `priority/p1-high` | d93f0b | 현재 sprint 내 처리 |
| `priority/p2-medium` | fbca04 | 다음 sprint 후보 |
| `priority/p3-low` | 0e8a16 | backlog |

### 3.3 Area (어느 영역인가) — prefix `area/`

| Label | 색상 | 대응 파일/디렉터리 |
|-------|------|-------------------|
| `area/panes` | 1d76db | `crates/moai-studio-ui/src/panes/**` |
| `area/tabs` | 1d76db | `crates/moai-studio-ui/src/tabs/**` |
| `area/terminal` | 1d76db | `crates/moai-studio-terminal/**` |
| `area/ui-shell` | 1d76db | `crates/moai-studio-ui/src/lib.rs` + `crates/moai-studio-ui/src/terminal/**` |
| `area/workspace` | 1d76db | `crates/moai-studio-workspace/**` |
| `area/persistence` | 1d76db | `crates/moai-studio-workspace/src/persistence.rs` + 관련 |
| `area/ci` | 1d76db | `.github/**` |
| `area/deps` | 1d76db | `Cargo.toml`, `Cargo.lock`, `rust-toolchain` |
| `area/docs` | 1d76db | `.moai/specs/**`, `docs/**`, `README*` |
| `area/spec` | 1d76db | `.moai/specs/**` spec authoring only |
| `area/infra` | 1d76db | `scripts/**`, toolchain, release 인프라 |

### 3.4 Release Drafter 보조 라벨 (자동 drafter 동작)

| Label | 역할 |
|-------|------|
| `release/major` | MAJOR version bump (X.y.z) |
| `release/minor` | MINOR version bump (x.Y.z) |
| `release/patch` | PATCH version bump (x.y.Z) |
| `skip-changelog` | CHANGELOG 에서 제외 (chore, internal refactor) |

---

## 4. Merge Strategy [HARD]

머지 방식은 **대상 브랜치 기준** 으로 결정한다. GitHub `Settings → General → Pull Requests` 에서 세 방식 모두 활성화해 두고, **운영 규칙으로만** 제한한다.

| 소스 | 대상 | 머지 방식 | 비고 |
|------|------|----------|------|
| `feature/*` | `main` | **Squash merge** | PR 제목 = squash commit subject. SPEC 단위 1개 커밋으로 축약. Scope 명시. |
| `hotfix/*` | `main` | **Merge commit (--no-ff)** + tag | `merge(hotfix): v{x.y.z}` + tag |

### 4.1 Squash Commit 메시지 규칙 (feature → develop)

Conventional Commits 포맷:
```
<type>(<scope>): <subject> [AC-... 또는 SPEC 참조]

<body — 왜 바꿨는가 + 주요 decision>

🗿 MoAI <email@mo.ai.kr>
```

예시:
```
feat(panes): T8 TabContainer 자료구조 + last_focused_pane 복원 (AC-P-8/10/11)
```

### 4.2 Merge Commit 메시지 규칙 (release / hotfix)

```
merge(<release|hotfix>): <source> → <target> [v{x.y.z}]

<summary of contained PRs/commits>

🗿 MoAI <email@mo.ai.kr>
```

---

## 5. Release Drafter — CHANGELOG 자동화

`.github/release-drafter.yml` + `.github/workflows/release-drafter.yml` 로 PR 라벨 기반 CHANGELOG 초안을 자동 생성한다.

### 5.1 동작 원리

1. `main` 에 PR 머지 시 Release Drafter 가 실행
2. PR 라벨 (`type/*`, `release/*`) 을 읽어 카테고리 분류
3. `v{next}` draft release 에 항목 누적
4. 릴리스 담당자가 tag 생성 후 draft 를 publish

### 5.2 카테고리 매핑 (release-drafter.yml)

| PR Label | Drafter Category |
|----------|------------------|
| `type/feature` | `## Added` |
| `type/bug` | `## Fixed` |
| `type/security` | `## Security` |
| `type/perf` | `## Performance` |
| `type/refactor` | `## Refactored` |
| `type/docs` | `## Documentation` |
| `type/chore` + `type/test` | (stealth, `skip-changelog` 없는 한 `## Internal`) |
| `skip-changelog` | 제외 |

### 5.3 Version Bump

- `release/major` → X.y.z
- `release/minor` → x.Y.z
- `release/patch` → x.y.Z (default)

---

## 6. 일상 워크플로 체크리스트

### 6.1 Feature 작업 착수 (SPEC-XXX 구현)

1. `git checkout main && git pull`
2. `git checkout -b feature/SPEC-XXX-short-slug`
3. `/moai run SPEC-XXX` 로 TDD 사이클 진행 (현행)
4. 로컬 커밋 누적 (auto_commit=true per `.moai/config/sections/git-strategy.yaml`)
5. 구현 완료 시: `git push origin feature/SPEC-XXX-short-slug`
6. GitHub UI 에서 PR 생성 → base: `main`
7. PR 에 **type/ + priority/ + area/** 3축 라벨 부착
8. CI GREEN + 1 review → **Squash merge**
9. 머지 후 feature 브랜치 삭제 (GitHub 자동 삭제 활성화 권장)

### 6.2 Hotfix (production 긴급)

1. `git checkout main && git pull`
2. `git checkout -b hotfix/v0.1.1-{slug}`
3. 최소 수정 + reproduction test 추가 (per CLAUDE.md §7 Rule 4)
4. Push + PR `hotfix/*` → `main` (**Merge commit** + tag)
5. `hotfix/*` 브랜치 삭제

---

## 7. MoAI 에이전트 운영 조정

### 7.1 `manager-git` subagent 위임 시 주입 컨텍스트

- Default target branch: `main`
- Feature 브랜치 생성 시 prefix: `feature/` (단, `SPEC-XXX` 연계 시 `feature/SPEC-XXX-slug`)
- `git-strategy.manual` 유지: auto_commit=true, auto_push=false, auto_pr=false
- Push 는 사용자 명시적 지시 시에만

### 7.2 `/moai sync` Subcommand 개정 힌트

Phase 3 PR 생성 시:
- base branch: `main`
- PR 라벨 추천 (type/area 필수, priority 선택)
- Auto-merge 활성: PR 생성 직후 `gh pr merge --auto --squash <PR#>` 호출 권장 (§2.2)
- Required status checks (§2.1 의 7 contexts) PASS 시 자동 머지
- conflict 또는 별개 이슈 fail 시 auto-merge 자동 해제 — 수동 개입

---

## 9. Code Comments Policy [HARD]

### 9.1 영어 주석 강제

[HARD] 본 레포의 모든 코드 주석은 **영어** 로 작성한다. `.moai/config/sections/language.yaml` 의 `code_comments: en` 설정과 일치.

적용 범위:
- inline comment (`//`, `#`, `--`, `;` 등 모든 언어 prefix)
- docstring / doc-comment (`///`, `/** */`, `//!`, `"""..."""` 등)
- module / file 헤더 주석 (`//!` Rust file-level, package docstring Python 등)
- @MX 태그 description 및 @MX:REASON sub-line
- 테스트 함수 주석 및 assert message (가능한 한 영어)

### 9.2 적용 시점 및 점진 전환

- **2026-04-26 이후 작성·수정되는 모든 코드** 는 즉시 영어 주석 적용 (HARD)
- 기존 한국어 주석 코드: 정책 활성 이후 해당 파일을 touch 할 때 그 시점에 영어로 전환 (점진 마이그레이션)
- 일괄 변환은 별도 SPEC (`SPEC-V3-COMMENTS-MIGRATION` 후보) 으로 분리 — 본 정책은 신규 코드와 touch-on-modify 만 강제

### 9.3 한국어 유지 영역 (제외)

다음은 영어 정책에서 제외된다 (별도 language.yaml 설정 따라감):
- SPEC 문서 (`.moai/specs/**/*.md`) — `documentation: ko`
- README / CHANGELOG / 사용자 가이드 — `documentation: ko`
- git commit message subject 및 body — `git_commit_messages: ko`
- 사용자 응답 (orchestrator → user) — `conversation_language: ko`
- error_messages 의 사용자 메시지 — `error_messages: en` (이미 영어, 별도)

### 9.4 Skill / Agent / Rule 정의

`.claude/skills/**`, `.claude/agents/**`, `.claude/rules/**` 의 instruction document 는 영어로 작성 (CLAUDE.md / coding-standards.md 의 Language Policy 따라감). 본 §9 와 동일한 영어 강제.

### 9.5 Agent 위임 시 명시

코드를 작성하는 subagent (`manager-tdd`, `manager-ddd`, `expert-backend`, `expert-frontend` 등) 의 위임 프롬프트에 다음 라인을 권장 포함:

> All code comments and docstrings MUST be in English. Variable / function / type / module names are also in English (already enforced by Rust convention). Korean is reserved for SPEC documents (`.moai/specs/`), git commit messages, README / docs, and user-facing orchestrator responses only.

### 9.6 위반 처리

- 코드 리뷰 시 한국어 주석 발견 → 영어로 수정 요청 (블로킹 사유)
- 자동 검증 도구 부재: `cargo clippy` / `eslint` 등 표준 lint 는 주석 언어 검사 없음 → 수동 리뷰
- 향후 자동 검사 도입 후보: pre-commit hook 에서 정규식 기반 한글 unicode 검출 (단, 의도적 한글 string literal 은 false positive 가능 — 신중히 도입)

---

## 10. Troubleshooting

| 상황 | 대응 |
|------|------|
| feature 브랜치가 main 에서 오래 방치 → merge conflict 우려 | `git rebase main` 또는 main merge 로 최신화. 주기적 sync 권장. |
| Release Drafter 가 라벨 없는 PR 을 미분류로 표시 | PR 작성자는 머지 전 3축 라벨 부착 필수. 미부착 PR 은 review 에서 reject. |
| 실수로 main 에 직접 push | Branch protection rule 활성화로 차단됨. 우회 시 즉시 revert + hotfix 브랜치로 이관. |
| 한국어 주석이 신규 코드에 들어감 | §9.1 위반. 머지 전 영어로 수정. agent 가 작성한 경우 위임 프롬프트에 §9.5 라인 누락 → 다음 위임에 추가. |
| Doc-only PR (README, .moai/specs/, LICENSE) auto-merge 차단 | **해소**: `.github/workflows/ci-required-stubs.yml` (2026-04-26 추가) 가 7 required contexts 를 stub 으로 SUCCESS 보고. doc-only PR 도 추가 조치 없이 auto-merge 가능. |

---

Version: 2.0.0
Last Updated: 2026-04-27
Scope: github.com/modu-ai/moai-studio (PUBLIC, transferred from GoosLab/moai-studio 2026-04-26)

Changelog:
- 2.0.0 (2026-04-27): **GitHub Flow 전환**. Enhanced GitHub Flow 폐지, develop 브랜치 삭제. feature → main 직접 머지. 모든 develop 관련 섹션 제거 (§1, §2, §4, §5, §6, §7, §10).
- 1.3.0 (2026-04-26): PUBLIC visibility 전환 완료 (v0.1.0 이전 선제 처리). §8 에 PUBLIC 전환 메모 추가. §10 troubleshooting 에 doc-only PR auto-merge 해소 (`ci-required-stubs.yml`) + Release Drafter 정상 동작 + GHA billing 해소 항목 갱신. CLAUDE.local.md 자체는 PUBLIC repo 에 commit 되어 외부 노출 (정책 텍스트만, 민감 정보 0).
- 1.2.0 (2026-04-26): §2 branch protection 활성 (main + develop, 7 required contexts), §2.4 Auto-merge 운영 가이드 신설. §7.2 sync subcommand 에 auto-merge 패턴 주입. Repo transfer (GoosLab → modu-ai).
- 1.1.0 (2026-04-26): §9 Code Comments Policy 신설 (HARD: 모든 코드 주석 영어). Troubleshooting → §10 이동. CI billing / Release Drafter config troubleshooting 항목 추가.
- 1.0.0 (2026-04-24): 초안. Enhanced GitHub Flow + 3축 라벨 + Release Drafter + branch protection 가이드.
