use neo_vm_rs::{StackValue, VmState};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Vector {
    pub(super) name: String,
    pub(super) description: String,
    pub(super) script_hex: String,
    pub(super) expected_state: VmState,
    pub(super) expected_stack: Vec<StackValue>,
}
