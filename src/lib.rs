extern crate winapi;

use winapi::um::{
    libloaderapi::{FreeLibraryAndExitThread, GetModuleHandleA},
    psapi::{MODULEINFO, GetModuleInformation},
    processthreadsapi::GetCurrentProcess,
};
use winapi::shared::minwindef::*;
use fltk::{app, 
    button::{Button, ToggleButton}, 
    frame::Frame, prelude::*, 
    window::Window, 
    enums::{Color, Align},
    input::IntInput,
};
use std::{
    mem,
    ffi::CString,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};
use byteorder::{LittleEndian, WriteBytesExt};

const HEALTHOFFSET : usize = 0x04768E78;
const LEVELOFFSET  : usize = 0x04740178;
const SOULOFFSET   : usize = 0x04740178;
const FPOFFSET     : usize = 0x04768E78;
const STAMINAOFFSET: usize = 0x04768E78;

/// Use the offsets to traverse multi-level pointers starting at {ptr}
unsafe fn bypass_dma(ptr: usize, offsets: Vec<usize>) -> usize {
    let mut addr = ptr;
    for offset in offsets {
        addr = *(addr as *const usize);
        addr += offset;
    }
    addr
}

/// Write {val} to {ptr}, uses the {offsets} to traverse multi level pointers
unsafe fn write_mem(ptr: usize, val: u32, offsets: Vec<usize>) {
    let addr = bypass_dma(ptr, offsets);
    let mut val_vec = vec![0u8; 4];
    val_vec.write_u32::<LittleEndian>(val).unwrap();
    *(addr as *mut u32) = val;
}

/// Read value from {ptr}, uses the {offsets} to traverse multi level pointers
unsafe fn read_mem(ptr: usize, offsets: Vec<usize>) -> u32 {
    let addr = bypass_dma(ptr, offsets);
    *(addr as *mut u32)
}

#[allow(non_snake_case)]
/// Entry point that gets called once the dll is injected into Dark Souls
unsafe fn entry_point(base: winapi::shared::minwindef::LPVOID) -> u32 {
    let HASINFST = Rc::new(AtomicBool::new(false));
    let HASINFHP = Rc::new(AtomicBool::new(false));
    let HASINFFP = Rc::new(AtomicBool::new(false));

    // Get program base address
    let modname = CString::new("DarkSoulsIII.exe").unwrap();
    let mut m_info: MODULEINFO = MODULEINFO {
        EntryPoint: std::ptr::null_mut(),
        SizeOfImage: 0,
        lpBaseOfDll: std::ptr::null_mut(),
    };
    let size = mem::size_of::<MODULEINFO>() as u32;
    GetModuleInformation(GetCurrentProcess(), GetModuleHandleA(
            modname.as_c_str().as_ptr()), &mut m_info, size);

    let base_addr = m_info.lpBaseOfDll as usize;

    // Draw the gui window and create the buttons
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 500, "Dark Souls 3 Hack");

    let mut quit_button = Button::new(360, 0, 40, 40, "Quit");

    let mut godmode     = ToggleButton::new(20, 20 , 30, 30, "Off");
    let mut inf_stamina = ToggleButton::new(20, 60 , 30, 30, "Off");
    let mut inf_hp      = ToggleButton::new(20, 100, 30, 30, "Off");
    let mut inf_fp      = ToggleButton::new(20, 140, 30, 30, "Off");

    let mut health      = IntInput::new(20, 180, 30, 30, "");
    let mut level       = IntInput::new(20, 220, 30, 30, "");
    let mut souls       = IntInput::new(20, 260, 30, 30, "");
    let mut fp          = IntInput::new(20, 300, 30, 30, "");
    let mut stamina     = IntInput::new(20, 340, 30, 30, "");

    Frame::new(55, 20 , 0, 30, "GODMODE").with_align(Align::Right);
    Frame::new(55, 60 , 0, 30, "INFINITE STAMINA").with_align(Align::Right);
    Frame::new(55, 100, 0, 30, "INFINITE HEALTH").with_align(Align::Right);
    Frame::new(55, 140, 0, 30, "INFINITE FP").with_align(Align::Right);

    let mut health_frame  = Frame::new(55, 180, 0, 30, "").with_align(Align::Right);
    let mut level_frame   = Frame::new(55, 220, 0, 30, "").with_align(Align::Right);
    let mut souls_frame   = Frame::new(55, 260, 0, 30, "").with_align(Align::Right);
    let mut fp_frame      = Frame::new(55, 300, 0, 30, "").with_align(Align::Right);
    let mut stamina_frame = Frame::new(55, 340, 0, 30, "").with_align(Align::Right);

    wind.set_color(Color::White);
    wind.end();
    wind.show();

    // Set up the instructions that get executed whenever a button/field is used
    quit_button.set_callback(move |_| {
        app.quit();
        wind.clear();
        FreeLibraryAndExitThread(base as HMODULE, 0);
    });

    health.set_callback(move |e| {
        let val: u32 = e.value().parse().unwrap();
        write_mem(base_addr + HEALTHOFFSET, val, vec![0x40, 0x28, 0x3A0, 0x70, 0x98]);
        write_mem(base_addr + HEALTHOFFSET, val, vec![0x40, 0x28, 0x3A0, 0x70, 0x90]);
    });

    level.set_callback(move |e| {
        let val: u32 = e.value().parse().unwrap();
        write_mem(base_addr + LEVELOFFSET, val, vec![0x10, 0x70]);
    });

    souls.set_callback(move |e| {
        let val: u32 = e.value().parse().unwrap();
        write_mem(base_addr + SOULOFFSET, val, vec![0xDD4]);
    });

    fp.set_callback(move |e| {
        let val: u32 = e.value().parse().unwrap();
        write_mem(base_addr + FPOFFSET, val, vec![0x40, 0x28, 0x3A0, 0x70, 0xA4]);
        write_mem(base_addr + FPOFFSET, val, vec![0x40, 0x28, 0x3A0, 0x70, 0x9C]);
    });

    stamina.set_callback(move |e| {
        let val: u32 = e.value().parse().unwrap();
        write_mem(base_addr + STAMINAOFFSET, val, vec![0x40, 0x28, 0x3A0, 0x70, 0xB0]);
        write_mem(base_addr + STAMINAOFFSET, val, vec![0x40, 0x28, 0x3A0, 0x70, 0xA8]);
    });

    inf_stamina.set_callback({
        let HASINFST = HASINFST.clone();
        move |e| {
            if e.is_toggled() {
                e.set_label("On");
                HASINFST.store(true, Ordering::Relaxed);
                e.toggle(true);
                e.set_label("On");
            } else { 
                e.set_label("Off");
                HASINFST.store(false, Ordering::Relaxed);
                e.toggle(false);
                e.set_label("Off");
            }
        }
    });

    inf_hp.set_callback({
        let HASINFHP = HASINFHP.clone();
        move |e| {
            if e.is_toggled() {
                e.set_label("On");
                HASINFHP.store(true, Ordering::Relaxed);
                e.toggle(true);
                e.set_label("On");
            } else { 
                e.set_label("Off");
                HASINFHP.store(false, Ordering::Relaxed);
                e.toggle(false);
                e.set_label("Off");
            }
        }
    });

    inf_fp.set_callback({
        let HASINFFP = HASINFFP.clone();
        move |e| {
            if e.is_toggled() {
                e.set_label("On");
                HASINFFP.store(true, Ordering::Relaxed);
                e.toggle(true);
                e.set_label("On");
            } else { 
                e.set_label("Off");
                HASINFFP.store(false, Ordering::Relaxed);
                e.toggle(false);
                e.set_label("Off");
            }
        }
    });

    godmode.set_callback({
        let HASINFST = HASINFST.clone();
        let HASINFHP = HASINFHP.clone();
        let HASINFFP = HASINFFP.clone();
        move |e| {
            if e.is_toggled() {
                e.set_label("On"); 
                HASINFST.store(true, Ordering::Relaxed);
                HASINFHP.store(true, Ordering::Relaxed);
                HASINFFP.store(true, Ordering::Relaxed);
                inf_stamina.toggle(true);
                inf_stamina.set_label("On");
                inf_hp.toggle(true);
                inf_hp.set_label("On");
                inf_fp.toggle(true);
                inf_fp.set_label("On");
            } else {
                e.set_label("Off"); 
                HASINFST.store(false, Ordering::Relaxed);
                HASINFHP.store(false, Ordering::Relaxed);
                HASINFFP.store(false, Ordering::Relaxed);
                inf_stamina.toggle(false);
                inf_stamina.set_label("Off");
                inf_hp.toggle(false);
                inf_hp.set_label("Off");
                inf_fp.toggle(false);
                inf_fp.set_label("Off");
            }
        }
    });

    // Add functions that consistently get executed during event loop. These are mostly used
    // to read out values and provide users with realtime feedback about current values
    app::add_idle(move || {
        let val = read_mem(base_addr + HEALTHOFFSET, vec![0x40, 0x28, 0x3A0, 0x70, 0x90]);
        let new = format!("Set HEALTH (Current: {})", val);
        health_frame.set_label(&new);
    });

    app::add_idle(move || {
        let val = read_mem(base_addr + LEVELOFFSET, vec![0x10, 0x70]);
        let new = format!("Set LEVEL (Current: {})", val);
        level_frame.set_label(&new);
    });

    app::add_idle(move || {
        let val = read_mem(base_addr + SOULOFFSET, vec![0xDD4]);
        let new = format!("Set SOULS (Current: {})", val);
        souls_frame.set_label(&new);
    });

    app::add_idle(move || {
        let val = read_mem(base_addr + FPOFFSET, vec![0x40, 0x28, 0x3A0, 0x70, 0x9C]);
        let new = format!("Set FP (Current: {})", val);
        fp_frame.set_label(&new);
    });

    app::add_idle(move || {
        let val = read_mem(base_addr + STAMINAOFFSET, vec![0x40, 0x28, 0x3A0, 0x70, 0xA8]);
        let new = format!("Set STAMINA (Current: {})", val);
        stamina_frame.set_label(&new);
    });

    app::add_idle(move || {
        if HASINFST.load(Ordering::Relaxed) {
            write_mem(base_addr + STAMINAOFFSET, 9999, vec![0x40, 0x28, 0x3A0, 0x70, 0xB0]);
            write_mem(base_addr + STAMINAOFFSET, 9999, vec![0x40, 0x28, 0x3A0, 0x70, 0xA8]);
        }
        if HASINFHP.load(Ordering::Relaxed) {
            write_mem(base_addr + HEALTHOFFSET, 9999, vec![0x40, 0x28, 0x3A0, 0x70, 0x98]);
            write_mem(base_addr + HEALTHOFFSET, 9999, vec![0x40, 0x28, 0x3A0, 0x70, 0x90]);
        }
        if HASINFFP.load(Ordering::Relaxed) {
            write_mem(base_addr + FPOFFSET, 9999, vec![0x40, 0x28, 0x3A0, 0x70, 0xA4]);
            write_mem(base_addr + FPOFFSET, 9999, vec![0x40, 0x28, 0x3A0, 0x70, 0x9C]);
        }
    });

    app.run().unwrap();
    0
}

/// Small wrapper for the entry point
unsafe extern "system" fn dll_attach_wrapper(base: winapi::shared::minwindef::LPVOID) -> u32 {
    entry_point(base);
    0
}

/// DllMain is the main function that gets called when the dll is first attached
/// It creates a new thread to run the hack in
#[no_mangle]
pub unsafe extern "stdcall" fn DllMain(hinst_dll: HINSTANCE, fdw_reason: DWORD, 
                                _lpv_reserved: LPVOID) {
    if fdw_reason == winapi::um::winnt::DLL_PROCESS_ATTACH {
        winapi::um::processthreadsapi::CreateThread(std::ptr::null_mut(), 0, 
            Some(dll_attach_wrapper), hinst_dll as _, 0, std::ptr::null_mut());
    }
}
