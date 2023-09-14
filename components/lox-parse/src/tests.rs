use std::{path::{PathBuf, Path}, fs};

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
                res.push(TestCase { lox: rs, syntax: rast, text });
            }
        }
        res.sort();
        res
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use lox_ir::{word::Word, input_file::InputFile};
    use salsa::DebugWithDb;

    use crate::file_parser::parse_file;

    use super::TestCase;

    #[salsa::db(crate::Jar, lox_ir::Jar, lox_lex::Jar)]
    #[derive(Default)]
    struct Database {
        storage: salsa::Storage<Self>,
    }

    impl salsa::Database for Database {}

    impl lox_ir::Db for Database {}

    impl lox_lex::Db for Database {}


    #[test]
    fn parse() {
        let db = Database::default();
        for case in TestCase::list("") {
            let input_file = InputFile::new(&db, Word::intern(&db, case.lox.to_str().unwrap()), case.text.clone());
            let expr = parse_file(&db, input_file);
            if let Some(expr) = expr {
                expect_file![case.syntax].assert_eq(&format!("{:?}", expr.debug(&db)));
            } else {
                panic!("failed to parse {:?}", case.lox);
            }
        }
    }
}

