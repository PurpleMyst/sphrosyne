//! An opinionated client for the [Virtual Gamepad Emulation Framework](https://vigem.org/) utilizing [ViGEmClient](https://github.com/ViGEm/ViGEmClient/)

#![warn(
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    or_patterns_back_compat,
    pointer_structural_match,
    semicolon_in_expressions_from_macros,
    single_use_lifetimes,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]

pub mod client;
pub mod error;
pub mod gamepad_state;

pub use client::Client;
pub use error::*;
pub use gamepad_state::*;
