use std::{io::{self, BufRead}};

pub fn get_line() -> Option<String> {
    let mut ret = String::new();
    io::stdin()
        .lock()
        .read_line(&mut ret)
        .and_then(|_| Ok(String::from(&ret[..ret.len() - 2])))
        .ok()
}

pub fn get_line_number() -> Option<usize> {
    let line = get_line()?;

    match line.parse() {
        Ok(ret) => Some(ret),
        _ => None
    }
}

#[macro_export]
macro_rules! get_line_enum {
    ($E:ty) => {
        match utils::get_line_number().and_then(|n| Some(n-1)) {
            Some(v @ 0..<$E>::VARIANT_COUNT) => {
                Some(unsafe {std::mem::transmute::<u8, $E>(v as u8)})
            },
            _ => None
        }
    };
}