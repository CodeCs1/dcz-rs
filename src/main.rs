#![allow(non_snake_case)]

use std::{fs::File, panic, path::{Path, PathBuf}};
use clap::Parser;
use inkwell::context::Context;
use token::Token;
use AST::{AST as dcz_ast, ast_checker::Checker};

use crate::MessageHandler::throw_message;

//mod object_out;
mod AST;
mod token;
mod Value;
mod test;
mod codegen;
mod MessageHandler;


#[derive(Debug, Clone, clap::ValueEnum)]
enum ObjectArch {
    X32,
    X64
}

#[derive(Debug, Clone, clap::ValueEnum, PartialEq,Eq)]
#[repr(C)]
enum LLDFlavor {
  Invaild,
  Gnu,     // -flavor gnu
  MinGW,   // -flavor gnu MinGW
  WinLink, // -flavor link
  Darwin,  // -flavor darwin
  Wasm,    // -flavor wasm
}


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cmd {
    /// Input file
    file: String,
    /// LLD Flavor
    #[arg(long, value_enum,default_value_t=LLDFlavor::Invaild)]
    flavor: LLDFlavor,
    #[arg(short='O', default_value_t = String::from("0"), value_parser = clap::builder::PossibleValuesParser::new(
        [
            "0","1","2","3","z"
        ]
    ))]
    ///Optimization flags
    Optimization: String,
    #[arg(short,default_value_t=format!(""))]
    /// Output file
    OutputFile: String,
    ///Architecture flags
    #[arg(short, value_enum, default_value_t=ObjectArch::X64)]
    Architecture: ObjectArch,
    #[arg(short, long, default_value_t=false)]
    /// Enable program log
    Verbose: bool,
    #[arg(short='f', required=false)]
    MachineIndependant: Vec<String>,
    #[arg(long, required=false)]
    /// Show AST Tree and exit
    ShowAST: bool
}


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut args = Cmd::parse();
    if args.flavor == LLDFlavor::Invaild {
        args.flavor = match std::env::consts::OS {
            "windows" => LLDFlavor::WinLink,
            "linux" => LLDFlavor::Gnu,
            "macos" => LLDFlavor::Darwin,
            &_ => unimplemented!()
        };
    }

    let file_path = Path::new(args.file.as_str());
    // let _out_file = if args.OutputFile.is_empty() {
    //     let mut fp = PathBuf::from(&args.file);
    //     fp.set_extension("exe");
    //     String::from(fp.as_os_str().to_str().unwrap())
    // } else {
    //     args.OutputFile
    // };

    let file_io=File::open(file_path)?;
    let t = Token::FromIO(file_path,file_io);
    let mut p=dcz_ast::new(t?.tokenize());
    let ast_tree = p.parse()?;

    let mut c = Checker::new(
        &ast_tree,
        file_path.to_str().unwrap_or("source").to_string()
    );
    let expr = c.check()?;

    if args.ShowAST {
        println!("{:#?}", expr);
        return Ok(())
    }

    let ctx = Context::create();
    let codegen =codegen::llvm_codegen::LLVMCodegen::new(
        expr, &ctx, &args.file
    );

    codegen.compile()?;
    codegen.dump();

    /*
    let llvm_obj = llvm_object::LLVMObject::new(cg_c.get_module(), args.Architecture);
    llvm_obj.obj2exe(
        llvm_obj.ir2obj(
            &out_file,
            args.Verbose
        ),
        args.Verbose,
        args.flavor
    );*/
    Ok(())
}
