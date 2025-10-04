use cairo_bootloader_hints::types::{CairoPieTask, RunProgramTask, TaskSpec};
use cairo_vm::types::errors::program_errors::ProgramError;
use cairo_lang_executable::executable::{EntryPointKind, Executable};
use cairo_lang_runner::{Arg, CairoHintProcessor, build_hints_dict};

use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::runners::cairo_pie::CairoPie;
use cairo_vm::Felt252;
use std::collections::HashMap;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum BootloaderTaskError {
    #[error("Failed to read program: {0}")]
    Program(#[from] ProgramError),

    #[error("Failed to read PIE: {0}")]
    Pie(#[from] std::io::Error),
}

pub fn make_bootloader_tasks(
    program: &Path,
    program_input: Option<serde_json::Value>,
) -> Result<TaskSpec, BootloaderTaskError> {
    let program = get_program_from_executable(program);

    let task = TaskSpec::RunProgram(RunProgramTask {
        program,
        // program_input: program_input.unwrap().into_object().unwrap(),
        program_input: HashMap::new(),
        use_poseidon: true,
    });
    Ok(task)
}

fn get_program_from_executable(program: &Path) -> Program {
    let executable: Executable = serde_json::from_reader(std::fs::File::open(program).unwrap())
        .expect("Failed to read executable");

    let data: Vec<MaybeRelocatable> = executable
        .program
        .bytecode
        .iter()
        .map(Felt252::from)
        .map(MaybeRelocatable::from)
        .collect();
    let (hints, _) = build_hints_dict(&executable.program.hints);
    let entrypoint = executable
        .entrypoints
        .iter()
        .find(|e| matches!(e.kind, EntryPointKind::Standalone))
        .expect("Failed to find entrypoint");

    let program = Program::new_for_proof(
        entrypoint.builtins.clone(),
        data,
        entrypoint.offset,
        entrypoint.offset + 4,
        hints,
        Default::default(),
        Default::default(),
        vec![],
        None,
    )
    .unwrap();
    program
}