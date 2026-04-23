//! trybuild compile-fail 테스트 하네스
//!
//! Windows target 에서 ConPtyStub::spawn() / read_available() 호출이
//! compile_error! 로 차단되는지 검증 (AC-T-10).
//!
//! SPEC-V3-002 AC-T-10: "Windows target (on-demand, CI matrix 미포함)".
//! Unix 플랫폼 에서는 rustc 버전/플랫폼별 stderr 포맷 차이로 false negative
//! 가 발생하므로 `#[ignore]` 로 기본 제외. 개발자 on-demand 실행 전용.
//!
//! 실행법:
//!   cargo test -p moai-studio-terminal --test compile_fail -- --ignored
//!   TRYBUILD=overwrite cargo test ... -- --ignored  (stderr 스냅샷 재생성)

#[test]
#[ignore = "AC-T-10 on-demand: cross-platform stderr 포맷 불일치 방지"]
fn conpty_compile_error_gate() {
    // compile_fail/ 디렉터리의 .rs 파일들이 예상된 에러로 실패하는지 확인
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/conpty_spawn.rs");
}
