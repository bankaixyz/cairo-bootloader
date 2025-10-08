use cairo_bootloader_hints::types::{CairoPieTask, RunProgramTask, TaskSpec};
use cairo_lang_casm::hints::Hint;
use cairo_lang_executable::executable::{EntryPointKind, Executable, ExecutableEntryPoint};
use cairo_lang_runner::{build_hints_dict, Arg, CairoHintProcessor};
use cairo_vm::types::errors::program_errors::ProgramError;
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
) -> Result<(TaskSpec, HashMap<String, Hint>), BootloaderTaskError> {
    // let program = get_program_from_executable(program);
    let executable: Executable = serde_json::from_reader(std::fs::File::open(program).unwrap())
        .expect("Failed to read executable");

    let entrypoint = executable
        .entrypoints
        .iter()
        .find(|e| matches!(e.kind, EntryPointKind::Bootloader))
        .expect("Failed to find entrypoint");

    let (program, string_to_hint) = program_and_hints_from_executable(&executable, entrypoint);

    let program_input_map = if let Some(input) = program_input {
        if let serde_json::Value::Object(map) = input {
            map.into_iter()
                .map(|(k, v)| (k, v))
                .collect::<HashMap<String, serde_json::Value>>()
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    let task = TaskSpec::RunProgram(RunProgramTask {
        program,
        // program_input: program_input.unwrap().into_object().unwrap(),
        program_input: HashMap::new(),
        use_poseidon: false,
    });
    Ok((task, string_to_hint))
}

pub fn program_and_hints_from_executable(
    executable: &Executable,
    entrypoint: &ExecutableEntryPoint,
) -> (Program, HashMap<String, Hint>) {
    let data: Vec<MaybeRelocatable> = executable
        .program
        .bytecode
        .iter()
        .map(Felt252::from)
        .map(MaybeRelocatable::from)
        .collect();
    let (hints, string_to_hint) = build_hints_dict(&executable.program.hints);
    let program = match entrypoint.kind {
        EntryPointKind::Standalone => Program::new_for_proof(
            entrypoint.builtins.clone(),
            data,
            entrypoint.offset,
            entrypoint.offset + 4,
            hints,
            Default::default(),
            Default::default(),
            vec![],
            None,
        ),
        EntryPointKind::Bootloader => Program::new(
            entrypoint.builtins.clone(),
            data,
            Some(entrypoint.offset),
            hints,
            Default::default(),
            Default::default(),
            vec![],
            None,
        ),
    };

    (program.unwrap(), string_to_hint)
}
