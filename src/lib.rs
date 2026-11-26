#![feature(
    concat_idents,
    proc_macro_hygiene
)]
#![allow(
    unused_imports,
    non_upper_case_globals,
    non_snake_case,
    clippy::borrow_interior_mutable_const
)]

use {
    smash::{
        lua2cpp::*,
        phx::*,
        app::{sv_animcmd::*, lua_bind::*, *},
        lib::{lua_const::*, L2CValue, L2CAgent},
        hash40
    },
    smash_script::*,
    smashline::{*, Priority::*}
};
use table_const::*;
pub use variables::*;

pub static mut MARKED_COLORS: [bool; 256] = [false; 256];
pub static mut VOICE_COLORS: [bool; 256] = [false; 256];

pub fn check_deps() -> bool {
    let mut passed = true;

    for dep in [
        "rom:/skyline/plugins/libarcropolis.nro",
        "rom:/skyline/plugins/libnro_hook.nro",
        "rom:/skyline/plugins/libsmashline_plugin.nro",
    ] {
        if !std::path::Path::new(dep).is_file() {
            println!("{} not found! This installation is incomplete. Please download all dependencies listed in the README file.", dep);
            passed = false;
        }
    }

    passed
}

extern "C" fn mods_mounted(_ev: arcropolis_api::Event) {
    const FIGHTER_NAME: &str = "younglink";
    const MARKER_FILE: &str = "navi.marker";
    const VOICE_MARKER_FILE: &str = "navi_voice.marker";
    let mut any_marked = false;
    for x in 0..256 {
        // Marker
        if let Ok(_) = std::fs::read(format!(
            "mods:/fighter/{}/motion/navy/c{:02}/{}",
            FIGHTER_NAME, x, MARKER_FILE
        )) {
            unsafe {
                MARKED_COLORS[x as usize] = true;
            }
            any_marked = true;
        }

        // Voice marker
        if let Ok(_) = std::fs::read(format!(
            "mods:/fighter/{}/motion/navy/c{:02}/{}",
            FIGHTER_NAME, x, VOICE_MARKER_FILE
        )) {
            unsafe {
                VOICE_COLORS[x as usize] = true;
            }
        }
    }

    if !any_marked {
        return;
    }

    younglink::install();
}

pub unsafe fn voice_enabled(boma: *mut BattleObjectModuleAccessor) -> bool {
    let color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
    color >= 0 && color < 256 && VOICE_COLORS[color as usize]
}

pub fn marked_costumes() -> Vec<usize> {
    let mut slots = Vec::new();
    for i in 0..256 {
        if unsafe { MARKED_COLORS[i] } {
            slots.push(i);
        }
    }
    slots
}

mod younglink;
mod variables;

#[skyline::main(name = "perma-navi-ssbu")]
pub fn main() {
    if !check_deps() {
        return;
    }

    unsafe {
        extern "C" {
            fn arcrop_register_event_callback(
                ty: arcropolis_api::Event,
                callback: arcropolis_api::EventCallbackFn,
            );
        }

        arcrop_register_event_callback(arcropolis_api::Event::ModFilesystemMounted, mods_mounted);
    }
}
