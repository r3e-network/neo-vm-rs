use super::runtime_types::{
    structurally_equal, to_abi_stack, to_abi_value, CompoundIds, StackValue,
};
use core::cell::UnsafeCell;

const POST_SYSCALL_STACK_HEADROOM: usize = 8;
const RETAINED_PREFIX_BUF_SIZE: usize = 2 * 1024 * 1024;
const MAX_RETAINED_DECODE_DEPTH: usize = 64;
const MAX_RETAINED_COLLECTION_LEN: usize = 4096;
const MAX_INTEGER_SIZE: usize = 32;

const RETAINED_TAG_INTEGER: u8 = 0x01;
const RETAINED_TAG_BIGINTEGER: u8 = 0x02;
const RETAINED_TAG_BYTESTRING: u8 = 0x03;
const RETAINED_TAG_BOOLEAN: u8 = 0x04;
const RETAINED_TAG_ARRAY: u8 = 0x05;
const RETAINED_TAG_STRUCT: u8 = 0x06;
const RETAINED_TAG_MAP: u8 = 0x07;
const RETAINED_TAG_INTEROP: u8 = 0x08;
const RETAINED_TAG_ITERATOR: u8 = 0x09;
const RETAINED_TAG_NULL: u8 = 0x0A;
const RETAINED_TAG_POINTER: u8 = 0x0B;
const RETAINED_TAG_BUFFER: u8 = 0x0C;

pub(crate) struct RetainedPrefixBuffer(UnsafeCell<[u8; RETAINED_PREFIX_BUF_SIZE]>);

unsafe impl Sync for RetainedPrefixBuffer {}

impl RetainedPrefixBuffer {
    const fn new() -> Self {
        Self(UnsafeCell::new([0; RETAINED_PREFIX_BUF_SIZE]))
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn as_mut_slice(&self) -> &mut [u8] {
        &mut *self.0.get()
    }

    pub(crate) unsafe fn as_slice(&self, len: usize) -> &[u8] {
        &(&*self.0.get())[..len]
    }
}

static RETAINED_STACK_BUF: RetainedPrefixBuffer = RetainedPrefixBuffer::new();
pub(crate) static RETAINED_ARGS_BUF: RetainedPrefixBuffer = RetainedPrefixBuffer::new();
static RETAINED_LOCALS_BUF: RetainedPrefixBuffer = RetainedPrefixBuffer::new();
static RETAINED_STATIC_FIELDS_BUF: RetainedPrefixBuffer = RetainedPrefixBuffer::new();
static RETAINED_CONSUMED_MUTATIONS_BUF: RetainedPrefixBuffer = RetainedPrefixBuffer::new();
#[cfg(target_arch = "riscv32")]
static RETAINED_INITIAL_STACK_BUF: RetainedPrefixBuffer = RetainedPrefixBuffer::new();
#[cfg(target_arch = "riscv32")]
pub(crate) static RETAINED_CALL_STACK_BUF: RetainedPrefixBuffer = RetainedPrefixBuffer::new();

use super::SyscallProvider;
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use num_bigint::BigInt;
use num_traits::Zero;

mod bridge;
mod retained;
mod values;

pub(crate) use bridge::*;
pub(crate) use retained::*;
pub(crate) use values::*;
