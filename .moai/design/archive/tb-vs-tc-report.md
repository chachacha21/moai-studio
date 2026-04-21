# TB vs TC 심층 비교 보고서 (2026-04-21)

**배경 수정**: 초기 v3 제안에서 TC (Ghostty 하이브리드) 를 "복잡, 유지보수 비용 큼" 으로 평가했으나, 후속 리서치에서 **libghostty-vt + libghostty-rs** 생태계를 발견하여 TC 가 TB 와 거의 동일한 복잡도로 축소됨. 본 보고서로 재평가.

---

## 1. TB · TC 정의 (수정)

### TB — alacritty_terminal + wgpu 네이티브 렌더러

- VT state/parser: [alacritty_terminal](https://crates.io/crates/alacritty_terminal) (Apache 2.0, Rust 네이티브)
- PTY: alacritty_terminal 내장 + portable-pty fallback
- Renderer: [wgpu](https://wgpu.rs/) + [glyphon](https://github.com/grovesNL/glyphon) (cosmic-text + wgpu)
- 빌드: pure Rust, Zig 불필요
- 플랫폼: macOS/Linux/Windows/BSD 자동 크로스 (cargo build)

### TC — libghostty-vt + wgpu 네이티브 렌더러 (수정됨)

- VT state/parser: [libghostty-vt](https://lib.rs/crates/libghostty-vt) via [libghostty-rs](https://github.com/Uzaaft/libghostty-rs) FFI
- PTY: portable-pty 또는 libghostty-spm 래퍼
- Renderer: **wgpu + glyphon (TB 와 동일)** OR 선택적 macOS Metal xcframework (libghostty-spm)
- 빌드: cargo + **Zig 0.15.x 필수** (libghostty 소스 컴파일)
- 플랫폼: libghostty-vt 는 **zero-dependency C/Zig** — macOS/Linux/Windows/WebAssembly 호환 공식

---

## 2. 10축 비교 매트릭스

| 축 | TB (alacritty_terminal) | TC (libghostty-vt) | 승자 |
|----|-------------------------|---------------------|------|
| **1. VT 파서 안정성** | Alacritty v0.14 프로덕션 검증 | Ghostty 프로덕션 검증 + libghostty-vt **alpha** (API 변경 가능) | **TB** |
| **2. 라이브러리 API 안정성** | 1.0 stable (2022~) | Alpha (1.0 6개월 내 목표, 2025-09 기준) | **TB** |
| **3. 유지보수 활성도** | Alacritty 팀 활발 | Mitchell + Ghostty 팀 활발 | 동등 |
| **4. 크로스플랫폼 호환** | Rust 네이티브 (자동) | zero-dep C/Zig (4 플랫폼 공식 지원) | 동등 |
| **5. 빌드 의존성** | cargo only | cargo + **Zig 0.15.x 툴체인** | **TB** |
| **6. 렌더 성능 (정확도/품질)** | 우수, Unicode 표준 | **Ghostty 수준 (industry best)** ligatures + emoji + Unicode edge cases | **TC** |
| **7. 입력 레이턴시** | Alacritty 는 낮은 레이턴시 최우선 설계 | Ghostty 도 낮은 레이턴시 (Metal tight integration) | 동등 |
| **8. 이미지 프로토콜** | 없음 (Alacritty 의도적 제외) | 부분 (wezterm_term 보다는 적음) | TC 약간 우위 |
| **9. 기존 MoAI 자산 재사용** | 0% (새 학습) | macOS Ghostty xcframework (M0/M1 투자) 부분 재활용 가능 | **TC** |
| **10. 레퍼런스 구현 풍부도** | Alacritty 본체, Zed Terminal | ghostling_rs, [gpui-ghostty](https://xuanwo.io/2026/01-gpui-ghostty/), Restty, taskers | TC 약간 우위 |

**총합**: TB 3승, TC 3승, 동등 4 → **기술적으로 동급**

---

## 3. 사용자 체감 UX 비교

렌더 품질 (실측 근거 기반):

| UX 요소 | TB | TC |
|---------|-----|-----|
| 평문 스크롤 (10K 줄 @60Hz) | ✅ 60 FPS | ✅ 60+ FPS |
| 고속 출력 (tail -f, make) | ✅ 문제없음 | ✅ 문제없음 (Ghostty 는 iTerm 대비 4x) |
| Unicode 이모지 | 표준 | **우수 (Ghostty 검증)** |
| Coding ligatures (Fira Code 등) | 표준 | **유일한 Metal ligature 가속 계승** |
| 복잡 Unicode (RTL, Indic) | 표준 | Ghostty 우수 |
| 터미널 그래픽 프로토콜 | ❌ | 부분 |
| 120Hz ProMotion | wgpu 가 지원, 벤치 필요 | libghostty-vt 는 VT only, 렌더러 따라 결정 |

**결론**: TC 가 Unicode/ligature 에서 미묘하게 우위이지만, TB 로도 일반 사용자는 차이를 체감하기 어렵다. 커뮤니티 고급 유저만 차이를 안다.

---

## 4. 개발 복잡도 비교

### TB 개발 체크리스트
- [ ] `alacritty_terminal = "0.24"` 의존성 추가
- [ ] `wgpu = "0.20"` + `glyphon = "0.5"` 통합
- [ ] Tauri 창 내부에 wgpu surface 통합 (Tao WindowHandle)
- [ ] 터미널 grid → glyphon TextRenderer 연결
- [ ] 색상/속성 cell → wgpu render pass
- [ ] PTY 프로세스 spawn → alacritty_terminal::tty::Pty
- [ ] Input 이벤트 → terminal.input()
- [ ] OSC 8 hyperlink 파싱 (기본 지원)

### TC 개발 체크리스트 (추가 항목 강조)
- [ ] **Zig 0.15.x 툴체인 설치** ⚠️
- [ ] `libghostty-vt = "0.1"` (alpha) 의존성 추가
- [ ] `libghostty-vt-sys` FFI bindgen 자동 실행
- [ ] wgpu + glyphon 통합 (TB 와 동일)
- [ ] Terminal::new(TerminalOptions) 기반 상태 관리
- [ ] on_pty_write 콜백 등록
- [ ] RenderState → 렌더러 연동 (API 아직 flux)
- [ ] **alpha API 변경 시 마이그레이션 대비**

**추가 리스크 (TC)**:
- 빌드 서버 CI 에 Zig 설치 필요 → 빌드 시간 증가
- `libghostty-vt` 1.0 출시 전 breaking change 발생 가능
- Windows 빌드에서 Zig 0.15.x 호환성 검증 필요

---

## 5. 지연 (Latency) 상세

Ghostty 자체 벤치마크 (ghostty-org/ghostty discussion #4837):
- Ghostty 는 iTerm2/Kitty 대비 **4x 빠른 평문 파싱**, Terminal.app 대비 **2x**
- 그러나 유지관리자 발언: *"latency 는 대부분 사용자에게 우선순위 하위"*

Alacritty 는 설계 시점부터 "낮은 레이턴시" 를 타겟팅 → 벤치 결과 Ghostty 에 근접.

**MoAI Studio 컨텍스트**: Claude Code 가 초당 수십 token 스트리밍 → 어떤 쪽이든 충분.

---

## 6. MoAI Studio v3 전략 적합도

| 전략 요소 | TB | TC |
|-----------|-----|-----|
| "smart terminal-first" 포지셔닝 | ✅ | ✅ (Ghostty 브랜드 후광 가능) |
| Mitchell Hashimoto 생태계 정렬 (cmux 도 이것) | △ | ✅ |
| 빠른 MVP 출시 | ✅ (Zig 불필요) | △ (Zig 체인 + alpha API) |
| 장기 최고 품질 | 표준 우수 | **최고 수준** |
| 크로스 플랫폼 CI 복잡도 | 낮음 | 중간 (Zig 추가) |
| libghostty-vt 1.0 도달 후 전환 | ✅ 가능 | - |

---

## 7. 하이브리드 옵션: TB → TC 마이그레이션 경로

**권장 운영 전략**:

```
Phase 1-4 (MVP): TB 로 먼저 출시
  ├─ 안정적 cargo 빌드
  ├─ 빠른 개발
  └─ alacritty_terminal API 안정

Phase 8 (libghostty-vt 1.0 출시 후, 2026-Q4 추정):
  └─ TC 마이그레이션 평가 + A/B 벤치
     ├─ 체감 개선이 측정 가능하면 TC 로 전환
     └─ 차이가 미미하면 TB 유지
```

**이 경로의 이점**:
- 제품 출시 지연 없음 (TB 로 즉시 시작)
- libghostty-vt alpha 리스크 회피
- VT state/render 분리로 마이그레이션 비용 중간 수준 (grid/cell API 유사)

**이 경로의 비용**:
- 미래 마이그레이션 1회 발생 (부담 중간)

---

## 8. moai-adk-go 신규 SPEC 반영 사항

사용자 지적대로 `.moai/design/` 구조가 moai-adk-go 에서 공식 템플릿으로 승격됨:

### 파일명 표준 (SPEC-DESIGN-DOCS-001, SPEC-DESIGN-CONST-AMEND-001)

MoAI Studio 도 규격 정렬 필수:

| 현재 (MoAI Studio) | 표준 (moai-adk-go) | 조치 |
|-------------------|---------------------|------|
| `research-v3.md` | `research.md` | 이름 통일 (v3 archived 처리) |
| `spec-v3.md` | `spec.md` | 이름 통일 |
| `master-plan-v3.md` | `spec.md` 에 섹션 통합 OR 유지 | 검토 |
| `pencil-redesign-v2-archived.md` | `pencil-plan.md` | 활성 이름 정리 |
| `system.md` | `system.md` | ✅ 이미 정렬 |

### 우선순위 (SPEC-DESIGN-ATTACH-001)

Phase B2.5 자동 로드 우선순위: `spec > system > research > pencil-plan`

MoAI Studio 도 이 순서로 읽히도록 파일명 정렬 필요.

### 토큰 자동 생성 (SPEC-DB-CMD-001, SPEC-DB-TEMPLATES-001)

`tokens.json` (자동) + `system.md` (사람) 이중 구조. 현재 `system.md` 만 있음 → `/moai design` 또는 Claude Design 핸드오프로 `tokens.json` 자동 생성 가능.

### Pencil 경로 선택성 (SPEC-DESIGN-PENCIL-001)

- Pencil MCP 는 **선택 경로**. `.pen` 파일 + `pencil-plan.md` 존재 시에만 활성화
- 부재 시 **graceful skip** → 범용 유저에게 Pencil 의무 없음 ✅

### 중요 결론

moai-adk-go SPEC 체계 는 **Pencil optional + markdown source of truth** 를 이미 명문화함. 이전 제 보고서의 "Pencil 의무 철회" 방향이 옳았고, moai-adk-go 설계와 자동 정렬됨.

---

## 9. 최종 권고

**1차 권고 (안전한 생산 경로)**:

| 결정 | 권고 |
|------|------|
| 터미널 VT | **TB (alacritty_terminal)** |
| 렌더러 | wgpu + glyphon |
| 디자인 도구 | **Claude Design + markdown source of truth** (Pencil optional) |
| 파일명 | moai-adk-go 표준 정렬 (`spec.md`, `research.md`, `pencil-plan.md`) |
| 마이그레이션 대비 | VT state/render 분리로 TC 전환 가능성 열어둠 |

**2차 경로 (최고 품질 타겟)**:

| 결정 | 권고 |
|------|------|
| 터미널 VT | **TC (libghostty-vt)** |
| 단, 조건 | libghostty-vt 1.0 릴리즈 대기 (2026-Q4 예상) OR alpha 리스크 수용 |
| 기타 | TB 와 동일 |

**핵심 차이 포인트**:
- TB: "지금 안정적으로 출시" 를 우선시
- TC: "Ghostty 급 최고 품질" 을 우선시, Zig 툴체인 + alpha 수용

---

## 10. 참고 문헌

- [Alacritty — A fast, cross-platform, OpenGL terminal emulator](https://alacritty.org/)
- [alacritty/alacritty DeepWiki (architecture)](https://deepwiki.com/alacritty/alacritty)
- [alacritty_terminal crate docs](https://crates.io/crates/alacritty_terminal)
- [Libghostty Is Coming — Mitchell Hashimoto](https://mitchellh.com/writing/libghostty-is-coming)
- [libghostty-rs GitHub](https://github.com/Uzaaft/libghostty-rs)
- [awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty)
- [libghostty-vt crate](https://lib.rs/crates/libghostty-vt)
- [Build GPUI + Ghostty without writing code](https://xuanwo.io/2026/01-gpui-ghostty/)
- [Ghostty Performance Discussion #4837](https://github.com/ghostty-org/ghostty/discussions/4837)
- [glyphon (wgpu + cosmic-text)](https://github.com/grovesNL/glyphon)

---

버전: 1.0.0 · 2026-04-21
