#![allow(unused_imports, dead_code)]
use crate::{AST::AST, token::Token};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[cfg(test)]
mod ast_stub {
    use super::*;
    #[test]
    fn simple_ast() -> TestResult {
        let mut test_ast = AST::new( Token::new("i;".to_string(),None).tokenize() );
        assert_eq!(&format!("{:?}",test_ast.parse()?), "[[1:0]:Var(\"i\")]");
        Ok(())
    }
    #[test]
    #[should_panic]
    fn error_expect_missing_block()  {
        let _ = AST::new( Token::new("func main() a=3;".to_string(),None).tokenize() ).parse().expect("");
    }
}

mod operator_test {
    #[cfg(test)]
    mod term {
        use super::super::*;
        #[test]
        fn plus()  -> TestResult {
            let mut test_ast = AST::new(Token::new("1+2;".to_string(), None).tokenize());
            assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:Binary([1:0]:Literal((I64)1), TokenData { tok_type: Plus, start: 1, end: 2, line: 1, identifier: \"\", value: Null, sub_tok: None }, [1:2]:Literal((I64)2))]");
            test_ast = AST::new(Token::new("1+a;".to_string(), None).tokenize());
            assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:Binary([1:0]:Literal((I64)1), TokenData { tok_type: Plus, start: 1, end: 2, line: 1, identifier: \"\", value: Null, sub_tok: None }, [1:2]:Var(\"a\"))]");
            Ok(())
        }
        #[test]
        fn minus() -> TestResult  {
            let mut test_ast = AST::new(Token::new("1-2;".to_string(), None).tokenize());
            assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:Binary([1:0]:Literal((I64)1), TokenData { tok_type: Minus, start: 1, end: 2, line: 1, identifier: \"\", value: Null, sub_tok: None }, [1:2]:Literal((I64)2))]");
            test_ast = AST::new(Token::new("1-a;".to_string(), None).tokenize());
            assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:Binary([1:0]:Literal((I64)1), TokenData { tok_type: Minus, start: 1, end: 2, line: 1, identifier: \"\", value: Null, sub_tok: None }, [1:2]:Var(\"a\"))]");
            Ok(())
        }
    }
}

#[cfg(test)]
mod stmt_test {
    use ntest::timeout;

use super::*;
    #[test]
    fn vardecl_test() -> TestResult  {
        let mut test_ast = AST::new(Token::new("let k = 4;".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:VarDecl(Variable { vData: None, name: \"k\", init: Some([1:8]:Literal((I64)4)) })]");

        test_ast = AST::new(Token::new("const k = 4;".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:VarDecl(Variable { vData: None, name: \"k\", init: Some([1:10]:Literal((I64)4)) })]");

        test_ast = AST::new(Token::new("let k:u16 = 4;".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:VarDecl(Variable { vData: Some(U16), name: \"k\", init: Some([1:12]:Literal((I64)4)) })]");

        test_ast = AST::new(Token::new("let k:u16* = 4;".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:VarDecl(Variable { vData: Some(U16*), name: \"k\", init: Some([1:13]:Literal((I64)4)) })]");

        test_ast = AST::new(Token::new("let k:u16 = a;".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:VarDecl(Variable { vData: Some(U16), name: \"k\", init: Some([1:12]:Var(\"a\")) })]");
        Ok(())
    }
    #[test]
    fn assignment_test() -> TestResult {
        let mut test_ast = AST::new(Token::new("a=3;".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:Assign(\"a\", [1:2]:Literal((I64)3))]");

        test_ast = AST::new(Token::new("a=b+3;".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:0]:Assign(\"a\", [1:2]:Binary([1:2]:Var(\"b\"), TokenData { tok_type: Plus, start: 3, end: 4, line: 1, identifier: \"\", value: Null, sub_tok: None }, [1:4]:Literal((I64)3)))]");
        Ok(())
    }
}

#[cfg(test)]
mod function_test {
    use super::*;
    #[test]
    fn simple_function() -> TestResult  {
        let mut test_ast = AST::new(Token::new("func test() { }".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:12]:FuncStmt(test([]) -> None, [1:12]:Block([]))]");

        test_ast = AST::new(Token::new("func test(value: i32) { }".to_string(), None).tokenize());
        assert_eq!(&format!("{:?}", test_ast.parse()?), "[[1:22]:FuncStmt(test([Variable { vData: Some(I32), name: \"value\", init: None }]) -> None, [1:22]:Block([]))]");
        Ok(())
    }
}
