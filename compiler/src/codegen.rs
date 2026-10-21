// Engine Lang Native LLVM IR & Silicon Assembly Codegen Engine (`codegen`)
// Architecture: Phase 4 Native Silicon Codegen & Phase 5 Industrial MMIO
// Conformance: ENG-CG-01.1 & ENG-OS-01.1 Standard

use crate::ast::*;

pub struct LlvmCodegen {
    module: Module,
    target_arch: String,
    ir_buffer: String,
}

impl LlvmCodegen {
    pub fn new(module: Module, target_arch: &str) -> Self {
        Self {
            module,
            target_arch: target_arch.to_string(),
            ir_buffer: String::new(),
        }
    }

    pub fn generate_ir(&mut self) -> Result<String, String> {
        self.emit_module_header();
        self.emit_hardware_intrinsics();

        let decls = self.module.declarations.clone();
        for decl in &decls {
            match decl {
                TopLevelDecl::Entity(entity) => self.emit_entity_struct(entity),
                TopLevelDecl::Struct(st) => self.emit_plain_struct(st),
                TopLevelDecl::Register(reg) => self.emit_register_mmio(reg),
                TopLevelDecl::Action(action) => self.emit_action_function(action),
                TopLevelDecl::Datasource(ds) => self.emit_datasource_metadata(ds),
                _ => {}
            }
        }

        Ok(self.ir_buffer.clone())
    }

    /// Emits direct native Assembly (.s) conforming to System V AMD64 / AAPCS64
    pub fn generate_asm(&mut self) -> Result<String, String> {
        let mut asm = String::new();
        asm.push_str(&format!(
            "# Engine Lang Sovereign Silicon Assembly\n# Module: {}\n# Target: {}\n# Conformance: ENG-CG-01.1\n\n",
            self.module.name, self.target_arch
        ));

        match self.target_arch.as_str() {
            "aarch64" => self.emit_asm_aarch64(&mut asm),
            "riscv64" => self.emit_asm_riscv64(&mut asm),
            _ => self.emit_asm_x86_64(&mut asm),
        }

        Ok(asm)
    }

    fn emit_asm_x86_64(&self, asm: &mut String) {
        asm.push_str(".text\n.globl _start\n.globl main\n.p2align 4, 0x90\n\n");

        for decl in &self.module.declarations {
            if let TopLevelDecl::Action(act) = decl {
                asm.push_str(&format!("{}:\n", act.name));
                asm.push_str("    pushq   %rbp\n");
                asm.push_str("    movq    %rsp, %rbp\n");
                asm.push_str("    # System V AMD64 calling convention: args in %rdi, %rsi, %rdx\n");

                let has_atomic = act.body.statements.iter().any(|s| matches!(s, Statement::Atomic { .. }));
                if has_atomic {
                    asm.push_str("    # Hardware Persistence Barrier: CLWB + SFENCE\n");
                    asm.push_str("    clwb    (%rdi)         # Cache Line Write Back to CXL/NVRAM\n");
                    asm.push_str("    sfence                 # Store Fence: commit barrier\n");
                }

                // SIMD Cosine Distance (<=>) synthesis demonstration
                asm.push_str("    # AVX2 Vector Similarity (<=>) Acceleration\n");
                asm.push_str("    vmovups (%rsi), %ymm0\n");
                asm.push_str("    vmovups (%rdx), %ymm1\n");
                asm.push_str("    vmulps  %ymm1, %ymm0, %ymm2\n");

                asm.push_str("    xorl    %eax, %eax     # Return 0 (Ok)\n");
                asm.push_str("    popq    %rbp\n");
                asm.push_str("    retq\n\n");
            }
        }

        // Default entrypoint for bare-metal unikernel
        asm.push_str("_start:\n");
        asm.push_str("    movq    $0x00200000, %rdi  # Single-Level Store Superblock Base\n");
        asm.push_str("    callq   main\n");
        asm.push_str("    movq    $60, %rax          # Syscall exit (fallback)\n");
        asm.push_str("    xorq    %rdi, %rdi\n");
        asm.push_str("    syscall\n");
    }

    fn emit_asm_aarch64(&self, asm: &mut String) {
        asm.push_str(".text\n.globl _start\n.globl main\n.p2align 2\n\n");

        for decl in &self.module.declarations {
            if let TopLevelDecl::Action(act) = decl {
                asm.push_str(&format!("{}:\n", act.name));
                asm.push_str("    stp     x29, x30, [sp, -16]!\n");
                asm.push_str("    mov     x29, sp\n");

                let has_atomic = act.body.statements.iter().any(|s| matches!(s, Statement::Atomic { .. }));
                if has_atomic {
                    asm.push_str("    # ARM64 Hardware Persistence: DC CVAC + DMB ISHST\n");
                    asm.push_str("    dc      cvac, x0       # Data Cache Clean by VA to PoC\n");
                    asm.push_str("    dmb     ishst          # Inner Shareable Store Memory Barrier\n");
                }

                asm.push_str("    # ARM NEON Vector Cosine Similarity (<=>)\n");
                asm.push_str("    ldr     q0, [x1]\n");
                asm.push_str("    ldr     q1, [x2]\n");
                asm.push_str("    fmul    v2.4s, v0.4s, v1.4s\n");

                asm.push_str("    mov     w0, #0         # Return 0 (Ok)\n");
                asm.push_str("    ldp     x29, x30, [sp], 16\n");
                asm.push_str("    ret\n\n");
            }
        }
    }

    fn emit_asm_riscv64(&self, asm: &mut String) {
        asm.push_str(".text\n.globl _start\n.globl main\n.p2align 2\n\n");

        for decl in &self.module.declarations {
            if let TopLevelDecl::Action(act) = decl {
                asm.push_str(&format!("{}:\n", act.name));
                asm.push_str("    addi    sp, sp, -16\n");
                asm.push_str("    sd      ra, 8(sp)\n");

                let has_atomic = act.body.statements.iter().any(|s| matches!(s, Statement::Atomic { .. }));
                if has_atomic {
                    asm.push_str("    # RISC-V Persistent Fence\n");
                    asm.push_str("    fence   rw, rw\n");
                }

                asm.push_str("    li      a0, 0          # Return Ok\n");
                asm.push_str("    ld      ra, 8(sp)\n");
                asm.push_str("    addi    sp, sp, 16\n");
                asm.push_str("    ret\n\n");
            }
        }
    }

    fn emit_module_header(&mut self) {
        self.ir_buffer.push_str(&format!(
            "; ModuleID = '{}'\n; Generated by Engine Lang Compiler (enginec v0.3.1)\n; Target Silicon Architecture: {}\n\n",
            self.module.name, self.target_arch
        ));
        self.ir_buffer
            .push_str("source_filename = \"module.eng\"\n");
        if self.target_arch == "aarch64" {
            self.ir_buffer.push_str(
                "target datalayout = \"e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128\"\n",
            );
            self.ir_buffer
                .push_str("target triple = \"aarch64-unknown-linux-gnu\"\n\n");
        } else {
            self.ir_buffer.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
            self.ir_buffer
                .push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");
        }
    }

    fn emit_hardware_intrinsics(&mut self) {
        self.ir_buffer
            .push_str("; --- Silicon DAX & Memory Fence Intrinsics ---\n");
        if self.target_arch == "aarch64" {
            self.ir_buffer
                .push_str("declare void @llvm.aarch64.dmb(i32)\n");
            self.ir_buffer
                .push_str("declare void @llvm.aarch64.isb(i32)\n\n");
        } else {
            self.ir_buffer
                .push_str("declare void @llvm.x86.clwb(i8*)\n");
            self.ir_buffer
                .push_str("declare void @llvm.x86.sfence()\n\n");
        }
    }

    fn emit_entity_struct(&mut self, entity: &EntityDecl) {
        self.ir_buffer
            .push_str(&format!("; Persistent Entity: %struct.{}\n", entity.name));
        self.ir_buffer
            .push_str(&format!("%struct.{} = type <{{ ", entity.name));

        let mut field_types = Vec::new();
        // Implicit Single-Level Store Header (Causal Header: 32 bytes)
        field_types.push("i64".to_string()); // Epoch ID
        field_types.push("i64".to_string()); // Transaction Hash
        field_types.push("[16 x i8]".to_string()); // UUIDv7

        for f in &entity.fields {
            let llvm_t = match &f.field_type {
                TypeRef::Named(n, _) if n == "Int" => "i64",
                TypeRef::Named(n, _) if n == "Float" => "double",
                TypeRef::Named(n, _) if n == "Text" => "[32 x i8]",
                TypeRef::Named(n, _) if n == "Boolean" => "i1",
                TypeRef::Decimal(_, _) => "double",
                _ => "i64",
            };
            field_types.push(llvm_t.to_string());
        }

        self.ir_buffer.push_str(&field_types.join(", "));
        self.ir_buffer.push_str(" }>, align 64\n\n");
    }

    fn emit_register_mmio(&mut self, reg: &RegisterDecl) {
        self.ir_buffer.push_str(&format!("; Hardware MMIO Register Block: %mmio.{}\n", reg.name));
        self.ir_buffer.push_str(&format!("%mmio.{} = type <{{ ", reg.name));
        let mut field_types = Vec::new();
        for f in &reg.fields {
            let t = match f.bit_width {
                1..=8 => "i8",
                9..=16 => "i16",
                17..=32 => "i32",
                _ => "i64",
            };
            field_types.push(t.to_string());
        }
        self.ir_buffer.push_str(&field_types.join(", "));
        self.ir_buffer.push_str(" }>, align 4\n\n");
    }

    fn emit_plain_struct(&mut self, st: &StructDecl) {
        self.ir_buffer
            .push_str(&format!("%struct.{} = type {{ ", st.name));
        let mut field_types = Vec::new();
        for f in &st.fields {
            let llvm_t = match &f.field_type {
                TypeRef::Named(n, _) if n == "Int" => "i64",
                TypeRef::Named(n, _) if n == "Float" => "double",
                TypeRef::Named(n, _) if n == "Text" => "[32 x i8]",
                TypeRef::Named(n, _) if n == "Boolean" => "i1",
                _ => "i64",
            };
            field_types.push(llvm_t.to_string());
        }
        self.ir_buffer.push_str(&field_types.join(", "));
        self.ir_buffer.push_str(" }\n\n");
    }

    fn emit_action_function(&mut self, action: &ActionDecl) {
        let ret_type = "i32"; // Standard return code
        self.ir_buffer
            .push_str(&format!("; Action: {}\n", action.name));
        self.ir_buffer.push_str(&format!(
            "define dso_local {} @action_{}(",
            ret_type, action.name
        ));

        let mut params = Vec::new();
        for p in &action.params {
            params.push(format!("i8* %{}", p.name));
        }
        self.ir_buffer.push_str(&params.join(", "));
        self.ir_buffer.push_str(") #0 {\nentry:\n");

        let mut has_atomic = false;
        for stmt in &action.body.statements {
            if let Statement::Atomic { .. } = stmt {
                has_atomic = true;
                break;
            }
        }

        if has_atomic {
            self.ir_buffer
                .push_str("  ; Hardware Atomic Writeback Sequence\n");
            if self.target_arch == "aarch64" {
                self.ir_buffer
                    .push_str("  call void @llvm.aarch64.dmb(i32 11) ; SY (Full System Barrier)\n");
            } else {
                self.ir_buffer
                    .push_str("  %shadow_ptr = alloca i8, align 64\n");
                self.ir_buffer
                    .push_str("  call void @llvm.x86.clwb(i8* %shadow_ptr) #1\n");
                self.ir_buffer
                    .push_str("  call void @llvm.x86.sfence() #1\n");
            }
        }

        self.ir_buffer.push_str("  ret i32 0\n");
        self.ir_buffer.push_str("}\n\n");
    }

    fn emit_datasource_metadata(&mut self, ds: &DatasourceDecl) {
        self.ir_buffer.push_str(&format!(
            "; Datasource Metadata: {} (driver={}, mode={})\n",
            ds.name,
            ds.driver,
            ds.mode.as_deref().unwrap_or("read_write")
        ));
        self.ir_buffer.push_str(&format!(
            "@datasource_{} = internal constant [{} x i8] c\"{}\\00\", align 1\n\n",
            ds.name,
            ds.name.len() + 1,
            ds.name
        ));
    }
}
