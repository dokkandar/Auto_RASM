// RSM — RUST_CAD's native binary Document format.
//
// Placeholder; Slice I fills this in.

use cad_kernel::Document;

pub fn read_rsm(_bytes: &[u8]) -> Result<Document, String> {
    Err("RSM reader not implemented yet (Slice I)".into())
}

pub fn write_rsm(_doc: &Document) -> Vec<u8> {
    // Empty placeholder — Slice I lands the real format.
    Vec::new()
}
