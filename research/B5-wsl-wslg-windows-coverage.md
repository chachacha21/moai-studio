# B5 — WSL + WSLg 를 통한 Windows 커버리지 분석

> **Date**: 2026-04-11
> **Method**: WebFetch Microsoft official docs + WSLg GitHub issues + Ghostty WSL discussions + user reports
> **Thesis under test**: "Linux 네이티브 빌드를 내면 WSLg 로 Windows 유저가 자동으로 사용 가능하다"

---

## Verdict

**부분적으로 참**이지만 **공짜 점심 아님**.

- **Developer-tier Windows coverage**: YES (~30-50% Windows 개발자)
- **General-purpose Windows support**: NO
- **Transparent Windows story**: NO

**Bottom line**: "Linux 빌드 = 무료 Windows 지원" 은 통째로 주장할 수 없지만, **개발자 대상 비공식 tier** 로 문서화해서 제공하면 실제로 작동합니다.

---

## 1. WSLg Architecture

### 그래픽 스택
- **Compositor**: Weston (Wayland 레퍼런스) + Microsoft 의 **RDP-RAIL/VAIL 백엔드**로 개별 창을 RDP 로 remoting
- **X11 지원**: XWayland 번들로 legacy X11 앱도 실행 가능
- **통합 방식**: Windows 가 로컬에서 RDP 로 합성. 픽셀 스트리밍 아님. virtio-fs 공유 메모리로 오버헤드 최소화
- **GPU 가속**: Mesa 의 D3D12 Gallium 드라이버 (Ubuntu 24.04 이상) — OpenGL → DirectX 12 변환
- **Vulkan**: [UNVERIFIED] Mesa 를 통해 일부 지원, 공식 primary path 아님
- **최소 Windows**: Windows 10 Build 19044+ (21H2) 또는 Windows 11 (권장)

근거:
- [Microsoft Learn: gui-apps](https://learn.microsoft.com/en-us/windows/wsl/tutorials/gui-apps)
- [DevBlogs: WSLg Architecture](https://devblogs.microsoft.com/commandline/wslg-architecture/)
- [DeepWiki: GPU Acceleration](https://deepwiki.com/microsoft/wslg/5.3-gpu-acceleration)

---

## 2. GTK4 / Ghostty / WSLg 호환성 (★ 가장 중요한 발견)

### GTK4 on WSLg 는 버그 다수

Microsoft 공식 이슈 트래커에서 확인된 GTK4 관련 open bugs:

1. **[wslg#922 — Gtk4 applications display nothing](https://github.com/microsoft/wslg/issues/922)**: `gtk4-demo` 등 GTK4 앱이 빈 창만 띄움. Workaround: `LIBGL_ALWAYS_SOFTWARE=1` (하드웨어 가속 포기).
2. **[wslg#754 — GTK4 windows cannot resize with wayland backend](https://github.com/microsoft/wslg/issues/754)**: GTK4 창이 리사이즈 안 됨, minimize/maximize 버튼 없음. Workaround: `GDK_BACKEND=x11`.
3. **[wslg#1265 — popups in gnome-text-editor won't disappear](https://github.com/microsoft/wslg/issues/1265)**: GTK4 앱의 팝업 메뉴가 사라지지 않고 창 뒤로 이동. Segfault 가끔 발생.
4. **[wslg#1299 — GTK4 Popup menus unresponsive after first click](https://github.com/microsoft/wslg/issues/1299)**

**공통 진단**: Microsoft 의 RDP-RAIL Weston 백엔드가 GTK4 의 Wayland 프로토콜 사용과 호환성 문제. 일관된 workaround 는 `GDK_BACKEND=x11` 강제 (GTK4 네이티브 Wayland 경로 포기).

### Ghostty 는 공식적으로 WSL 미지원

[ghostty#2563](https://github.com/ghostty-org/ghostty/discussions/2563) 에서 Ghostty 메인테이너 인용:

> "WSL isn't really a supported target at the moment, and there's no guarantee it will work... issues won't be actively fixed... official WSL support comes with the Windows timeline."

비공식 Windows 타임라인: **Ghostty 1.4 또는 1.5**, 확정 없음.

### Ghostty-specific WSL 버그

2024-2025 사용자 보고 ([ghostty GitHub discussions](https://github.com/ghostty-org/ghostty/discussions)):

- **창 리사이즈 불가** — maximized 와 hardcoded config size 만
- **Context menu 가 멈춤, 때로 크래시** ([#9638](https://github.com/ghostty-org/ghostty/discussions/9638))
- **Shell integration 깨짐** — Ghostty 열고 `source ~/.zshrc` 수동 실행 필요 ([#7107](https://github.com/ghostty-org/ghostty/discussions/7107))
- **Config hot-reload 작동 안 함** — 재시작 필요
- **GNOME libadwaita 버튼 미표시** (snap build)

### 커뮤니티 네이티브 포트

[InsipidPoint/ghostty-windows](https://github.com/InsipidPoint/ghostty-windows) — Win32 + OpenGL (WGL) + ConPTY 를 추가한 비공식 fork. 업스트림 머지 안 됨.

### moai-cli 에 주는 의미

**libghostty 를 Linux 빌드의 터미널 백엔드로 쓰면 WSL 사용자는 고생합니다.**

**결론**: **Linux 빌드에서는 VTE (libvte, GNOME 의 GTK 네이티브 가상 터미널)를 사용해야 합니다.**

VTE 의 장점:
- GTK4 네이티브 통합 (공식 widget)
- 수년간 GNOME Terminal / Terminator / Tilix 에서 프로덕션 검증
- WSLg 에서 안정적 (GNOME Terminal 은 WSLg 에서 잘 작동)
- `libvte` 는 패키지 매니저로 즉시 설치 가능

VTE 의 단점:
- macOS 에서는 사용 안 함 (macOS 는 libghostty)
- Ghostty 만큼 예쁘지는 않음 (GPU 가속 제한적)

**v4 설계**: macOS = libghostty, Linux/WSL = VTE. 2개 터미널 백엔드, 같은 Claude Code 프로토콜.

---

## 3. 9P 파일 시스템 병목 (★ 두 번째로 중요)

### 성능 절벽

WSL2 는 9P (Plan 9) 프로토콜로 Windows/Linux 파일 브리지. 이것이 WSL2 의 가장 큰 성능 문제입니다.

- **~9배 느림** (네이티브 ext4 대비 cross-boundary 연산)
- Phoronix 벤치: WSL2 bare-metal Linux 의 ~87% 성능 (네이티브 ext4 워크로드만)
- [microsoft/WSL#13846](https://github.com/microsoft/WSL/issues/13846): Windows 드라이브의 `ls` 가 30초 걸림
- `git status` / `git diff` 특히 느림
- Visual Studio, ripgrep, 풀텍스트 인덱싱이 `\\wsl.localhost\` 에서 "극단적 지연 + 시스템 크래시" 보고

### moai-cli 권고사항

**사용자가 프로젝트를 WSL2 ext4 파일 시스템 안에 보관해야 합니다**, `C:\Users\...` 아님.

이는 "윈도우 사용자" 가 기대하는 워크플로우 (Downloads 폴더 더블클릭) 와 **충돌**합니다. Documentation 에 명시적으로 설명 필요:

```
[HARD] WSL 사용자 가이드:
- 프로젝트는 반드시 ~/projects/ (Linux FS) 안에
- /mnt/c/Users/... 는 사용 금지
- git clone 은 WSL 내부에서만
```

근거:
- [Microsoft Learn: compare-versions](https://learn.microsoft.com/en-us/windows/wsl/compare-versions)
- [Tributary AI: Optimizing WSL2 for Claude Code](https://www.thetributary.ai/blog/optimizing-wsl2-claude-code-performance-guide/)
- [Allen Kuo: 9P vs Samba benchmarks](https://allenkuo.medium.com/windows-wsl2-i-o-performance-benchmarking-9p-vs-samba-file-systems-cf2559be41ac)

---

## 4. AF_UNIX 크로스 바운더리 (WSL ↔ Windows)

### 결론: 깨져있음, 사용 불가

- WSL1 시절에는 [AF_UNIX Windows/WSL interop](https://devblogs.microsoft.com/commandline/windowswsl-interop-with-af_unix/) 이 작동
- **WSL2 에서 regression**, 2026 현재도 미해결
- 공식 버그: [microsoft/WSL#5961](https://github.com/microsoft/WSL/issues/5961), [#8321](https://github.com/microsoft/WSL/issues/8321)
- Workaround: localhost TCP (자동 포워딩됨) 또는 Windows named pipe

### moai-cli 에 주는 의미

- **WSL 안에서 Linux-to-Linux**: Unix socket 완벽 작동 (양쪽 Linux 프로세스)
- **WSL ↔ Windows 바운더리**: Unix socket 불가. localhost TCP 필수.
- **우리 아키텍처는 영향 없음** — moai-cli Linux 빌드와 Claude Code 가 모두 WSL 안에서 실행됨. 바운더리 넘기 없음.

---

## 5. Claude Code CLI in WSL — 공식 지원

### 공식 설치 방법 (2025-2026)

[Claude Code setup docs](https://code.claude.com/docs/en/setup) 에서 Windows 2가지 공식 경로:

1. **Native Windows** (2025 년 이후 권장 기본)
   - 공식 installer 또는 `winget install Anthropic.ClaudeCode`
   - 내부에서 **Git Bash** 사용
   - Node.js 불필요
   - "Anthropic, PBC" 로 서명된 바이너리

2. **WSL2** (Ubuntu 20.04+ / Debian 10+)
   - `npm install -g @anthropic-ai/claude-code` 또는 공식 installer
   - 여전히 공식 지원
   - 권장 시나리오: "WSL 환경 이미 있음, Docker 통합 필요, bash tool sandboxing 원함"

**두 설치 공존 가능**: PowerShell 의 `claude` 와 WSL 의 `claude` 는 독립.

### moai-cli 에 주는 의미

moai-cli Linux 빌드가 WSLg 에서 실행될 때, **WSL 내부의 `claude` 바이너리를 사용**하면 됩니다. Windows 측과 통신 필요 없음. Authentication 도 `~/.claude/` (WSL 내부) 에서 처리.

---

## 6. Desktop Integration

### WSLg 가 제공하는 것

- ✅ **Taskbar**: Linux GUI 앱이 Windows taskbar 에 개별 표시
- ✅ **Alt-Tab**: Linux/Windows 창 간 전환
- ✅ **Start Menu**: `.desktop` 파일 등록 시 "Ubuntu → AppName" 형태로 자동 표시
- ✅ **Clipboard**: Bidirectional, text/HTML/bitmap
- ✅ **Window snapping**: Windows 가 처리
- ✅ **Audio**: PulseAudio → PipeWire → Windows
- ✅ **High-DPI**: Per-monitor DPI via RDP client-side scaling

### WSLg 가 제공하지 않는 것

- ❌ **Notifications**: [UNVERIFIED] 공식 문서에 libnotify/D-Bus → Windows Notification Center 브리지 확인 안 됨. 불안정하다고 가정
- ❌ **File associations**: Linux 앱을 Windows 파일 확장자의 default handler 로 등록 불가
- ❌ **Desktop 바로가기**: 자동 아님. 사용자가 `wslg.exe -d Ubuntu -- /path/to/app` 수동 설정
- ❌ **Installer (MSI/EXE/MSIX)**: 없음. 배포 = `.deb` 또는 소스 빌드
- ❌ **Windows Update / winget 배포**: 없음
- ❌ **Code signing**: Linux ELF 는 Authenticode 서명 불가. SmartScreen 우회 불가

---

## 7. 사용자 세그먼트 vs 커버리지

| 세그먼트 | WSL+WSLg 로 커버? |
|---|---|
| WSL2 매일 쓰는 Windows 개발자 | ✅ (libghostty → VTE 전환 시) |
| 네이티브 Windows 도구 (VS, Rider) 쓰는 개발자 | ❌ (WSL 설치 거부) |
| 비개발자 Windows 사용자 (디자이너, PM) | ❌ (WSL 접근 불가) |
| 엔터프라이즈 Windows 펫 (IT 관리) | ❌ (WSL 종종 금지) |

**"Linux 빌드 = 무료 Windows 지원" 은 WSL-literate 개발자 (~30-50%) 만 커버**합니다. 나머지 50-70% 는 네이티브 Windows 포트가 필요합니다.

---

## 8. moai-cli v4 전략 권장

### Tier Ladder (점진적 Windows 지원)

| Tier | 작업 | 대상 |
|---|---|---|
| **Tier 0 — Free** (Linux 빌드만) | Ubuntu `.deb` + "WSL2 사용자 설치 가이드" 문서 + VTE 기반 터미널 + 프로젝트 배치 가이드 | WSL 숙련 개발자 |
| **Tier 1 — Small** (~1주) | `.reg` 또는 PowerShell 스크립트로 Start Menu shortcut + 파일 association via `wslg.exe` wrapper | WSL 중급 |
| **Tier 2 — Medium** (~2개월) | 서명된 Windows helper exe (Notification bridge, Windows file picker) → WSL moai-cli 와 localhost TCP 통신 | WSL + 표준 Windows UX |
| **Tier 3 — Full** (~3-6개월) | Win32 네이티브 포트 (WGL + ConPTY, InsipidPoint/ghostty-windows 패턴 차용) | 전체 Windows 생태계 |

### moai-cli 단기 전략

- **M0-M5**: macOS 만 타겟. Linux 포팅 생각 안 함
- **M6**: Linux 포팅 개시 (macOS 1.0 출시 후)
- **M7**: Linux + **WSL Tier 0 문서** 포함 출시
- **M8**: Windows 수요가 있다면 Tier 1 → Tier 2 검토
- **Tier 3 (네이티브 Windows)**: 연간 1000+ Windows 사용자 신호가 있을 때 착수

### 피해야 할 것

- ❌ "Windows 에서 WSLg 로 완벽 작동" 이라고 마케팅
- ❌ WSLg notification / file association 에 의존하는 기능
- ❌ WSL/Windows 바운더리를 넘는 Unix socket IPC
- ❌ libghostty 를 Linux 기본 터미널로 사용 (VTE 대신)

---

## 9. Source Inventory

### Microsoft 공식
- https://learn.microsoft.com/en-us/windows/wsl/tutorials/gui-apps
- https://github.com/microsoft/wslg
- https://devblogs.microsoft.com/commandline/wslg-architecture/
- https://learn.microsoft.com/en-us/windows/wsl/compare-versions
- https://devblogs.microsoft.com/commandline/windowswsl-interop-with-af_unix/

### WSLg GitHub Issues
- https://github.com/microsoft/wslg/issues/922 (GTK4 empty windows)
- https://github.com/microsoft/wslg/issues/754 (GTK4 resize broken)
- https://github.com/microsoft/wslg/issues/1265 (popup issues)
- https://github.com/microsoft/wslg/issues/1299 (popup unresponsive)
- https://github.com/microsoft/WSL/issues/5961 (AF_UNIX interop)
- https://github.com/microsoft/WSL/issues/13846 (9P slowness)

### Ghostty
- https://github.com/ghostty-org/ghostty/discussions/2563 (Windows support)
- https://github.com/ghostty-org/ghostty/discussions/7107 (shell integration)
- https://github.com/ghostty-org/ghostty/discussions/9638 (menu issues)
- https://github.com/InsipidPoint/ghostty-windows (native port)

### Claude Code on Windows/WSL
- https://code.claude.com/docs/en/setup
- https://claudelab.net/en/articles/claude-code/claude-code-windows-native-wsl2-complete-guide
- https://smartscope.blog/en/generative-ai/claude/claude-code-windows-native-installation/

### User Reports
- https://brucelim.com/blog/ghostty-wsl-install
- https://www.thetributary.ai/blog/optimizing-wsl2-claude-code-performance-guide/
- https://allenkuo.medium.com/windows-wsl2-i-o-performance-benchmarking-9p-vs-samba-file-systems-cf2559be41ac
- https://markaicode.com/wslg-2-ubuntu-gui-app-performance-benchmarks/
