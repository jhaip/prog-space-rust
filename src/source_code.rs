use crate::database::Database;
use crate::fact::{Fact, Term};

use lazy_static::lazy_static;
use mlua::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;

pub struct SourceCodeManager {
    source_code_folder_path: String,
    lua_state: Lua,
    script_paths: HashMap<i32, String>,
    script_source_codes: HashMap<i32, String>,
}
impl SourceCodeManager {
    pub fn new(source_code_folder_path: String) -> SourceCodeManager {
        SourceCodeManager {
            source_code_folder_path,
            lua_state: Lua::new(),
            script_paths: HashMap::new(),
            script_source_codes: HashMap::new(),
        }
    }

    fn get_program_id_from_filename(filename: &String) -> Option<i32> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r#"(\./scripts/(\d+)(?:__.+)*\.lua$)"#).unwrap();
        }
        RE.captures_iter(filename).last().and_then(|cap| {
            cap.get(2)
                .and_then(|id| Some(id.as_str().parse::<i32>().unwrap()))
        })
    }

    pub fn init(&mut self, db: &mut Database) {
        let paths = fs::read_dir("./scripts").unwrap();
        for path in paths {
            let file_path = path.unwrap().path().display().to_string();
            println!("Name: {}", file_path);
            if let Some(program_id) = SourceCodeManager::get_program_id_from_filename(&file_path) {
                let source_code =
                    fs::read_to_string(&file_path).expect("Should have been able to read the file");
                self.script_paths.insert(program_id, file_path);
                self.script_source_codes.insert(program_id, source_code.clone());
                let terms: Vec<Term> = vec![
                    Term::Id("00".to_string()),
                    Term::Text(format!("{}", program_id)),
                    Term::Text("source".to_string()),
                    Term::Text("code".to_string()),
                    Term::Text(source_code.clone()),
                ];
                db.claim(Fact::from_terms(&terms[..]));
            }
        }

        self.init_lua_state(db);

        // // run boot program automatically
        // run_program(0);
    }

    fn init_lua_state(&self, db: &Database) {
        // self.lua_state
        db.print();
        println!("TODO init_lua_state");
    }
}
