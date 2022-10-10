use crate::database::{Database, QueryResult, Subscription};
use crate::fact::{Fact, Term};

use lazy_static::lazy_static;
use mlua::{prelude::*, Function, Lua, RegistryKey, Table, Variadic};
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::sync::{Mutex, MutexGuard};

// enum ProgramUpdate {
//     Claim(String),
//     Retract(String),
//     Cleanup(),
// }

pub struct SourceCodeManager {
    source_code_folder_path: String,
    // subscriptions: Vec<Subscription>,
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
            // subscriptions: vec![],
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

    // pub fn init(&mut self, db: &'static mut Database) {
    pub fn init(&mut self, static_db: &'static Mutex<Database>) {
        let mut db = static_db.lock().unwrap();
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

        std::mem::drop(db);

        // run boot program automatically
        self.run_program(0, &static_db);
        self.run_program(1, &static_db);
        self.run_program(2, &static_db);
    }

    pub fn update(&mut self, static_db: &'static Mutex<Database>) {
        self.run_subscriptions(&static_db);

        // self.run_lua(db, )
        // handler.call::<_, ()>(results)
        // self.lua_state.load(source_code).exec()
    }

    fn run_subscriptions(&mut self, static_db: &'static Mutex<Database>) {
        // how to iterate over subscriptions when it will also be modified?
        let mut db = static_db.lock().unwrap();
        let mut stuff: Vec<(LuaFunction, Table)> = vec![];
        for sub in &db.subscriptions {
            println!("running a subscription");
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

            stuff.push((handler, results));

            // handler.call::<_, ()>(results);
            // self.lua_state.scope(|scope| {
            //     self.lua_state.globals().set(
            //         "retract",
            //         {
            //             // let mut db2 = db;
            //             scope.create_function_mut(|_, fact_string: String| {
            //                 println!("callback retract");
            //                 Ok(())
            //             }).unwrap()
            //         },
            //     ).unwrap();

            //     handler.call::<_, ()>(results)
            // }).unwrap();
            // self.run_lua(&sub.program_source_id, db, || handler.call::<_, ()>(results));
        }
        std::mem::drop(db);
        for (handler, results) in stuff {
            handler.call::<_, ()>(results);
        }
    }

    // fn init_lua_state<'a, 'b: 'a>(&self, program_id: i32, scope: &'a mlua::Scope<'_, 'a>, db: &'a mut Database) {

    // }

    // fn run_lua<R, F: FnOnce() -> Result<R, LuaError>>(&mut self, program_id: &String, db: &mut Database, func: F)
    //     where R: 'static {
    //     let updates: RefCell<Vec<ProgramUpdate>> = RefCell::new(vec![]);

    //     self.lua_state.scope(|scope| {
    //         // We create a 'claim' Lua callback that holds a mutable reference to the variable
    //         // `rust_val`. Outside of a `Lua::scope` call, this would not be allowed
    //         // because it could be unsafe.

    //         // let _db = Rc::new(RefCell::new(5));

    //         self.lua_state.globals().set(
    //             "claim",
    //             {
    //                 scope.create_function_mut(|_, va: Variadic<String>| {
    //                     if let Some(fact_string) = va.first() {
    //                         // let mut f = Fact::from_string(&fact_string);
    //                         // f.terms.insert(0, Term::Id(program_id.to_string()));
    //                         // db.claim(f);
    //                         updates.borrow_mut().push(ProgramUpdate::Claim(fact_string.to_string()));
    //                     } else {
    //                         // TODO: handle variadic input so programs can claim raw types of data
    //                         // maybe by defining a type that accepts both a string or a table and implments to ToLua trait
    //                         println!("unhandle empty or non-string input to claim function from lua");
    //                     }
    //                     Ok(())
    //                 }).unwrap()
    //             },
    //         ).unwrap();

    //         self.lua_state.globals().set(
    //             "retract",
    //             {
    //                 // let mut db2 = db;
    //                 scope.create_function_mut(|_, fact_string: String| {
    //                     // db.retract(&fact_string);
    //                     updates.borrow_mut().push(ProgramUpdate::Retract(fact_string.to_string()));
    //                     Ok(())
    //                 }).unwrap()
    //             },
    //         ).unwrap();

    //         // TODO: handle retract, etc.

    //         // This function probaby doesn't need to be within a scope?
    //         self.lua_state.globals().set(
    //             "when",
    //             scope.create_function_mut(|_, (query_parts, callback_func): (Vec<String>, Function)| {
    //                 let handler = self.lua_state
    //                     .create_registry_value(callback_func)
    //                     .expect("cannot store Lua handler");
    //                 db.subscriptions.push(Subscription::new(&program_id.to_string(), &query_parts, handler));

    //                 Ok(())
    //             }).unwrap(),
    //         ).unwrap();

    //         // self.lua_state.load(source_code).exec()
    //         func()
    //     }).unwrap();

    //     for u in updates.borrow().iter() {
    //         match u {
    //             ProgramUpdate::Claim(fact_string) => {
    //                 let mut f = Fact::from_string(&fact_string);
    //                 f.terms.insert(0, Term::Id(program_id.to_string()));
    //                 println!("claim! {}", program_id);
    //                 db.print();
    //                 db.claim(f);
    //                 db.print();
    //             },
    //             ProgramUpdate::Retract(fact_string) => {
    //                 println!("retract! {}", program_id);
    //                 db.print();
    //                 db.retract(&fact_string);
    //                 db.print();
    //             },
    //             ProgramUpdate::Cleanup() => println!("TODO: handle cleanup in database")
    //         }
    //     }
    // }

    fn run_program(&mut self, program_id: i32, static_db: &'static Mutex<Database>) {
        let script_source_codes = self.script_source_codes.clone();
        if let Some(source_code) = script_source_codes.get(&program_id) {
            // match self.lua_state.load(source_code).exec() {
            //     Err(e) => println!("Exception when running program {}: {:?}", program_id, e),
            //     Ok(_) => println!("Ran program {} with no errors", program_id)
            // }

            // let v = RefCell::new(5);

            let claim = self
                .lua_state
                .create_function_mut(move |_, va: Variadic<String>| {
                    let mut db = static_db.lock().unwrap();
                    if let Some(fact_string) = va.first() {
                        let mut f = Fact::from_string(&fact_string);
                        f.terms.insert(0, Term::Id(program_id.to_string()));
                        db.claim(f);
                        std::mem::drop(db);
                    } else {
                        // TODO: handle variadic input so programs can claim raw types of data
                        // maybe by defining a type that accepts both a string or a table and implments to ToLua trait
                        println!("unhandle empty or non-string input to claim function from lua");
                    }
                    Ok(())
                })
                .unwrap();
            self.lua_state.globals().set("claim", claim).unwrap();

            let retract = self
                .lua_state
                .create_function_mut(|_, fact_string: String| {
                    println!("retract waiting for db");
                    let mut db = static_db.lock().unwrap();
                    db.retract(&fact_string);
                    std::mem::drop(db);
                    Ok(())
                })
                .unwrap();
            self.lua_state.globals().set("retract", retract).unwrap();

            // This function probaby doesn't need to be within a scope?
            let when_func = self
                .lua_state
                .create_function_mut(
                    move |lua, (query_parts, callback_func): (Vec<String>, Function)| {
                        let mut db = static_db.lock().unwrap();
                        let handler = lua
                            .create_registry_value(callback_func)
                            .expect("cannot store Lua handler");
                        // db.subscriptions.push(Subscription::new(&program_id.to_string(), &query_parts, handler));
                        db.subscriptions.push(Subscription::new(
                            &program_id.to_string(),
                            &query_parts,
                            handler,
                        ));
                        std::mem::drop(db);
                        Ok(())
                    },
                )
                .unwrap();
            self.lua_state.globals().set("when", when_func).unwrap();

            // self.lua_state.globals().set(
            //     "when",
            //     scope.create_function_mut(|_, (query_parts, callback_func): (Vec<String>, Function)| {
            //         let handler = self.lua_state
            //             .create_registry_value(callback_func)
            //             .expect("cannot store Lua handler");
            //         self.subscriptions.push(Subscription::new(&program_id.to_string(), &query_parts, handler));

            //         Ok(())
            //     }).unwrap(),
            // ).unwrap();

            // self.lua_state.globals().set(
            //     "claim",
            //     {
            //         scope.create_function_mut(|_, va: Variadic<String>| {
            //             if let Some(fact_string) = va.first() {
            //                 // let mut f = Fact::from_string(&fact_string);
            //                 // f.terms.insert(0, Term::Id(program_id.to_string()));
            //                 // db.claim(f);
            //                 updates.borrow_mut().push(ProgramUpdate::Claim(fact_string.to_string()));
            //             } else {
            //                 // TODO: handle variadic input so programs can claim raw types of data
            //                 // maybe by defining a type that accepts both a string or a table and implments to ToLua trait
            //                 println!("unhandle empty or non-string input to claim function from lua");
            //             }
            //             Ok(())
            //         }).unwrap()
            //     },
            // ).unwrap();

            // self.run_lua(&program_id.to_string(), db, || self.lua_state.load(source_code).exec());
            self.lua_state.load(source_code).exec().unwrap();
        } else {
            println!("Exception when running program {program_id}: program ID not found");
        }
    }
}
