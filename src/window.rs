use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HWND};
use winapi::um::winuser::{ChangeDisplaySettingsW, CreateWindowExW, DefWindowProcW, DestroyWindow,
                          DispatchMessageW, GetSystemMetrics, GetWindowLongPtrW, PeekMessageW,
                          PostQuitMessage, RegisterClassExW, SetFocus, SetForegroundWindow,
                          SetWindowLongPtrW, ShowWindow, TranslateMessage, UpdateWindow,
                          CDS_FULLSCREEN, CREATESTRUCTW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW,
                          GWLP_USERDATA, MSG, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW,
                          VK_ESCAPE, WM_CREATE, WM_KEYDOWN, WM_QUIT, WM_SHOWWINDOW, WNDCLASSEXW,
                          WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_APPWINDOW, WS_POPUP};
use winapi::um::wingdi::{GetStockObject, BLACK_BRUSH, DEVMODEW, DM_BITSPERPEL, DM_PELSHEIGHT,
                         DM_PELSWIDTH};
use winapi::um::libloaderapi::GetModuleHandleW;
use std::mem::{size_of, transmute, zeroed};
use std::ptr::{null, null_mut};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use super::application::Application;
use super::events::Event;

pub struct Window {
    pub screen_width: i32,
    pub screen_height: i32,
    pub instance: HINSTANCE,
    pub window: HWND,
}

extern "system" fn wnd_process(
    window: HWND,
    message: UINT,
    param1: WPARAM,
    param2: LPARAM,
) -> LRESULT {
    let mut app_ptr: *mut Application =
        unsafe { transmute(GetWindowLongPtrW(window, GWLP_USERDATA)) };
    if WM_CREATE == message {
        let create_structure: &mut CREATESTRUCTW = unsafe { transmute(param2) };
        app_ptr = unsafe { transmute(create_structure.lpCreateParams) };
        unsafe {
            SetWindowLongPtrW(window, GWLP_USERDATA, transmute(app_ptr));
        }
    }
    if app_ptr == null_mut() {
        eprintln!(
            "Unexpected message for nullptr sys app uMsg is: {}",
            message
        );
        return unsafe { DefWindowProcW(window, message, param1, param2) };
    }
    let app: &mut Application = unsafe { transmute(app_ptr) };
    match message {
        WM_QUIT => {
            unsafe {
                DestroyWindow(window);
                PostQuitMessage(0);
            }
            app.handle(Event::Quit);
        }
        WM_KEYDOWN => {
            if param1 == VK_ESCAPE as WPARAM {
                unsafe {
                    DestroyWindow(window);
                    PostQuitMessage(0);
                }
                app.handle(Event::Quit);
            }
        }
        WM_SHOWWINDOW => {
            println!("windows shown");
        }
        _ => {
            eprintln!("Unhandled window message");
        }
    }
    return unsafe { DefWindowProcW(window, message, param1, param2) };
}

impl Window {
    pub fn new(appptr: *mut Application) {
        let app: &mut Application = unsafe { transmute(appptr) };
        let mut wc: WNDCLASSEXW = unsafe { zeroed() };
        let application_name: Vec<u16> = OsStr::new("Dariush")
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect();
        let application_name = application_name.as_ptr();
        app.window.instance = unsafe { GetModuleHandleW(null()) } as HINSTANCE;
        wc.style = CS_HREDRAW | CS_VREDRAW | CS_OWNDC;
        wc.lpfnWndProc = Some(wnd_process);
        wc.hInstance = app.window.instance;
        wc.hbrBackground = unsafe { GetStockObject(BLACK_BRUSH as i32) } as HBRUSH;
        wc.lpszClassName = application_name;
        wc.cbSize = size_of::<WNDCLASSEXW>() as u32;
        unsafe {
            RegisterClassExW(&wc);
        }
        app.window.screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) } as i32;
        app.window.screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) } as i32;
        let mut screen_settings: DEVMODEW = unsafe { zeroed() };
        screen_settings.dmSize = size_of::<DEVMODEW>() as u16;
        screen_settings.dmPelsWidth = app.window.screen_width as u32;
        screen_settings.dmPelsHeight = app.window.screen_height as u32;
        screen_settings.dmBitsPerPel = 32;
        screen_settings.dmFields = DM_BITSPERPEL | DM_PELSWIDTH | DM_PELSHEIGHT;
        unsafe {
            ChangeDisplaySettingsW(&mut screen_settings, CDS_FULLSCREEN);
        }
        app.window.window = unsafe {
            CreateWindowExW(
                WS_EX_APPWINDOW,
                application_name,
                application_name,
                WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_POPUP,
                0,
                0,
                app.window.screen_width,
                app.window.screen_height,
                null_mut(),
                null_mut(),
                app.window.instance,
                transmute(appptr),
            )
        };
        unsafe {
            ShowWindow(app.window.window, SW_SHOW);
            SetForegroundWindow(app.window.window);
            SetFocus(app.window.window);
            UpdateWindow(app.window.window);
        }
    }

    pub fn main_loop(&mut self) {
        let mut msg: MSG = unsafe { zeroed() };
        loop {
            while unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) } != 0 {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
}
