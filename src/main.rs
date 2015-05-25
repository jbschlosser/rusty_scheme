use std::env;
use std::fs::File;
use std::path::Path;
use std::io::Read;

mod lexer;
mod parser;
mod ast_walk_interpreter;
mod cps_interpreter;

#[cfg(not(test))]
mod repl;

macro_rules! try_or_err_to_string {
    ($inp:expr) => (
        match $inp {
            Ok(v) => v,
            Err(e) => return Err(e.to_string())
        }
    )
}

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => start_repl(),
        2 => run_file(&args[1]),
        _ => panic!("You must provide 0 or 1 arguments to RustyScheme: {:?}", args)
    }
}

#[allow(unused_must_use)]
#[cfg(not(test))]
fn run_file(filename: &String) {
    let path = Path::new(&filename);
    let mut file = File::open(&path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    let ctx = cps_interpreter::new().unwrap();
    execute_cps(&contents, ctx);
}

#[cfg(not(test))]
fn start_repl() {
    println!("\nWelcome to the RustyScheme REPL!");
    repl::start("> ", (|s| execute_cps(&s, cps_interpreter::new().unwrap())));
}

fn parse(input: &str) -> Result<Vec<parser::Node>, String> {
    let tokens = try_or_err_to_string!(lexer::tokenize(input));
    let ast = try_or_err_to_string!(parser::parse(&tokens));
    Ok(ast)
}

fn execute_ast_walk(input: &str, ctx: ast_walk_interpreter::Interpreter) -> Result<String, String> {
    let result = try_or_err_to_string!(ctx.run(&try!(parse(input))));
    Ok(format!("{}", result))
}

fn execute_cps(input: &str, ctx: cps_interpreter::Interpreter) -> Result<String, String> {
    let result = try_or_err_to_string!(ctx.run(&try!(parse(input))));
    Ok(format!("{}", result))
}

macro_rules! assert_execute {
    ($src:expr, $res:expr) => (
        assert_eq!(execute_ast_walk($src, ast_walk_interpreter::new()).unwrap(), $res);
        assert_eq!(execute_cps($src, cps_interpreter::new().unwrap()).unwrap(), $res);
    )
}

macro_rules! assert_execute_fail {
    ($src:expr, $res:expr) => (
        assert_eq!(execute_ast_walk($src, ast_walk_interpreter::new()).err().unwrap(), $res);
        assert_eq!(execute_cps($src, cps_interpreter::new().unwrap()).err().unwrap(), $res);
    )
}

#[test]
fn test_basic_identities() {
    assert_execute!("1", "1");
    assert_execute!("#f", "#f");
    assert_execute!("\"hi\"", "\"hi\"");
    assert_execute!("(lambda (x) x)", "#<procedure>");
}

#[test]
fn test_simple_function() {
    assert_execute!("(+ 2 3)", "5");
}

#[test]
fn test_multiple_expression_return() {
    assert_execute!("(+ 2 3)\n(+ 1 2)", "3");
}

#[test]
fn test_nested_expressions() {
    assert_execute!("(+ 2 (- (+ 9 1) 4))", "8");
}

#[test]
fn test_list_creation() {
    assert_execute!("(list)", "'()");
    assert_execute!("(list 1 2 3)", "'(1 2 3)");
    assert_execute!("(list 1 (list 2 3) (list 4) (list))", "'(1 (2 3) (4) ())");
}

#[test]
fn test_cons() {
    assert_execute!("(cons 1 '())", "'(1)");
    assert_execute!("(cons 1 '(2))", "'(1 2)");
    assert_execute!("(cons '(1) '(2))", "'((1) 2)");
}

#[test]
fn test_variable_definition() {
    assert_execute!("(define x 2) (+ x x x)", "6");
    assert_execute!("(define x 2) ((lambda (x) x) 3)", "3");
    assert_execute!("(define x 2) (let ((x 3)) x)", "3");
    assert_execute!("(define x 2) ((lambda (x) (define x 4) x) 3)", "4");
    assert_execute!("(define x 2) (let ((x 3)) (define x 4) x)", "4");
}

#[test]
fn test_duplicate_variable_definition() {
    assert_execute_fail!("(define x 2) (define x 3)", "RuntimeError: Duplicate define: \"x\"");
    assert_execute_fail!("((lambda () (define x 2) (define x 3)))", "RuntimeError: Duplicate define: \"x\"");
    assert_execute_fail!("(let ((y 2)) (define x 2) (define x 3))", "RuntimeError: Duplicate define: \"x\"");
}

#[test]
fn test_variable_modification() {
    assert_execute!("(define x 2) (set! x 3) (+ x x x)", "9");
    assert_execute!("(define x 2) ((lambda () (set! x 3))) x", "3");
    assert_execute!("(define x 2) (let ((y 2)) (set! x 3)) x", "3");
}

#[test]
fn test_unknown_variable_modification() {
    assert_execute_fail!("(set! x 3)", "RuntimeError: Can't set! an undefined variable: \"x\"");
}

#[test]
fn test_procedure_definition() {
    assert_execute!("(define double (lambda (x) (+ x x))) (double 8)", "16");
    assert_execute!("(define twice (lambda (f v) (f (f v)))) (twice (lambda (x) (+ x x)) 8)", "32");
    assert_execute!("(define twice (λ (f v) (f (f v)))) (twice (λ (x) (+ x x)) 8)", "32");
    assert_execute!("((λ (x) (+ x x)) 8)", "16");
    assert_execute!("(define foo (λ (x) (λ (y) (+ x y)))) (define add2 (foo 2)) (add2 5)", "7");
    assert_execute!("(define foo (λ (x) (λ (y) (+ x y)))) (define add2 (foo 2)) ((λ (x) (add2 (+ x 1))) 1)", "4");
    assert_execute!("(define (twice f v) (f (f v))) (twice (lambda (x) (+ x x)) 8)", "32");
}

#[test]
fn test_begin_statement() {
    assert_execute!("(define x 1) (begin (set! x 5) (set! x (+ x 2)) x)", "7");
}

#[test]
fn test_let_statement() {
    assert_execute!("(let ((x 2)) (+ x x))", "4");
    assert_execute!("(let ((x 2) (y 3)) (+ x y))", "5");
    assert_execute!("(let ((x 2) (y 3)) (set! y (+ y 1)) (+ x y))", "6");
}

#[test]
fn test_conditional_execution() {
    assert_execute!("(if #t 1 2)", "1");
    assert_execute!("(if #f 1 2)", "2");
    assert_execute!("(if 0 1 2)", "1");
    assert_execute!("(if \"\" 1 2)", "1");
}

#[test]
fn test_conditional_execution_doesnt_run_other_case() {
    assert_execute!("(if #t 1 (error \"bad\"))", "1");
    assert_execute!("(if #f (error \"bad\") 2)", "2");
}

#[test]
fn test_boolean_operators() {
    assert_execute!("(and)", "#t");
    assert_execute!("(and #t)", "#t");
    assert_execute!("(and 1)", "1");
    assert_execute!("(and 1 2 3)", "3");
    assert_execute!("(and 1 #f 3)", "#f");
    assert_execute!("(and 1 #f (error \"bad\"))", "#f");
    assert_execute!("(or)", "#f");
    assert_execute!("(or #f)", "#f");
    assert_execute!("(or 1)", "1");
    assert_execute!("(or 1 2)", "1");
    assert_execute!("(or 1 #f)", "1");
    assert_execute!("(or #f 3)", "3");
    assert_execute!("(or #f #f)", "#f");
    assert_execute!("(or 1 (error \"bad\"))", "1");
}

#[test]
fn test_quoting() {
    assert_execute!("(quote #t)", "#t");
    assert_execute!("(quote 1)", "1");
    assert_execute!("(quote sym)", "'sym");
    assert_execute!("(quote \"hi\")", "\"hi\"");
    assert_execute!("(quote (1 2))", "'(1 2)");
    assert_execute!("(quote (a b))", "'(a b)");
    assert_execute!("(quote (a b (c (d) e ())))", "'(a b (c (d) e ()))");
    assert_execute!("(quote (a (quote b)))", "'(a (quote b))");
    assert_execute!("'(1 2)", "'(1 2)");
    assert_execute!("'(a b (c (d) e ()))", "'(a b (c (d) e ()))");
    assert_execute!("'(1 '2)", "'(1 (quote 2))");
}

#[test]
fn test_quasiquoting() {
    assert_execute!("(quasiquote (1 2))", "'(1 2)");
    assert_execute!("(quasiquote (2 (unquote (+ 1 2)) 4))", "'(2 3 4)");
    assert_execute!("`(2 ,(+ 1 2) 4)", "'(2 3 4)");
    assert_execute!("(define formula '(+ x y)) `((lambda (x y) ,formula) 2 3)", "'((lambda (x y) (+ x y)) 2 3)");
}

#[test]
fn test_apply() {
    assert_execute!("(apply + '(1 2 3))", "6");
    assert_execute!("(define foo (lambda (f) (lambda (x y) (f (f x y) y)))) (apply (apply foo (list +)) '(5 3))", "11");
}

#[test]
fn test_eval() {
    assert_execute!("(eval '(+ 1 2 3))", "6");
    assert_execute!("(define eval-formula (lambda (formula) (eval `((lambda (x y) ,formula) 2 3)))) (eval-formula '(+ (- y x) y))", "4");
    assert_execute_fail!("(define bad-eval-formula (lambda (formula) ((lambda (x y) (eval formula)) 2 3))) (bad-eval-formula '(+ x y))", "RuntimeError: Identifier not found: 'x");
}

#[test]
fn test_bad_syntax() {
    assert_execute_fail!("(22+)", "SyntaxError: Unexpected character when looking for a delimiter: + (line: 1, column: 4)");
    assert_execute_fail!("(+ 2 3)\n(+ 1 2-)", "SyntaxError: Unexpected character when looking for a delimiter: - (line: 2, column: 7)");
}

#[test]
fn test_generated_runtime_error() {
    assert_execute_fail!("(error \"fail, please\")", "RuntimeError: \"fail, please\"");
    assert_execute_fail!("(error (+ 2 3))", "RuntimeError: 5");
}

#[test]
fn test_errors_halt_execution() {
    assert_execute_fail!("(error \"fail, please\") 5", "RuntimeError: \"fail, please\"");
}

#[test]
fn test_unicode_identifiers() {
    assert_execute!("(define ★ 3) (define ♫ 4) (+ ★ ♫)", "7");
}

#[test]
fn test_macros() {
    assert_execute!("(define-syntax-rule (incr x) (set! x (+ x 1))) (define a 1) (incr a) a", "2");
    assert_execute!("(define-syntax-rule (incr x) (set! x (+ x 1))) (define x 1) (incr x) x", "2");
    assert_execute!("(define-syntax-rule (incr x) (set! x (+ x 1))) (define-syntax-rule (foo x y z) (if x (incr y) (incr z))) (define a #t) (define b 10) (define c 20) (foo a b c) (set! a #f) (foo a b c) (list b c)", "'(11 21)");
    assert_execute!("(define-syntax-rule (foo x) (if x (+ (foo #f) 3) 10)) (foo #t)", "13");
    assert_execute!("(define-syntax-rule (testy a b c) (if a b c)) (testy #t 1 (error \"test\")) (testy #f (error \"test\") 2)", "2");
}
