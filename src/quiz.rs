use std::path::Path;
use rand::{distributions::WeightedIndex, prelude::*};

use crate::{QuestionType, files, questions, utils};

type QuizResult = Result<(), String>;

pub fn do_quiz(
    q_type: QuestionType, 
    question_amount: usize
) -> QuizResult {

    let type_val = q_type as u8 + 1;
    let mut all_questions = Vec::new();
        
    //ask single
    if type_val & 1 != 0 {
        println!("-----------------");
        select_classes::<questions::QuestionSingle>(
            "Parole con un singolo kanji", 
            Path::new("singles"), 
            &mut all_questions
        )?;
    }
    
    //ask multi
    if type_val & 2 != 0 {
        println!("-----------------");
        select_classes::<questions::QuestionMultiple>(
            "Parole con più kanji", 
            Path::new("multiples"), 
            &mut all_questions
        )?;
    }

    //do the quiz
    println!("------------------");
    for _ in 0..question_amount {
        let rng_dist = WeightedIndex::new(
            all_questions.iter().map(|qst| qst.weight())
        ).unwrap();
        let mut rng = rand::thread_rng();

        let current_question = &all_questions[rng_dist.sample(&mut rng)];

        current_question.ask();
    }

    Ok(())

}

fn select_classes<Kind>(
    message: &str, 
    path: &Path, 
    all: &mut Vec<questions::BoxedQuestion>
) -> QuizResult 
    where Kind: questions::Question + 'static
{
    let list_files = files::list_files(path).unwrap();
        println!("{}: seleziona tra le classi di caratteri disponibili. ", message);

        for qfile in &list_files {
            let filename = match qfile.filename() {
                Some(v) => v,
                None => continue,
            };
            println!("- {}, {}", filename, qfile.desc());
        }

        println!("(per inserire più classi di caratteri separale con /)");
        println!("Inserisci:");
        
        for index in get_line_indices() {
            println!("Index: {}", index);
            let question_file = match list_files.get(index-1) {
                Some(v) => v.read_questions::<Kind>(),
                None => return Err(format!("Classe di caratteri numero {} inesistente!", index))
            };
            match question_file {
                Ok(mut questions) => all.append(&mut questions),
                Err(err) => return Err(err.to_string())
            }
        }

        Ok(())
}

fn get_line_indices() -> Vec<usize> {
    let mut ret = Vec::new();
    let line = utils::get_line().unwrap();

    for split in line.split("/") {
        match split.parse::<usize>() {
            Ok(value) => ret.push(value),
            _ => ()
        };
    }

    ret
}