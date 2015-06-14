use libc::{c_uint, c_int};
use ffi::prelude::LLVMValueRef;
use ffi::{core, LLVMAttribute};
use std::ffi::CString;
use std::{fmt, mem};
use std::ops::{Deref, Index};
use block::BasicBlock;
use context::{Context, GetContext};
use ty::{FunctionType, Type};
use util;

/// A typed value that can be used as an operand in instructions
pub struct Value;
native_ref!(&Value = LLVMValueRef);
impl Value {
    /// Create a new constant struct from the values given
    pub fn new_struct<'a>(context: &'a Context, vals: &[&'a Value], packed: bool) -> &'a Value {
        unsafe { core::LLVMConstStructInContext(context.into(), vals.as_ptr() as *mut LLVMValueRef, vals.len() as c_uint, packed as c_int) }.into()
    }
    pub fn get_name(&self) -> Option<&str> {
        unsafe {
            let c_name = core::LLVMGetValueName(self.into());
            util::to_null_str(c_name as *mut i8)
        }
    }
    pub fn set_name(&self, name: &str) {
        let c_name = CString::new(name).unwrap();
        unsafe {
            core::LLVMSetValueName(self.into(), c_name.as_ptr())
        }
    }
    pub fn get_type(&self) -> &Type {
        unsafe { core::LLVMTypeOf(self.into()) }.into()
    }
}
/// A `Value` that represents an argument to a function
pub struct Arg;
native_ref!(&Arg = LLVMValueRef);
impl Deref for Arg {
    type Target = Value;
    fn deref(&self) -> &Value {
        unsafe { mem::transmute(self) }
    }
}
impl Arg {
    /// Add an attribute to a function argument
    pub fn add_attribute(&self, attr: Attribute) {
        unsafe { core::LLVMAddAttribute(self.into(), attr.into()) }
    }
    /// Add attributes to this function argument
    pub fn add_attributes(&self, attrs: &[Attribute]) {
        let mut sum = LLVMAttribute::empty();
        for attr in attrs {
            let attr:LLVMAttribute = (*attr).into();
            sum = sum | attr;
        }
        unsafe { core::LLVMAddAttribute(self.into(), sum.into()) }
    }
    /// Get the attributes set for a function argument
    pub fn has_attribute(&self, attr: Attribute) -> bool {
        unsafe {
            let other = core::LLVMGetAttribute(self.into());
            other.contains(attr.into())
        }
    }
    /// Remove an attribute from a function argument
    pub fn remove_attribute(&self, attr: Attribute) {
        unsafe { core::LLVMRemoveAttribute(self.into(), attr.into()) }
    }
}
/// A `Value` that represents a `Function`
pub struct Function;
native_ref!(&Function = LLVMValueRef);
impl Deref for Function {
    type Target = Value;
    fn deref(&self) -> &Value {
        unsafe { mem::transmute(self) }
    }
}
impl Index<usize> for Function {
    type Output = Arg;
    fn index(&self, index: usize) -> &Arg {
        unsafe {
            if index < core::LLVMCountParams(self.into()) as usize {
                core::LLVMGetParam(self.into(), index as c_uint).into()
            } else {
                panic!("no such index {} on {:?}", index, self.get_type())
            }
        }
    }
}
impl Function {
    pub fn append<'a>(&'a self, name: &str) -> &'a BasicBlock {
        util::with_cstr(name, |ptr| unsafe {
            core::LLVMAppendBasicBlockInContext(self.get_context().into(), self.into(), ptr).into()
        })
    }
    pub fn get_entry(&self) -> Option<&BasicBlock> {
        unsafe { mem::transmute(core::LLVMGetEntryBasicBlock(self.into())) }
    }

    pub fn get_name(&self) -> &str {
        unsafe {
            let c_name = core::LLVMGetValueName(self.into());
            util::to_str(c_name as *mut i8)
        }
    }
    pub fn get_signature(&self) -> &FunctionType {
        unsafe { core::LLVMTypeOf(self.into()) }.into()
    }
    /// Add an attribute to this function
    pub fn add_attribute(&self, attr: Attribute) {
        unsafe { core::LLVMAddFunctionAttr(self.into(), attr.into()) }
    }
    /// Add attributes to this function
    pub fn add_attributes(&self, attrs: &[Attribute]) {
        let mut sum = LLVMAttribute::empty();
        for attr in attrs {
            let attr:LLVMAttribute = (*attr).into();
            sum = sum | attr;
        }
        unsafe { core::LLVMAddFunctionAttr(self.into(), sum.into()) }
    }
    /// Check if the attribute is set
    pub fn has_attribute(&self, attr: Attribute) -> bool {
        unsafe {
            let other = core::LLVMGetFunctionAttr(self.into());
            other.contains(attr.into())
        }
    }
    /// Remove an attribute from the function
    pub fn remove_attribute(&self, attr: Attribute) {
        unsafe { core::LLVMRemoveAttribute(self.into(), attr.into()) }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C)]
pub enum Attribute {
    /// Zero-extended before or after call
    ZExt =              0b1,
    /// Sign-extended before or after call
    SExt =              0b10,
    /// Mark the function as not returning
    NoReturn =          0b100,
    /// Force argument to be passed in register
    InReg =             0b1000,
    /// Hidden pointer to structure to return
    StructRet =         0b10000,
    /// Function doesn't unwind stack
    NoUnwind =          0b100000,
    /// Consider to not alias after call
    NoAlias =           0b1000000,
    /// Pass structure by value
    ByVal =             0b10000000,
    /// Nested function static chain
    Nest =              0b100000000,
    /// Function doesn't access memory
    ReadNone =          0b1000000000,
    /// Function only reads from memory
    ReadOnly =          0b10000000000,
    /// Never inline this function
    NoInline =          0b100000000000,
    /// Always inline this function
    AlwaysInline =      0b1000000000000,
    /// Optimize this function for size
    OptimizeForSize =   0b10000000000000,
    /// Stack protection
    StackProtect =      0b100000000000000,
    /// Stack protection required
    StackProtectReq =   0b1000000000000000,
    /// Alignment of parameter (5 bits) stored as log2 of alignment with +1 bias 0 means unaligned (different from align(1))
    Alignment =         0b10000000000000000,
    /// Function creates no aliases of pointer
    NoCapture =         0b100000000000000000,
    /// Disable redzone
    NoRedZone =         0b1000000000000000000,
    /// Disable implicit float instructions
    NoImplicitFloat =   0b10000000000000000000,
    /// Naked function
    Naked =             0b100000000000000000000,
    /// The source language has marked this function as inline
    InlineHint =        0b1000000000000000000000,
    /// Alignment of stack for function (3 bits) stored as log2 of alignment with +1 bias 0 means unaligned (different from alignstack=(1))
    StackAlignment =    0b11100000000000000000000000000,
    /// This function returns twice
    ReturnsTwice =      0b100000000000000000000000000000,
    /// Function must be in unwind table
    UWTable =           0b1000000000000000000000000000000,
    /// Function is called early/often, so lazy binding isn't effective
    NonLazyBind =       0b10000000000000000000000000000000
}
impl From<LLVMAttribute> for Attribute {
    fn from(attr: LLVMAttribute) -> Attribute {
        unsafe { mem::transmute(attr) }
    }
}
impl From<Attribute> for LLVMAttribute {
    fn from(attr: Attribute) -> LLVMAttribute {
        unsafe { mem::transmute(attr) }
    }
}
impl GetContext for Value {
    fn get_context(&self) -> &Context {
        self.get_type().get_context()
    }
}
to_str!(Value, LLVMPrintValueToString);
