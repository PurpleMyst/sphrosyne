#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(test, allow(deref_nullptr, unaligned_references))]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
