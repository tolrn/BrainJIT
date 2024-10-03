use dynasmrt::{x64::X64Relocation, Assembler, AssemblyOffset};

use crate::{execution::native::state::State, syntax::Instruction};
use dynasmrt::dynasm;
use dynasmrt::DynasmApi;
use dynasmrt::DynasmLabelApi;

use super::codegen::NativeCodeGenBackend;

pub struct X64CodeGen;

macro_rules! x64_alias_asm {
    ($ops:expr, $($t:tt)*) => {
        dynasm!($ops
            ; .arch x64
            ; .alias cell_ptr, rdx
            ; .alias retval, rax
            $($t)*
        )
    }
}

impl NativeCodeGenBackend for X64CodeGen {
    type Relocation = X64Relocation;

    fn generate_prolouge(&self, ops: &mut Assembler<Self::Relocation>) -> AssemblyOffset {
        let start = ops.offset();
        x64_alias_asm!(ops,
            ; sub rsp,0x28
            ; mov[rsp+0x30],rcx
            ; mov[rsp+0x40],r8
            ; mov[rsp+0x48],r9
        );
        start
    }

    fn generate_epilouge(&self, ops: &mut Assembler<Self::Relocation>) {
        macro_rules! epilogue {
            ($ops:expr, $e:expr) => {
                x64_alias_asm!($ops,
                    ; mov retval, $e
                    ; add rsp, 0x28
                    ; ret
                );
            };
        }

        x64_alias_asm!(ops,
            ;; epilogue!(ops, 0)
            ;->overflow:
            ;; epilogue!(ops, 1)
            ;->io_failure:
            ;; epilogue!(ops, 2)
        );
    }

    fn generate_increment(&self, ops: &mut Assembler<Self::Relocation>, value: i8) {
        x64_alias_asm!(ops,
            ; add BYTE [rdx], value
        );
    }

    fn generate_cell_increment(&self, ops: &mut Assembler<Self::Relocation>, value: i32) {
        x64_alias_asm!(ops,
            ; add cell_ptr, value
        );
    }

    fn generate_loop(&self, ops: &mut Assembler<Self::Relocation>, nodes: &[Instruction]) {
        let backward_label = ops.new_dynamic_label();
        let forward_label = ops.new_dynamic_label();

        // Start of the loop: Check if the current cell is 0, jump to the forward label (end of loop) if true.
        x64_alias_asm!(ops,
            ; cmp BYTE [cell_ptr], 0
            ; jz =>forward_label
            ;=>backward_label
        );

        // Generate the instructions inside the loop
        for node in nodes {
            self.generate_instruction(ops, node);
        }

        // End of the loop: Jump back to the start of the loop if the condition is still true.
        x64_alias_asm!(ops,
            ; cmp BYTE [cell_ptr], 0
            ; jnz =>backward_label
            ;=>forward_label
        );
    }

    fn generate_write(&self, ops: &mut Assembler<Self::Relocation>) {
        X64CodeGen::generate_extern_call(ops, State::putchar as _);
        x64_alias_asm!(ops,
            ; cmp al, 0
            ; jnz ->io_failure
        );
    }

    fn generate_read(&self, ops: &mut Assembler<Self::Relocation>) {
        {
            X64CodeGen::generate_extern_call(ops, State::getchar as _);
            x64_alias_asm!(ops,
                ; cmp al, 0
                ; jnz ->io_failure
            );
        };
    }
}

impl X64CodeGen {
    pub fn generate_extern_call(ops: &mut Assembler<X64Relocation>, addr: *const ()) {
        x64_alias_asm!(ops,
            ; mov[rsp+0x38],rdx
            ; mov rax,QWORD addr as _
            ; call rax
            ; mov rcx,[rsp+0x30]
            ; mov rdx,[rsp+0x38]
            ; mov r8,[rsp+0x40]
            ; mov r9,[rsp+0x48]
        );
    }
}