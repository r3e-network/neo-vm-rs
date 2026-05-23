use serde::Deserialize;

use super::vector::Vector;

#[derive(Debug, Deserialize)]
pub(super) struct Suite {
    pub(super) neo_node_tag: String,
    pub(super) neo_package_version: String,
    pub(super) neo_vm_package_version: String,
    pub(super) source: String,
    pub(super) vectors: Vec<Vector>,
}
