use std::{any::Any, collections::HashMap, rc::Rc};

use cairo_bootloader_hints::{ExtensiveHintImpl, HintImpl};
use cairo_lang_casm::{hints::{ExternalHint, Hint}, operand::ResOperand};
use cairo_lang_runner::{casm_run::{cell_ref_to_relocatable, extract_relocatable, get_val}, Arg};
use cairo_vm::{
    any_box, hint_processor::{
        builtin_hint_processor::builtin_hint_processor_definition::{
            BuiltinHintProcessor, HintProcessorData,
        },
        cairo_1_hint_processor::hint_processor::Cairo1HintProcessor,
        hint_processor_definition::{HintExtension, HintProcessorLogic, HintReference},
    }, serde::deserialize_program::ApTracking, types::{exec_scope::ExecutionScopes, relocatable::Relocatable}, vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError}, runners::cairo_runner::ResourceTracker,
        vm_core::VirtualMachine,
    }, Felt252
};

use num_traits::{Signed, ToPrimitive, Zero};
pub struct BootloaderHintProcessor {
    builtin_hint_proc: BuiltinHintProcessor,
    cairo1_builtin_hint_proc: Cairo1HintProcessor,
    hints: HashMap<String, HintImpl>,
    extensive_hints: HashMap<String, ExtensiveHintImpl>,
    external_hint_proc: ExternalHintProcessor,
    pub string_to_hint: HashMap<String, Hint>,
}

impl BootloaderHintProcessor {
    pub fn new(string_to_hint: HashMap<String, Hint>, user_args: Vec<Vec<Arg>>) -> Self {
        Self {
            builtin_hint_proc: BuiltinHintProcessor::new_empty(),
            cairo1_builtin_hint_proc: Cairo1HintProcessor::new(Default::default(), Default::default(), false),
            hints: Self::hints(),
            extensive_hints: Self::extensive_hints(),
            string_to_hint,
            external_hint_proc: ExternalHintProcessor::new(user_args),
        }
    }

    fn hints() -> HashMap<String, HintImpl> {
        let mut hints = HashMap::new();
        hints.extend(cairo_bootloader_hints::get_hints());
        hints
    }

    fn extensive_hints() -> HashMap<String, ExtensiveHintImpl> {
        let mut hints = HashMap::new();
        hints.extend(cairo_bootloader_hints::get_extensive_hints());
        hints
    }
}

impl HintProcessorLogic for BootloaderHintProcessor {
    fn execute_hint(
        &mut self,
        _vm: &mut VirtualMachine,
        _exec_scopes: &mut ExecutionScopes,
        _hint_data: &Box<dyn Any>,
        _constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        unreachable!();
    }

    fn execute_hint_extensive(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<HintExtension, HintError> {
        // // // If this is a Cairo 1 hint (cairo_lang_casm::hints::Hint), execute it directly
        if let Some(hint) = hint_data.downcast_ref::<Hint>() {
            match hint {
                Hint::External(external_hint) => {
                    println!("Cairo 1 External Hint: {:?}", hint);
                    let r = self.external_hint_proc.execute_external_hint(vm, external_hint)?;
                    // println!("Cairo 1 External Hint Result: {:?}", r);
                    return Ok(HintExtension::default());
                }
                _ => {
                    println!("Cairo 1 Builtin Hint: {:?}", hint);
                    let r = self.cairo1_builtin_hint_proc
                        .execute(vm, exec_scopes, hint).map(|_| HintExtension::default());
                    // println!("Cairo 1 Builtin Hint Result: {:?}", r);
                    return r;
                }
            }
        }

        if let Some(hpd) = hint_data.downcast_ref::<HintProcessorData>() {
            let hint_code: &str = hpd.code.as_str();
            if let Some(hint_impl) = self.hints.get(hint_code) {
                println!("Custom Hint: {:?}", hint_code);
                return hint_impl(vm, exec_scopes, hpd, &HashMap::new())
                    .map(|_| HintExtension::default());
            }

            if let Some(hint_impl) = self.extensive_hints.get(hint_code) {
                let r = hint_impl(
                    &mut self.builtin_hint_proc,
                    &self.string_to_hint,
                    vm,
                    exec_scopes,
                    hpd,
                    &HashMap::new(),
                );
                return r;
            }

            return self
                .builtin_hint_proc
                .execute_hint_extensive(vm, exec_scopes, hint_data, constants)
                .map(|_| HintExtension::default());
        }

        Err(HintError::WrongHintData)
    }
}

impl ResourceTracker for BootloaderHintProcessor {}


pub struct ExternalHintProcessor {
    pub user_args: Vec<Vec<Arg>>,
    markers: Vec<Vec<Felt252>>,
    panic_traceback: Vec<(Relocatable, Relocatable)>,
}

impl ExternalHintProcessor {
    pub fn new(user_args: Vec<Vec<Arg>>) -> Self {
        Self {
            user_args,
            markers: Vec::new(),
            panic_traceback: Vec::new(),
        }
    }

    fn execute_external_hint(
        &mut self,
        vm: &mut VirtualMachine,
        core_hint: &ExternalHint,
    ) -> Result<(), HintError> {
        match core_hint {
            ExternalHint::AddRelocationRule { src, dst } => vm.add_relocation_rule(
                extract_relocatable(vm, src)?,
                // The following is needed for when the `extensive_hints` feature is used in the
                // VM, in which case `dst_ptr` is a `MaybeRelocatable` type.
                #[allow(clippy::useless_conversion)]
                {
                    extract_relocatable(vm, dst)?.into()
                },
            )?,
            ExternalHint::WriteRunParam { index, dst } => {
                let index = get_val(vm, index)?.to_usize().expect("Got a bad index.");
                let mut stack = vec![(cell_ref_to_relocatable(dst, vm), &self.user_args[index])];
                while let Some((mut buffer, values)) = stack.pop() {
                    for value in values {
                        match value {
                            Arg::Value(v) => {
                                vm.insert_value(buffer, v)?;
                                buffer += 1;
                            }
                            Arg::Array(arr) => {
                                let arr_buffer = vm.add_memory_segment();
                                stack.push((arr_buffer, arr));
                                vm.insert_value(buffer, arr_buffer)?;
                                buffer += 1;
                                vm.insert_value(buffer, (arr_buffer + Self::args_size(arr))?)?;
                                buffer += 1;
                            }
                        }
                    }
                }
            }
            ExternalHint::AddMarker { start, end } => {
                self.markers.push(Self::read_felts(vm, start, end)?);
            }
            ExternalHint::AddTrace { flag } => {
                let flag = get_val(vm, flag)?;
                // Setting the panic backtrace if the given flag is panic.
                if flag == 0x70616e6963u64.into() {
                    let mut fp = vm.get_fp();
                    self.panic_traceback = vec![(vm.get_pc(), fp)];
                    // Fetch the fp and pc traceback entries
                    loop {
                        let ptr_at_offset = |offset: usize| {
                            (fp - offset).ok().and_then(|r| vm.get_relocatable(r).ok())
                        };
                        // Get return pc.
                        let Some(ret_pc) = ptr_at_offset(1) else {
                            break;
                        };
                        println!("ret_pc: {ret_pc}");
                        // Get fp traceback.
                        let Some(ret_fp) = ptr_at_offset(2) else {
                            break;
                        };
                        println!("ret_fp: {ret_fp}");
                        if ret_fp == fp {
                            break;
                        }
                        fp = ret_fp;

                        let call_instruction = |offset: usize| -> Option<Relocatable> {
                            let ptr = (ret_pc - offset).ok()?;
                            println!("ptr: {ptr}");
                            let inst = vm.get_integer(ptr).ok()?;
                            println!("inst: {inst}");
                            let inst_short = inst.to_u64()?;
                            (inst_short & 0x7000_0000_0000_0000 == 0x1000_0000_0000_0000)
                                .then_some(ptr)
                        };
                        if let Some(call_pc) = call_instruction(1).or_else(|| call_instruction(2)) {
                            self.panic_traceback.push((call_pc, fp));
                        } else {
                            break;
                        }
                    }
                    self.panic_traceback.reverse();
                }
            }
        }
        Ok(())
    }

    /// Reads a range of `Felt252`s from the VM.
    fn read_felts(
        vm: &mut VirtualMachine,
        start: &ResOperand,
        end: &ResOperand,
    ) -> Result<Vec<Felt252>, HintError> {
        let mut curr = extract_relocatable(vm, start)?;
        let end = extract_relocatable(vm, end)?;

        let mut felts = Vec::new();
        while curr != end {
            let value = *vm.get_integer(curr)?;
            felts.push(value);
            curr = (curr + 1)?;
        }

        Ok(felts)
    }

    /// The size in memory of the arguments.
    fn args_size(args: &[Arg]) -> usize {
        args.iter().map(Arg::size).sum()
    }
}