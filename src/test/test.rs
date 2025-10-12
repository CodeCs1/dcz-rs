use crate::{codegen::{llvm::Module, llvm_codegen}, object_out::llvm_object, token::{token_type::TokenType, Token, TokenData}, Value::Value, AST::{ast_checker::Checker, expr_node::{ClassFunction, Expr, Func_Header}, AST}};
use std::{fs::File, path::Path};
fn CompileDcz2Executable(file: &str) -> Result<(), Box<dyn std::error::Error>>{
    let file_path = Path::new(file);
    let file_io=File::open(file_path)?;
    let t = Token::FromIO(file_path,file_io);
    let mut p=AST::new(t?.tokenize());
    let ast_tree = p.parse();

    let mut c = Checker::new(&ast_tree);
    let expr = c.check()?;
    let binding = Module::new(file.to_string());
    let cg_c = llvm_codegen::LLVMCodegen::compile(expr, &binding);
    cg_c.codegen_all();
    let file= file_path.with_extension("").as_os_str().to_str().unwrap_or("a.out").to_string();
    llvm_object::LLVMObject::new(cg_c.get_module(), crate::ObjectArch::X64).ir2obj(&file);
    Ok(())
}

#[cfg(test)]
mod test {
    use std::process::Command;

    use super::*;

    #[test]
    fn tokenizer_test_simple() {
        let mut t = Token::new("(()".to_string(), None);
        let meta_data = t.tokenize();
        assert_eq!(meta_data.tok_data, vec![
            TokenData {
                tok_type: TokenType::LeftParen,
                start:0,
                end:1,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::LeftParen,
                start:1,
                end:2,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::RightParen,
                start: 2,
                end: 3,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::EOF,
                start: 2,
                end: 3,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            }
        ])
    }
    #[test]
    fn tokenizer_test_string() {
        let mut t = Token::new("\"Hello World\"".to_string(), None);
        let meta_data = t.tokenize();
        assert_eq!(meta_data.tok_data, vec![
            TokenData {
                tok_type: TokenType::String,
                start: 0,
                end: 13,
                identifier: "\"Hello World\"".to_string(),
                line: 1,
                value: Value::Str("Hello World".to_string()),
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::EOF,
                start: 0,
                end: 13,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            }
        ])
    }
    #[test]
    fn tokenizer_test_identifier_keyword() {
        let mut t = Token::new("abcxyz".to_string(), None);
        let meta_data1 = t.tokenize();
        assert_eq!(meta_data1.tok_data, vec![
            TokenData {
                tok_type: TokenType::Identifier,
                start: 0,
                end: 6,
                identifier: "abcxyz".to_string(),
                line: 1,
                value: Value::Object("abcxyz".to_string()),
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::EOF,
                start: 0,
                end: 6,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            }
        ]);
        let mut t1 = Token::new("suu number".to_string(),None);
        let meta_data2 = t1.tokenize();
        assert_eq!(meta_data2.tok_data, vec![
            TokenData {
                tok_type: TokenType::DataType,
                start: 0,
                end: 3,
                identifier: "suu".to_string(),
                line: 1,
                value: Value::Object("suu".to_string()),
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::Identifier,
                start: 4,
                end: 10,
                identifier: "number".to_string(),
                line: 1,
                value: Value::Object("number".to_string()),
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::EOF,
                start: 4,
                end: 10,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            }
        ])
    }

    #[test]
    fn value_test() {
        let v = Value::new("1".to_string());
        assert_eq!(*v.value().unwrap().downcast_ref::<i64>().unwrap(), 1)
    }

    #[test]
    fn classDecl_Test() {
        let p = Path::new("examples/class.dcz");
        let file_io=File::open(p).expect("Failed to open sample file: class.dcz");
        let t = Token::FromIO(p,file_io).expect("Failed to open file").tokenize();
        let p = AST::new(t).parse();
        assert_eq!(p, vec![
            Expr::Class("Test".to_string(), vec![
                ClassFunction {
                    func_type: crate::AST::expr_node::ClassFunctionType::Initializer,
                    access_level: crate::AST::expr_node::AccessLevel::Public,
                    function: Expr::FuncStmt(Func_Header {
                        args: vec![],
                        name: "Test".to_string(),
                        is_ptr_dt: false,
                        return_type: None,
                    }, Box::new(
                        Expr::Block(vec![])
                    ))
                },
                ClassFunction {
                    func_type: crate::AST::expr_node::ClassFunctionType::Deconstructor,
                    access_level: crate::AST::expr_node::AccessLevel::Public,
                    function: Expr::FuncStmt(Func_Header {
                        args: vec![],
                        name: "Test".to_string(),
                        is_ptr_dt: false,
                        return_type: None,
                    }, Box::new(
                        Expr::Block(vec![])
                    ))
                },
                ClassFunction {
                    func_type: crate::AST::expr_node::ClassFunctionType::Function,
                    access_level: crate::AST::expr_node::AccessLevel::Private,
                    function: Expr::FuncStmt(Func_Header {
                        args: vec![],
                        name: "private_function".to_string(),
                        is_ptr_dt: false,
                        return_type: None,
                    }, Box::new(
                        Expr::Block(vec![])
                    ))
                },
                ClassFunction {
                    func_type: crate::AST::expr_node::ClassFunctionType::Function,
                    access_level: crate::AST::expr_node::AccessLevel::Public,
                    function: Expr::FuncStmt(Func_Header {
                        args: vec![],
                        name: "public_function".to_string(),
                        is_ptr_dt: false,
                        return_type: None,
                    }, Box::new(
                        Expr::Block(vec![])
                    ))
                }
            ])
        ])
    }
    #[test]
    fn simple_hello_world_test() {
        CompileDcz2Executable("examples/HelloWorld.dcz").expect("failed to compile program");

        let mut binding = Command::new("gcc");
        let cmd = binding.args(["examples/HelloWorld.o", "-o", "examples/HelloWorld"]);
        match cmd.spawn() {
            Err(e) => {
                if let std::io::ErrorKind::NotFound = e.kind() {
                    eprintln!("`ld` was not found! Check your PATH!")
                } else {
                    eprintln!("Unknown error occur while linking sample script");
                }
                std::process::exit(1);
            }
            _ => {}
        }
        assert!(cmd.status().expect("cannot get status").success());
        let prog = Command::new("examples/HelloWorld").output().expect("Unable to run program");
        assert_eq!(String::from_utf8(prog.stdout).expect("failed to conv 2 String"), "Hello world!".to_string());

        std::fs::remove_file("examples/HelloWorld").expect("Failed to remove test file");
        std::fs::remove_file("examples/HelloWorld.o").expect("Failed to remove test file");
    }

}
