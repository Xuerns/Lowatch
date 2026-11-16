use tauri::{LogicalSize, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::AppState;
use crate::db;
use crate::error::AppError;

/// Lebar jendela Focus Mode dalam satuan logical pixel (sesuai PRD 5.4 & 10).
pub const FOCUS_WIDTH_LOGICAL: f64 = 280.0;
/// Tinggi jendela Focus Mode dalam satuan logical pixel (cukup untuk dial stopwatch bulat).
pub const FOCUS_HEIGHT_LOGICAL: f64 = 240.0;
/// Margin dari tepi kanan dan atas work area monitor dalam satuan logical pixel (sesuai PRD 5.4).
pub const FOCUS_MARGIN_LOGICAL: f64 = 16.0;

/// Ukuran minimum jendela utama dalam mode normal (sesuai `tauri.conf.json` & PRD 10).
pub const NORMAL_MIN_WIDTH_LOGICAL: f64 = 360.0;
/// Tinggi minimum jendela utama dalam mode normal (sesuai `tauri.conf.json` & PRD 10).
pub const NORMAL_MIN_HEIGHT_LOGICAL: f64 = 400.0;

/// Geometri jendela mode normal yang disimpan sebelum beralih ke Focus Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedWindowGeometry {
    pub size: PhysicalSize<u32>,
    pub position: PhysicalPosition<i32>,
    pub is_maximized: bool,
    pub decorations: bool,
    pub resizable: bool,
}

/// State Focus Mode di memori (tidak dipersist ke DB; saat restart selalu mulai di mode normal).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FocusState {
    pub saved_geometry: Option<SavedWindowGeometry>,
    pub focused_stopwatch_id: Option<i64>,
}

/// Representasi area kerja (work area) sebuah monitor dalam satuan physical pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorWorkArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Menghitung koordinat physical `(x, y)` pojok kanan atas di dalam `work_area` monitor,
/// dengan memperhatikan `scale_factor` DPI dan margin logical.
pub fn compute_top_right_position(
    work_area: MonitorWorkArea,
    scale_factor: f64,
    widget_width_logical: f64,
    margin_logical: f64,
) -> PhysicalPosition<i32> {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };

    let widget_width_phys = (widget_width_logical * scale).round() as i64;
    let margin_phys = (margin_logical * scale).round() as i64;

    let area_x = i64::from(work_area.x);
    let area_y = i64::from(work_area.y);
    let area_w = i64::from(work_area.width);

    let right_edge = area_x.saturating_add(area_w);
    let raw_x = right_edge
        .saturating_sub(widget_width_phys)
        .saturating_sub(margin_phys);

    // Clamp agar tidak melewati batas kiri monitor jika layar sangat sempit
    let clamped_x = raw_x.max(area_x);
    let target_y = area_y.saturating_add(margin_phys);

    let final_x = i32::try_from(clamped_x).unwrap_or(work_area.x);
    let final_y = i32::try_from(target_y).unwrap_or(work_area.y);

    PhysicalPosition::new(final_x, final_y)
}

/// Memvalidasi keberadaan stopwatch `id` di database dan memperbarui `FocusState`.
/// Hanya memanggil `capture_geometry` apabila stopwatch ditemukan dan aplikasi belum berada di Focus Mode.
pub fn prepare_enter_focus_state<F>(
    state: &AppState,
    id: i64,
    capture_geometry: F,
) -> Result<(), AppError>
where
    F: FnOnce() -> Result<SavedWindowGeometry, AppError>,
{
    {
        let conn = state.lock_db()?;
        let _ = db::get_stopwatch(&conn, id)?;
    }

    let needs_capture = {
        let focus = state.lock_focus()?;
        focus.saved_geometry.is_none()
    };

    let captured = if needs_capture {
        Some(capture_geometry()?)
    } else {
        None
    };

    let mut focus = state.lock_focus()?;
    if focus.saved_geometry.is_none() {
        focus.saved_geometry = captured;
    }
    focus.focused_stopwatch_id = Some(id);
    Ok(())
}

/// Membersihkan state Focus Mode dan mengembalikan geometri jendela normal yang tersimpan (jika ada).
pub fn clear_focus_state(state: &AppState) -> Result<Option<SavedWindowGeometry>, AppError> {
    let mut focus = state.lock_focus()?;
    focus.focused_stopwatch_id = None;
    Ok(focus.saved_geometry.take())
}

fn capture_window_geometry(window: &WebviewWindow) -> Result<SavedWindowGeometry, AppError> {
    let is_maximized = window.is_maximized()?;
    let decorations = window.is_decorated()?;
    let resizable = window.is_resizable()?;
    if is_maximized {
        window.unmaximize()?;
    }
    let size = window.outer_size()?;
    let position = window.outer_position()?;

    Ok(SavedWindowGeometry {
        size,
        position,
        is_maximized,
        decorations,
        resizable,
    })
}

/// Mengubah jendela utama ke Focus Mode (`280 × 240` logical, pojok kanan atas work area, always-on-top, tanpa dekorasi).
pub fn enter_focus_mode(
    window: &WebviewWindow,
    state: &AppState,
    id: i64,
) -> Result<(), AppError> {
    prepare_enter_focus_state(state, id, || capture_window_geometry(window))?;

    window.set_min_size(Some(LogicalSize::new(
        FOCUS_WIDTH_LOGICAL,
        FOCUS_HEIGHT_LOGICAL,
    )))?;
    window.set_decorations(false)?;
    window.set_size(LogicalSize::new(FOCUS_WIDTH_LOGICAL, FOCUS_HEIGHT_LOGICAL))?;
    window.set_resizable(false)?;
    window.set_always_on_top(true)?;

    let monitor = match window.current_monitor()? {
        Some(m) => Some(m),
        None => window.primary_monitor()?,
    };

    if let Some(m) = monitor {
        let work_area = m.work_area();
        let scale_factor = m.scale_factor();
        let pos = compute_top_right_position(
            MonitorWorkArea {
                x: work_area.position.x,
                y: work_area.position.y,
                width: work_area.size.width,
                height: work_area.size.height,
            },
            scale_factor,
            FOCUS_WIDTH_LOGICAL,
            FOCUS_MARGIN_LOGICAL,
        );
        window.set_position(pos)?;
    }

    Ok(())
}

/// Mengembalikan jendela utama dari Focus Mode ke ukuran, posisi, dan dekorasi normal sebelumnya.
pub fn exit_focus_mode(window: &WebviewWindow, state: &AppState) -> Result<(), AppError> {
    let saved_opt = clear_focus_state(state)?;

    window.set_always_on_top(false)?;

    if let Some(saved) = saved_opt {
        window.set_resizable(saved.resizable)?;
        window.set_decorations(saved.decorations)?;
        window.set_min_size(Some(LogicalSize::new(
            NORMAL_MIN_WIDTH_LOGICAL,
            NORMAL_MIN_HEIGHT_LOGICAL,
        )))?;
        window.set_size(saved.size)?;
        window.set_position(saved.position)?;
        if saved.is_maximized {
            window.maximize()?;
        }
    } else {
        window.set_decorations(false)?;
        window.set_resizable(true)?;
        window.set_min_size(Some(LogicalSize::new(
            NORMAL_MIN_WIDTH_LOGICAL,
            NORMAL_MIN_HEIGHT_LOGICAL,
        )))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("test-focus.db");
        let conn = db::init_db(&db_path).expect("init_db should succeed");
        (dir, AppState::new(conn))
    }

    #[test]
    fn test_compute_top_right_single_monitor_scale_100_percent() {
        let work_area = MonitorWorkArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1040,
        };
        let pos = compute_top_right_position(
            work_area,
            1.0,
            FOCUS_WIDTH_LOGICAL,
            FOCUS_MARGIN_LOGICAL,
        );
        assert_eq!(pos.x, 1920 - 280 - 16);
        assert_eq!(pos.x, 1624);
        assert_eq!(pos.y, 16);
    }

    #[test]
    fn test_compute_top_right_hidpi_scale_125_and_150_and_200_percent() {
        // Scale 125% pada monitor 1920x1080 (work area tinggi 1030):
        // widget_width_phys = 280 * 1.25 = 350, margin_phys = 16 * 1.25 = 20
        let pos_125 = compute_top_right_position(
            MonitorWorkArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1030,
            },
            1.25,
            FOCUS_WIDTH_LOGICAL,
            FOCUS_MARGIN_LOGICAL,
        );
        assert_eq!(pos_125, PhysicalPosition::new(1920 - 350 - 20, 20));
        assert_eq!(pos_125, PhysicalPosition::new(1550, 20));

        // Scale 150% pada monitor 4K 3840x2160:
        // widget_width_phys = 280 * 1.5 = 420, margin_phys = 16 * 1.5 = 24
        let pos_150 = compute_top_right_position(
            MonitorWorkArea {
                x: 0,
                y: 0,
                width: 3840,
                height: 2100,
            },
            1.5,
            FOCUS_WIDTH_LOGICAL,
            FOCUS_MARGIN_LOGICAL,
        );
        assert_eq!(pos_150, PhysicalPosition::new(3840 - 420 - 24, 24));
        assert_eq!(pos_150, PhysicalPosition::new(3396, 24));

        // Scale 200% pada monitor Retina/HiDPI 2880x1800:
        // widget_width_phys = 280 * 2.0 = 560, margin_phys = 16 * 2.0 = 32
        let pos_200 = compute_top_right_position(
            MonitorWorkArea {
                x: 0,
                y: 0,
                width: 2880,
                height: 1720,
            },
            2.0,
            FOCUS_WIDTH_LOGICAL,
            FOCUS_MARGIN_LOGICAL,
        );
        assert_eq!(pos_200, PhysicalPosition::new(2880 - 560 - 32, 32));
        assert_eq!(pos_200, PhysicalPosition::new(2288, 32));
    }

    #[test]
    fn test_compute_top_right_secondary_monitor_positive_and_negative_offsets() {
        // Monitor kedua di sebelah kanan (1920, 0) ukuran 2560x1400, scale 1.0
        let right_monitor = MonitorWorkArea {
            x: 1920,
            y: 0,
            width: 2560,
            height: 1400,
        };
        let pos_right = compute_top_right_position(
            right_monitor,
            1.0,
            FOCUS_WIDTH_LOGICAL,
            FOCUS_MARGIN_LOGICAL,
        );
        assert_eq!(pos_right, PhysicalPosition::new(1920 + 2560 - 280 - 16, 16));
        assert_eq!(pos_right, PhysicalPosition::new(4184, 16));

        // Monitor kedua di sebelah kiri/atas dengan koordinat negatif (-1920, -120)
        let left_monitor = MonitorWorkArea {
            x: -1920,
            y: -120,
            width: 1920,
            height: 1040,
        };
        let pos_left = compute_top_right_position(
            left_monitor,
            1.0,
            FOCUS_WIDTH_LOGICAL,
            FOCUS_MARGIN_LOGICAL,
        );
        assert_eq!(pos_left, PhysicalPosition::new(-296, -104));

        // Monitor dengan taskbar di bagian atas (work_area.y = 40)
        let top_taskbar_monitor = MonitorWorkArea {
            x: 0,
            y: 40,
            width: 1920,
            height: 1040,
        };
        let pos_top_bar = compute_top_right_position(
            top_taskbar_monitor,
            1.0,
            FOCUS_WIDTH_LOGICAL,
            FOCUS_MARGIN_LOGICAL,
        );
        assert_eq!(pos_top_bar, PhysicalPosition::new(1624, 56));
    }

    #[test]
    fn test_compute_top_right_invalid_scale_factor_and_tiny_work_area() {
        let area = MonitorWorkArea {
            x: 100,
            y: 50,
            width: 1920,
            height: 1080,
        };

        // Scale factor <= 0 atau NaN harus fallback ke 1.0
        let pos_zero =
            compute_top_right_position(area, 0.0, FOCUS_WIDTH_LOGICAL, FOCUS_MARGIN_LOGICAL);
        let pos_neg =
            compute_top_right_position(area, -1.5, FOCUS_WIDTH_LOGICAL, FOCUS_MARGIN_LOGICAL);
        let pos_nan =
            compute_top_right_position(area, f64::NAN, FOCUS_WIDTH_LOGICAL, FOCUS_MARGIN_LOGICAL);
        let expected_fallback = PhysicalPosition::new(100 + 1920 - 280 - 16, 50 + 16);

        assert_eq!(pos_zero, expected_fallback);
        assert_eq!(pos_neg, expected_fallback);
        assert_eq!(pos_nan, expected_fallback);

        // Work area sangat sempit (lebar 200 < lebar widget 280): x harus di-clamp ke work_area.x
        let tiny_area = MonitorWorkArea {
            x: 300,
            y: 20,
            width: 200,
            height: 150,
        };
        let pos_tiny =
            compute_top_right_position(tiny_area, 1.0, FOCUS_WIDTH_LOGICAL, FOCUS_MARGIN_LOGICAL);
        assert_eq!(pos_tiny.x, 300);
        assert_eq!(pos_tiny.y, 36);
    }

    #[test]
    fn test_focus_state_validation_and_geometry_preservation() {
        let (_dir, state) = make_test_state();

        let (sw1, sw2) = {
            let conn = state.lock_db().expect("lock db");
            let s1 = db::create_stopwatch(&conn, Some("Fokus 1"), 1_000).expect("create sw1");
            let s2 = db::create_stopwatch(&conn, Some("Fokus 2"), 1_100).expect("create sw2");
            (s1, s2)
        };

        let normal_geom = SavedWindowGeometry {
            size: PhysicalSize::new(480, 640),
            position: PhysicalPosition::new(200, 150),
            is_maximized: false,
            decorations: true,
            resizable: true,
        };

        // 1. Masuk Focus Mode dengan ID yang tidak ada harus gagal dengan NotFound dan tidak mengubah FocusState
        let invalid_res = prepare_enter_focus_state(&state, 9999, || Ok(normal_geom));
        assert!(matches!(invalid_res, Err(AppError::NotFound(9999))));
        {
            let focus = state.lock_focus().expect("lock focus");
            assert_eq!(focus.saved_geometry, None);
            assert_eq!(focus.focused_stopwatch_id, None);
        }

        // 2. Masuk Focus Mode pertama kali dengan sw1 menyimpan normal_geom
        prepare_enter_focus_state(&state, sw1.id, || Ok(normal_geom)).expect("enter focus sw1");
        {
            let focus = state.lock_focus().expect("lock focus");
            assert_eq!(focus.saved_geometry, Some(normal_geom));
            assert_eq!(focus.focused_stopwatch_id, Some(sw1.id));
        }

        // 3. Saat sudah berada di Focus Mode, berpindah target ke sw2 tidak boleh menimpa saved_geometry awal
        let focus_widget_geom = SavedWindowGeometry {
            size: PhysicalSize::new(280, 96),
            position: PhysicalPosition::new(1624, 16),
            is_maximized: false,
            decorations: false,
            resizable: false,
        };
        prepare_enter_focus_state(&state, sw2.id, || Ok(focus_widget_geom))
            .expect("switch focus to sw2");
        {
            let focus = state.lock_focus().expect("lock focus");
            assert_eq!(focus.saved_geometry, Some(normal_geom));
            assert_eq!(focus.focused_stopwatch_id, Some(sw2.id));
        }

        // 4. Keluar dari Focus Mode membersihkan state dan mengembalikan normal_geom asli
        let restored = clear_focus_state(&state).expect("clear focus state");
        assert_eq!(restored, Some(normal_geom));
        {
            let focus = state.lock_focus().expect("lock focus");
            assert_eq!(focus.saved_geometry, None);
            assert_eq!(focus.focused_stopwatch_id, None);
        }
    }
}
