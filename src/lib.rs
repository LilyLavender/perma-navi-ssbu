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

mod younglink;

#[skyline::main(name = "perma-navi-ssbu")]
pub fn main() {
    younglink::install();
}