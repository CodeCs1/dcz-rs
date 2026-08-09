This is a simple syntax tree that is implemented using recursion descent

```
dcz grammar

Program => Decl*

Type => (u<8,16,32,64> | i<8,16,32,64> | f<32,64>) | Identifier

Decl => Macro | Class | Func | Var | Stmt
Macro => '#! [' MacroFunction ']'
Class => 'class' Identifier '{' ClassDeclare* '}'
Func => 'func' Identifier ('->' Type) Body
Body => '{' Decl* '}'

Stmt => If | While | Expr
If => 'if' Expr Body ('else' Body)?
While => 'while' Expr Body
Expr => OtherStuff

```
