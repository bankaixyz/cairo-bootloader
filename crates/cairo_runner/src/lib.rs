use cairo_bootloader_hints::{types::BootloaderInput, vars::BOOTLOADER_INPUT};
use cairo_vm::types::exec_scope::ExecutionScopes;

pub mod bootloaders;
pub mod hint_processor;
pub mod task;

/// Inserts the bootloader input in the execution scopes.
pub fn insert_bootloader_input(
    exec_scopes: &mut ExecutionScopes,
    bootloader_input: BootloaderInput,
) {
    exec_scopes.insert_value(BOOTLOADER_INPUT, bootloader_input);
}