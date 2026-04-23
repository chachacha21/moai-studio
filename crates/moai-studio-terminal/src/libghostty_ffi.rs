//! @MX:ANCHOR(libghostty-ffi-boundary)
//! @MX:REASON: libghostty-vt FFI wrapper — upstream churn 격리 경계.
//!   이 파일만 libghostty-vt 를 직접 참조한다.
//!   외부 모듈은 여기서 re-export 된 타입만 사용해야 한다 (SPEC-V3-002 RG-V3-002-1).
//!
//! 주요 FFI 심볼:
//!   Terminal, RenderState (libghostty-vt pinned SHA dfac6f3e)

use libghostty_vt::{RenderState, Terminal, TerminalOptions};
use thiserror::Error;

/// FFI 계층 에러 타입
#[derive(Debug, Error)]
pub enum FfiError {
    #[error("Terminal 초기화 실패: {0}")]
    InitError(String),
}

/// 불투명 Terminal 핸들 (libghostty_ffi 계층 내부에서만 사용).
///
/// libghostty-vt 의 `Terminal` 은 `!Send + !Sync` 이므로
/// 항상 단일 스레드(PTY worker) 에서만 접근한다.
pub struct TerminalHandle {
    /// 내부 터미널 — libghostty-vt pinned SHA
    pub(crate) inner: Terminal<'static, 'static>,
    /// 렌더 상태 — update() 로 갱신
    pub(crate) render: RenderState<'static>,
}

// @MX:WARN(unsafe-send-single-owner-invariant)
// @MX:REASON: libghostty-vt 의 Terminal 은 !Send + !Sync 인데 tokio::spawn 으로 이동시키려면 Send 가 필요하다.
//   안전성은 "TerminalHandle 은 PTY worker thread 단독 소유" 라는 invariant 에 의존한다.
//   invariant 는 worker.rs 의 PtyWorker::run 이 block_in_place 로 async boundary 를 격리하고,
//   TerminalHandle 참조를 외부로 노출하지 않는 것으로 보장된다.
//   invariant 가 깨지면 data race 로 UB 발생 — 외부에서 핸들 복제/전송 금지.
unsafe impl Send for TerminalHandle {}

/// RenderState 스냅샷 — GPUI render thread 가 소비하는 Grid 정보.
pub struct RenderSnapshot {
    /// 커서 위치 (row, col) — 0-indexed
    pub cursor_row: u16,
    pub cursor_col: u16,
    /// 첫 번째 행 텍스트 (ASCII 기준)
    pub row0_text: String,
    /// 첫 번째 행 비어있지 않은 셀 수
    pub row0_cell_count: usize,
}

/// Terminal 을 생성한다 (cols, rows 기준).
///
/// AC-T-8(b): 테스트에서 호출하는 진입점.
pub fn new_terminal(cols: u16, rows: u16) -> Result<TerminalHandle, FfiError> {
    // @MX:NOTE(max-scrollback-default): 1000 행은 Phase 2 기본값 — Ghostty/Zed 기본 (10,000) 보다 보수적.
    //   Phase 2.5 scrollback UI 구현 시 config 값으로 이관 예정 (SPEC-V3-002 §6 Exclusions).
    let opts = TerminalOptions {
        cols,
        rows,
        max_scrollback: 1000,
    };
    let inner = Terminal::new(opts).map_err(|e| FfiError::InitError(e.to_string()))?;
    let render = RenderState::new().map_err(|e| FfiError::InitError(e.to_string()))?;
    Ok(TerminalHandle { inner, render })
}

/// PTY 에서 읽은 바이트를 VT parser 에 주입한다.
///
/// 내부적으로 `Terminal::vt_write()` 를 호출한다.
pub fn feed(handle: &mut TerminalHandle, data: &[u8]) {
    handle.inner.vt_write(data);
    // render 상태 갱신
    let _ = handle.render.update(&handle.inner);
}

/// 현재 RenderState 스냅샷을 반환한다.
///
/// GPUI render thread 가 소비하는 Grid<Cell> 을 포함한다.
pub fn render_state(handle: &TerminalHandle) -> RenderSnapshot {
    // 커서 위치 추출 (실패 시 0,0)
    let cursor_col = handle.inner.cursor_x().unwrap_or(0);
    let cursor_row = handle.inner.cursor_y().unwrap_or(0);

    // @MX:TODO(render-state-row-iter): row0_text / row0_cell_count 는 현재 stub.
    //   RenderState 의 row iterator API 로 Cell::text() 수집 필요. T3 후속 또는 SPEC-V3-003 에서 연동.
    //   현재 상태에서는 테스트 (libghostty_api_compat) 가 커서 위치만 검증한다.
    RenderSnapshot {
        cursor_row,
        cursor_col,
        row0_text: String::new(),
        row0_cell_count: 0,
    }
}

/// 터미널 크기를 변경한다.
pub fn resize(handle: &mut TerminalHandle, cols: u16, rows: u16) -> Result<(), FfiError> {
    handle
        .inner
        .resize(cols, rows, 0, 0)
        .map_err(|e| FfiError::InitError(e.to_string()))
}
