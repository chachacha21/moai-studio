# UITest CI Signing 설정 가이드

## 개요

C-1 carry-over 해소. Apple Developer Program 1회 설정 후 CI UITest 완전 자동화.

Secrets 업로드가 완료되면 모든 PR에서 `MoAIStudioUITests` 가 자동으로 실행됩니다.
Secrets 가 없는 경우(fork PR 포함) CI 는 자동으로 UITest 를 skip 하고 skip 안내 메시지를 출력합니다.

---

## 선행 조건

- Apple ID (무료 ID 아닌 유료 Developer Program 멤버십, $99/year)
- macOS (Keychain Access 앱 필요)
- GitHub 저장소 admin 권한 (Secrets 설정)

---

## 단계 1 — Apple Developer Program 가입

1. <https://developer.apple.com/programs/> 접속
2. "Enroll" 클릭 → Apple ID 로그인 → $99 결제 선택
   - 개인(Individual): 보통 즉시 승인
   - 조직(Organization): 1-2일 소요 (D-U-N-S 번호 필요)
3. 가입 승인 이메일 수신 확인

---

## 단계 2 — Developer ID Application 인증서 생성

UITest 용으로 **Developer ID Application** 인증서를 권장합니다 (macOS 앱 서명용).

### 2-1. CSR 생성 (로컬 macOS)

```
Keychain Access 앱 열기
→ 메뉴 바 [Keychain Access] → [Certificate Assistant] → [Request a Certificate from a Certificate Authority...]
→ 이메일 주소 및 이름 입력
→ "Saved to disk" 선택
→ Continue → CSR.certSigningRequest 파일 저장
```

### 2-2. Apple Developer 콘솔에서 인증서 생성

1. <https://developer.apple.com/account/resources/certificates/list> 접속
2. "+" 버튼 → "Developer ID Application" 선택 → Continue
3. CSR 파일 업로드 → Continue
4. 인증서 Download `.cer` 파일 저장
5. `.cer` 파일을 더블클릭하여 Keychain 에 설치

### 2-3. .p12 내보내기

1. Keychain Access 앱 → 로그인 키체인
2. `Developer ID Application: <이름> (<TEAM_ID>)` 인증서 찾기
3. 우클릭 → "Export <인증서명>..."
4. 포맷: "Personal Information Exchange (.p12)" 선택
5. 파일명: `moai-studio-signing.p12` 으로 저장
6. 강력한 암호 설정 후 저장 (이 암호가 `P12_PASSWORD` 가 됩니다)

---

## 단계 3 — Team ID 확인

<https://developer.apple.com/account> → Membership Details → **Team ID** (10자 영숫자 문자열)

---

## 단계 4 — GitHub Secrets 업로드

저장소 → Settings → Secrets and variables → Actions → **New repository secret**

먼저 헬퍼 스크립트를 실행합니다:

```bash
./scripts/export-signing-assets.sh moai-studio-signing.p12
```

스크립트 출력을 참고하여 아래 4개 Secret 을 등록합니다:

| Secret 이름 | 값 | 출처 |
|------------|---|------|
| `BUILD_CERTIFICATE_BASE64` | base64 문자열 | 스크립트 출력 |
| `P12_PASSWORD` | .p12 내보내기 암호 | 단계 2-3 에서 설정 |
| `KEYCHAIN_PASSWORD` | 임의 문자열 | 스크립트 제안값 사용 권장 |
| `APPLE_TEAM_ID` | 10자 Team ID | 단계 3 에서 확인 |

> **선택 사항**: Distribution 서명(App Store) 시 `PROVISIONING_PROFILE_BASE64` 도 필요하지만, UITest 단독 실행에는 불필요합니다.

---

## 단계 5 — 검증

1. `main` 브랜치에 대한 PR 또는 push 생성
2. GitHub Actions 탭 → `Swift CI` 워크플로우 확인
3. 아래 스텝이 순서대로 성공해야 합니다:
   - `Install signing (if available)` → 초록색
   - `UITest (signed)` → 초록색 (MoAIStudioUITests 실행)

---

## 로컬 개발자 설정

Xcode 에서 로컬 서명을 구성하려면:

```bash
./scripts/setup-signing-local.sh moai-studio-signing.p12 <p12-password> <TEAM_ID>
```

스크립트 실행 후 Xcode 에서 정상 서명이 가능합니다.

---

## 롤백

Secrets 에서 `BUILD_CERTIFICATE_BASE64` 를 삭제하면 CI 가 자동으로 UITest skip 모드로 전환됩니다. 기존 ad-hoc 빌드는 그대로 유지됩니다.

---

## 문제 해결

| 증상 | 원인 | 해결 |
|------|------|------|
| "Unable to build chain to self-signed root" | 인증서 체인 불완전 | Apple WWDR CA 인증서 Keychain 설치 확인 |
| "User interaction is not allowed" | 키체인 잠금 해제 실패 | `KEYCHAIN_PASSWORD` Secret 값 확인 |
| "No identity found" | 인증서 임포트 실패 | `P12_PASSWORD` 와 `.p12` 파일 일치 여부 확인 |
| UITest timeout | macOS 권한 다이얼로그 | Developer ID 인증서 사용 여부 확인 (ad-hoc 아닌지) |

> **Notarization 관련**: App Store 배포 또는 Gatekeeper 통과가 목적인 경우 notarization 이 추가로 필요합니다. 이는 본 가이드 범위를 벗어나며 M6 배포 스프린트에서 `AC_USERNAME` / `AC_PASSWORD` secrets 와 함께 구성 예정입니다.

---

## 보안 유의 사항

- Secrets 값을 커밋, PR 본문, 로그, 슬랙 등 어디에도 노출하지 마세요.
- 임시 키체인은 CI job 종료 시 composite action 의 `post` 단계에서 자동 삭제됩니다.
- `.p12` 파일을 절대 저장소에 커밋하지 마세요 (`.gitignore` 에 `*.p12` 포함됨).
- 인증서 유효기간(5년)이 만료되기 전에 교체하세요.

---

*관련 파일: `.github/actions/install-signing/action.yml`, `scripts/export-signing-assets.sh`, `scripts/setup-signing-local.sh`*
*관련 문서: `.github/WORKFLOWS.md`*
