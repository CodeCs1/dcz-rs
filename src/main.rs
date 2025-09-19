#![allow(non_snake_case)]

use std::{fs::File, io::Read, path::Path/*, iter::zip*/};
use clap::Parser;
use codegen::codegen::Codegen;
use object_out::ObjectOut;
//use codegen::codegen::Codegen;
use token::Token;
use AST::{AST as dcz_ast, ast_checker::Checker};

use crate::{codegen::{llvm::Module, llvm_codegen::TypeValue}, object_out::llvm_object};

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

    #[arg(short, default_value="x64")]
    Architecture: String
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

    let binding = Module::new(args.file);
    let cg_c = codegen::llvm_codegen::LLVMCodegen::compile(expr, &binding);
    cg_c.codegen_all();
    cg_c.get_module().dump();

    llvm_object::LLVMObject::new(cg_c.get_module(), arch).ir2obj();

    
    /*
    let mut ast2ir = codegen::ast_2_ir::Ast2Ir::new(c.check()?);
    let opcode_list = ast2ir.to_ir();*/
    /*

    if args.run {
        use VM::vm;
        vm::VM::new(ast2ir.const_pool.clone()).run(opcode_list.instr,0).expect("VMError");
        return Ok(());
    }

    
    let mut code_gen = Codegen::new();

    code_gen.instr(opcode_list.instr,0);

    let mut obj = ObjectOut::new();
    let mut func_vec = Vec::new();
    for (n,o) in code_gen.func_location.iter_mut() {
        func_vec.push((n.clone(),obj.add_func(n.as_str(), o.assemble(0)?)));
    }
    /*
    if opcode.instructions().len() > 0 {
        obj.add_func("_start",opcode.assemble(0)?);
    }*/
    
    code_gen.call_location.iter().for_each(|(n,loc)| {
        if let Some(idx) = func_vec.iter().position(|(f,_)| f == n) {
            let (_,sym) = func_vec[idx];
            //dbg!(loc);
            obj.add_text_reloc(sym, *loc as u64, -4);
        }
    });

    code_gen.assign_location.iter().for_each(|(n,v)| {
        obj.add_value_data(n.clone(),v.clone());
    });

    std::fs::write("output.o", obj.write_buff()).expect("Failed to save output.o");

*/
    Ok(())
}
