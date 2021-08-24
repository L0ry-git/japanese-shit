use crate::questions;

use std::{fs::{self, File}, io::{self, BufRead}, path::{Path, PathBuf}};

#[derive(Debug)]
pub struct QuestionFile {
    path: PathBuf,
    desc: String,
}

impl QuestionFile {
    
    pub fn filename(&self) -> Option<String> {
        Some(String::from(self.path.file_name()?.to_str()?))
    }

    pub fn desc(&self) -> &String {
        &self.desc
    }

    pub fn read_questions<Kind>(&self) -> io::Result<Vec<questions::BoxedQuestion>> 
        where Kind: questions::Question + 'static
    {
        let file = File::open(self.path.as_path())?;
        let mut ret = Vec::new();

        let mut lines_iter = io::BufReader::new(file)
            .lines()
            .filter_map(Result::ok)
            .peekable();
        let gen_check = lines_iter.peek().ok_or(
            io::Error::new(
                io::ErrorKind::UnexpectedEof, 
                "Errore durante la lettura di un file vuoto!"
            )
        )?.starts_with(questions::gen::GENERATOR_BEGIN);

        let parser = if gen_check {
            questions::parse_from_iter::<questions::gen::QuestionGenerated>
        } else {questions::parse_from_iter::<Kind>};

        while lines_iter.peek().is_some() {
            let question_opt = parser(&mut lines_iter);
            if let Some(question) = question_opt {
                ret.push(question);
            };
        }

        Ok(ret)
    }

    pub fn from_line((line, path): (String, &PathBuf)) -> Self {
        Self {
            path: path.to_path_buf(),
            desc: line
        }
    }

}

#[inline(always)]
fn desc_file_check(path: PathBuf) -> Option<PathBuf> {
    if path.file_name()? == "_desc.txt" {
        Some(path)
    } else {
        None
    }
}

pub fn list_files<P>(path: P) -> io::Result<Vec<QuestionFile>>
where
    P: AsRef<Path>
{
    let dir_path = path.as_ref();
    let dir_str = dir_path.clone();

    let list_files = fs::read_dir(dir_path).unwrap();
    let mut paths: Vec<_> = list_files.map(|dir| dir.unwrap().path()).collect();

    let desc_path = paths.pop().and_then(desc_file_check).ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "Il file _desc.txt non esiste nel percorso {}",
            dir_str.display()
        ),
    ))?;
    let desc_file = File::open(desc_path)?;
    let mut paths_iter = paths.iter();

    let desc_lines: Vec<_> = io::BufReader::new(desc_file)
        .lines()
        .filter_map(Result::ok)
        .map(|path| (path, paths_iter.next().unwrap()))
        .map(QuestionFile::from_line)
        .collect();
    Ok(desc_lines)
}
