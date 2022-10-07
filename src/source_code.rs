use crate::database::{Database, QueryResult};
use crate::fact::{Fact, Term};

use lazy_static::lazy_static;
use mlua::{prelude::*, Function, Lua, RegistryKey, Variadic};
use regex::Regex;
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
struct Subscription {
    program_source_id: String,
    query_parts: Vec<String>,
    callback_func: RegistryKey,
    last_results: Vec<QueryResult>,
}
impl Subscription {
    fn new(
        program_source_id: &String,
        query_parts: &Vec<String>,
        callback_func: RegistryKey,
    ) -> Self {
        Subscription {
            program_source_id: program_source_id.to_owned(),
            query_parts: query_parts.to_owned(),
            callback_func,
            last_results: vec![],
        }
    }
}

pub struct SourceCodeManager {
    source_code_folder_path: String,
    subscriptions: Vec<Subscription>,
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
            subscriptions: vec![],
        }
    }

    fn get_program_id_from_filename(filename: &String) -> Option<i32> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"(\./scripts/(\d+)(?:__.+)*\.lua$)"#).unwrap();
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
                self.script_source_codes
                    .insert(program_id, source_code.clone());
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

        // self.init_lua_state(db);

        // run boot program automatically
        self.run_program(0, db);
        self.run_program(1, db);
        self.run_program(2, db);
    }

    pub fn update(&mut self, db: &mut Database) {
        self.run_subscriptions(db);
    }

    fn run_subscriptions(&self, db: &Database) {
        for sub in &self.subscriptions {
            let handler = &sub.callback_func;
            let handler: Function = self
                .lua_state
                .registry_value(&handler)
                .expect("cannot get Lua handler");
            let results = self.lua_state.create_table().unwrap();
            for v in db.select(&sub.query_parts) {
                let result = self.lua_state.create_table().unwrap();
                for r in v.result {
                    result.set(r.variable_name, r.term.to_string()).unwrap();
                }
                results.set(1, result).unwrap();
            }
            handler.call::<_, ()>(results).unwrap();
        }
    }

    // fn init_lua_state<'a, 'b: 'a>(&self, program_id: i32, scope: &'a mlua::Scope<'_, 'a>, db: &'a mut Database) {

    // }

    fn run_program(&mut self, program_id: i32, db: &mut Database) {
        if let Some(source_code) = self.script_source_codes.get(&program_id) {
            // match self.lua_state.load(source_code).exec() {
            //     Err(e) => println!("Exception when running program {}: {:?}", program_id, e),
            //     Ok(_) => println!("Ran program {} with no errors", program_id)
            // }

            self.lua_state.scope(|scope| {
                // We create a 'claim' Lua callback that holds a mutable reference to the variable
                // `rust_val`. Outside of a `Lua::scope` call, this would not be allowed
                // because it could be unsafe.
                self.lua_state.globals().set(
                    "claim",
                    scope.create_function_mut(|_, va: Variadic<String>| {
                        if let Some(fact_string) = va.first() {
                            let mut f = Fact::from_string(&fact_string);
                            f.terms.insert(0, Term::Id(program_id.to_string()));
                            db.claim(f);
                        } else {
                            // TODO: handle variadic input so programs can claim raw types of data
                            // maybe by defining a type that accepts both a string or a table and implments to ToLua trait
                            println!("unhandle empty or non-string input to claim function from lua");
                        }
                        Ok(())
                    }).unwrap(),
                ).unwrap();

                // TODO: handle retract, etc.

                // This function probaby doesn't need to be within a scope?
                self.lua_state.globals().set(
                    "when",
                    scope.create_function_mut(|_, (query_parts, callback_func): (Vec<String>, Function)| {
                        let handler = self.lua_state
                            .create_registry_value(callback_func)
                            .expect("cannot store Lua handler");
                        self.subscriptions.push(Subscription::new(&program_id.to_string(), &query_parts, handler));

                        Ok(())
                    }).unwrap(),
                ).unwrap();

                self.lua_state.load(source_code).exec()
            }).unwrap();
        } else {
            println!("Exception when running program {program_id}: program ID not found");
        }
    }
}
