use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct TestCase {
    lox: PathBuf,
    syntax: PathBuf,
    bytecode: PathBuf,
    execute: PathBuf,
    text: String,
}

impl TestCase {
    fn list(path: &'static str) -> Vec<TestCase> {
        let crate_root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

        // FIXME: do not hardcode the path
        let test_data_dir = crate_root_dir.join("lox_tests");

        let dir = test_data_dir.join(path);

        let mut res = Vec::new();
        let read_dir = fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("can't `read_dir` {}: {err}", dir.display()));
        for file in read_dir {
            let file = file.unwrap();
            let path = file.path();
            if path.extension().unwrap_or_default() == "lox" {
                let lox = path;
                let syntax = lox.with_extension("syntax");
                let bytecode = lox.with_extension("bytecode");
                let execute = lox.with_extension("execute");
                let text = fs::read_to_string(&lox).unwrap();
                res.push(TestCase {
                    lox,
                    syntax,
                    bytecode,
                    execute,
                    text,
                });
            }
        }
        res.sort();
        res
    }
}

use expect_test::expect_file;
use lox_ir::{diagnostic::Diagnostics, input_file::InputFile, word::Word, bytecode};

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

use lox_parse::parse_file;
use salsa::DebugWithDb;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let db = Database::default();

    // use env var to filter test cases
    let filter = std::env::var("TEST_FILTER").unwrap_or_default();

    for case in TestCase::list("") {
        if !filter.is_empty() && !case.lox.to_str().unwrap().contains(&filter) {
            continue;
        }

        tracing::info!("test case: {:?}", case.lox);
        let input_file = InputFile::new(
            &db,
            Word::intern(&db, case.lox.to_str().unwrap()),
            case.text.clone(),
        );
        let exprs = parse_file(&db, input_file);

        // test syntax
        let mut buf = String::new();
        for expr in exprs.iter() {
            buf.push_str(&format!("{:#?}\n", expr.debug(&db)));
        }

        let diagnostics = parse_file::accumulated::<Diagnostics>(&db, input_file);
        for diagnostic in diagnostics.iter() {
            buf.push_str(&format!("{:#?}\n", diagnostic));
        }
        expect_file![case.syntax].assert_eq(&buf);

        // test bytecode
        let chunk = lox_compile::compile_file(&db, input_file);
        expect_file![case.bytecode].assert_eq(&format!("{:#?}", chunk));

        // test execute
        let mut buf = String::new();
        let step_inspect = |code: bytecode::Code, vm: &lox_execute::VM| {
            buf.push_str(&format!("execute: {:#?}\n", code));
            buf.push_str(&format!("stack: {:#?}\n", vm.stack));
            buf.push('\n');
        };
        lox_execute::execute_file(&db, input_file, Some(step_inspect));
        expect_file![case.execute].assert_eq(&buf);
    }
}
