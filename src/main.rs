#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;

use config::{Config, seed_rng};
use std::mem::size_of;
use windows::Win32::Foundation::{
    COLORREF, ERROR_ALREADY_EXISTS, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT,
    WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    COLOR_WINDOW, COLOR_WINDOWTEXT, DEFAULT_GUI_FONT, GetStockObject, GetSysColor,
    GetSysColorBrush, HDC, OPAQUE, SetBkColor, SetBkMode, SetTextColor,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::{
    ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, SetThreadExecutionState,
};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Controls::{
    BST_CHECKED, BST_UNCHECKED, CheckDlgButton, EM_LIMITTEXT, ICC_BAR_CLASSES,
    INITCOMMONCONTROLSEX, InitCommonControlsEx, IsDlgButtonChecked, SB_SETTEXTW, SB_SIMPLE,
    SB_SIMPLEID,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT, SendInput,
};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO, NIIF_NOSOUND, NIM_ADD, NIM_DELETE,
    NIM_SETVERSION, NIN_BALLOONUSERCLICK, NIN_SELECT, NOTIFYICONDATAW, NOTIFYICON_VERSION_4,
    Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, AppendMenuW, BS_AUTOCHECKBOX, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW,
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow,
    DispatchMessageW, EN_CHANGE, ES_NUMBER, FindWindowW, GWLP_USERDATA, GetCursorPos, GetDlgItem,
    GetMessageW, GetSystemMetrics, GetWindowLongPtrW, GetWindowTextW, HICON, HMENU, IDC_ARROW,
    IDI_APPLICATION, IMAGE_FLAGS, IMAGE_ICON, KillTimer, LR_DEFAULTCOLOR, LoadCursorW, LoadIconW,
    LoadImageW, MB_ICONERROR, MB_OK, MF_STRING, MSG, MessageBoxW, PostMessageW, PostQuitMessage,
    RegisterClassExW, SC_CLOSE, SC_MINIMIZE, SM_CXSCREEN, SM_CXSMICON, SM_CXVIRTUALSCREEN,
    SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_HIDE, SW_RESTORE,
    SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SendMessageW, SetCursorPos,
    SetForegroundWindow, SetProcessDPIAware, SetTimer, SetWindowLongPtrW, SetWindowPos,
    SetWindowTextW, ShowWindow, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU, WM_CREATE,
    WM_CTLCOLORBTN, WM_CTLCOLORSTATIC, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_NCCREATE,
    WM_NULL, WM_RBUTTONUP, WM_SETFONT, WM_SIZE, WM_SYSCOMMAND, WM_TIMER, WNDCLASSEXW, WS_BORDER,
    WS_CAPTION, WS_CHILD, WS_EX_APPWINDOW, WS_EX_CLIENTEDGE, WS_MINIMIZEBOX, WS_OVERLAPPED,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

const CLASS_NAME: PCWSTR = w!("MouseMoverWnd");
const WINDOW_TITLE: PCWSTR = w!("Mouse Mover");
const MUTEX_NAME: PCWSTR = w!("Local\\MouseMover.SingleInstance");

const IDC_ENABLE: i32 = 101;
const IDC_ZEN: i32 = 102;
const IDC_INTERVAL_LABEL: i32 = 103;
const IDC_INTERVAL: i32 = 104;
const IDC_RANDOM: i32 = 105;
const IDC_JITTER_LABEL: i32 = 106;
const IDC_JITTER: i32 = 107;
const IDC_JITTER_PCT: i32 = 108;
const IDC_MINIMIZE: i32 = 109;
const IDC_HINT: i32 = 110;
const IDC_STATUS: i32 = 111;

const IDM_OPEN: usize = 201;
const IDM_EXIT: usize = 202;

const WM_TRAY: u32 = WM_APP + 1;
const TIMER_JIGGLE: usize = 1;
const TIMER_ANIM: usize = 2;
const TRAY_UID: u32 = 1;

const CLIENT_W: i32 = 428;
const CLIENT_H: i32 = 214;

const VISIBLE_DELTA: i32 = 64;
const ANIM_STEP_MS: u32 = 90;
const VISIBLE_PATH: [(i32, i32); 4] = [
    (VISIBLE_DELTA, 0),
    (VISIBLE_DELTA, VISIBLE_DELTA),
    (0, VISIBLE_DELTA),
    (0, 0),
];

struct AppState {
    cfg: Config,
    icon: HICON,
    rng: u32,
    last_cursor: POINT,
    next_delay_ms: u32,
    ui_ready: bool,
    own_icon: bool,
    anim_origin: POINT,
    anim_step: u8,
}

fn main() {
    if let Err(err) = run() {
        let text: Vec<u16> = format!("{err}\0").encode_utf16().collect();
        unsafe {
            let _ = MessageBoxW(
                None,
                PCWSTR(text.as_ptr()),
                WINDOW_TITLE,
                MB_OK | MB_ICONERROR,
            );
        }
    }
}

fn run() -> windows::core::Result<()> {
    unsafe {
        let _ = SetProcessDPIAware();
        let icc = INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_BAR_CLASSES,
        };
        let _ = InitCommonControlsEx(&icc);

        let module = GetModuleHandleW(None)?;
        let hinst = HINSTANCE(module.0);

        let _mutex = CreateMutexW(None, true, MUTEX_NAME)?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            if let Ok(existing) = FindWindowW(CLASS_NAME, WINDOW_TITLE) {
                restore_window(existing);
            }
            return Ok(());
        }

        let (icon, own_icon) = load_app_icon(hinst);

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: hinst,
            hIcon: icon,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: GetSysColorBrush(COLOR_WINDOW),
            lpszClassName: CLASS_NAME,
            hIconSm: icon,
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let mut cfg = Config::load();
        cfg.sanitize();
        let start_hidden = cfg.minimize_to_tray;

        let mut state = Box::new(AppState {
            cfg,
            icon,
            rng: seed_rng(),
            last_cursor: cursor_pos(),
            next_delay_ms: 0,
            ui_ready: false,
            own_icon,
            anim_origin: POINT { x: 0, y: 0 },
            anim_step: 0,
        });
        state.next_delay_ms = state.cfg.next_delay_ms(&mut state.rng);
        let ptr = Box::into_raw(state);

        let (win_w, win_h) = window_size();
        let x = (GetSystemMetrics(SM_CXSCREEN) - win_w).max(0) / 2;
        let y = (GetSystemMetrics(SM_CYSCREEN) - win_h).max(0) / 2;

        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
        let hwnd = match CreateWindowExW(
            WS_EX_APPWINDOW,
            CLASS_NAME,
            WINDOW_TITLE,
            style,
            x,
            y,
            win_w,
            win_h,
            None,
            None,
            Some(hinst),
            Some(ptr as *const _),
        ) {
            Ok(h) => h,
            Err(e) => {
                drop(Box::from_raw(ptr));
                return Err(e);
            }
        };

        tray_add(hwnd, icon, start_hidden);
        if start_hidden {
            let _ = ShowWindow(hwnd, SW_HIDE);
        } else {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }

        apply_awake(ptr_state(ptr).cfg.enabled);
        restart_timer(hwnd, ptr_state(ptr));
        update_status(hwnd, ptr_state(ptr));

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Ok(())
    }
}

fn ptr_state(ptr: *mut AppState) -> &'static mut AppState {
    unsafe { &mut *ptr }
}

fn window_size() -> (i32, i32) {
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: CLIENT_W,
        bottom: CLIENT_H,
    };
    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    unsafe {
        let _ = AdjustWindowRectEx(&mut rc, style, false, WS_EX_APPWINDOW);
    }
    (rc.right - rc.left, rc.bottom - rc.top)
}

fn load_app_icon(hinst: HINSTANCE) -> (HICON, bool) {
    unsafe {
        let sm = GetSystemMetrics(SM_CXSMICON);
        if let Ok(handle) = LoadImageW(
            Some(hinst),
            PCWSTR(1u16 as *const u16),
            IMAGE_ICON,
            sm,
            sm,
            LR_DEFAULTCOLOR,
        ) {
            return (HICON(handle.0), true);
        }
        if let Ok(handle) = LoadImageW(
            Some(hinst),
            PCWSTR(1u16 as *const u16),
            IMAGE_ICON,
            32,
            32,
            IMAGE_FLAGS(0),
        ) {
            return (HICON(handle.0), true);
        }
        match LoadIconW(Some(hinst), PCWSTR(1u16 as *const u16)) {
            Ok(icon) if !icon.0.is_null() => (icon, false),
            _ => (LoadIconW(None, IDI_APPLICATION).unwrap_or_default(), false),
        }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let cs = lparam.0 as *const CREATESTRUCTW;
        if !cs.is_null() {
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*cs).lpCreateParams as _);
            }
        }
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    let state = {
        let p = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut AppState;
        if p.is_null() {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }
        unsafe { &mut *p }
    };

    match msg {
        WM_CREATE => {
            unsafe { create_controls(hwnd) };
            sync_ui(hwnd, state);
            state.ui_ready = true;
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 as u32) & 0xFFFF;
            let code = (wparam.0 as u32) >> 16;
            let edit_changed =
                (id == IDC_INTERVAL as u32 || id == IDC_JITTER as u32) && code == EN_CHANGE as u32;
            let toggle = matches!(
                id,
                x if x == IDC_ENABLE as u32
                    || x == IDC_ZEN as u32
                    || x == IDC_RANDOM as u32
                    || x == IDC_MINIMIZE as u32
            );
            if state.ui_ready && (edit_changed || toggle) {
                let was_enabled = state.cfg.enabled;
                let old_interval = state.cfg.interval_secs;
                let old_rand = state.cfg.randomize;
                let old_jitter = state.cfg.jitter_percent;
                pull_ui(hwnd, state);
                sync_jitter_enabled(hwnd, state.cfg.randomize);
                apply_awake(state.cfg.enabled);
                let timing_changed = state.cfg.enabled != was_enabled
                    || state.cfg.interval_secs != old_interval
                    || state.cfg.randomize != old_rand
                    || state.cfg.jitter_percent != old_jitter;
                if state.cfg.enabled && timing_changed {
                    restart_timer(hwnd, state);
                } else if !state.cfg.enabled {
                    restart_timer(hwnd, state);
                }
                update_status(hwnd, state);
            }
            LRESULT(0)
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => paint_light_control(hwnd, wparam, lparam),
        WM_SIZE => {
            layout_status(hwnd);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_JIGGLE {
                on_timer(hwnd, state);
            } else if wparam.0 == TIMER_ANIM {
                on_anim(hwnd, state);
            }
            LRESULT(0)
        }
        WM_TRAY => {
            // NOTIFYICON_VERSION_4: событие в LOWORD(lParam). Классика: весь lParam.
            let event = (lparam.0 as u32) & 0xFFFF;
            const NIN_KEYSELECT: u32 = 1025;
            match event {
                NIN_BALLOONUSERCLICK | NIN_SELECT | NIN_KEYSELECT | WM_LBUTTONUP
                | WM_LBUTTONDBLCLK => restore_window(hwnd),
                WM_RBUTTONUP | WM_CONTEXTMENU => unsafe { show_tray_menu(hwnd) },
                _ => {}
            }
            LRESULT(0)
        }
        WM_SYSCOMMAND => {
            let cmd = (wparam.0 as u32) & 0xFFF0;
            if cmd == SC_MINIMIZE || cmd == SC_CLOSE {
                unsafe {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
                return LRESULT(0);
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_CLOSE => {
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0 as _);
                let _ = KillTimer(Some(hwnd), TIMER_ANIM);
                tray_remove(hwnd);
                apply_awake(false);
                if !p.is_null() {
                    let state = Box::from_raw(p);
                    if state.own_icon && !state.icon.0.is_null() {
                        let _ = DestroyIcon(state.icon);
                    }
                }
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn create_controls(parent: HWND) {
    let font = unsafe { GetStockObject(DEFAULT_GUI_FONT) };
    let font_wp = WPARAM(font.0 as usize);

    let items: &[(
        &str,
        &str,
        WINDOW_STYLE,
        WINDOW_EX_STYLE,
        i32,
        i32,
        i32,
        i32,
        i32,
    )] = &[
        (
            "BUTTON",
            "Двигать мышь",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            WINDOW_EX_STYLE(0),
            16,
            16,
            380,
            22,
            IDC_ENABLE,
        ),
        (
            "BUTTON",
            "Скрытый режим (курсор не прыгает)",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            WINDOW_EX_STYLE(0),
            16,
            44,
            380,
            22,
            IDC_ZEN,
        ),
        (
            "STATIC",
            "Интервал, сек:",
            WS_CHILD | WS_VISIBLE,
            WINDOW_EX_STYLE(0),
            16,
            78,
            130,
            22,
            IDC_INTERVAL_LABEL,
        ),
        (
            "EDIT",
            "",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER | WINDOW_STYLE(ES_NUMBER as u32),
            WS_EX_CLIENTEDGE,
            150,
            74,
            64,
            24,
            IDC_INTERVAL,
        ),
        (
            "BUTTON",
            "Случайный интервал",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            WINDOW_EX_STYLE(0),
            16,
            110,
            168,
            22,
            IDC_RANDOM,
        ),
        (
            "STATIC",
            "±",
            WS_CHILD | WS_VISIBLE,
            WINDOW_EX_STYLE(0),
            188,
            112,
            18,
            20,
            IDC_JITTER_LABEL,
        ),
        (
            "EDIT",
            "",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER | WINDOW_STYLE(ES_NUMBER as u32),
            WS_EX_CLIENTEDGE,
            208,
            108,
            48,
            24,
            IDC_JITTER,
        ),
        (
            "STATIC",
            "%",
            WS_CHILD | WS_VISIBLE,
            WINDOW_EX_STYLE(0),
            262,
            112,
            24,
            20,
            IDC_JITTER_PCT,
        ),
        (
            "BUTTON",
            "Сворачивать в трей при запуске",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            WINDOW_EX_STYLE(0),
            16,
            142,
            390,
            22,
            IDC_MINIMIZE,
        ),
        (
            "STATIC",
            "Закрытие окна сворачивает в трей. Выход — через меню иконки.",
            WS_CHILD | WS_VISIBLE,
            WINDOW_EX_STYLE(0),
            16,
            172,
            396,
            20,
            IDC_HINT,
        ),
    ];

    for (class, text, style, ex, x, y, w, h, id) in items {
        let class_w: Vec<u16> = class.encode_utf16().chain([0]).collect();
        let text_w: Vec<u16> = text.encode_utf16().chain([0]).collect();
        if let Ok(ctrl) = unsafe {
            CreateWindowExW(
                *ex,
                PCWSTR(class_w.as_ptr()),
                PCWSTR(text_w.as_ptr()),
                *style,
                *x,
                *y,
                *w,
                *h,
                Some(parent),
                Some(HMENU(*id as isize as *mut core::ffi::c_void)),
                None,
                None,
            )
        } {
            unsafe {
                SendMessageW(ctrl, WM_SETFONT, Some(font_wp), Some(LPARAM(1)));
                if *id == IDC_INTERVAL {
                    SendMessageW(ctrl, EM_LIMITTEXT, Some(WPARAM(3)), Some(LPARAM(0)));
                }
                if *id == IDC_JITTER {
                    SendMessageW(ctrl, EM_LIMITTEXT, Some(WPARAM(2)), Some(LPARAM(0)));
                }
            }
        }
    }

    let status_class: Vec<u16> = "msctls_statusbar32".encode_utf16().chain([0]).collect();
    if let Ok(sb) = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(status_class.as_ptr()),
            w!(""),
            WS_CHILD | WS_VISIBLE,
            0,
            0,
            0,
            0,
            Some(parent),
            Some(HMENU(IDC_STATUS as isize as *mut core::ffi::c_void)),
            None,
            None,
        )
    } {
        unsafe {
            SendMessageW(sb, WM_SETFONT, Some(font_wp), Some(LPARAM(1)));
            SendMessageW(sb, SB_SIMPLE, Some(WPARAM(1)), None);
        }
        layout_status(parent);
    }
}

fn layout_status(parent: HWND) {
    if let Ok(sb) = unsafe { GetDlgItem(Some(parent), IDC_STATUS) } {
        unsafe {
            SendMessageW(sb, WM_SIZE, None, None);
        }
    }
}

fn paint_light_control(parent: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ctrl = HWND(lparam.0 as *mut core::ffi::c_void);
    if let Ok(sb) = unsafe { GetDlgItem(Some(parent), IDC_STATUS) } {
        if ctrl == sb {
            return unsafe { DefWindowProcW(parent, WM_CTLCOLORSTATIC, wparam, lparam) };
        }
    }
    let hdc = HDC(wparam.0 as *mut core::ffi::c_void);
    unsafe {
        let text = COLORREF(GetSysColor(COLOR_WINDOWTEXT));
        let back = COLORREF(GetSysColor(COLOR_WINDOW));
        SetTextColor(hdc, text);
        SetBkColor(hdc, back);
        SetBkMode(hdc, OPAQUE);
        LRESULT(GetSysColorBrush(COLOR_WINDOW).0 as isize)
    }
}

fn checked(parent: HWND, id: i32) -> bool {
    unsafe { IsDlgButtonChecked(parent, id) == BST_CHECKED.0 }
}

fn set_checked(parent: HWND, id: i32, value: bool) {
    unsafe {
        let _ = CheckDlgButton(parent, id, if value { BST_CHECKED } else { BST_UNCHECKED });
    }
}

fn set_edit(hwnd: HWND, id: i32, value: u32) {
    if let Ok(edit) = unsafe { GetDlgItem(Some(hwnd), id) } {
        let text: Vec<u16> = format!("{value}\0").encode_utf16().collect();
        unsafe {
            let _ = SetWindowTextW(edit, PCWSTR(text.as_ptr()));
        }
    }
}

fn read_u32(hwnd: HWND, id: i32) -> Option<u32> {
    let edit = unsafe { GetDlgItem(Some(hwnd), id) }.ok()?;
    let mut buf = [0u16; 8];
    let n = unsafe { GetWindowTextW(edit, &mut buf) };
    if n <= 0 {
        return None;
    }
    String::from_utf16_lossy(&buf[..n as usize])
        .trim()
        .parse()
        .ok()
}

fn sync_jitter_enabled(hwnd: HWND, on: bool) {
    if let Ok(edit) = unsafe { GetDlgItem(Some(hwnd), IDC_JITTER) } {
        unsafe {
            let _ = EnableWindow(edit, on);
        }
    }
}

fn sync_ui(hwnd: HWND, state: &AppState) {
    set_checked(hwnd, IDC_ENABLE, state.cfg.enabled);
    set_checked(hwnd, IDC_ZEN, state.cfg.zen);
    set_checked(hwnd, IDC_RANDOM, state.cfg.randomize);
    set_checked(hwnd, IDC_MINIMIZE, state.cfg.minimize_to_tray);
    set_edit(hwnd, IDC_INTERVAL, state.cfg.interval_secs);
    set_edit(hwnd, IDC_JITTER, state.cfg.jitter_percent);
    sync_jitter_enabled(hwnd, state.cfg.randomize);
}

fn pull_ui(hwnd: HWND, state: &mut AppState) {
    state.cfg.enabled = checked(hwnd, IDC_ENABLE);
    state.cfg.zen = checked(hwnd, IDC_ZEN);
    state.cfg.randomize = checked(hwnd, IDC_RANDOM);
    state.cfg.minimize_to_tray = checked(hwnd, IDC_MINIMIZE);
    if let Some(v) = read_u32(hwnd, IDC_INTERVAL) {
        state.cfg.interval_secs = v;
    }
    if let Some(v) = read_u32(hwnd, IDC_JITTER) {
        state.cfg.jitter_percent = v;
    }
    state.cfg.sanitize();
    state.cfg.save();
}

fn restart_timer(hwnd: HWND, state: &mut AppState) {
    unsafe {
        let _ = KillTimer(Some(hwnd), TIMER_JIGGLE);
        if !state.cfg.enabled {
            let _ = KillTimer(Some(hwnd), TIMER_ANIM);
            state.anim_step = 0;
        }
    }
    if !state.cfg.enabled {
        return;
    }
    state.next_delay_ms = state.cfg.next_delay_ms(&mut state.rng);
    unsafe {
        SetTimer(Some(hwnd), TIMER_JIGGLE, state.next_delay_ms, None);
    }
}

fn on_timer(hwnd: HWND, state: &mut AppState) {
    if state.cfg.enabled {
        let now = cursor_pos();
        let user_moved = now.x != state.last_cursor.x || now.y != state.last_cursor.y;
        if !user_moved {
            start_jiggle(hwnd, state);
        }
        state.last_cursor = cursor_pos();
        apply_awake(true);
    }
    restart_timer(hwnd, state);
    update_status(hwnd, state);
}

fn start_jiggle(hwnd: HWND, state: &mut AppState) {
    if state.cfg.zen {
        send_rel(0, 0);
        return;
    }
    if state.anim_step != 0 {
        return;
    }
    state.anim_origin = cursor_pos();
    state.anim_step = 1;
    move_cursor_to(
        state.anim_origin.x + VISIBLE_PATH[0].0,
        state.anim_origin.y + VISIBLE_PATH[0].1,
    );
    unsafe {
        SetTimer(Some(hwnd), TIMER_ANIM, ANIM_STEP_MS, None);
    }
}

fn on_anim(hwnd: HWND, state: &mut AppState) {
    if state.anim_step == 0 {
        return;
    }
    let idx = state.anim_step as usize;
    if idx >= VISIBLE_PATH.len() {
        unsafe {
            let _ = KillTimer(Some(hwnd), TIMER_ANIM);
        }
        state.anim_step = 0;
        state.last_cursor = cursor_pos();
        return;
    }
    move_cursor_to(
        state.anim_origin.x + VISIBLE_PATH[idx].0,
        state.anim_origin.y + VISIBLE_PATH[idx].1,
    );
    state.anim_step += 1;
    state.last_cursor = cursor_pos();
}

fn update_status(hwnd: HWND, state: &AppState) {
    let text = if !state.cfg.enabled {
        "Пауза".to_string()
    } else {
        let mode = if state.cfg.zen {
            "скрытый"
        } else {
            "видимый"
        };
        let secs = (state.next_delay_ms + 500) / 1000;
        if state.cfg.randomize {
            format!(
                "Вкл · {mode} · {base} с ±{pct}% · след. ~{secs} с",
                base = state.cfg.interval_secs,
                pct = state.cfg.jitter_percent
            )
        } else {
            format!(
                "Вкл · {mode} · каждые {base} с · след. ~{secs} с",
                base = state.cfg.interval_secs
            )
        }
    };
    if let Ok(sb) = unsafe { GetDlgItem(Some(hwnd), IDC_STATUS) } {
        let w: Vec<u16> = text.encode_utf16().chain([0]).collect();
        unsafe {
            SendMessageW(
                sb,
                SB_SETTEXTW,
                Some(WPARAM(SB_SIMPLEID as usize)),
                Some(LPARAM(w.as_ptr() as isize)),
            );
        }
    }
}

fn send_rel(dx: i32, dy: i32) {
    let inp = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        let _ = SendInput(&[inp], size_of::<INPUT>() as i32);
    }
}

fn move_cursor_to(x: i32, y: i32) {
    send_abs(x, y);
    let _ = unsafe { SetCursorPos(x, y) };
}

fn send_abs(x: i32, y: i32) {
    let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) }.max(1);
    let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) }.max(1);
    let ax = ((x - vx) as i64 * 65535 / (vw - 1).max(1) as i64) as i32;
    let ay = ((y - vy) as i64 * 65535 / (vh - 1).max(1) as i64) as i32;
    let inp = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: ax,
                dy: ay,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        let _ = SendInput(&[inp], size_of::<INPUT>() as i32);
    }
}

fn cursor_pos() -> POINT {
    let mut pt = POINT { x: 0, y: 0 };
    let _ = unsafe { GetCursorPos(&mut pt) };
    pt
}

fn apply_awake(on: bool) {
    unsafe {
        if on {
            SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
        } else {
            SetThreadExecutionState(ES_CONTINUOUS);
        }
    }
}

fn wsz<const N: usize>(s: &str) -> [u16; N] {
    let mut buf = [0u16; N];
    let mut i = 0;
    for unit in s.encode_utf16() {
        if i + 1 >= N {
            break;
        }
        buf[i] = unit;
        i += 1;
    }
    buf
}

fn tray_add(hwnd: HWND, icon: HICON, balloon: bool) {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = TRAY_UID;
    let mut flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    if balloon {
        flags |= NIF_INFO;
        nid.szInfoTitle = wsz("Mouse Mover");
        nid.szInfo = wsz("Mouse Mover запущен в свёрнутом режиме");
        nid.dwInfoFlags = NIIF_INFO | NIIF_NOSOUND;
    }
    nid.uFlags = flags;
    nid.uCallbackMessage = WM_TRAY;
    nid.hIcon = icon;
    nid.szTip = wsz("Mouse Mover");
    unsafe {
        let _ = Shell_NotifyIconW(NIM_ADD, &nid);
        nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        let _ = Shell_NotifyIconW(NIM_SETVERSION, &nid);
    }
}

fn tray_remove(hwnd: HWND) {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = TRAY_UID;
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}

unsafe fn show_tray_menu(hwnd: HWND) {
    unsafe {
        let Ok(menu) = CreatePopupMenu() else {
            return;
        };
        let _ = AppendMenuW(menu, MF_STRING, IDM_OPEN, w!("Открыть"));
        let _ = AppendMenuW(menu, MF_STRING, IDM_EXIT, w!("Завершить работу"));

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_RETURNCMD,
            pt.x,
            pt.y,
            Some(0),
            hwnd,
            None,
        );
        let _ = DestroyMenu(menu);
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));

        match cmd.0 as usize {
            IDM_OPEN => restore_window(hwnd),
            IDM_EXIT => {
                let _ = DestroyWindow(hwnd);
            }
            _ => {}
        }
    }
}

fn restore_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOZORDER | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
    }
}
