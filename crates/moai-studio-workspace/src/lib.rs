//! MoAI Studio Multi-Project Workspace.
//!
//! Phase 1.4 (SPEC-V3-001 RG-V3-2): 네이티브 파일 picker + 워크스페이스 CRUD 기초.
//!
//! ## 기능 (현재)
//! - `pick_project_folder()` — NSOpenPanel/GTK/IFileDialog 네이티브 picker
//! - `WorkspaceDescriptor` — 워크스페이스 메타데이터 스키마
//! - `Workspace::from_path()` — 선택된 폴더로부터 워크스페이스 객체 생성
//!
//! ## Phase 5 (SPEC-V3-004) 확장 예정
//! - `~/.moai/studio/workspaces.json` 저장/로드 (VS Code `.code-workspace` 모델)
//! - 사이드바 스위처, 최근 사용, 글로벌 검색

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

// ============================================================
// Workspace 메타데이터
// ============================================================

/// 하나의 MoAI Studio 워크스페이스 (= 하나의 프로젝트 폴더).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// UUID (자동 생성).
    pub id: String,
    /// 사람이 읽을 수 있는 이름 (기본: 폴더명).
    pub name: String,
    /// 프로젝트 절대 경로.
    pub project_path: PathBuf,
    /// `.moai/` 디렉토리 상대 경로 (기본 ".moai/").
    pub moai_config: PathBuf,
    /// 사이드바 색상 태그 (RGB 24-bit).
    pub color: u32,
    /// 마지막 활성 시각 (Unix 초).
    pub last_active: u64,
}

impl Workspace {
    /// 절대 경로로부터 새 워크스페이스 생성.
    ///
    /// 폴더명을 이름으로 사용하며 UUID v4 를 id 로 할당.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(WorkspaceError::NotADirectory(path.to_path_buf()));
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        Ok(Self {
            id: generate_id(),
            name,
            project_path: path.to_path_buf(),
            moai_config: PathBuf::from(".moai"),
            color: 0xff6a3d, // ACCENT_MOAI (system.md §4)
            last_active: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        })
    }
}

/// UUID 대체 간이 ID 생성 (Phase 5 에서 `uuid` crate 로 교체 예정).
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ws-{:x}", nanos)
}

// ============================================================
// 네이티브 파일 picker (rfd crate)
// ============================================================

/// 네이티브 폴더 선택 다이얼로그 (blocking).
///
/// macOS: NSOpenPanel — 한국어 로케일 자동.
/// Linux: GTK FileChooser (GTK3+ 설치 전제) 또는 XDG Portal.
/// Windows: IFileDialog (modern Vista+ 다이얼로그).
///
/// 반환: 사용자가 폴더를 선택하면 `Some(PathBuf)`, 취소 시 `None`.
///
/// Phase 1.4 에서는 blocking. 향후 `pick_folder_async()` 도 추가 고려.
pub fn pick_project_folder() -> Option<PathBuf> {
    info!("pick_project_folder: 네이티브 폴더 다이얼로그 호출");
    let path = rfd::FileDialog::new()
        .set_title("MoAI Studio — 프로젝트 폴더 선택")
        .pick_folder();

    match &path {
        Some(p) => info!("pick_project_folder: 선택됨 — {}", p.display()),
        None => warn!("pick_project_folder: 취소됨"),
    }
    path
}

/// 선택된 폴더로부터 워크스페이스 생성 (편의 함수).
///
/// `pick_project_folder()` 결과를 즉시 Workspace 로 변환한다.
pub fn pick_and_create() -> Result<Option<Workspace>, WorkspaceError> {
    match pick_project_folder() {
        Some(path) => Workspace::from_path(&path).map(Some),
        None => Ok(None),
    }
}

// ============================================================
// 에러 타입
// ============================================================

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("경로가 디렉토리가 아닙니다: {0}")]
    NotADirectory(PathBuf),

    #[error("워크스페이스 I/O 실패: {0}")]
    Io(#[from] std::io::Error),

    #[error("워크스페이스 직렬화 실패: {0}")]
    Serde(#[from] serde_json::Error),
}

// ============================================================
// 스캐폴드 hello 유지
// ============================================================

pub fn hello() {
    info!("moai-studio-workspace: Phase 1.4 — 네이티브 파일 picker 활성");
}

// ============================================================
// 유닛 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_from_path_extracts_folder_name() {
        let tmp = std::env::temp_dir().join("moai-test-workspace");
        std::fs::create_dir_all(&tmp).unwrap();
        let ws = Workspace::from_path(&tmp).expect("should create workspace");
        assert_eq!(ws.name, "moai-test-workspace");
        assert!(!ws.id.is_empty());
        assert_eq!(ws.color, 0xff6a3d);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn workspace_from_path_rejects_non_directory() {
        let tmp = std::env::temp_dir().join("moai-test-not-a-dir.txt");
        std::fs::write(&tmp, b"not a dir").unwrap();
        let result = Workspace::from_path(&tmp);
        assert!(matches!(result, Err(WorkspaceError::NotADirectory(_))));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn workspace_serializes_to_json() {
        let tmp = std::env::temp_dir().join("moai-test-serialize");
        std::fs::create_dir_all(&tmp).unwrap();
        let ws = Workspace::from_path(&tmp).unwrap();
        let json = serde_json::to_string(&ws).unwrap();
        assert!(json.contains("\"name\":\"moai-test-serialize\""));
        // 0xff6a3d = 16738877 (ACCENT_MOAI)
        let expected_color = 0xff6a3du32;
        assert_eq!(expected_color, 16_738_877);
        assert!(json.contains(&format!("\"color\":{}", expected_color)));
        std::fs::remove_dir_all(&tmp).ok();
    }
}
