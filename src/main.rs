#![windows_subsystem = "windows"]


use std::convert::TryInto;
use std::mem::size_of;

use windows::core::{BSTR, PCWSTR, PWSTR, w};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    CreateProcessW, CREATE_UNICODE_ENVIRONMENT, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::{IDYES, MB_ICONERROR, MB_OK, MB_YESNO, MessageBoxW};


fn collect_arg_string<S: AsRef<str>>(args: &[S]) -> String {
    let mut pieces: Vec<String> = Vec::new();

    // the docs for CommandLineToArgvW describe the opposite operation in more detail
    for arg in args {
        let arg_str = arg.as_ref();
        let mut need_quotes = false;
        let mut piece = String::with_capacity(arg_str.len());

        for c in arg_str.chars() {
            if c.is_whitespace() {
                need_quotes = true;
            }

            if c == '\\' || c == '"' {
                piece.push('\\');
            }
            piece.push(c);
        }

        if need_quotes {
            piece.push('"');
            piece.insert(0, '"');
        }

        pieces.push(piece);
    }

    pieces.join(" ")
}


fn main() {
    let args: Vec<String> = std::env::args()
        .collect();

    const REALLY_START: PCWSTR = w!("Really Start?");

    if args.len() < 3 {
        let usage_text: BSTR = format!("Usage: {} TITLE PROGRAM [ARG...]", args[0]).into();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(usage_text.as_wide().as_ptr()),
                REALLY_START,
                MB_ICONERROR | MB_OK,
            )
        };
        return;
    }

    let result = unsafe {
        let really_start_text: BSTR = format!("Really start {}?", args[1]).into();
        MessageBoxW(
            None,
            PCWSTR(really_start_text.as_wide().as_ptr()),
            REALLY_START,
            MB_YESNO,
        )
    };
    if result != IDYES {
        return;
    }

    // args[0]: reallystart.exe
    // args[1]: Notepad
    // args[2]: C:\Windows\notepad.exe
    // args[3]: C:\test.txt

    let app_name: BSTR = args[2].as_str().into();
    let mut command_line: Vec<u16> = collect_arg_string(&args[2..]).encode_utf16().collect();

    let mut startup_info = STARTUPINFOW::default();
    startup_info.cb = size_of::<STARTUPINFOW>().try_into()
        .expect("size doesn't fit into u32");
    let mut process_info = PROCESS_INFORMATION::default();
    let create_process_res = unsafe {
        CreateProcessW(
            PCWSTR(app_name.as_wide().as_ptr()),
            PWSTR(command_line.as_mut_ptr()),
            None,
            None,
            false,
            CREATE_UNICODE_ENVIRONMENT,
            None,
            None,
            &mut startup_info,
            &mut process_info,
        )
    };
    if let Err(e) = create_process_res {
        let message: BSTR = e.message().into();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(message.as_ptr()),
                REALLY_START,
                MB_ICONERROR | MB_OK,
            )
        };
        return;
    }

    let mut close_handle_result = unsafe { CloseHandle(process_info.hProcess) };
    if let Err(e) = close_handle_result {
        let message: BSTR = format!("Failed to close process handle: {}", e.message()).into();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(message.as_ptr()),
                REALLY_START,
                MB_ICONERROR | MB_OK,
            )
        };
    }
    close_handle_result = unsafe { CloseHandle(process_info.hThread) };
    if let Err(e) = close_handle_result {
        let message: BSTR = format!("Failed to close thread handle: {}", e.message()).into();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(message.as_ptr()),
                REALLY_START,
                MB_ICONERROR | MB_OK,
            )
        };
    }
}
