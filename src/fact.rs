#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Term {
    Text(String),
    Id(String),
    Variable(String),
    Postfix(String),
}
impl Term {
    fn new(input: &str) -> Term {
        match input.chars().nth(0) {
            Some(c) => match c {
                '$' => Term::Variable(input[1..].to_string()),
                '%' => Term::Postfix(input[1..].to_string()),
                '#' => Term::Id(input[1..].to_string()),
                _ => Term::Text(input[..].to_string()),
            },
            None => Term::Text("".to_string()),
        }
    }

    fn to_string(&self) -> String {
        match &self {
            Term::Text(text) => text.to_owned(),
            Term::Id(text) => "#".to_string() + &text,
            Term::Variable(text) => "$".to_string() + &text,
            Term::Postfix(text) => "%".to_string() + &text,
        }
    }
}

// #[derive(Copy, Clone)]
pub struct Fact {
    pub terms: Vec<Term>,
}
impl Fact {
    pub fn from_string(input: &str) -> Fact {
        Fact {
            terms: input
                .split_whitespace() // TODO: split on quotes here
                .map(|chunk| Term::new(chunk))
                .collect(),
        }
    }

    pub fn from_terms(input_terms: &[Term]) -> Fact {
        Fact {
            terms: input_terms.to_owned(),
        }
    }

    pub fn to_string(&self) -> String {
        self.terms
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}
