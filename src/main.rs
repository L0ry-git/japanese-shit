#![feature(box_syntax)]
#![feature(exclusive_range_pattern)]
#![feature(drain_filter)]

mod questions;
mod files;
mod quiz;
mod utils;

use variant_count::VariantCount;

#[derive(Debug, VariantCount)]
pub enum QuestionType {
    SingleWord,
    Multi,
    Both
}

fn main() {

    println!("Kanji Exercise Helper");
    println!("---------------------");

    print!("Che gruppo di vocaboli vuoi utilizzare? ");
    println!("(1: solo vocaboli a un kanji, 2: solo espressioni con più di un kanji, 3: entrambi)");
    let q_type = match get_line_enum!(QuestionType) {
        Some(qt) => qt,
        _ => {
            println!("Gruppo non valido!");
            return;
        }
    };

    println!("Quanti vocaboli vuoi mostrare?");
    let amount = match utils::get_line_number() {
        Some(value) => value,
        _ => {
            println!("Valore non valido!");
            return;
        }
    };

    match quiz::do_quiz(q_type, amount) {
        Err(es) => println!("Errore imprevisto: {}", es),
        _ => (),
    };
}