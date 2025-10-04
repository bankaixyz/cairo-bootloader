use std::{any::Any, collections::HashMap};

use cairo_bootloader_hints::{ExtensiveHintImpl, HintImpl};
use cairo_vm::{hint_processor::{builtin_hint_processor::builtin_hint_processor_definition::{BuiltinHintProcessor, HintProcessorData}, cairo_1_hint_processor::hint_processor::Cairo1HintProcessor, hint_processor_definition::{HintExtension, HintProcessorLogic}}, types::exec_scope::ExecutionScopes, vm::{errors::hint_errors::HintError, runners::cairo_runner::ResourceTracker, vm_core::VirtualMachine}, Felt252};
use cairo_lang_casm::hints::Hint;
pub struct BootloaderHintProcessor {
    builtin_hint_proc: BuiltinHintProcessor,
    cairo1_builtin_hint_proc: Cairo1HintProcessor,
    hints: HashMap<String, HintImpl>,
    extensive_hints: HashMap<String, ExtensiveHintImpl>,
}

impl BootloaderHintProcessor {
    pub fn new() -> Self {
        Self { 
            builtin_hint_proc: BuiltinHintProcessor::new_empty(),
            cairo1_builtin_hint_proc: Cairo1HintProcessor::new(Default::default(), Default::default(), true),
            hints: HashMap::new(),
            extensive_hints: HashMap::new(),
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
        if let Some(hpd) = hint_data.downcast_ref::<HintProcessorData>() {
            let hint_code = hpd.code.as_str();

            if let Some(hint_impl) = self.hints.get(hint_code) {
                return hint_impl(vm, exec_scopes, hpd, constants).map(|_| HintExtension::default());
            }

            if let Some(hint_impl) = self.extensive_hints.get(hint_code) {
                let r = hint_impl(self, vm, exec_scopes, hpd, constants);
                return r;
            }

            return self
                .builtin_hint_proc
                .execute_hint(vm, exec_scopes, hint_data, constants)
                .map(|_| HintExtension::default());
        }

        if let Some(hint) = hint_data.downcast_ref::<Hint>() {
            return self
                .cairo1_builtin_hint_proc
                .execute(vm, exec_scopes, hint)
                .map(|_| HintExtension::default());
        }

        Err(HintError::WrongHintData)
    }
}
    
impl ResourceTracker for BootloaderHintProcessor {}
