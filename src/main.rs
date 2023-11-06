use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use clap::{Parser, Subcommand};
use expect_test::expect_file;
use lox_db::Database;
use lox_error_format::FormatOptions;
use lox_ir::{bytecode, diagnostic::Diagnostics, input_file::InputFile, word::Word};
use salsa::DebugWithDb;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct TestCase {
    lox: PathBuf,
    token: PathBuf,
    syntax: PathBuf,
    bytecode: PathBuf,
    execute: PathBuf,
    diagnostic: PathBuf,
    stdout: PathBuf,
    text: String,
}

impl TestCase {
    fn new(lox: &Path) -> TestCase {
        let lox = TestCase::absolute_path(lox);
        if lox.extension().unwrap_or_default() != "lox" {
            panic!("expected lox file, got {}", lox.display());
        }

        // if the lox file is `foo/bar.lox`, then the generated files will be
        // `foo/bar/{token,syntax,bytecode,execute}`
        let parent = lox.parent().unwrap();
        let lox_dir = parent.join(lox.file_stem().unwrap());
        if !lox_dir.exists() {
            fs::create_dir(&lox_dir).unwrap();
        }

        let token = lox_dir.join("token");
        let syntax = lox_dir.join("syntax");
        let bytecode = lox_dir.join("bytecode");
        let execute = lox_dir.join("execute");
        let diagnostic = lox_dir.join("diagnostic");
        let output = lox_dir.join("output");
        let text = fs::read_to_string(&lox).unwrap();
        TestCase {
            lox: lox.to_owned(),
            token,
            syntax,
            bytecode,
            execute,
            diagnostic,
            stdout: output,
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
        print!("test {} ... ", self.lox.display());
        let input_file = InputFile::new(
            db,
            Word::intern(db, self.lox.to_str().unwrap()),
            self.text.clone(),
        );

        // check if we should ignore this test
        let ignore = input_file
            .source_text(db)
            .lines()
            .any(|line| line.trim_start().starts_with("# ignore"));
        if ignore {
            println!("ignored");
            return;
        }

        // test lex
        let token_tree = lox_lex::lex_file(db, input_file);
        expect_file![self.token].assert_eq(&format!("{:#?}", token_tree.debug(db)));

        // test syntax
        let exprs = lox_parse::parse_file(db, input_file);

        let mut buf = String::new();
        for expr in exprs.iter() {
            buf.push_str(&format!("{:#?}\n", expr.debug(db)));
        }
        expect_file![self.syntax].assert_eq(&buf);

        // test diagnostics
        let diagnostics = lox_compile::compile_file::accumulated::<Diagnostics>(db, input_file);
        let output = lox_error_format::format_diagnostics_with_options(
            db,
            &diagnostics,
            FormatOptions::no_color(),
        );
        if let Ok(output) = output {
            expect_file![self.diagnostic].assert_eq(&output);
        }

        // test bytecode
        let chunk = lox_compile::compile_file(db, input_file);
        expect_file![self.bytecode].assert_eq(&format!("{:#?}", chunk));

        // test execute
        let buf = Arc::new(Mutex::new(String::new()));
        let step_inspect = |code: Option<bytecode::Code>, vm: &lox_execute::VM| {
            let mut buf = buf.lock().unwrap();
            buf.push_str(&format!("execute: {:#?}\n", code));
            buf.push_str(&format!("stack: {:?}\n", &vm.stack_values()));
            buf.push_str(&format!("stdout: {:#?}\n", vm.output));
            buf.push('\n');
        };
        let output = lox_execute::execute_file(db, input_file, Some(step_inspect));
        expect_file![self.execute].assert_eq(&buf.lock().unwrap());

        // test stdout
        expect_file![self.stdout].assert_eq(&output);

        println!("ok");
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
        #[arg(default_value = "lox_tests")]
        path: PathBuf,

        /// instead of validating the output, generate or update it
        #[arg(long)]
        bless: bool,
    },

    Run {
        /// path to lox file
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
        Commands::Test { path, bless } => {
            if bless {
                // add `UPDATE_EXPECT` to the environment to update the expected output
                std::env::set_var("UPDATE_EXPECT", "1");
            }
            if path.is_dir() {
                let test_cases = TestCase::list(&path);
                for test_case in test_cases {
                    test_case.test(&db);
                }
            } else {
                let test_case = TestCase::new(&path);
                test_case.test(&db);
            }
        }
        Commands::Run { path } => {
            let input_file = InputFile::new(
                &db,
                Word::intern(&db, path.to_str().unwrap()),
                fs::read_to_string(&path).unwrap(),
            );
            lox_compile::compile_file(&db, input_file);
            let diagnostics =
                lox_compile::compile_file::accumulated::<Diagnostics>(&db, input_file);
            if !diagnostics.is_empty() {
                for diagnostic in &diagnostics {
                    lox_error_format::print_diagnostic(&db, diagnostic).unwrap();
                    println!(
                        "{}",
                        lox_error_format::format_diagnostics(&db, &diagnostics).unwrap()
                    );
                }
            } else {
                let output =
                    lox_execute::execute_file(&db, input_file, None::<fn(_, &lox_execute::VM)>);
                println!("{}", output);
            }
        }
    }
}
