#![allow(non_snake_case)]

use std::{fs::File, path::Path/*, iter::zip*/};
use clap::Parser;
//use codegen::codegen::Codegen;
use token::Token;
use AST::{AST as dcz_ast, ast_checker::Checker};

use crate::{codegen::{llvm::Module}, object_out::llvm_object};

//use object_out::ObjectOut;

mod object_out;
mod AST;
mod token;
mod DataSection;
mod Value;
mod test;
mod codegen;
mod VM;
mod MessageHandler;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cmd {
    file: String,

    #[arg(short, long)]
    /// Run speficied script.
    run: bool,

    #[arg(short, long, default_value_t='0')]
    ///Optimization flags
    // It could be: (0: basic optimization)
    Optimization: char,

    ///Architecture flags
    #[arg(short, default_value="x64")]
    Architecture: String,

    #[arg(short, long, default_value_t=false)]
    Verbose: bool
}

#[derive(Debug, Clone)]
enum ObjectArch {
    X16,
    X32,
    X64
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Cmd::parse();

    let file_path = Path::new(args.file.as_str());

    let arch = {
        match args.Architecture.as_str() {
            "x16" => ObjectArch::X16,
            "x32" => ObjectArch::X32,
            "x64" => ObjectArch::X64,
            _ => { return Err(format!("No architecture found: {:?}!", args.Architecture).into()); }
        }
    };

    let file_io=File::open(file_path)?;
    let t = Token::FromIO(file_path,file_io);
    let mut p=dcz_ast::new(t?.tokenize());
    let ast_tree = p.parse();

    let mut c = Checker::new(&ast_tree);
    let expr = c.check()?;
    println!("{:#?}", expr);

    let binding = Module::new(args.file.clone());
    let cg_c = codegen::llvm_codegen::LLVMCodegen::compile(expr, &binding);
    cg_c.codegen_all();
    cg_c.get_module().dump();

    llvm_object::LLVMObject::new(cg_c.get_module(), arch).ir2obj(&args.file);
    Ok(())
}
