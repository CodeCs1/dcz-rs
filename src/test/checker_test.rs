
use crate::{AST::AST, token::Token, Checker};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[cfg(test)]
mod simple_tests {
    use super::*;
    #[test]
    fn constant_folding() {
        let mut ast = AST::new( Token::new("1+1;".to_string(),None).tokenize() );
        let test_ast = Checker::new(
            &ast.parse().expect("Parser Failed"),
            ast.get_filename()
        ).check().expect("Checker Failed");
        assert_eq!(&format!("{:?}",test_ast), "[[1:0]:Literal((I64)2)]");
    }
}

#[cfg(test)]
mod checker {
    use super::*;
    #[test]
    #[should_panic(expected = "Variable 'i' not found")]
    fn variable_not_found() {
        let mut ast = AST::new( Token::new("i;".to_string(),None).tokenize() );
        let k = Checker::new(
            &ast.parse().expect("Parser Failed"),
            ast.get_filename()
        ).check().expect("Checker Failed");
        println!("Checker 'variable not found' result (shouldn't ok):{:?}", k);
    }

    #[test]
    #[should_panic]
    fn variable_already_defined() {
        let mut ast = AST::new( Token::new("let k = 3; let k = 5;".to_string(),None).tokenize() );
        let k = Checker::new(
            &ast.parse().expect("Parser Failed"),
            ast.get_filename()
        ).check().expect("Checker Failed");
        println!("Checker 'variable already defined' result (shouldn't ok):{:?}", k);
    }
}
