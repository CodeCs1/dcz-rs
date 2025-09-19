use std::ffi::CStr;

use llvm_sys_201::target_machine::{LLVMOpaqueTargetMachine, LLVMTarget, LLVMTargetRef};

use crate::{codegen::{llvm::Module}, ObjectArch};

pub struct LLVMObject<'llvm> {
    llvm_module: &'llvm Module,
    target_machine: *mut LLVMOpaqueTargetMachine

}

impl<'llvm> LLVMObject<'llvm> {
    pub fn new(
        module: &'llvm Module,
        arch: ObjectArch
    ) ->Self {
        unsafe {
            /*
            llvm_sys_201::target::LLVM_InitializeAllTargetInfos();
            llvm_sys_201::target::LLVM_InitializeAllTargets();
            llvm_sys_201::target::LLVM_InitializeAllTargetMCs();
            llvm_sys_201::target::LLVM_InitializeAllAsmParsers();
            llvm_sys_201::target::LLVM_InitializeAllAsmPrinters();
            */
            let default_triple = llvm_sys_201::target_machine::LLVMGetDefaultTargetTriple();
            match arch {
                ObjectArch::X64 => {
                    llvm_sys_201::target::LLVMInitializeX86TargetInfo();
                    llvm_sys_201::target::LLVMInitializeX86Target();
                    llvm_sys_201::target::LLVMInitializeX86TargetMC();
                    llvm_sys_201::target::LLVMInitializeX86AsmParser();
                    llvm_sys_201::target::LLVMInitializeX86AsmPrinter();
                }
                _ => todo!(
                    "not implemented or llvm just does not support =)"
                )
            }


            let mut target: LLVMTargetRef = libc::malloc(
                size_of::<LLVMTarget>()
            ) as LLVMTargetRef;
            let mut err: *mut i8 = std::mem::MaybeUninit::zeroed().as_mut_ptr();
            llvm_sys_201::target_machine::LLVMGetTargetFromTriple(
                default_triple, &mut target, &mut err
            );
            let target_machine = llvm_sys_201::target_machine::LLVMCreateTargetMachine(
            target, default_triple, b"generic\0".as_ptr().cast(), 
        b"\0".as_ptr().cast(), llvm_sys_201::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault, 
        llvm_sys_201::target_machine::LLVMRelocMode::LLVMRelocPIC, 
    llvm_sys_201::target_machine::LLVMCodeModel::LLVMCodeModelDefault);

            module.set_data_layout(target_machine);
            Self {
                llvm_module: module,
                target_machine: target_machine
            }
        }
    }
    pub fn ir2obj(&self) {
        unsafe {
            let mut err: *mut i8 = std::mem::MaybeUninit::zeroed().as_mut_ptr();
            let is_ok = llvm_sys_201::target_machine::LLVMTargetMachineEmitToFile(
                self.target_machine,
                self.llvm_module.module,
                b"test.o\0".as_ptr().cast(),
                llvm_sys_201::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
                &mut err
            );
            println!("isok: {:?}", is_ok);
            println!("e: {:?}", CStr::from_ptr(err).to_string_lossy().to_string());
        }
    }
}