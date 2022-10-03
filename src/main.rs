use crate::database::Database;
use crate::fact::Fact;

pub mod database;
pub mod fact;

fn main() {
    let mut db = Database::new();
    db.claim(Fact::from_string("fox is red"));
    db.claim(Fact::from_string("rock is red"));
    db.print();
    for v in db.select(&vec!["$x is red".to_string()]) {
        println!("{:?}", v);
    }
    for v in db.select(&vec!["fox is $".to_string()]) {
        println!("{:?}", v);
    }
    for v in db.select(&vec!["%fact".to_string()]) {
        println!("{:?}", v);
    }
    db.retract("fox is $");
    db.print();
}
