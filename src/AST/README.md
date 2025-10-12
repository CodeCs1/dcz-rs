This is a simple syntax tree that is implemented using recursion descent
(This script is written in ohmjs syntax)
```
DucaScript {
	Program = Decl*
    Decl = VarDecl -- var
    			| ClassDecl --class
                | FuncDecl -- funcdecl
                | StructDecl -- structdecl
    			| Stmt
    dataType = "char"
    					| "short"
                        | "int"
                        | "long"
                        | "float"
                        | "suu"
   	alpha = "a" .. "z" | "A" .. "Z" | "_"
    string = "\"" (~"\"" any)* "\""
    id = alpha ( alpha | digit )*
    Args = Expr ("," Expr)*
    FuncHeader = id "(" param? ")" Block
    param = id ( "," id )*
    FuncClass = FuncHeader --func1
    					  | FuncDecl -- func2
                          | VarDecl -- vardecl
    StructDecl = "struct" id "{" VarDecl* "}"
    ClassDecl = "class" id ("extends" id)? "{" (("public" | "private")?FuncClass )* "}"
    FuncDecl = "func" FuncHeader
    VarDecl =  ("unsigned"?) dataType "*"? id ( "=" Expr )? ";"
    Stmt = ExprStmt --expr
    			| Block -- block
                | Return -- ret
    Return = "return" Expr? ";"
    Block = "{"  Decl* "}"
    ExprStmt = Expr ";"
    Expr = Casting
    Casting =  "<" ("unsigned")? dataType  "*"? ">"  Expr  -- cast
    				| Equality
    Equality = Compare ( ("!=" | "==") Compare )*
    Compare= Term  ( (">" | ">=" | "<" | "<=") Term)*
    Term = Factor( ("-" | "+") Factor )*
    Factor = Unary( ("*" | "/" | "%") Unary)*
    Unary = ("!" | "-") Unary -- unary
    				| Call
    CallArgs = "(" Args? ")" --args1
    					| "." id  --args2
    Call = Primary ( CallArgs )*
    Primary = number -- number
    				| "(" Expr ")" -- group
                    | id -- id
                    | string --str
                    | "self"--self
                    | "super" "." id --super
    number = digit+ ("." digit+)?
 }
```