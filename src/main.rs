#![allow(non_snake_case)]

use std::{fs::File, io::Read/*, iter::zip*/};
use clap::Parser;
//use codegen::codegen::Codegen;
use token::Token;
use AST::{AST as dcz_ast, ast_checker::Checker};


//use object_out::ObjectOut;

mod object_out;
mod AST;
mod token;
mod DataSection;
mod Value;
mod test;
mod codegen;
mod VM;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cmd {
    file: String,

    #[arg(short, long, default_value_t=true)]
    /// Run speficied script.
    run: bool,

    #[arg(short, long, default_value_t='0')]
    ///Optimization flags
    Optimization: char
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Cmd::parse();
    let mut file_content=String::new();
    File::open(args.file)?.read_to_string(&mut file_content)?;

    let mut t = Token::new(file_content.trim().to_string());
    t.tokenize();

    let mut p=dcz_ast::new(t.tok_data);
    
    let ast_tree = p.parse();

    let _ = Checker::new(&ast_tree).check()?;

    let mut ast2ir = codegen::ast_2_ir::Ast2Ir::new(ast_tree);

    let opcode_list = ast2ir.to_ir();

    if args.run {
        use VM::vm;
        vm::VM::new(ast2ir.const_pool.clone(),opcode_list).run().expect("VMError");
    }

    /*
    let mut code_gen = Codegen::new(ast_tree);

    let opcode = code_gen.instr().assemble(0)?;

    let mut obj = ObjectOut::new();
    obj.init_with_opcode(opcode);
    

    let code_zip = zip(t.data.return_data(), code_gen.blob_location);

    code_zip.for_each(|((n,v),o)| {
        let str_sym = obj.add_str_data(n.clone(), v.clone().to_string());
        obj.add_reloc(str_sym, o as u64);
    });

    std::fs::write("output.o", obj.write_buff()).expect("Failed to save output.o");*/


    Ok(())
}
