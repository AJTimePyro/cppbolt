//! Integration tests verifying C++ exception unwinding, scope lifetimes, and stack integrity.

pub(crate) mod common;

mod basic;
mod copy_move;
mod dispatch;
mod function_try;
mod lifecycle;
mod noexcept;
mod parameters;
mod stdlib;
mod temporaries;
mod unwinding;
