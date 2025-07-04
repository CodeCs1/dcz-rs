#![allow(non_snake_case)]

use std::{fs::File, io::Read/*, iter::zip*/};
use clap::Parser;
use codegen::codegen::Codegen;
use object_out::ObjectOut;
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
    let mut ast2ir = codegen::ast_2_ir::Ast2Ir::new(Checker::new(&ast_tree).check()?);
    let opcode_list = ast2ir.to_ir();

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


    Ok(())
}
