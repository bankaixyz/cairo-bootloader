mod bootloader_hints;
mod codes;
mod execute_task_hints;
mod fact_topologies;
mod hint_processors;
mod inner_select_builtins;
mod load_cairo_pie;
mod program_hash;
mod program_loader;
mod select_builtins;
mod simple_bootloader_hints;
mod types;
mod vars;

use std::collections::HashMap;

use cairo_vm::{hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData, types::exec_scope::ExecutionScopes, vm::{errors::hint_errors::HintError, vm_core::VirtualMachine}, Felt252};
pub use types::{
    BootloaderConfig, BootloaderInput, CairoPiePath, CairoPieTask, PackedOutput, RunProgramTask,
    SimpleBootloaderInput, Task, TaskSpec,
};

pub use vars::BOOTLOADER_INPUT;

use crate::{bootloader_hints::{assert_is_composite_packed_output, assert_program_address, compute_and_configure_fact_topologies, enter_packed_output_scope, guess_pre_image_of_subtasks_output_hash, import_packed_output_schemas, is_plain_packed_output, load_bootloader_config, prepare_simple_bootloader_input, prepare_simple_bootloader_output_segment, restore_bootloader_output, save_output_pointer, save_packed_outputs, set_packed_output_to_subtasks}, codes::{BOOTLOADER_ASSERT_IS_COMPOSITE_PACKED_OUTPUT, BOOTLOADER_COMPUTE_FACT_TOPOLOGIES, BOOTLOADER_ENTER_PACKED_OUTPUT_SCOPE, BOOTLOADER_GUESS_PRE_IMAGE_OF_SUBTASKS_OUTPUT_HASH, BOOTLOADER_IMPORT_PACKED_OUTPUT_SCHEMAS, BOOTLOADER_IS_PLAIN_PACKED_OUTPUT, BOOTLOADER_LOAD_BOOTLOADER_CONFIG, BOOTLOADER_PREPARE_SIMPLE_BOOTLOADER_INPUT, BOOTLOADER_PREPARE_SIMPLE_BOOTLOADER_OUTPUT_SEGMENT, BOOTLOADER_RESTORE_BOOTLOADER_OUTPUT, BOOTLOADER_SAVE_OUTPUT_POINTER, BOOTLOADER_SAVE_PACKED_OUTPUTS, BOOTLOADER_SET_PACKED_OUTPUT_TO_SUBTASKS, EXECUTE_TASK_ALLOCATE_PROGRAM_DATA_SEGMENT, EXECUTE_TASK_APPEND_FACT_TOPOLOGIES, EXECUTE_TASK_ASSERT_PROGRAM_ADDRESS, EXECUTE_TASK_CALL_TASK, EXECUTE_TASK_EXIT_SCOPE, EXECUTE_TASK_LOAD_PROGRAM, EXECUTE_TASK_VALIDATE_HASH_V0_13_0, EXECUTE_TASK_VALIDATE_HASH_V0_13_1, EXECUTE_TASK_WRITE_RETURN_BUILTINS, INNER_SELECT_BUILTINS_SELECT_BUILTIN, SELECT_BUILTINS_ENTER_SCOPE, SIMPLE_BOOTLOADER_DIVIDE_NUM_BY_2, SIMPLE_BOOTLOADER_PREPARE_TASK_RANGE_CHECKS, SIMPLE_BOOTLOADER_SET_CURRENT_TASK, SIMPLE_BOOTLOADER_SET_TASKS_VARIABLE, SIMPLE_BOOTLOADER_ZERO_V0_13_0, SIMPLE_BOOTLOADER_ZERO_V0_13_1}, execute_task_hints::{allocate_program_data_segment, append_fact_topologies, call_task, exit_scope_with_comments, load_program_hint, validate_hash, write_return_builtins_hint}, inner_select_builtins::select_builtin, select_builtins::select_builtins_enter_scope, simple_bootloader_hints::{divide_num_by_2, prepare_task_range_checks, set_ap_to_zero, set_ap_to_zero_or_one, set_current_task, set_tasks_variable}};


pub type HintImpl = fn(
    &mut VirtualMachine,
    &mut ExecutionScopes,
    &HintProcessorData,
    &HashMap<String, Felt252>,
) -> Result<(), HintError>;

pub fn get_hints() -> HashMap<String, HintImpl> {
    let mut hints = HashMap::<String, HintImpl>::new();
    hints.insert(
        BOOTLOADER_RESTORE_BOOTLOADER_OUTPUT.into(),
        restore_bootloader_output
    );
    hints.insert(
        BOOTLOADER_PREPARE_SIMPLE_BOOTLOADER_INPUT.into(),
        prepare_simple_bootloader_input
    );
    hints.insert(
        BOOTLOADER_LOAD_BOOTLOADER_CONFIG.into(),
        load_bootloader_config
    );
    hints.insert(
        BOOTLOADER_ENTER_PACKED_OUTPUT_SCOPE.into(),
        enter_packed_output_scope
    );
    hints.insert(
        BOOTLOADER_SAVE_OUTPUT_POINTER.into(),
        save_output_pointer
    );
    hints.insert(
        BOOTLOADER_SAVE_PACKED_OUTPUTS.into(),
        save_packed_outputs
    );
    hints.insert(
        BOOTLOADER_GUESS_PRE_IMAGE_OF_SUBTASKS_OUTPUT_HASH.into(),
        guess_pre_image_of_subtasks_output_hash
    );
    hints.insert(
        BOOTLOADER_PREPARE_SIMPLE_BOOTLOADER_OUTPUT_SEGMENT.into(),
        prepare_simple_bootloader_output_segment
    );
    hints.insert(
        BOOTLOADER_COMPUTE_FACT_TOPOLOGIES.into(),
        compute_and_configure_fact_topologies   
    );
    hints.insert(
        BOOTLOADER_SET_PACKED_OUTPUT_TO_SUBTASKS.into(),
        set_packed_output_to_subtasks
    );
    hints.insert(
        BOOTLOADER_IMPORT_PACKED_OUTPUT_SCHEMAS.into(),
        import_packed_output_schemas
    );
    hints.insert(
        BOOTLOADER_IS_PLAIN_PACKED_OUTPUT.into(),
        is_plain_packed_output
    );
    hints.insert(
        BOOTLOADER_ASSERT_IS_COMPOSITE_PACKED_OUTPUT.into(),
        assert_is_composite_packed_output
    );
    hints.insert(
        SIMPLE_BOOTLOADER_PREPARE_TASK_RANGE_CHECKS.into(),
        prepare_task_range_checks
    );
    hints.insert(
        SIMPLE_BOOTLOADER_SET_TASKS_VARIABLE.into(),
        set_tasks_variable
    );
    hints.insert(
        SIMPLE_BOOTLOADER_DIVIDE_NUM_BY_2.into(),
        divide_num_by_2
    );
    hints.insert(
        SIMPLE_BOOTLOADER_SET_CURRENT_TASK.into(),
        set_current_task
    );
    hints.insert(
        SIMPLE_BOOTLOADER_ZERO_V0_13_0.into(),
        set_ap_to_zero
    );
    hints.insert(
        SIMPLE_BOOTLOADER_ZERO_V0_13_1.into(),
        set_ap_to_zero_or_one
    );
    hints.insert(
        EXECUTE_TASK_ALLOCATE_PROGRAM_DATA_SEGMENT.into(),
        allocate_program_data_segment
    );
    hints.insert(
        EXECUTE_TASK_LOAD_PROGRAM.into(),
        load_program_hint
    );
    hints.insert(
        EXECUTE_TASK_VALIDATE_HASH_V0_13_0.into(),
        validate_hash
    );
    hints.insert(
        EXECUTE_TASK_VALIDATE_HASH_V0_13_1.into(),
        validate_hash
    );
    hints.insert(
        EXECUTE_TASK_ASSERT_PROGRAM_ADDRESS.into(),
        assert_program_address
    );
    hints.insert(
        EXECUTE_TASK_CALL_TASK.into(),
        call_task
    );
    hints.insert(
        EXECUTE_TASK_WRITE_RETURN_BUILTINS.into(),
        write_return_builtins_hint
    );
    hints.insert(
        EXECUTE_TASK_APPEND_FACT_TOPOLOGIES.into(),
        append_fact_topologies
    );
    hints.insert(
        EXECUTE_TASK_EXIT_SCOPE.into(),
        exit_scope_with_comments
    );
    hints.insert(
        SELECT_BUILTINS_ENTER_SCOPE.into(),
        select_builtins_enter_scope
    );
    hints.insert(
        INNER_SELECT_BUILTINS_SELECT_BUILTIN.into(),
        select_builtin
    );

    
    hints
}