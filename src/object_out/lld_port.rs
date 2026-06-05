use crate::LLDFlavor;

unsafe extern "C" {
    //int LLDMain(int argc, const char **argv);
    pub fn LLDMain(argc: i32, argv: *const *const libc::c_char, isVerbose: bool, flavor: LLDFlavor) -> i32;
}