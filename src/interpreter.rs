use lexer;
use parser;
use ast_walk_interpreter;
use cps_interpreter;

#[cfg(not(test))]
use repl;

#[cfg(not(test))]
use std::fs::File;

#[cfg(not(test))]
use std::path::Path;

#[cfg(not(test))]
use std::io::Read;

macro_rules! try_or_err_to_string {
    ($inp:expr) => (
        match $inp {
            Ok(v) => v,
            Err(e) => return Err(e.to_string())
        }
    )
}

pub fn new(t: &str) -> Interpreter {
    Interpreter::new(t)
}

pub enum Interpreter {
    AstWalk(ast_walk_interpreter::Interpreter),
    Cps(cps_interpreter::Interpreter),
}

impl Interpreter {
    pub fn new(t: &str) -> Interpreter {
        match t.as_ref() {
            "cps" => Interpreter::Cps(cps_interpreter::new().unwrap()),
            "ast_walk" => Interpreter::AstWalk(ast_walk_interpreter::new()),
            _ => panic!("Interpreter type must be 'cps' or 'ast_walk'")
        }
    }

    fn parse(&self, input: &str) -> Result<Vec<parser::Node>, String> {
        let tokens = try_or_err_to_string!(lexer::tokenize(input));
        let ast = try_or_err_to_string!(parser::parse(&tokens));
        Ok(ast)
    }

    pub fn execute(&self, input: &str) -> Result<String, String> {
        let parsed = try!(self.parse(input));
        match *self {
            Interpreter::AstWalk(ref i) => Ok(format!("{:?}", try_or_err_to_string!(i.run(&parsed)))),
            Interpreter::Cps(ref i)     => Ok(format!("{:?}", try_or_err_to_string!(i.run(&parsed)))),
        }
    }

    pub fn define_custom(&mut self, name: &str, op: ast_walk_interpreter::ValueOperation) {
        match *self {
            Interpreter::AstWalk(ref mut i) => {
                //let root = i.root.borrow_mut();
                i.add_to_environment(String::from(name), op).unwrap(); // TODO: Don't unwrap
            }
            Interpreter::Cps(_) => () // Do nothing for the moment.
        }
    }

    #[cfg(not(test))]
    pub fn start_repl(&self) {
        println!("\nWelcome to the RustyScheme REPL!");
        repl::start("> ", (|s| self.execute(&s)))
    }

    pub fn run_str(&self, s: &str) -> Result<String, String> {
        self.execute(s)
    }

    #[cfg(not(test))]
    pub fn run_file(&self, filename: &String) {
        let path = Path::new(&filename);
        let mut file = File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        match self.execute(&contents) {
            Ok(_) => {},
            Err(e) => println!("{}", e),
        }
    }
}
