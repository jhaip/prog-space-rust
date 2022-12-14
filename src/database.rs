use crate::fact::{Fact, Term};
use mlua::RegistryKey;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryResultVariable {
    pub variable_name: String,
    pub term: Term,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryResult {
    pub result: Vec<QueryResultVariable>,
}

#[derive(Debug)]
pub struct Subscription {
    pub program_source_id: String,
    pub query_parts: Vec<String>,
    pub callback_func: RegistryKey,
    pub last_results: Vec<QueryResult>,
}
impl Subscription {
    pub fn new(
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

pub struct Database {
    facts: Vec<Fact>,
    pub subscriptions: Vec<Subscription>,
}
impl Database {
    pub fn new() -> Self {
        Database {
            facts: vec![],
            subscriptions: vec![],
        }
    }

    pub fn print(&self) {
        println!("DATABASE:");
        self.facts
            .iter()
            .for_each(|f| println!("{}", f.to_string()));
    }

    pub fn claim(&mut self, fact: Fact) {
        self.facts.push(fact);
    }

    pub fn retract(&mut self, fact_query_str: &str) {
        let fact_query = Fact::from_string(fact_query_str);
        let mut empty_query_result = QueryResult { result: vec![] };
        self.facts
            .retain(|fact| !Self::fact_match(&fact_query, fact, &mut empty_query_result))
    }

    fn term_match(a: &Term, b: &Term, env: &mut QueryResult) -> bool {
        match a {
            Term::Variable(variable_name) | Term::Postfix(variable_name) => {
                if variable_name.is_empty() {
                    true
                } else {
                    let term_query_result_match = env
                        .result
                        .iter()
                        .find(|r| r.variable_name.eq(variable_name));
                    if let Some(t) = term_query_result_match {
                        return Self::term_match(&t.term.clone(), b, env);
                    } else {
                        env.result.push(QueryResultVariable {
                            variable_name: variable_name.to_owned(),
                            term: b.clone(),
                        });
                        true
                    }
                }
            }
            _ if a == b => true,
            _ => false,
        }
    }

    fn fact_match(fact_a: &Fact, fact_b: &Fact, env: &mut QueryResult) -> bool {
        if fact_a.terms.is_empty() {
            return false;
        }
        if let Some(Term::Postfix(_)) = fact_a.terms.last() {
            if fact_a.terms.len() > fact_b.terms.len() {
                return false;
            }
        } else if fact_a.terms.len() != fact_b.terms.len() {
            return false;
        }
        for (i, a_term) in fact_a.terms.iter().enumerate() {
            if !Self::term_match(a_term, &fact_b.terms[i], env) {
                return false;
            }
            if let Term::Postfix(variable_name) = a_term {
                if !variable_name.is_empty() {
                    env.result.push(QueryResultVariable {
                        variable_name: variable_name.to_string(),
                        term: Term::Text(Fact::from_terms(&fact_b.terms[i..]).to_string()),
                    })
                }
                break;
            }
        }
        return true;
    }

    fn collect_solutions(&self, query: &[Fact], env: &QueryResult) -> Vec<QueryResult> {
        if query.is_empty() {
            vec![env.clone()]
        } else {
            let mut solutions: Vec<QueryResult> = vec![];
            for f in &self.facts {
                let mut new_env: QueryResult = env.clone();
                if Self::fact_match(&query[0], f, &mut new_env) {
                    for solution in self.collect_solutions(&query[1..], &new_env) {
                        solutions.push(solution);
                    }
                }
            }
            return solutions;
        }
    }

    pub fn select(&self, query_parts: &Vec<String>) -> Vec<QueryResult> {
        let query: Vec<Fact> = query_parts.iter().map(|p| Fact::from_string(p)).collect();
        return self.collect_solutions(&query[..], &QueryResult { result: vec![] });
    }

    pub fn remove_subscriptions_by_program(&mut self, program_source_id: &String) {
        self.subscriptions
            .retain(|sub| sub.program_source_id.ne(program_source_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_tests() {
        let mut db = Database::new();
        db.claim(Fact::from_string("fox is red"));
        db.claim(Fact::from_string("rock is red"));
        db.print();
        let q1 = db.select(&vec!["$x is red".to_string()]);
        let mut q1 = q1.iter();
        assert_eq!(
            q1.next().unwrap(),
            &QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "x".to_string(),
                    term: Term::Text("fox".to_string())
                }]
            }
        );
        assert_eq!(
            q1.next().unwrap(),
            &QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "x".to_string(),
                    term: Term::Text("rock".to_string())
                }]
            }
        );

        let q2 = db.select(&vec!["fox is $".to_string()]);
        let mut q2 = q2.iter();
        assert_eq!(q2.next().unwrap(), &QueryResult { result: vec![] });

        let q3 = db.select(&vec!["%fact".to_string()]);
        let mut q3 = q3.iter();
        assert_eq!(
            q3.next().unwrap(),
            &QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "fact".to_string(),
                    term: Term::Text("fox is red".to_string())
                }]
            }
        );
        assert_eq!(
            q3.next().unwrap(),
            &QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "fact".to_string(),
                    term: Term::Text("rock is red".to_string())
                }]
            }
        );

        db.retract("fox is $");

        let q4 = db.select(&vec!["$x is red".to_string()]);
        let mut q4 = q4.iter();
        assert_eq!(
            q4.next().unwrap(),
            &QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "x".to_string(),
                    term: Term::Text("rock".to_string())
                }]
            }
        );
    }
}