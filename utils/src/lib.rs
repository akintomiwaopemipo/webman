use std::fs;

use prelude::PregReplace;
use rand::{Rng, distributions::Alphanumeric, thread_rng};

pub mod cmd;
pub mod substrings;
pub mod ssh;



pub fn dot_file_extension(file_extension_opt: Option<&str>) -> String {
    if let Some(file_extension) = file_extension_opt{
        if !file_extension.trim().is_empty(){
            format!(".{}", file_extension.trim().to_string().preg_replace(r"^\.", ""))
        }else{
            format!("")
        }
    }else{
        format!("")
    }
}



pub fn file_exists(path: &str) -> bool {
    let metadata = fs::metadata(path);

    if metadata.is_ok(){
        metadata.unwrap().is_file()
    }else{
        false
    }
}


pub fn directory_exists(path: &str) -> bool {
    let metadata = fs::metadata(path);

    if metadata.is_ok(){
        metadata.unwrap().is_dir()
    }else{
        false
    }
}


pub fn random_varchar(charset: &[u8], length: usize) -> String {
    let mut rng = rand::thread_rng();

    (0..length).map(|_| {
            let idx = rng.gen_range(0, charset.len());
            charset[idx] as char
        })
        .collect()
    
}



pub fn random_digits(length: usize)->String{
    random_varchar(b"0123456789", length)
    
}


pub fn random_characters(length: usize) -> String{
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect()
}


pub fn random_alphabets(length: usize)->String {
    random_varchar(b"abcdefghijklmnopqrstuvwxyz", length)
}

pub fn random_hex(length: usize) -> String {
    random_varchar(b"0123456789abcdef", length)
}


pub fn unique_from_vec(vec: Vec<String>, length: usize, context: &str)->String {
    let  mut content: String;
    loop{
        if context=="digits"{
            content = random_digits(length)
        }else{
            content = random_characters(length);
        }

        
        if !vec.contains(&content){
            return content;
        }
    }
    
}



pub fn unique_digits_from_vec(vec: Vec<String>, length: usize)->String{
    unique_from_vec(vec, length, "digits")
}


pub fn unique_characters_from_vec(vec: Vec<String>, length: usize)->String{
    unique_from_vec(vec, length, "characters")
}


pub enum UniqueFromFsContext{
    Digits,
    Characters,
    Hex
}

pub fn unique_from_fs(directory_path: &str, length: usize, file_extension_opt: Option<&str>, context: UniqueFromFsContext)->String{
    
    loop{

        let content = match context{
            UniqueFromFsContext::Digits => format!("{}{}", random_digits(length), dot_file_extension(file_extension_opt)),
            UniqueFromFsContext::Characters => format!("{}{}", random_characters(length), dot_file_extension(file_extension_opt)),
            UniqueFromFsContext::Hex => format!("{}{}", random_hex(length), dot_file_extension(file_extension_opt)),
        };

        
        if !directory_exists(&format!("{}/{}",directory_path,content)){
            return content;
        }
    }
    
}



pub fn unique_digits_from_fs(directory_path: &str, length: usize, file_extension_opt: Option<&str>)->String{
    unique_from_fs(directory_path, length, file_extension_opt, UniqueFromFsContext::Digits)
}



pub fn unique_characters_from_fs(directory_path: &str, length: usize, file_extension_opt: Option<&str>)->String{
    unique_from_fs(directory_path, length, file_extension_opt, UniqueFromFsContext::Characters)
}


pub fn unique_hex_from_fs(directory_path: &str, length: usize, file_extension_opt: Option<&str>)->String{
    unique_from_fs(directory_path, length, file_extension_opt, UniqueFromFsContext::Hex)
}