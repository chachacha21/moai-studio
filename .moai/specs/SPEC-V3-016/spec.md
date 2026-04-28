---
id: SPEC-V3-016
version: 1.0.0
status: ready
created: 2026-04-29
updated: 2026-04-29
author: manager-spec
priority: medium
issue_number: TBD
depends_on:
  - SPEC-V3-004
  - SPEC-V3-006
parallel_with:
  - SPEC-V3-007
  - SPEC-V3-008
labels:
  - phase-3
  - ui
  - gpui
  - image-viewer
  - exif
  - surface
  - viewer
milestones:
  - id: MS-1
    name: Image decoding + rendering + file routing
    priority: high
    acceptance:
      - AC-IV-1
      - AC-IV-2
      - AC-IV-3
      - AC-IV-4
  - id: MS-2
    name: EXIF metadata panel + zoom controls
    priority: medium
    acceptance:
      - AC-IV-5
      - AC-IV-6
      - AC-IV-7
      - AC-IV-8
  - id: MS-3
    name: Advanced formats + cursor feedback + test completion
    priority: low
    acceptance:
      - AC-IV-9
      - AC-IV-10
      - AC-IV-11
---

# SPEC-V3-016: Image Surface Enhancement -- image decoding, EXIF metadata, zoom controls, format routing (Design v3 C-5)

## HISTORY

- 2026-04-29: Initial draft. Design v3 C-5 feature "Image Surface with zoom/pan/EXIF" 의 구현 SPEC. `crates/moai-studio-ui/src/viewer/image.rs` 의 기존 scaffold (zoom/pan state + placeholder render) 를 실제 이미지 디코딩, EXIF 추출, 확장자 라우팅, 단위 테스트로 보강. `LeafKind::Image` 는 SPEC-V3-006 에서 이미 배선 완료. (manager-spec)

---

## 1. Overview

### 1.1 Purpose

moai-studio v3 의 viewer surface 중 Image Viewer (C-5) 를 완성한다. 현재 `crates/moai-studio-ui/src/viewer/image.rs` 는 zoom/pan 상태 관리와 placeholder 렌더링만 제공하며, **실제 이미지 디코딩, EXIF 메타데이터 추출, 파일 확장자 라우팅, 단위 테스트가 모두 누락**된 상태다.

본 SPEC 은 다음을 달성한다:

1. **Image decoding**: `image` crate 을 사용하여 PNG, JPEG, GIF, WebP, BMP, ICO 포맷을 GPUI `Render` 트레잇 내에서 실제 픽셀로 디코딩하여 표시.
2. **EXIF metadata**: JPEG/PNG 파일의 카메라 정보, 촬영 일시, 해상도, 파일 크기를 추출하여 사이드 패널에 표시.
3. **Zoom controls**: 마우스 휠 외에 +/- 버튼, fit-to-view, 100% (actual size) 버튼 제공.
4. **File extension routing**: `viewer/mod.rs` 의 `resolve_event` 가 이미지 확장자를 `EventResolution::Binary` 가 아닌 `EventResolution::Image` 로 라우팅.
5. **Unit tests**: image viewer 의 핵심 로직에 대한 단위 테스트 작성 (현재 0건).

### 1.2 Existing Code

| Location | Current State | Action |
|----------|---------------|--------|
| `viewer/image.rs` | `ImageViewer` struct with zoom/pan state, `Render` impl with placeholder | Replace placeholder with actual image rendering |
| `viewer/mod.rs` | `LeafKind::Image(Entity<ImageViewer>)` variant wired in `Render for LeafKind` | No structural change needed |
| `viewer/mod.rs` `resolve_event` | Image extensions (png, jpg, jpeg, gif, bmp, ico, svg, webp) routed to `EventResolution::Binary` | Route to new `EventResolution::Image` |
| `viewer/mod.rs` `EventResolution` | `Open(SurfaceHint)` / `Binary` enum | Add `Image` variant |
| `viewer/mod.rs` `route_by_extension` | No image extension routing | Add image extensions |
| `Cargo.toml` (workspace) | No `image` or EXIF crate dependency | Add `image` + `kamadak-exif` |

### 1.3 Dependency on Existing SPECs

- **SPEC-V3-004**: `render_pane_tree<L>` generic, `LeafKind` enum, pane/tab infrastructure. No API changes required.
- **SPEC-V3-006**: `LeafKind::Image(Entity<ImageViewer>)` already added and wired in `Render for LeafKind`. `viewer/mod.rs` is the shared entry point.

### 1.4 Reference Documents

- `crates/moai-studio-ui/src/viewer/image.rs` -- existing scaffold (175 LOC).
- `crates/moai-studio-ui/src/viewer/mod.rs` -- routing + `LeafKind` + `EventResolution` + `resolve_event`.
- `.moai/specs/SPEC-V3-006/spec.md` -- viewer surface architecture, `LeafKind` design.
- `.moai/specs/SPEC-V3-004/spec.md` -- `render_pane_tree<L>` generic, `TabContainer` + `PaneTree`.
- [image crate](https://crates.io/crates/image) -- PNG/JPEG/GIF/WebP/BMP/ICO decoder.
- [kamadak-exif crate](https://crates.io/crates/kamadak-exif) -- EXIF metadata parser.

---

## 2. Background and Motivation

Design v3 C-5 는 "Image Surface with zoom/pan/EXIF" 를 명시한다. 현재 구현은 zoom/pan 상태 관리와 placeholder rendering 만 갖추고 있으며:

- **이미지 로딩 불가**: 사용자가 이미지 파일을 열어도 `resolve_event` 가 `Binary` 로 거부하여 ImageViewer entity 가 생성되지 않는다.
- **EXIF 정보 없음**: 카메라 모델, 촬영 일시, 렌즈 정보 등 사진 메타데이터를 볼 수 없다.
- **Zoom 컨트롤 없음**: 마우스 휠만 지원. +/- 버튼, fit-to-view, 100% 버튼이 없다.
- **테스트 없음**: image viewer 영역의 단위 테스트가 0건이다.

본 SPEC 은 이 네 가지 갭을 메운다.

---

## 3. Goals / Non-Goals

### 3.1 Goals

- G1. `ImageViewer` 가 `image` crate 으로 PNG/JPEG/GIF/WebP/BMP/ICO 파일을 디코딩하여 GPUI element 로 렌더링한다.
- G2. `resolve_event` 가 이미지 확장자 (.png, .jpg, .jpeg, .gif, .webp, .bmp, .ico) 를 `EventResolution::Image` 로 라우팅하고, `RootView` 가 `LeafKind::Image(Entity<ImageViewer>)` leaf 를 마운트한다.
- G3. EXIF metadata panel 이 JPEG/PNG 파일의 카메라 모델, 촬영 일시, 해상도, 파일 크기를 표시한다.
- G4. Zoom toolbar (+/-, fit-to-view, 100%) 가 마우스 휠과 함께 동작한다.
- G5. Pan (click-drag) 시 커서가 `grabbing` 으로 변경된다.
- G6. SPEC-V3-002 (terminal core), SPEC-V3-003 (panes/tabs), SPEC-V3-004 (render layer), SPEC-V3-006 (viewer mod.rs structure) 의 공개 API 를 변경하지 않는다.
- G7. Minimum 15 unit tests for image viewer core logic (decoding, zoom math, EXIF parsing, routing).

### 3.2 Non-Goals

- N1. **SVG rendering** -- 별도 SPEC (SPEC-V3-016-SVG) 에서 resvg 기반으로 다룬다. 본 SPEC 은 SVG 확장자를 라우팅만 하고 "SVG not yet supported" placeholder 표시.
- N2. **Animated GIF playback** -- 본 SPEC v1 은 GIF 의 첫 프레임만 표시. 애니메이션 재생은 MS-3 선택 범위 또는 별도 SPEC.
- N3. **Image editing (crop, rotate, filter)** -- viewer 전용. 편집 기능은 별도 SPEC.
- N4. **Color picker / eyedropper** -- 별도 SPEC.
- N5. **Thumbnail grid / gallery view** -- 별도 SPEC.
- N6. **HEIF/HEIC, TIFF, AVIF support** -- 본 SPEC v1 은 6 포맷 (PNG/JPEG/GIF/WebP/BMP/ICO) 만. 추가 포맷은 별도 SPEC.
- N7. **Image diff / comparison (side-by-side)** -- v2 의 Vision SSIM 기능 이관이나 본 SPEC 비대상.
- N8. **Clipboard paste image** -- 별도 SPEC.
- N9. **Drag-and-drop file onto viewer** -- 별도 SPEC.
- N10. **Persistence schema major version bump** -- `LeafState::Image { path }` variant 추가만 (minor extension).

---

## 4. User Stories

- **US-IV1**: 사용자가 파일 탐색기에서 `screenshot.png` 를 더블클릭한다 -> 활성 탭의 leaf 가 ImageViewer 로 교체되어 실제 이미지가 가시. placeholder 가 아님.
- **US-IV2**: 사용자가 이미지 위에서 마우스 휠을 위로 굴린다 -> 이미지가 10% 단위로 확대된다. zoom overlay 에 "150%" 가 표시된다.
- **US-IV3**: 사용자가 이미지 위에서 마우스 왼쪽 버튼을 누른 채 드래그한다 -> 이미지가 패닝되고, 커서가 grabbing 손 모양으로 변경된다.
- **US-IV4**: 사용자가 zoom toolbar 의 "100%" 버튼을 클릭한다 -> 이미지가 실제 픽셀 크기 (1:1) 로 표시된다.
- **US-IV5**: 사용자가 zoom toolbar 의 "Fit" 버튼을 클릭한다 -> 이미지가 뷰어 영역에 맞게 축소/확대되어 전체가 가시.
- **US-IV6**: 사용자가 JPEG 사진을 열고 화면 우측 EXIF 패널을 본다 -> 카메라 모델 "Canon EOS R5", 촬영 일시 "2026-04-15 14:32:00", 해상도 "8192x5464", 파일 크기 "24.3 MB" 가 표시된다.
- **US-IV7**: 사용자가 `.gif` 파일을 연다 -> 첫 프레임이 표시되고, zoom/pan 이 정상 동작한다.

---

## 5. Requirements (EARS)

### RG-IV-1 -- Image Decoding and Rendering

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-IV-001 | Ubiquitous | The system **shall** provide an `ImageData` struct containing decoded pixel buffer (`Vec<u8>` RGBA), width, height, and source path in `ImageViewer`. | `ImageViewer` holds `Option<ImageData>` with decoded RGBA pixels, dimensions, and path. |
| REQ-IV-002 | Event-Driven | **When** `ImageViewer::load_image(path)` is called with a valid image file path, the system **shall** read the file bytes, decode via `image::io::Reader`, and store the result in `ImageData`. | When `load_image` is called, the system decodes the file and stores `ImageData`. |
| REQ-IV-003 | Event-Driven | **When** `ImageViewer::load_image(path)` is called with a corrupt or unsupported file, the system **shall** set an error state and render "Failed to load image: {error}" instead of panicking. | When loading fails, the system shows an error message without panicking. |
| REQ-IV-004 | Ubiquitous | The system **shall** support decoding of PNG, JPEG, GIF (first frame), WebP, BMP, and ICO formats. | The system supports 6 image formats. |
| REQ-IV-005 | State-Driven | **While** `ImageViewer.image_data` is `Some`, the system **shall** render the decoded image at the current zoom level with pan offset applied. | While image data exists, render it at current zoom/pan state. |
| REQ-IV-006 | State-Driven | **While** `ImageViewer.image_data` is `None`, the system **shall** render the existing placeholder ("Image Viewer (C-5)"). | While no image is loaded, render the placeholder. |

### RG-IV-2 -- File Extension Routing

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-IV-010 | Ubiquitous | The system **shall** define an `EventResolution::Image` variant in `viewer/mod.rs` alongside the existing `Open` and `Binary` variants. | Add `Image` variant to `EventResolution`. |
| REQ-IV-011 | Event-Driven | **When** `resolve_event` receives an `OpenFileEvent` with an image extension (.png, .jpg, .jpeg, .gif, .webp, .bmp, .ico), the system **shall** return `EventResolution::Image` instead of `EventResolution::Binary`. | Image extensions route to `Image`, not `Binary`. |
| REQ-IV-012 | Event-Driven | **When** `resolve_event` returns `EventResolution::Image`, the caller (RootView) **shall** create an `Entity<ImageViewer>`, call `load_image(path)`, and set the active leaf to `LeafKind::Image(entity)`. | RootView creates ImageViewer entity and loads the image on `Image` resolution. |
| REQ-IV-013 | Event-Driven | **When** `resolve_event` receives an `.svg` extension, the system **shall** return `EventResolution::Image`. The ImageViewer **shall** render "SVG preview not yet supported" placeholder. | SVG routes to ImageViewer but shows unsupported placeholder. |
| REQ-IV-014 | Ubiquitous | The system **shall** add `Image` to `SurfaceHint` enum (or use a separate routing path) so that `route_by_extension` returns an image-appropriate hint for image extensions. | Surface routing handles image extensions. |

### RG-IV-3 -- Zoom Controls

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-IV-020 | Ubiquitous | The system **shall** provide a zoom toolbar rendered below the image area with four buttons: zoom-in (+), zoom-out (-), fit-to-view, and 100% (actual size). | Zoom toolbar with 4 buttons. |
| REQ-IV-021 | Event-Driven | **When** the user clicks the zoom-in button, the system **shall** increase zoom by 10% (capped at 10x). | Zoom-in button increases zoom by 10%. |
| REQ-IV-022 | Event-Driven | **When** the user clicks the zoom-out button, the system **shall** decrease zoom by 10% (floored at 0.1x). | Zoom-out button decreases zoom by 10%. |
| REQ-IV-023 | Event-Driven | **When** the user clicks the fit-to-view button, the system **shall** calculate the zoom level that makes the image fit entirely within the viewer bounds, centered, and reset pan offsets to (0, 0). | Fit-to-view calculates optimal zoom and resets pan. |
| REQ-IV-024 | Event-Driven | **When** the user clicks the 100% button, the system **shall** set zoom to 1.0 and reset pan offsets to (0, 0). | 100% button sets zoom to 1.0 and resets pan. |
| REQ-IV-025 | Ubiquitous | The system **shall** display the current zoom percentage as a text overlay in the top-left corner of the image area (existing behavior preserved). | Zoom percentage overlay preserved. |

### RG-IV-4 -- Pan Cursor Feedback

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-IV-030 | State-Driven | **While** the user is dragging (is_dragging == true), the system **shall** set the cursor style to `grabbing`. | While dragging, cursor is `grabbing`. |
| REQ-IV-031 | State-Driven | **While** the mouse is over the image area and is_dragging == false, the system **shall** set the cursor style to `grab`. | While hovering (not dragging), cursor is `grab`. |

### RG-IV-5 -- EXIF Metadata Panel

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-IV-040 | Ubiquitous | The system **shall** define an `ExifData` struct containing: camera_make, camera_model, datetime_original, exposure_time, f_number, iso, focal_length, image_width, image_height, file_size. | `ExifData` struct with standard photo metadata fields. |
| REQ-IV-041 | Event-Driven | **When** `ImageViewer::load_image(path)` successfully loads a JPEG or PNG file, the system **shall** attempt to extract EXIF data via `kamadak-exif` and store it in `Option<ExifData>`. | On JPEG/PNG load, extract EXIF data. |
| REQ-IV-042 | State-Driven | **While** `ExifData` is `Some`, the system **shall** render a toggleable metadata panel on the right side of the viewer showing all available EXIF fields. Non-available fields show "--". | When EXIF exists, show metadata panel with all fields. |
| REQ-IV-043 | Event-Driven | **When** the user presses `Cmd+I` (macOS) / `Ctrl+I` (Linux) while the image viewer is active, the system **shall** toggle the EXIF panel visibility. | `Cmd+I` / `Ctrl+I` toggles EXIF panel. |
| REQ-IV-044 | State-Driven | **While** the EXIF panel is visible, the image rendering area **shall** shrink to accommodate the panel width (estimated 280px). | EXIF panel takes space from image area. |
| REQ-IV-045 | Unwanted | The system **shall not** display EXIF panel by default. It starts hidden. | EXIF panel default is hidden. |

### RG-IV-6 -- Frozen Zone Carry

| REQ ID | Pattern | Requirement (Korean) | English |
|--------|---------|---------------------|---------|
| REQ-IV-050 | Ubiquitous | The system **shall not** modify any file in `crates/moai-studio-terminal/**`. SPEC-V3-002 carry. | Terminal core unchanged. |
| REQ-IV-051 | Ubiquitous | The system **shall not** modify the public API of `crates/moai-studio-ui/src/panes/{tree.rs, constraints.rs, focus.rs, splitter.rs, divider.rs}`. | Panes API unchanged. |
| REQ-IV-052 | Ubiquitous | The system **shall not** modify the public API of `crates/moai-studio-ui/src/tabs/{container.rs, bar.rs, keys.rs}`. Existing method signatures preserved. `LeafKind::Image` enum variant addition is enum extension only (already done in SPEC-V3-006). | Tabs API unchanged. |
| REQ-IV-053 | Ubiquitous | The system **shall not** modify the `moai-studio/panes-v1` schema existing fields in `crates/moai-studio-workspace/src/persistence.rs`. `LeafState::Image { path }` variant addition is a minor extension. | Persistence schema backward compatible. |

---

## 6. Non-Functional Requirements

### 6.1 Performance

- NFR-IV-1. Image decode + first render for a 10 MB JPEG within 500ms on Apple Silicon M1.
- NFR-IV-2. Zoom/pan interaction at 60fps (no frame drops during mouse wheel zoom or drag).
- NFR-IV-3. EXIF extraction within 50ms (metadata only, no re-decode).
- NFR-IV-4. Memory usage for a single 50 MP image: decoded RGBA buffer + GPUI texture only. No duplicate copies.

### 6.2 Memory

- NFR-IV-5. Decoded image RGBA buffer is held as `Vec<u8>` in `ImageData`. When a new image is loaded, the previous buffer is dropped before decoding the new one.
- NFR-IV-6. Maximum supported image dimension: 16384 x 16384 pixels (2.1 billion pixels RGBA = ~8 GB theoretical max; practical limit enforced at 100 MP).

### 6.3 Reliability

- NFR-IV-7. Corrupt image files produce "Failed to load image" message, never panic.
- NFR-IV-8. Loading a non-image file (wrong extension, magic mismatch) produces a clear error message.

### 6.4 Security

- NFR-IV-9. Image decoding uses the `image` crate which has memory-safe Rust implementation. No unsafe image parsing.
- NFR-IV-10. File size limit: images exceeding 200 MB are rejected (reuse `MAX_FILE_BYTES` from `viewer/mod.rs`).

### 6.5 Compatibility

- NFR-IV-11. macOS 14+ and Ubuntu 22.04+ both supported. No platform-specific image handling.
- NFR-IV-12. `image` crate version pinned to latest stable in workspace `Cargo.toml`.

---

## 7. Architecture

### 7.1 Module Topology

```
crates/moai-studio-ui/src/viewer/
  image.rs          -- ImageViewer struct + Render impl (MODIFIED)
  image_data.rs     -- ImageData + image decoding logic (NEW)
  exif.rs           -- ExifData struct + kamadak-exif extraction (NEW)
  mod.rs            -- LeafKind, EventResolution, resolve_event (MODIFIED)
```

### 7.2 Data Flow

```
OpenFileEvent { path: "photo.jpg" }
    |
    v
resolve_event -> EventResolution::Image
    |
    v
RootView creates Entity<ImageViewer>
    |
    v
ImageViewer::load_image("photo.jpg")
    |
    +-- image::io::Reader::open("photo.jpg") -> DynamicImage -> RGBA buffer
    |       |
    |       v
    |   ImageData { pixels: Vec<u8>, width, height, path }
    |
    +-- exif::Reader::new().read_from_path("photo.jpg") -> Option<ExifData>
    |
    v
cx.notify() -> Render
    |
    +-- Image area (zoomed/panned RGBA)
    +-- Zoom toolbar (+/-, Fit, 100%)
    +-- Zoom overlay ("150%")
    +-- EXIF panel (if visible)
```

### 7.3 Component Layout

```
+------------------------------------------------------+
| Image Viewer Surface                                 |
| +--------------------------------------------------+ |
| | Image Area (zoomed/panned)          | EXIF Panel  | |
| |                                     | (280px)     | |
| |  [Zoom: 150%]                       |             | |
| |                                     | Camera:     | |
| |                                     | Canon EOS   | |
| |                                     |             | |
| |                                     | Date:       | |
| |                                     | 2026-04-15  | |
| |                                     |             | |
| |                                     | Size:       | |
| |                                     | 8192x5464   | |
| |                                     |             | |
| |                                     | File:       | |
| |                                     | 24.3 MB     | |
| +--------------------------------------------------+ |
| [Zoom Toolbar: - | 150% | + | Fit | 100%]          |
+------------------------------------------------------+
```

### 7.4 Key Structs

```rust
/// Decoded image data stored in ImageViewer.
pub struct ImageData {
    pub pixels: Vec<u8>,     // RGBA8
    pub width: u32,
    pub height: u32,
    pub path: PathBuf,
    pub file_size: u64,
}

/// EXIF metadata extracted from JPEG/PNG.
pub struct ExifData {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub datetime_original: Option<String>,
    pub exposure_time: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<String>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
}
```

---

## 8. Milestones

### MS-1: Image decoding + rendering + file routing (Priority: High)

- **Scope**: Add `image` crate dependency. Create `image_data.rs` with decode logic. Modify `ImageViewer` to load and render actual images. Fix `resolve_event` to route image extensions to `EventResolution::Image`. Add `EventResolution::Image` variant. Unit tests for routing + decode logic.
- **Requirements**: REQ-IV-001 ~ REQ-IV-006, REQ-IV-010 ~ REQ-IV-014, REQ-IV-050 ~ REQ-IV-053.
- **Demonstrable state**: User opens a PNG/JPEG file -> image is displayed in the viewer (not placeholder). Zoom via mouse wheel works. Routing no longer rejects image files as binary.

### MS-2: EXIF metadata panel + zoom controls (Priority: Medium)

- **Scope**: Add `kamadak-exif` dependency. Create `exif.rs`. Implement EXIF extraction in `load_image`. Add EXIF panel rendering (toggleable, `Cmd+I`). Add zoom toolbar buttons (+/-, fit-to-view, 100%). Update `Render` to include toolbar and conditional EXIF panel.
- **Requirements**: REQ-IV-020 ~ REQ-IV-025, REQ-IV-040 ~ REQ-IV-045.
- **Demonstrable state**: User opens a JPEG photo -> EXIF panel shows camera/date/size. Zoom toolbar buttons work. Fit-to-view correctly scales image to bounds.

### MS-3: Advanced formats + cursor feedback + test completion (Priority: Low)

- **Scope**: Cursor feedback (grab/grabbing). GIF first-frame-only handling. SVG placeholder ("not yet supported"). ICO format verification. Complete test suite (target 15+ unit tests). Edge case handling (very large images, corrupt files).
- **Requirements**: REQ-IV-030 ~ REQ-IV-031, REQ-IV-003, REQ-IV-013 (SVG placeholder).
- **Demonstrable state**: Cursor changes during pan. GIF files show first frame. SVG shows placeholder. All 15+ unit tests pass.

---

## 9. File Layout

### 9.1 New Files

- `crates/moai-studio-ui/src/viewer/image_data.rs` -- `ImageData` struct + `decode_image(path) -> Result<ImageData, ImageError>` function.
- `crates/moai-studio-ui/src/viewer/exif.rs` -- `ExifData` struct + `extract_exif(path) -> Option<ExifData>` function.

### 9.2 Modified Files

- `crates/moai-studio-ui/src/viewer/image.rs` -- Add `ImageData`, `ExifData` fields to `ImageViewer`. Add `load_image(path)` method. Replace placeholder render with actual image render + toolbar + EXIF panel. Add cursor feedback.
- `crates/moai-studio-ui/src/viewer/mod.rs` -- Add `EventResolution::Image` variant. Update `resolve_event` to route image extensions to `Image`. Update `LeafKind::Render` match arm for `Image` (already wired, may need `load_image` trigger). Add `image_data` and `exif` module declarations.
- `crates/moai-studio-ui/Cargo.toml` -- Add `image` (workspace), `kamadak-exif` (workspace) dependencies.
- `Cargo.toml` (workspace root) -- Add `image` and `kamadak-exif` to `[workspace.dependencies]`.
- `crates/moai-studio-ui/src/lib.rs` -- Update `handle_open_file` / `resolve_event` integration to handle `EventResolution::Image` by creating `Entity<ImageViewer>` and calling `load_image`.

### 9.3 Frozen (No Change)

- `crates/moai-studio-terminal/**` (SPEC-V3-002 carry).
- `crates/moai-studio-ui/src/panes/{tree.rs, constraints.rs, focus.rs, splitter.rs, divider.rs}` public API.
- `crates/moai-studio-ui/src/tabs/{container.rs, bar.rs, keys.rs}` public API.
- `crates/moai-studio-workspace/src/persistence.rs` existing schema fields.

---

## 10. Acceptance Criteria

| AC ID | Req Group | MS | Given | When | Then | Verification |
|-------|-----------|-----|-------|------|------|-------------|
| AC-IV-1 | RG-IV-2 (REQ-IV-010~012) | MS-1 | `cargo run -p moai-studio-app` running | User opens `test.png` from file explorer | `resolve_event` returns `EventResolution::Image`. RootView creates `Entity<ImageViewer>`. `load_image` decodes PNG. Leaf is `LeafKind::Image(...)`. Image pixels visible (not placeholder). | Unit test (routing) + manual smoke |
| AC-IV-2 | RG-IV-1 (REQ-IV-001~004) | MS-1 | `ImageViewer` instance exists | `load_image("test.jpg")` called with valid JPEG | `ImageData.pixels.len() == width * height * 4`. `image_data.width > 0`. `image_data.height > 0`. Render shows image (not placeholder). | Unit test (decode logic) |
| AC-IV-3 | RG-IV-1 (REQ-IV-003) | MS-1 | `ImageViewer` instance exists | `load_image("corrupt.jpg")` called with invalid image data | Error state set. Render shows "Failed to load image: {error}". No panic. | Unit test (error handling) |
| AC-IV-4 | RG-IV-2 (REQ-IV-011) | MS-1 | `resolve_event` function available | Call with `OpenFileEvent { path: "photo.webp", surface_hint: None }` | Returns `EventResolution::Image` (not `Binary`). | Unit test |
| AC-IV-5 | RG-IV-3 (REQ-IV-020~024) | MS-2 | Image loaded in viewer | User clicks fit-to-view button | Zoom recalculated to fit image within viewer bounds. Pan reset to (0, 0). Image entirely visible. | Unit test (fit-to-view math) + manual |
| AC-IV-6 | RG-IV-3 (REQ-IV-024) | MS-2 | Image loaded, zoom at 200% | User clicks 100% button | Zoom set to 1.0. Pan reset to (0, 0). Overlay shows "100%". | Unit test + manual |
| AC-IV-7 | RG-IV-5 (REQ-IV-040~043) | MS-2 | JPEG with EXIF loaded | User presses `Cmd+I` | EXIF panel becomes visible on right side (280px). Camera model, date, dimensions, file size displayed. | Unit test (ExifData extraction) + manual |
| AC-IV-8 | RG-IV-5 (REQ-IV-045) | MS-2 | Image viewer opened | Viewer initially rendered | EXIF panel is hidden by default. Image uses full width. | Manual / render state check |
| AC-IV-9 | RG-IV-4 (REQ-IV-030~031) | MS-3 | Image loaded, mouse over image area | Mouse hover (no drag) | Cursor style is `grab`. Mouse down + drag: cursor is `grabbing`. Mouse up: cursor returns to `grab`. | Manual / visual check |
| AC-IV-10 | RG-IV-2 (REQ-IV-013) | MS-3 | `resolve_event` available | Call with `OpenFileEvent { path: "icon.svg", surface_hint: None }` | Returns `EventResolution::Image`. ImageViewer renders "SVG preview not yet supported" placeholder. | Unit test |
| AC-IV-11 | All | MS-3 | Implementation complete | `cargo test -p moai-studio-ui --lib viewer::image` | 15+ unit tests pass: routing (6 extensions), decode (valid/invalid), zoom math (fit-to-view, 100%, clamp), EXIF extraction (JPEG with/without EXIF), pan state, cursor state. | CI gate / cargo test |

---

## 11. Dependencies and Constraints

### 11.1 External Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `image` | Latest stable (workspace) | PNG/JPEG/GIF/WebP/BMP/ICO decoding |
| `kamadak-exif` | Latest stable (workspace) | EXIF metadata extraction from JPEG/TIFF/PNG |
| `gpui` | 0.2.2 (carry) | UI framework, unchanged |

### 11.2 Internal Dependencies

- `crates/moai-studio-terminal` (SPEC-V3-002) -- no change (carry).
- `crates/moai-studio-ui::panes` (SPEC-V3-003) -- no API change.
- `crates/moai-studio-ui::tabs` (SPEC-V3-003) -- no API change.
- `crates/moai-studio-ui::viewer::mod` (SPEC-V3-006) -- modification within existing module (add variant, update routing).

### 11.3 System Constraints

- Rust stable 1.93+ (SPEC-V3-001 carry).
- macOS 14+ + Ubuntu 22.04+.
- 200 MB file size limit (reuse `MAX_FILE_BYTES`).
- 100 MP pixel limit (16384 x 16384 max dimension).

### 11.4 Git Constraints

- Implementation on `feature/SPEC-V3-016-image-viewer` branch.
- No direct commits to `main` (CLAUDE.local.md Section 1).
- Each MS is a squash merge candidate.

---

## 12. Risks and Mitigations

| ID | Risk | Impact | Mitigation |
|----|------|--------|-----------|
| R-IV1 | `image` crate decode performance on very large images (>50 MP) | UI freeze during decode | Decode in background task (cx.spawn). Show "Loading..." placeholder during decode. MS-1 starts with synchronous decode for simplicity. |
| R-IV2 | GPUI `img()` element may not support raw RGBA buffer directly | MS-1 blocked | Verify GPUI image rendering API before MS-1 implementation. Fallback: write RGBA to temporary PNG and use GPUI's image loading. |
| R-IV3 | `kamadak-exif` may not parse all EXIF variants | Missing metadata fields | Graceful degradation: missing fields show "--" in panel. |
| R-IV4 | Animated GIF memory overhead if all frames decoded | Memory spike | MS-1/MS-2 decode first frame only (`image::DynamicImage` default). Animation deferred. |
| R-IV5 | WebP decode requires `image` feature flag `webp` | Build failure if feature not enabled | Verify `image` crate features in Cargo.toml. Enable `webp` feature explicitly. |
| R-IV6 | Image routing change may break existing binary detection logic | Regression in binary file handling | Existing `is_binary` tests must continue to pass. New routing only changes `resolve_event`, not `is_binary`. |

---

## 13. Reference Documents

### 13.1 Internal

- `crates/moai-studio-ui/src/viewer/image.rs` -- existing scaffold (175 LOC).
- `crates/moai-studio-ui/src/viewer/mod.rs` -- routing, `LeafKind`, `EventResolution`.
- `crates/moai-studio-ui/src/viewer/markdown.rs` -- reference viewer pattern (Entity + Render).
- `.moai/specs/SPEC-V3-006/spec.md` -- viewer surface architecture.
- `.moai/specs/SPEC-V3-004/spec.md` -- `render_pane_tree<L>` generic.
- `crates/moai-studio-ui/src/lib.rs` -- RootView handle_open_file integration point.

### 13.2 External

- [image crate documentation](https://docs.rs/image/latest/image/) -- decoding API reference.
- [kamadak-exif crate](https://docs.rs/kamadak-exif/latest/exif/) -- EXIF parsing API.
- [GPUI image rendering](https://www.gpui.rs/) -- GPUI `img()` element usage.

---

## 14. Exclusions (What NOT to Build)

- E1. **SVG rendering** -- resvg integration is out of scope. SVG files route to ImageViewer but show placeholder.
- E2. **Animated GIF playback** -- First frame only. Animation requires frame timing, decoder loop, separate SPEC.
- E3. **Image editing** -- Crop, rotate, filter, draw are not viewer features.
- E4. **Color picker / eyedropper** -- Separate SPEC.
- E5. **Thumbnail grid / gallery view** -- Separate SPEC.
- E6. **HEIF/HEIC, TIFF, AVIF** -- Beyond 6 supported formats. Separate SPEC.
- E7. **Image diff / comparison** -- v2 Vision SSIM migration. Separate SPEC.
- E8. **Clipboard paste** -- Separate SPEC.
- E9. **Drag-and-drop file onto viewer** -- Separate SPEC.
- E10. **Terminal core changes** -- RG-IV-6 carry (REQ-IV-050).
- E11. **Persistence schema major version bump** -- Minor extension only (`LeafState::Image { path }`).
- E12. **New design tokens** -- Reuse existing `crate::design::tokens`.

---

## 15. Terminology

- **Image Surface**: moai-studio v3 viewer surface for image file display. Design v3 C-5 feature.
- **ImageData**: Decoded RGBA pixel buffer with dimensions and source path. Stored in `ImageViewer`.
- **ExifData**: Extracted photo metadata (camera, date, exposure, etc.) from JPEG/PNG files.
- **Fit-to-view**: Zoom mode that scales the image to fit entirely within the viewer area, centered.
- **100% (actual size)**: Zoom level 1.0 where 1 image pixel = 1 screen pixel.
- **EventResolution::Image**: New routing variant for image file extensions, directing to ImageViewer entity.

---

## 16. Open Decisions

| ID | Decision | Default / Recommendation | Decision Point |
|----|----------|-------------------------|----------------|
| OD-IV1 | GPUI image rendering approach (raw RGBA vs PNG re-encode) | (a) Verify GPUI `img()` supports raw buffer; fallback to PNG re-encode | MS-1 start |
| OD-IV2 | Synchronous vs asynchronous image decode | (a) Synchronous for MS-1 (simple); migrate to `cx.spawn` async if UI freezes observed | MS-1 (revisit if R-IV1 materializes) |
| OD-IV3 | `image` crate version pin | Latest stable from crates.io | MS-1 start |
| OD-IV4 | EXIF panel width | 280px fixed (matches existing sidebar patterns) | MS-2 start |
| OD-IV5 | `kamadak-exif` vs `rexif` | (a) kamadak-exif (more maintained, pure Rust) | MS-2 start |

---

Created: 2026-04-29
Branch (output): `feature/SPEC-V3-016-image-viewer`
Next output: plan.md (Milestone x Task table)
