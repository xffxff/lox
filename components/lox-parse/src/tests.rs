#[cfg(test)]
mod tests {
    use std::{path::{PathBuf, Path}, fs};

    use expect_test::expect_file;
    use lox_ir::{diagnostic::Diagnostics, input_file::InputFile, word::Word};
    use salsa::DebugWithDb;

    use crate::file_parser::parse_file;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
    struct TestCase {
        lox: PathBuf,
        syntax: PathBuf,
        text: String,
    }

    impl TestCase {
        fn list(path: &'static str) -> Vec<TestCase> {
            let crate_root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
            let test_data_dir = crate_root_dir.join("test_data");
            let dir = test_data_dir.join(path);

            let mut res = Vec::new();
            let read_dir = fs::read_dir(&dir)
                .unwrap_or_else(|err| panic!("can't `read_dir` {}: {err}", dir.display()));
            for file in read_dir {
                let file = file.unwrap();
                let path = file.path();
                if path.extension().unwrap_or_default() == "lox" {
                    let rs = path;
                    let rast = rs.with_extension("syntax");
                    let text = fs::read_to_string(&rs).unwrap();
                    res.push(TestCase {
                        lox: rs,
                        syntax: rast,
                        text,
                    });
                }
            }
            res.sort();
            res
        }
    }



    #[salsa::db(crate::Jar, lox_ir::Jar, lox_lex::Jar)]
    #[derive(Default)]
    struct Database {
        storage: salsa::Storage<Self>,
    }

    impl salsa::Database for Database {}

    impl lox_ir::Db for Database {}

    impl lox_lex::Db for Database {}

    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    #[test]
    fn parse() {
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

            let mut buf = String::new();
            for expr in exprs.iter() {
                buf.push_str(&format!("{:#?}\n", expr.debug(&db)));
            }

            let diagnostics = parse_file::accumulated::<Diagnostics>(&db, input_file);
            for diagnostic in diagnostics.iter() {
                buf.push_str(&format!("{:#?}\n", diagnostic));
            }
            expect_file![case.syntax].assert_eq(&buf);
        }
    }
}
