// @MX:WARN(zig-toolchain-precondition)
// @MX:REASON: Zig 0.15.x 미설치 시 전체 moai-studio-terminal 빌드가 차단된다.
//             libghostty-vt 의 build.rs 는 Zig 로 C 소스를 컴파일하므로
//             이 crate 의 build.rs 에서 선제 검증하여 명확한 에러를 제공한다.
//             CI: mlugg/setup-zig@v2.2.1 + actions/cache@v4 로 해결 (SPEC-V3-002 AC-T-7).

use std::path::Path;
use std::process;

fn main() {
    // Zig 설치 여부 확인 (libghostty-vt 빌드 전제)
    if let Err(e) = check_zig() {
        eprintln!("{}", e);
        process::exit(1);
    }

    // cargo 가 libghostty-vt build.rs 를 실행하기 전에 통과함을 보장
    println!("cargo:rerun-if-env-changed=PATH");
    println!("cargo:rerun-if-env-changed=GHOSTTY_SOURCE_DIR");

    // @MX:NOTE(libghostty-dylib-copy): libghostty-vt-sys 가 dylib 으로만 링크되므로
    //   binary 런타임에 @rpath/libghostty-vt.dylib (Linux: libghostty-vt.so) 이 필요하다.
    //   Cargo 는 외부 빌드 dylib 을 자동 복사하지 않으므로 본 build.rs 가
    //   libghostty-vt-sys 의 OUT_DIR 에서 dylib 을 찾아 target/<profile>/ 로 복사한다.
    //   .cargo/config.toml 의 rustflags rpath=@executable_path (macOS) / $ORIGIN (Linux)
    //   가 binary 옆의 dylib 을 찾는다.
    copy_libghostty_dylib();
}

fn copy_libghostty_dylib() {
    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        return;
    };
    // OUT_DIR = target/<profile>/build/moai-studio-terminal-<hash>/out
    let Some(profile_dir) = Path::new(&out_dir).ancestors().nth(3) else {
        return;
    };
    let build_dir = profile_dir.join("build");
    let Ok(entries) = std::fs::read_dir(&build_dir) else {
        return;
    };

    // Target OS 별 dylib 확장자 + 심볼릭 링크 파일들
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let dylib_patterns: &[&str] = match target_os.as_str() {
        "macos" => &[
            "libghostty-vt.dylib",
            "libghostty-vt.0.dylib",
            "libghostty-vt.0.1.0.dylib",
        ],
        "linux" => &[
            "libghostty-vt.so",
            "libghostty-vt.so.0",
            "libghostty-vt.so.0.1.0",
        ],
        _ => return, // Windows 는 SPEC-V3-002 §6 Exclusions — Phase 7 이관
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with("libghostty-vt-sys-") {
            continue;
        }
        let lib_dir = entry.path().join("out/ghostty-install/lib");
        if !lib_dir.exists() {
            continue;
        }
        for pattern in dylib_patterns {
            let src = lib_dir.join(pattern);
            let dst = profile_dir.join(pattern);
            if src.exists() {
                // 심볼릭 링크 보존: src 가 symlink 면 symlink 로 복사, 아니면 파일 복사
                let _ = std::fs::remove_file(&dst);
                if let Ok(link_target) = std::fs::read_link(&src) {
                    #[cfg(unix)]
                    let _ = std::os::unix::fs::symlink(&link_target, &dst);
                } else {
                    let _ = std::fs::copy(&src, &dst);
                }
            }
        }
        println!("cargo:rerun-if-changed={}", lib_dir.display());
    }
}

/// Zig 0.15.x 설치 여부를 검증한다.
///
/// 반환:
/// - Ok(()) — Zig 가 PATH 에 있고 버전이 0.15.x
/// - Err(String) — 표준 에러 메시지 (AC-T-2 규격)
pub fn check_zig() -> Result<(), String> {
    let output = std::process::Command::new("zig").arg("version").output();

    match output {
        Err(_) => Err("Zig 0.15.x required — install via mise/asdf/ziglang.org".to_string()),
        Ok(out) => {
            let version = String::from_utf8_lossy(&out.stdout);
            let version = version.trim();
            if !version.starts_with("0.15") {
                Err(format!(
                    "Zig 0.15.x required — install via mise/asdf/ziglang.org (found: {})",
                    version
                ))
            } else {
                Ok(())
            }
        }
    }
}
