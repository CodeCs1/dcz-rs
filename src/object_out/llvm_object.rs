/* 
use std::{ffi::{CStr, CString, c_char}, path::PathBuf};
use llvm_sys_201::target_machine::{LLVMOpaqueTargetMachine, LLVMTarget, LLVMTargetRef};
use crate::{LLDFlavor, ObjectArch, codegen::llvm::Module};
use crate::object_out::lld_port;

pub struct LLVMObject<'llvm> {
    llvm_module: &'llvm Module,
    target_machine: *mut LLVMOpaqueTargetMachine

}

impl<'llvm> LLVMObject<'llvm> {
    pub fn new(
        module: &'llvm Module,
        arch: ObjectArch
    ) ->Self {
        let mut uninit_ptr =std::mem::MaybeUninit::zeroed();
        unsafe {
            let default_triple = llvm_sys_201::target_machine::LLVMGetDefaultTargetTriple();

            let triple_str = CStr::from_ptr(default_triple);
            let triple_name = String::from(
                triple_str.to_str().expect("C string was not valid UTF-8")
            );
            let mut spl = triple_name.split('-').collect::<Vec<&str>>();

            spl[0] =match arch {
                ObjectArch::X32 => "i386",
                ObjectArch::X64 => "x86_64",
            };

            let triple_name = spl.join("-");

            let default_triple=llvm_sys_201::target_machine::LLVMNormalizeTargetTriple(
                CString::new(triple_name).expect("cstring failed").as_ptr()
            );
            match arch {
                ObjectArch::X64 | ObjectArch::X32 => {
                    llvm_sys_201::target::LLVMInitializeX86TargetInfo();
                    llvm_sys_201::target::LLVMInitializeX86Target();
                    llvm_sys_201::target::LLVMInitializeX86TargetMC();
                    llvm_sys_201::target::LLVMInitializeX86AsmParser();
                    llvm_sys_201::target::LLVMInitializeX86AsmPrinter();
                }
            }


            let mut target: LLVMTargetRef = libc::malloc(
                size_of::<LLVMTarget>()
            ) as LLVMTargetRef;
            let mut err: *mut i8 = uninit_ptr.as_mut_ptr();
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

    pub fn obj2exe(&self, filename: String, isVerbose: bool,flavor: LLDFlavor) {
        let mut link_path = PathBuf::from(filename.as_str());
        link_path.set_extension("exe");
        let link_str = String::from(link_path.as_os_str().to_str().unwrap());
        let win_out_args=format!("/out:{}", link_str);
        let k = [
            "lld-link",
            filename.as_str(),
            win_out_args.as_str(),
            "msvcrt.lib"
        ];

        let c_strings: Vec<CString> = k.iter().map(
            |&s| CString::new(s).expect("CString::new failed")
        ).collect();

        let ptr: Vec<*const c_char> = c_strings.iter().map(|cs| cs.as_ptr()).collect();
        unsafe {
            println!("LLD Return code: {}",lld_port::LLDMain(
                k.len() as i32, ptr.as_ptr(), isVerbose, flavor
            ));
        }
    }
    pub fn ir2obj(&self, filename: &str, isVerbose: bool) -> String {
        let path = PathBuf::from(filename);
        let obj_fn =path.as_os_str().to_str().unwrap();

        let mut uninit_ptr =std::mem::MaybeUninit::zeroed();
        unsafe {
            let mut err: *mut i8 = uninit_ptr.as_mut_ptr();
            let is_ok = llvm_sys_201::target_machine::LLVMTargetMachineEmitToFile(
                self.target_machine,
                self.llvm_module.module,
                CString::new(obj_fn).expect("cstring failed").as_ptr(),
                llvm_sys_201::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
                &mut err
            );
            if isVerbose{
                println!("isok: {:?}", is_ok);
                println!("e: {:?}", CStr::from_ptr(err).to_string_lossy().to_string());
            }
        }
        String::from(obj_fn)
    }
}
    */