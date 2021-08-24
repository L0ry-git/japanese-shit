use crate::utils;
use std::mem;

pub trait Question {
    unsafe fn parse(this_ptr: *mut Self, lines_iter: &mut dyn Iterator<Item = String>) -> Option<()>
        where Self: Sized;

    fn weight(&self) -> usize;
    fn ask(&self);
}

pub type BoxedQuestion = Box<dyn Question>;

pub fn parse_from_iter<T>(it: &mut dyn Iterator<Item = String>) -> Option<BoxedQuestion>
where
    T: Question + 'static,
{
    unsafe {
        let mut ret_v = mem::MaybeUninit::<T>::uninit();
        T::parse(ret_v.as_mut_ptr(), it)?;

        Some(box ret_v.assume_init())
    }
}

#[derive(Debug, Clone)]
enum PronunciationKind {
    Kun,
    On,
}

#[derive(Debug, Clone)]
pub struct QuestionSingle {
    expr: String,
    pron: String,
    pron_kind: PronunciationKind,
    meaning: String,
}

impl QuestionSingle {
    #[inline(always)]
    fn single_from_iter<'s, I>(split_iter: &mut I) -> Option<QuestionSingle>
    where
        I: Iterator<Item = &'s str>,
    {
        let pron_kind = match split_iter.next()? {
            "K" => PronunciationKind::Kun,
            "O" => PronunciationKind::On,
            _ => return None,
        };
        let expr = String::from(split_iter.next()?);
        let ret = QuestionSingle {
            expr,
            pron_kind,
            pron: String::from(split_iter.next()?),
            meaning: String::from(split_iter.next()?),
        };

        Some(ret)
    }

    fn pron_kind(&self) -> String {
        String::from(match self.pron_kind {
            PronunciationKind::Kun => "kun'yomi",
            PronunciationKind::On => "on'yomi",
        })
    }
}

impl Question for QuestionSingle {
    unsafe fn parse(this_ptr: *mut Self, lines_iter: &mut dyn Iterator<Item = String>) -> Option<()>
        where Self: Sized 
    {
        let next_line = lines_iter.next()?;
        let mut split_iter = next_line.split('|');
        this_ptr.write(Self::single_from_iter(&mut split_iter)?);

        Some(())
    }

    fn ask(&self) {
        println!("Leggere il seguente kanji: {}", self.expr);

        println!("Premere INVIO per mostrare la soluzione:");
        utils::get_line();
        println!(
            "- Pronuncia: {}, tipo di pronuncia: {}, significato: {}",
            self.pron,
            self.pron_kind(),
            self.meaning
        );
        println!("-----------------------");
    }

    fn weight(&self) -> usize {
        1
    }
}

#[derive(Debug, Clone)]
pub struct QuestionMultiple {
    inner: QuestionSingle,
    composition: String,
}

impl Question for QuestionMultiple {
    unsafe fn parse(this_ptr: *mut Self, lines_iter: &mut dyn Iterator<Item = String>) -> Option<()>
        where Self: Sized 
    {
        let next_line = lines_iter.next()?;
        let mut split_iter = next_line.split('|');
        let inner = QuestionSingle::single_from_iter(&mut split_iter)?;

        this_ptr.write(Self {
            inner,
            composition: String::from(split_iter.next()?),
        });
        Some(())
    }

    fn ask(&self) {
        let inner = &self.inner;
        println!("Leggere i seguenti kanji: {}", inner.expr);

        println!("Premere INVIO per mostrare la soluzione:");
        utils::get_line();
        println!(
            "- Pronuncia: {}, tipo di pronuncia: {}, composizione: {}, significato: {}",
            inner.pron,
            inner.pron_kind(),
            self.composition,
            inner.meaning
        );
        println!("-----------------------");
    }

    fn weight(&self) -> usize {
        1
    }

}

pub mod gen {

    use rand::Rng;

    use crate::utils;

    use super::{Question, QuestionSingle};

    pub const GENERATOR_BEGIN: &str = "GB";
    const GENERATOR_END: &str = "GE";
    const GENERATOR_OBLIGATORY: char = 'o';
    const GENERATOR_FACOLTATIVE: char = 'f';
    const GENERATOR_MULTI: char = 'm';

    #[derive(Debug, PartialEq)]
    enum ShardKind {
        Facoltative,
        Obligatory,
        Multi,
        Invalid //doesn't get considered because is invalid
    }

    impl From<char> for ShardKind {
        fn from(c: char) -> Self {
            match c {
                GENERATOR_OBLIGATORY => Self::Obligatory,
                GENERATOR_FACOLTATIVE => Self::Facoltative,
                GENERATOR_MULTI => Self::Multi,
                _ => Self::Invalid
            }
        }
    }

    #[derive(Debug)]
    struct QuestionGeneratorShard {
        expressions: Vec<QuestionSingle>,
        kind: ShardKind,
    }

    impl QuestionGeneratorShard {

        #[inline(always)]
        fn expr_count(&self) -> usize {
            self.expressions.len()
        }

        pub fn generate(&self) -> QuestionSingle {
            let mut gen = rand::thread_rng();
            self.expressions[gen.gen_range(0..self.expr_count())].clone()
        }
    }

    #[derive(Debug)]
    pub struct QuestionGenerated {
        shards: Vec<QuestionGeneratorShard>,
    }

    impl QuestionGenerated {

        fn yes_or_no() -> bool {
            rand::thread_rng().gen_bool(0.5)
        }

    }

    impl Question for QuestionGenerated {

        unsafe fn parse(this_ptr: *mut Self, lines_iter: &mut dyn Iterator<Item = String>) -> Option<()>
            where Self: Sized 
        {
            if lines_iter.next()? != GENERATOR_BEGIN {return None}
            let mut new_self = Self {
                shards: vec![],
            };

            let mut current_shard = None;
            while let Some(next_line) = lines_iter.next() {
                if next_line == GENERATOR_END {break};

                if next_line.starts_with(&[
                        GENERATOR_OBLIGATORY, 
                        GENERATOR_FACOLTATIVE,
                        GENERATOR_MULTI
                    ][..]) {
                    let first_char = next_line.chars().nth(0).unwrap();
                    if let Some(previous_shard) = current_shard {
                        new_self.shards.push(previous_shard);
                    }

                    current_shard = Some(QuestionGeneratorShard {
                        expressions: vec![],
                        kind: ShardKind::from(first_char),
                    });
                }
                else {
                    let mut split_line = next_line.split('|');
                    let shard_expr = QuestionSingle::single_from_iter(&mut split_line);

                    if let Some(built_expr) = shard_expr {
                        current_shard.as_mut()
                            .unwrap().expressions.push(built_expr); 
                    }
                }
            }
            if let Some(last_shard) = current_shard {new_self.shards.push(last_shard)};
            this_ptr.write(new_self);

            Some(())
        }

        fn ask(&self) {
            let mut full_sequence = vec![];
            let mut shards_iter = self.shards.iter().peekable();

            while shards_iter.peek().is_some() {
                let shard = shards_iter.next().unwrap();

                let generated = match shard.kind {
                    ShardKind::Obligatory => shard.generate(),
                    ShardKind::Facoltative if Self::yes_or_no() => shard.generate(),
                    ShardKind::Multi => {
                        let mut all_multi = vec![shard];

                        //gather all the next multi shard
                        while let Some(multi_shard @ QuestionGeneratorShard { 
                            kind: ShardKind::Multi, expressions: _ 
                        }) = shards_iter.peek() {
                            all_multi.push(multi_shard);
                            shards_iter.next();
                        }

                        //pick a random one
                        let multi_range = 0..all_multi.len();
                        all_multi[rand::thread_rng().gen_range(multi_range)].generate()
                    }
                    _ => continue,
                };

                full_sequence.push(generated);
            }

            let (qst, ans): (Vec<_>, Vec<(_, _)>) = full_sequence.into_iter()
                .map(|qst| (qst.expr, (qst.pron, qst.meaning)))
                .unzip();
            let (ans_pron, ans_meaning): (Vec<_>, Vec<_>) = ans.into_iter().unzip();

            println!("Leggere i seguenti kanji: {}", qst.join(""));
            println!("Premere INVIO per mostrare la soluzione:");
            utils::get_line();
            println!(
                "- Pronuncia: {}, significato: {}",
                ans_pron.join("-"),
                ans_meaning.join("")
            );
            println!("-----------------------");
        }

        //the bigger the more likely to be asked
        fn weight(&self) -> usize {
            self.shards.len() * 2
        }
    }
}
