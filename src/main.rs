use std::{
    fs,
    path::{Path, PathBuf},
};

use expect_test::expect_file;
use lox_ir::{bytecode, diagnostic::Diagnostics, input_file::InputFile, word::Word};
use salsa::DebugWithDb;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use clap::{Parser, Subcommand};


#[salsa::db(
    lox_parse::Jar,
    lox_ir::Jar,
    lox_lex::Jar,
    lox_compile::Jar,
    lox_execute::Jar
)]
#[derive(Default)]
struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}

impl lox_ir::Db for Database {}

impl lox_lex::Db for Database {}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct TestCase {
    lox: PathBuf,
    token: PathBuf,
    syntax: PathBuf,
    bytecode: PathBuf,
    execute: PathBuf,
    text: String,
}

impl TestCase {
    fn new(lox: &Path) -> TestCase {
        let lox = TestCase::absolute_path(lox);
        if lox.extension().unwrap_or_default() != "lox" {
            panic!("expected lox file, got {}", lox.display());
        }        
        let token = lox.with_extension("token");
        let syntax = lox.with_extension("syntax");
        let bytecode = lox.with_extension("bytecode");
        let execute = lox.with_extension("execute");
        let text = fs::read_to_string(&lox).unwrap();
        TestCase {
            lox: lox.to_owned(),
            token,
            syntax,
            bytecode,
            execute,
            text,
        }
    }

    fn list(dir: &Path) -> Vec<TestCase> {
        let dir = TestCase::absolute_path(dir);

        let mut res = Vec::new();
        let read_dir = fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("can't `read_dir` {}: {err}", &dir.display()));
        for file in read_dir {
            let file = file.unwrap();
            let path = file.path();
            if path.extension().unwrap_or_default() == "lox" {
                let lox = path;
                res.push(TestCase::new(&lox));
            }
        }
        res.sort();
        res
    }

    // absolute path relative to crate root
    fn absolute_path(path: &Path) -> PathBuf {
        if !path.is_absolute() {
            let crate_root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
            crate_root_dir.join(path)
        } else {
            path.to_owned()
        }
    }

    fn test(self, db: &Database) {
        let input_file = InputFile::new(
            db,
            Word::intern(db, self.lox.to_str().unwrap()),
            self.text.clone(),
        );

        // test lex
        let token_tree = lox_lex::lex_file(db, input_file);
        expect_file![self.token].assert_eq(&format!("{:#?}", token_tree.debug(db)));

        // test syntax
        let exprs = lox_parse::parse_file(db, input_file);

        let mut buf = String::new();
        for expr in exprs.iter() {
            buf.push_str(&format!("{:#?}\n", expr.debug(db)));
        }

        let diagnostics = lox_parse::parse_file::accumulated::<Diagnostics>(db, input_file);
        for diagnostic in diagnostics.iter() {
            buf.push_str(&format!("{:#?}\n", diagnostic));
        }
        expect_file![self.syntax].assert_eq(&buf);

        // test bytecode
        let chunk = lox_compile::compile_file(db, input_file);
        expect_file![self.bytecode].assert_eq(&format!("{:#?}", chunk));

        // test execute
        let mut buf = String::new();
        let step_inspect = |code: bytecode::Code, vm: &lox_execute::VM| {
            buf.push_str(&format!("execute: {:#?}\n", code));
            buf.push_str(&format!("stack: {:#?}\n", vm.stack));
            buf.push('\n');
        };
        lox_execute::execute_file(db, input_file, Some(step_inspect));
        expect_file![self.execute].assert_eq(&buf);
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// path to test file or directory
        #[arg(short, long, default_value = "lox_tests")]
        path: PathBuf,
    },
}

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();


    let cli = Cli::parse();

    let db = Database::default();

    match cli.command {
        Commands::Test { path } => {
            if path.is_dir() {
                let test_cases = TestCase::list(&path);
                for test_case in test_cases {
                    test_case.test(&db);
                }
            } else {
                let test_case = TestCase::new(&path);
                test_case.test(&db);
            }
        },
    }
}
