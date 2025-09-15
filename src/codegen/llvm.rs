//simple llvm wrapper for dcz

use std::{ffi::{CStr, CString}, marker::PhantomData, ops::Deref};

use llvm_sys_201::{
    analysis::LLVMVerifyFunction, core::*, prelude::*, LLVMTypeKind, LLVMValueKind
};

pub struct Module {
    module: LLVMModuleRef,
    ctx: LLVMContextRef
}

unsafe extern "C" {
    fn LLVMFunctionType(
        ReturnType: Type<'_>,
        ParamTypes: *mut Type<'_>,
        ParamCount: ::libc::c_uint,
        IsVarArg: LLVMBool,
    ) -> LLVMTypeRef;
    fn LLVMBuildCall2(
        arg1: LLVMBuilderRef,
        arg2: Type<'_>,
        Fn: FnValue<'_>,
        Args: *mut LlvmValue<'_>,
        NumArgs: ::libc::c_uint,
        Name: *const ::libc::c_char,
    ) -> LLVMValueRef;
}


impl<'llvm> Module {
    pub fn new(module_name: String) -> Self {
        let (ctx,module) = unsafe {
            let c = LLVMContextCreate();
            let m = LLVMModuleCreateWithNameInContext(module_name.as_ptr().cast(), c);
            assert!(!c.is_null() && !m.is_null());
            (c,m)
        };

        Module { module: module, ctx: ctx }
    }
    pub fn dump(&self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }
    pub fn type_u64(&self) -> Type<'llvm> {
        let t_ref = unsafe {
            LLVMIntTypeInContext(self.ctx, 64)
        };
        Type::new(t_ref)
    }

    pub fn type_i64(&self) -> Type<'llvm> {
        self.type_u64()
    }
    pub fn type_void(&self) -> Type<'llvm> {
        let t_ref = unsafe {
            LLVMVoidTypeInContext(self.ctx)
        };
        Type::new(t_ref)
    }

    pub fn type_i32(&self) -> Type<'llvm> {
        let t_ref = unsafe {
            LLVMInt32TypeInContext(self.ctx)
        };
        Type::new(t_ref)
    }

    pub fn add_fn(&'llvm self, name: &str, fn_type: Type<'llvm>) -> FnValue<'llvm> {
        debug_assert_eq!(
            fn_type.kind(),
            LLVMTypeKind::LLVMFunctionTypeKind,
            "Expected a function type when adding a function!"
        );

        let c_string = CString::new(name).expect("cstring failed");

        let value_ref = unsafe {
            LLVMAddFunction(self.module, c_string.as_ptr(), fn_type.0)
        };


        FnValue::new(value_ref)
    }

    pub fn type_fn(&'llvm self, args: &mut [Type<'llvm>], ret: Type<'llvm>) -> Type<'llvm> {
        let t_ref = unsafe {
            LLVMFunctionType(
                ret,
                args.as_mut_ptr(), 
                args.len() as libc::c_uint,
                0)
        };
        Type::new(t_ref)
    }
    pub fn get_fn(&'llvm self, name: &str) -> Option<FnValue<'llvm>>{
        let name = CString::new(name).expect("cstring Failed");
        let value_ref = unsafe{
            LLVMGetNamedFunction(self.module, name.as_ptr())
        };
        (!value_ref.is_null()).then(||FnValue::new(value_ref))
    }
    pub fn new_basic_block(&'llvm self, fn_v: FnValue<'llvm>) -> BasicBlock<'llvm> {
        let bl = unsafe {
            LLVMAppendBasicBlockInContext(
                self.ctx, 
                fn_v.value_ref(), 
                b"block\0".as_ptr().cast())
        };
        assert!(!bl.is_null());
        BasicBlock(bl, PhantomData)
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.ctx);
        }
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Type<'llvm>(LLVMTypeRef, PhantomData<&'llvm ()>);

impl<'llvm> Type<'llvm> {
    fn new(type_ref: LLVMTypeRef) -> Self {
        assert!(!type_ref.is_null());
        Type(type_ref, PhantomData)
    }

    fn kind(&self) -> LLVMTypeKind {
        unsafe { LLVMGetTypeKind(self.0) }
    }

    pub fn dump(&self) {
        unsafe { LLVMDumpType(self.0) };
    }
    pub fn const_f64(self, n: f64) -> LlvmValue<'llvm> {
        debug_assert_eq!(
            self.kind(),
            LLVMTypeKind::LLVMDoubleTypeKind,
            "Expected a double type when creating const f64 value!"
        );

        let value_ref = unsafe { LLVMConstReal(self.0, n) };
        LlvmValue::new(value_ref)
    }

    pub fn const_i32(self, n:i32) -> LlvmValue<'llvm> {
        debug_assert_eq!(
            self.kind(),
            LLVMTypeKind::LLVMIntegerTypeKind,
            "Expected a number type when creating const int value!"
        );


        let v_ref = unsafe {
            LLVMConstInt(self.0, n as u64, 0)
        };
        LlvmValue::new(v_ref)
    }

    pub fn const_u64(self, n:u64) -> LlvmValue<'llvm> {
        debug_assert_eq!(
            self.kind(),
            LLVMTypeKind::LLVMIntegerTypeKind,
            "Expected a number type when creating const int value!"
        );


        let v_ref = unsafe {
            LLVMConstInt(self.0, n, 0)
        };
        LlvmValue::new(v_ref)
    }
    pub fn const_i64(self, n:i64) -> LlvmValue<'llvm> {
        debug_assert_eq!(
            self.kind(),
            LLVMTypeKind::LLVMIntegerTypeKind,
            "Expected a number type when creating const int value!"
        );


        let v_ref = unsafe {
            LLVMConstInt(self.0, n as u64, 1)
        };
        LlvmValue::new(v_ref)
    }
}

pub struct Builder<'llvm> {
    builder: LLVMBuilderRef,
    _ctx: PhantomData<&'llvm()>
}

impl <'llvm>Builder<'llvm> {
    pub fn new(module: &'llvm Module) -> Self {
        let builder = unsafe { LLVMCreateBuilderInContext (module.ctx) };
        assert!(!builder.is_null());

        Self {
            builder,
            _ctx: PhantomData,
        }
    }

    pub fn iadd(&self, lhs: LlvmValue<'llvm>,rhs: LlvmValue<'llvm>) ->LlvmValue<'llvm> {
        let value_ref =unsafe{
            LLVMBuildAdd(
                self.builder,
                lhs.value_ref(),
                rhs.value_ref(),
                b"add\0".as_ptr().cast()
            )
        };

        LlvmValue::new(value_ref)
    }

    pub fn ret(&self, ret: LlvmValue<'llvm>) -> LlvmValue<'llvm> {
        let v = unsafe {
            LLVMBuildRet(self.builder,ret.value_ref())
        };
        assert!(!v.is_null());
        LlvmValue::new(v)
    }
    pub fn call(&self, fn_value: FnValue<'llvm>, args: &mut[LlvmValue<'llvm>]) -> LlvmValue<'llvm> {
        let value_ref = unsafe {
            LLVMBuildCall2(
                self.builder,
                fn_value.ret_type(),
                fn_value,
                args.as_mut_ptr(),
                args.len() as libc::c_uint,
                b"call\0".as_ptr().cast(),
            )
        };
        LlvmValue::new(value_ref)
    }
    pub fn pos_at_end(&self, bb: BasicBlock<'llvm>) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, bb.0);
        }
    }
}


impl Drop for Builder<'_> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.builder) }
    }
}

#[derive(Copy, Clone)]
pub struct BasicBlock<'llvm>(LLVMBasicBlockRef, PhantomData<&'llvm ()>);

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct LlvmValue<'llvm>(LLVMValueRef, PhantomData<&'llvm ()>);

impl <'llvm>LlvmValue<'llvm> {
    pub fn new(v: LLVMValueRef) -> Self {
        assert!(!v.is_null());
        LlvmValue(v,PhantomData)
    }

    #[inline]
    fn value_ref(&self) -> LLVMValueRef {
        self.0
    }


    fn kind(&self) -> LLVMValueKind {
        unsafe { LLVMGetValueKind(self.value_ref()) }
    }
    pub fn dump(&self) {
        unsafe { LLVMDumpValue(self.value_ref()) };
    }

    pub fn set_name(&self, name: &str) {
        unsafe { LLVMSetValueName2(self.value_ref(), name.as_ptr().cast(), name.len()) };
    }
    pub fn get_name(&self) -> &'llvm str {
        let name = unsafe {
            let mut len: libc::size_t = 0;
            let name = LLVMGetValueName2(self.0, &mut len as _);
            assert!(!name.is_null());

            CStr::from_ptr(name)
        };

        // TODO: Does this string live for the time of the LLVM context?!
        name.to_str()
            .expect("Expected valid UTF8 string from LLVM API")
    }
}


#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct FnValue<'llvm>(LlvmValue<'llvm>);

impl<'llvm> Deref for FnValue<'llvm> {
    type Target = LlvmValue<'llvm>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl <'llvm>FnValue<'llvm> {
    fn new(value_ref: LLVMValueRef) -> Self {
        let value = LlvmValue::new(value_ref);
        debug_assert_eq!(
            value.kind(),
            LLVMValueKind::LLVMFunctionValueKind,
            "Expected a fn value when constructing FnValue!"
        );

        FnValue(value)
    }
    pub fn args(&self) -> usize {
        unsafe { LLVMCountParams(self.value_ref()) as usize }
    }
    pub fn ret_type(&self) -> Type<'llvm> {
        let type_ref = unsafe {
            LLVMGlobalGetValueType(self.value_ref())
        };
        Type::new(type_ref)
    }
    pub fn verify(&self) -> bool {
        unsafe {
            LLVMVerifyFunction(
                self.value_ref(),
                llvm_sys_201::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
            ) == 0
        }
    }

    pub fn arg(&self, idx: usize) -> LlvmValue<'llvm> {
        assert!(idx < self.args());

        let value_ref = unsafe { LLVMGetParam(self.value_ref(), idx as libc::c_uint) };
        LlvmValue::new(value_ref)
    }
    pub fn basic_blocks(&self) -> usize {
        unsafe { LLVMCountBasicBlocks(self.value_ref()) as usize }
    }
}
