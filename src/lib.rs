#![feature(asm)]
#![feature(repr_simd)]

pub mod crq;
pub mod lcrq;
pub mod flag_and_u63; // TODO: Using `pub` only to suppress unused warnings
pub mod node; // TODO: Using `pub` only to suppress unused warnings
mod atomics;
