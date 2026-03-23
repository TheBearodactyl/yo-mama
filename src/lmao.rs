use {
    image::{AnimationDecoder, ImageReader},
    std::{
        ffi::c_void,
        io::Cursor,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    },
    windows::{
        Win32::{
            Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM, *},
            Graphics::Gdi::*,
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{
                DefWindowProcW, HTCAPTION, MB_ICONERROR, MB_OK, MessageBoxW, PostQuitMessage,
                WM_DESTROY, WM_NCHITTEST, *,
            },
        },
        core::{Result, *},
    },
};

pub fn crash() -> ! {
    unsafe {
        MessageBoxW(None, w!("lmao"), w!("bro crashed"), MB_OK | MB_ICONERROR);
    }

    std::process::exit(1);
}

fn decode_rgba_from_bytes(bytes: &[u8]) -> color_eyre::Result<(Vec<u8>, u32, u32)> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?
        .into_rgba8();

    let (w, h) = img.dimensions();
    Ok((img.into_raw(), w, h))
}

fn apply_scale(rgba: &[u8], w: u32, h: u32, scale: f32) -> (Vec<u8>, u32, u32) {
    let new_w = ((w as f32) * scale).round() as u32;
    let new_h = ((h as f32) * scale).round() as u32;
    let src = image::RgbaImage::from_raw(w, h, rgba.to_vec()).unwrap();
    let resized =
        image::imageops::resize(&src, new_w, new_h, image::imageops::FilterType::Triangle);
    let (rw, rh) = resized.dimensions();
    (resized.into_raw(), rw, rh)
}

pub struct TransparentOverlay {
    hwnd: HWND,
}

impl TransparentOverlay {
    pub fn new(
        image_data: &[u8],
        x: i32,
        y: i32,
        global_alpha: u8,
        scale: Option<f32>,
    ) -> Result<Self> {
        let (image_data, width, height) =
            decode_rgba_from_bytes(image_data).expect("Failed to decode");
        let (image_data, width, height) = match scale {
            Some(s) if s != 1.0 => apply_scale(&image_data, width, height, s),
            _ => (image_data, width, height),
        };
        let bgra = rgba_to_premul_bgra(image_data.as_slice());

        unsafe {
            let hinstance: HINSTANCE = GetModuleHandleW(None)?.into();
            let class_name = w!("TransparentOverlayClass");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpszClassName: class_name,
                lpfnWndProc: Some(window_proc),
                hInstance: hinstance,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                ..Default::default()
            };

            let _ = RegisterClassExW(&wc);
            let ex_style = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT;
            let hwnd = CreateWindowExW(
                ex_style,
                class_name,
                w!(""),
                WS_POPUP | WS_VISIBLE,
                x,
                y,
                width as i32,
                height as i32,
                None,
                None,
                Some(hinstance),
                None,
            )?;

            let hdc_screen = GetDC(None);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut dib_bits: *mut c_void = std::ptr::null_mut();
            let hbitmap = CreateDIBSection(
                Some(hdc_screen),
                &bmi,
                DIB_RGB_COLORS,
                &mut dib_bits,
                None,
                0,
            )?;

            std::ptr::copy_nonoverlapping(bgra.as_ptr(), dib_bits as *mut u8, bgra.len());

            let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
            let old_obj: HGDIOBJ = SelectObject(hdc_mem, hbitmap.into());

            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: global_alpha,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };

            let pt_dst = POINT { x, y };
            let size = SIZE {
                cx: width as i32,
                cy: height as i32,
            };
            let pt_src = POINT { x: 0, y: 0 };

            let result = UpdateLayeredWindow(
                hwnd,
                Some(hdc_screen),
                Some(&pt_dst),
                Some(&size),
                Some(hdc_mem),
                Some(&pt_src),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );

            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(hbitmap.into());
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);

            result?;

            Ok(Self { hwnd })
        }
    }
}

impl Drop for TransparentOverlay {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

struct DecodedFrame {
    bgra: Vec<u8>,
    width: u32,
    height: u32,
    delay_ms: u64,
}

fn decode_animation_frames(
    data: &[u8],
    scale: Option<f32>,
) -> color_eyre::Result<Vec<DecodedFrame>> {
    let cursor = Cursor::new(data);
    let reader = ImageReader::new(cursor).with_guessed_format()?;
    let format = reader.format();

    let cursor = Cursor::new(data);

    let scale_and_convert = |rgba: Vec<u8>, w: u32, h: u32| -> (Vec<u8>, u32, u32) {
        match scale {
            Some(s) if s != 1.0 => {
                let (scaled, nw, nh) = apply_scale(&rgba, w, h, s);
                (rgba_to_premul_bgra(&scaled), nw, nh)
            }
            _ => (rgba_to_premul_bgra(&rgba), w, h),
        }
    };

    let frames: Vec<DecodedFrame> = match format {
        Some(image::ImageFormat::Gif) => {
            let decoder = image::codecs::gif::GifDecoder::new(cursor)?;
            decoder
                .into_frames()
                .filter_map(|f| f.ok())
                .map(|frame| {
                    let (numer, denom) = frame.delay().numer_denom_ms();
                    let delay_ms = (numer as u64) / (denom as u64).max(1);
                    let (w, h) = frame.buffer().dimensions();
                    let rgba = frame.into_buffer().into_raw();
                    let (bgra, width, height) = scale_and_convert(rgba, w, h);
                    DecodedFrame {
                        bgra,
                        width,
                        height,
                        delay_ms: delay_ms.max(10),
                    }
                })
                .collect()
        }
        Some(image::ImageFormat::WebP) => {
            let decoder = image::codecs::webp::WebPDecoder::new(cursor)?;
            decoder
                .into_frames()
                .filter_map(|f| f.ok())
                .map(|frame| {
                    let (numer, denom) = frame.delay().numer_denom_ms();
                    let delay_ms = (numer as u64) / (denom as u64).max(1);
                    let (w, h) = frame.buffer().dimensions();
                    let rgba = frame.into_buffer().into_raw();
                    let (bgra, width, height) = scale_and_convert(rgba, w, h);
                    DecodedFrame {
                        bgra,
                        width,
                        height,
                        delay_ms: delay_ms.max(10),
                    }
                })
                .collect()
        }
        _ => {
            let (rgba, w, h) = decode_rgba_from_bytes(data)?;
            let (bgra, width, height) = scale_and_convert(rgba, w, h);
            vec![DecodedFrame {
                bgra,
                width,
                height,
                delay_ms: 100,
            }]
        }
    };

    Ok(frames)
}

pub struct AnimatedOverlay {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AnimatedOverlay {
    pub fn new(
        image_data: &[u8],
        x: i32,
        y: i32,
        global_alpha: u8,
        scale: Option<f32>,
    ) -> Result<Self> {
        let frames =
            decode_animation_frames(image_data, scale).expect("Failed to decode animated image");

        if frames.is_empty() {
            return Err(windows::core::Error::new(E_FAIL, "No frames decoded"));
        }

        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);

        let thread = std::thread::spawn(move || unsafe {
            let hinstance: HINSTANCE = GetModuleHandleW(None).unwrap().into();
            let class_name = w!("AnimatedOverlayClass");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpszClassName: class_name,
                lpfnWndProc: Some(window_proc),
                hInstance: hinstance,
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                ..Default::default()
            };

            let _ = RegisterClassExW(&wc);
            let ex_style = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT;

            let first = &frames[0];
            let hwnd = CreateWindowExW(
                ex_style,
                class_name,
                w!(""),
                WS_POPUP | WS_VISIBLE,
                x,
                y,
                first.width as i32,
                first.height as i32,
                None,
                None,
                Some(hinstance),
                None,
            )
            .unwrap();

            let hdc_screen = GetDC(None);

            loop {
                for frame in &frames {
                    if stop_clone.load(Ordering::Relaxed) {
                        let _ = DestroyWindow(hwnd);
                        ReleaseDC(None, hdc_screen);
                        return;
                    }

                    update_layered_frame(
                        hwnd,
                        hdc_screen,
                        &frame.bgra,
                        frame.width,
                        frame.height,
                        x,
                        y,
                        global_alpha,
                    );

                    std::thread::sleep(std::time::Duration::from_millis(frame.delay_ms));
                }
            }
        });

        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for AnimatedOverlay {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn update_layered_frame(
    hwnd: HWND,
    hdc_screen: HDC,
    bgra: &[u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    global_alpha: u8,
) {
    unsafe {
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut dib_bits: *mut c_void = std::ptr::null_mut();
        let hbitmap = CreateDIBSection(
            Some(hdc_screen),
            &bmi,
            DIB_RGB_COLORS,
            &mut dib_bits,
            None,
            0,
        );

        let Ok(hbitmap) = hbitmap else { return };

        std::ptr::copy_nonoverlapping(bgra.as_ptr(), dib_bits as *mut u8, bgra.len());

        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let old_obj = SelectObject(hdc_mem, hbitmap.into());

        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: global_alpha,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let pt_dst = POINT { x, y };
        let size = SIZE {
            cx: width as i32,
            cy: height as i32,
        };
        let pt_src = POINT { x: 0, y: 0 };

        let _ = UpdateLayeredWindow(
            hwnd,
            Some(hdc_screen),
            Some(&pt_dst),
            Some(&size),
            Some(hdc_mem),
            Some(&pt_src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );

        SelectObject(hdc_mem, old_obj);
        let _ = DeleteObject(hbitmap.into());
        let _ = DeleteDC(hdc_mem);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_NCHITTEST => LRESULT(HTCAPTION as isize),
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn rgba_to_premul_bgra(rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgba.len());

    for px in rgba.chunks_exact(4) {
        let (r, g, b, a) = (px[0] as u32, px[1] as u32, px[2] as u32, px[3] as u32);

        out.push((b * a / 255) as u8);
        out.push((g * a / 255) as u8);
        out.push((r * a / 255) as u8);
        out.push(a as u8);
    }

    out
}

pub fn get_center_of_screen() -> (i32, i32) {
    unsafe {
        let mut cursor = POINT::default();
        GetCursorPos(&mut cursor).expect("get cursor pos");

        let hmonitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        GetMonitorInfoW(hmonitor, &mut info).expect("get monitor info");

        let r = info.rcMonitor;
        let cx = (r.left + r.right) / 2;
        let cy = (r.top + r.bottom) / 2;

        (cx, cy)
    }
}
