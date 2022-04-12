use winapi::shared::minwindef::{LPARAM, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageA, GetMessageA, SendInput, SetWindowsHookExA, TranslateMessage,
    INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT, KEYEVENTF_KEYUP, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL,
    WM_KEYUP, WM_SYSKEYUP,
};

#[repr(u32)]
enum KEYCODE {
    CapsLock = 0x14,
    LCtrl = 0xA2,
}

static mut HOOK: HHOOK = std::ptr::null_mut();

unsafe extern "system" fn hook_callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> isize {
    let kb_struct = *(lparam as *const KBDLLHOOKSTRUCT);

    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: Default::default(),
    };
    const INPUT_SIZE: i32 = std::mem::size_of::<INPUT>() as i32;
    let ki = input.u.ki_mut();

    if wparam == WM_KEYUP as usize || wparam == WM_SYSKEYUP as usize {
        ki.dwFlags = KEYEVENTF_KEYUP;
    }

    let key: KEYCODE = std::mem::transmute(kb_struct.vkCode);
    let injected = kb_struct.flags & LLKHF_INJECTED == 0;
    match (key, injected) {
        (key @ (KEYCODE::CapsLock | KEYCODE::LCtrl), true) => {
            ki.wVk = match key {
                KEYCODE::CapsLock => KEYCODE::LCtrl as u16,
                KEYCODE::LCtrl => KEYCODE::CapsLock as u16,
            };
            SendInput(1, &mut input, INPUT_SIZE);

            1
        }
        _ => CallNextHookEx(HOOK, code, wparam, lparam),
    }
}

fn main() {
    unsafe {
        HOOK = SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), std::ptr::null_mut(), 0);

        if !HOOK.is_null() {
            let mut message: MSG = Default::default();
            while GetMessageA(&mut message, std::ptr::null_mut(), 0, 0) == 1 {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
        }
    }
}
