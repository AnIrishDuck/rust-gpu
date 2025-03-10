#![no_std]
#![feature(register_attr, repr_simd, core_intrinsics, lang_items)]
#![cfg_attr(target_arch = "spirv", feature(asm))]
#![register_attr(spirv)]
// Our standard Clippy lints that we use in Embark projects, we opt out of a few that are not appropriate for the specific crate (yet)
#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::pub_enum_variant_names,
    clippy::mem_forget,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    //clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

pub mod storage_class;
mod textures;

pub use glam;
pub use num_traits;
pub use textures::*;

#[cfg(all(not(test), target_arch = "spirv"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(all(not(test), target_arch = "spirv"))]
#[lang = "eh_personality"]
extern "C" fn rust_eh_personality() {}

pub trait Derivative {
    fn ddx(self) -> Self;
    fn ddx_fine(self) -> Self;
    fn ddx_coarse(self) -> Self;
    fn ddy(self) -> Self;
    fn ddy_fine(self) -> Self;
    fn ddy_coarse(self) -> Self;
    fn fwidth(self) -> Self;
    fn fwidth_fine(self) -> Self;
    fn fwidth_coarse(self) -> Self;
}

#[cfg(target_arch = "spirv")]
macro_rules! deriv_caps {
    (true) => {
        asm!("OpCapability DerivativeControl")
    };
    (false) => {};
}

macro_rules! deriv_fn {
    ($name:ident, $inst:ident, $needs_caps:tt) => {
        fn $name(self) -> Self {
            #[cfg(not(target_arch = "spirv"))]
            panic!(concat!(stringify!($name), " is not supported on the CPU"));
            #[cfg(target_arch = "spirv")]
            unsafe {
                let o;
                deriv_caps!($needs_caps);
                asm!(
                    concat!("{1} = ", stringify!($inst), " typeof{0} {0}"),
                    in(reg) self,
                    out(reg) o,
                );
                o
            }
        }
    };
}
macro_rules! deriv_impl {
    ($ty:ty) => {
        impl Derivative for $ty {
            deriv_fn!(ddx, OpDPdx, false);
            deriv_fn!(ddx_fine, OpDPdxFine, true);
            deriv_fn!(ddx_coarse, OpDPdxCoarse, true);
            deriv_fn!(ddy, OpDPdy, false);
            deriv_fn!(ddy_fine, OpDPdyFine, true);
            deriv_fn!(ddy_coarse, OpDPdyCoarse, true);
            deriv_fn!(fwidth, OpFwidth, false);
            deriv_fn!(fwidth_fine, OpFwidthFine, true);
            deriv_fn!(fwidth_coarse, OpFwidthCoarse, true);
        }
    };
}

// "must be a scalar or vector of floating-point type. The component width must be 32 bits."
deriv_impl!(f32);
// TODO: Fix rustc to support these
// deriv_impl!(glam::Vec2);
// deriv_impl!(glam::Vec3);
// deriv_impl!(glam::Vec4);

/// libcore requires a few external symbols to be defined:
/// <https://github.com/rust-lang/rust/blob/c2bc344eb23d8c1d18e803b3f1e631cf99926fbb/library/core/src/lib.rs#L23-L27>
/// TODO: This is copied from `compiler_builtins/mem.rs`. Can we use that one instead? The note in the above link says
/// "[the symbols] can also be provided by the compiler-builtins crate". The memcpy in `compiler_builtins` is behind a
/// "mem" feature flag - can we enable that somehow?
/// <https://github.com/rust-lang/compiler-builtins/blob/eff506cd49b637f1ab5931625a33cef7e91fbbf6/src/mem.rs#L12-L13>
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }
    0
}
