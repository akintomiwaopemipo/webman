use prelude::*;
use std::fs;
use serde_json::Value;
use serde::Serialize;
use im;
use std::env;
use std::path::{ Path, PathBuf };
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use indexmap::*;
use json::*;
use sha2::{Sha256, Digest};
// use glob::glob;
use colored::Colorize;
use subprocess::Exec;
use std::process::Command;
use rustyline::{ Editor, error::ReadlineError };
use dirs;
use eyre::Result;
// use clap::{ ArgMatches, arg };


pub fn _t(_str: &str)->String{
    String::from(_str)
}


pub fn _th(hashmap: im::HashMap<&str,&str>)->im::HashMap<String,String>{
    let mut h = im::HashMap::new();

    for (k,v) in hashmap{
        h.insert(_t(k), _t(v));
    }
    h
}


pub fn _tv(vec: Vec<&str>)->Vec<String>{
    let mut _vec = vec![];

    for t in vec{
        _vec.push(_t(t));
    }
    _vec
}


pub fn file_get_contents(path: &str) -> String{
    fs::read_to_string(path).expect("An error occcured")
}


pub fn file_put_contents(path: &str, content: &str){
    let mut file = fs::File::create(path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}



pub fn json_encode(json: &Value)->String{
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    json.serialize(&mut ser).unwrap();
    String::from_utf8(ser.into_inner()).unwrap()
}


pub fn json_stringify<T: serde::Serialize>(value: T) -> String{
    let string_value = serde_json::to_string(&value).unwrap();
    return json::stringify(json::parse(&string_value).unwrap())
}


pub fn php_stringify<T: serde::Serialize>(value: T) -> String{
    json_stringify(json_stringify(value))
}


pub fn json_stringify_pretty<T: serde::Serialize>(value: T) -> String{
    let string_value = serde_json::to_string(&value).unwrap();
    return json::stringify_pretty(json::parse(&string_value).unwrap(), 4);
}


pub fn json_decode(json_string: &str)->Value{
    serde_json::from_str(json_string).expect("Decoding failed")
}


pub fn hashmap_keys(hashmap: &im::HashMap<String,String>)->Vec<String>{
    let mut vec: Vec<String> = Vec::new();
    for (key,_value) in hashmap{
        vec.push(String::from(key));
    }
    vec
}


pub fn hashmap_values(hashmap: &im::HashMap<String,String>)->Vec<String>{
    let mut vec: Vec<String> = Vec::new();
    for (_key,value) in hashmap{
        vec.push(_t(value));
    }
    vec
}




pub fn indexmap_keys<K,V>(indexmap: IndexMap<K,V>)->Vec<K>{
    let mut vec: Vec<K> = Vec::new();
    for (key,_value) in indexmap{
        vec.push(key);
    }
    vec
}


pub fn indexmap_values<K,V>(indexmap: IndexMap<K,V>)->Vec<V>{
    let mut vec: Vec<V> = Vec::new();
    for (_key,value) in indexmap{
        vec.push(value);
    }
    vec
}



pub fn object_assign(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                }
                else {
                    object_assign(a.entry(k).or_insert(Value::Null), v);
                }
            } 

            return;
        }
    }

    *a = b;
}




pub fn vec_merge(vec1: &Vec<String>,vec2: &Vec<String>)->Vec<String>{
    let mut vec = vec1.to_owned();
    for i in vec2{
        vec.push(String::from(i));
    }
    vec
}


pub fn vec_multiply(vec: Vec<String>)
->im::HashMap<String,String>{
    let mut hashmap = im::HashMap::new();
    for t in vec{
        hashmap.insert(t.clone(), t.clone());
    }
    hashmap
}


pub fn serde_to_hashmap(serde: &serde_json::Value)->im::HashMap<String,String>{
    let mut h = im::HashMap::new();

    for (k,v) in serde.as_object().unwrap(){
        if v.as_str().is_some(){
            h.insert(String::from(k.as_str()), String::from(v.as_str().unwrap()));
        }
    }
    h
}


pub fn fetch_url(url: &str)->String{
    shell_exec_to_string(&format!("curl --location --request GET '{}'",url))
}



pub fn portal_properties_base_url(portal_id: &str)->String{
    format!("https://demo.icitifysolution.com/specs/assets/portal-properties/{}",portal_id)
}


pub fn portal_properties_url(portal_id: &str)->String{
    format!("{}/{}",portal_properties_base_url(portal_id),"portal-properties.json")
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


pub fn current_dir()->String{
    env::current_dir().unwrap().into_os_string().into_string().unwrap()
}


pub fn dirname(path_str: &str)->String{
    let path = Path::new(path_str);
    _t(path.parent().unwrap().to_str().unwrap())
}



pub fn mkdir(path: &str){
    std::fs::create_dir_all(path).expect("Error occured while creating directory");
}



pub fn random_varchar(charset: &[u8], length: usize)->String{
    let mut rng = rand::thread_rng();

    (0..length).map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
    
}



pub fn random_digits(length: usize)->String{
    random_varchar(b"0123456789", length)
    
}


pub fn random_characters(length: usize)->String{
    let bytes = thread_rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .collect::<Vec<u8>>();
    String::from_utf8(bytes).unwrap()
}


pub fn random_hex(length: usize) -> String{
    random_varchar(b"0123456789abcdef", length)
}



pub fn unique_from_vec(vec: Vec<String>, length: usize, context: &str)->String{
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



pub fn dot_file_extension(file_extension_opt: Option<&str>) -> String{
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


pub fn timestamp()->u64{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}


// pub fn stdin(title: &str)->String{
//     let mut input = String::new();
//     print!("{}",title.bright_cyan());
//     std::io::stdout().flush().unwrap();
//     std::io::stdin().read_line(&mut input).unwrap();
//     std::io::stdout().flush().unwrap();
//     let output = input.replace("\r\n", "");
//     _t(output.trim())

// }





pub fn stdin(title: &str)->String{
        
    let mut rl = Editor::<()>::new();
    let history_file = Path::new(&home_dir()).join(".history.txt");
    if rl.load_history(&history_file).is_err() {
        //println!("No previous history.");
    }
    let mut input: String = _t("");
    loop {
        let readline = rl.readline(&format!("{}",title.bright_cyan()));
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                input = line;
                rl.save_history(&history_file).unwrap();
                break
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                std::process::exit(-1);
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history(&history_file).unwrap();
    input
}


pub fn stdin_or_default(title: &str, default: &str) -> String{
    let _stdin = stdin(&format!("{title} ({default}): "));
    if _stdin.is_empty(){
        default.to_string()
    }else{
        _stdin
    }
}

pub fn get_info(info_title: &str)->String{
    stdin(&format!("\n{} > ",info_title))
}


pub fn base64_decode(string: &str)->String{
    String::from_utf8(base64::decode(string).unwrap()).unwrap()
}


pub fn args()->std::env::Args{
    std::env::args()
}


pub fn base64_arg()->String{
    let mut args = args();
    let arg = args.nth(2);
    base64_decode(&arg.unwrap())
}


pub fn json_base64_arg()->Value{
    json_decode(&base64_arg())
}


pub fn explode(string: &str, delimiter: &str)->Vec<String>{
    let split = string.split(delimiter);
    let mut accumulator: Vec<String> = vec![];
    for s in split {
        accumulator.push(_t(s.trim()));
    }
    if accumulator.len()==1 && accumulator[0].trim().len()==0{
        accumulator = vec![];
    }
    accumulator
}



pub fn _get()->JsonValue{
    json::parse(&base64_arg()).unwrap()
}


pub fn sha256_hash(key: String)->String{
    let mut hasher = Sha256::new();

    // write input message
    hasher.update(key);

    // read hash digest and consume hasher
    format!("{:X}",hasher.finalize())
}


pub fn shell_exec_to_string(command: &str)->String{
    
    _t(std::str::from_utf8(&Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process").stdout).unwrap()
    )
}


pub fn shell_exec(command: &str){
    Exec::shell(command).join().unwrap();  
}

pub fn shell_exec_opt(command: &str) -> Result<()>{
    Exec::shell(command).join()?;
    Ok(())  
}



pub fn sed_edit_sshd_like_variable(variable_name: &str, search_value: &str , replace_value: &str,file_path: &str){
    shell_exec(&format!("sed -i '/^{}/s/{}/{}/' {}",variable_name,search_value,replace_value,file_path));
    
    let from = format!("{}: {}",variable_name,search_value);
    let arrow = format!("->");
    let to = format!("{}: {}",variable_name,replace_value);
    
    println!("{} {} {}",from.bright_green(),arrow.bright_red(),to.bright_cyan());
}


pub fn sed_uncomment_then_edit_sshd_like_variable(variable_name: &str, search_value: &str , replace_value: &str,file_path: &str){ 
    shell_exec(&format!("sed -e 's/#.*{} {}/{} {}/' {} > {}.tmp && mv -f {}.tmp {}",variable_name,search_value,variable_name,replace_value,file_path,file_path,file_path,file_path));
    
    let from = format!("#{}: {}",variable_name,search_value);
    let arrow = format!("->");
    let to = format!("{}: {}",variable_name,replace_value);
    
    println!("{} {} {}",from.bright_green(),arrow.bright_red(),to.bright_cyan());
}




pub fn sed_edit_mysqld_like_variable(variable_name: &str ,replace_value: &str,file_path: &str){
    shell_exec(&format!(r#"sudo sed -i "s/.*{}.*/{} = {}/" {}"#,variable_name,variable_name,replace_value,file_path));
    
    //let from = format!("{}: {}",variable_name,search_value);
    // let arrow = format!("->");
    let var = format!("{}: {}",variable_name,replace_value);
    
    println!("{}",var.bright_green());
}



pub fn sed_uncomment_then_edit_mysqld_like_variable(variable_name: &str ,replace_value: &str,file_path: &str){
    shell_exec(&format!(r#"sudo sed -i "s/#.*{}.*/{} = {}/" {}"#,variable_name,variable_name,replace_value,file_path));
    
    //let from = format!("{}: {}",variable_name,search_value);
    // let arrow = format!("->");
    let var = format!("{} = {}",variable_name,replace_value);
    
    println!("{}",var.bright_green());
}



pub fn sed_add_new_line(value: &str, file_path: &str){
    shell_exec(&format!(r#"sed -i 's/*/{}/' {}"#,value,file_path));
    
    println!("{} append to {}",value.bright_cyan(),file_path.bright_green());
}

pub fn awk_update(property: &str, value: &str, file_path: &str, comment_character: Option<&str>, is_assignment: Option<bool>){
    let comment_character_str = if comment_character.is_some() { comment_character.unwrap() } else { "#" };
    let is_assignment_bool = if is_assignment.is_some() { is_assignment.unwrap() } else { false };

    if is_assignment_bool{

        shell_exec(&format!(r#"awk -i inplace '/^{}{}/{{gsub(/^{}{}/,"{}",$0);$3="{}"}}{{print}}' {}"#, comment_character_str, property, comment_character_str, property, property, value, file_path));

        shell_exec(&format!(r#"awk -i inplace '/^{}/{{$3="{}"}}{{print}}' {}"#, property, value, file_path));
    }else{
        shell_exec(&format!(r#"awk -i inplace '/^{}{}/{{gsub(/^{}{}/,"{}",$0);$2="{}"}}{{print}}' {}"#, comment_character_str, property, comment_character_str, property, property, value, file_path));
        shell_exec(&format!(r#"awk -i inplace '/^{}/{{$2="{}"}}{{print}}' {}"#, property, value, file_path));
    }

    let from = format!("{}: {}", property, value);
    let separator = format!("->");
    let to = format!("{}",file_path);
    
    println!("{} {} {}",from.bright_green(), separator.bright_red(),to.bright_cyan());

}


pub fn awk_add_line_after_match(pattern: &str, value: &str, file_path: &str){
    shell_exec(&format!("awk '/{}/{{print;print '{}';next}}1' {}", pattern, value, file_path));
}


pub fn add_new_line(value: &str, file_path: &str){
    let mut file_content = file_get_contents(file_path);
    
    file_content = format!("{}\n{}",file_content,value);

    file_put_contents(file_path,&file_content);

    println!("{} append to {}",value.bright_cyan(),file_path.bright_green());
}



pub fn change_unix_user_password(user: &str,password: &str){
    shell_exec(&format!(r#"echo "{username}:{password}" | chpasswd"#,username=user,password=password))
}


pub fn change_current_unix_user_password(password: &str){
    shell_exec(&format!("wget api.wpanel.dev/wpanel/bash/passwd.sh -q && chmod +x passwd.sh && ./passwd.sh {} && rm -rf passwd.sh",password));
}


pub fn change_wfm_password(user: &str, password: &str){
    let mut settings = object!{};
    settings["auth_users"][user] = fetch_url(&format!("api.wpanel.dev/password-hash?password={}",password)).into();

    file_put_contents(&format!("/home/{}/public_html/file-manager/settings.json",user),json::stringify_pretty(settings, 4).as_str());
}


pub fn stdin_lest_is_none(stdin_title: &str, option: Option<&str>)->String{
    let response: String;

    if option.is_some(){
        response = _t(option.unwrap());
    }else{
        response = stdin(stdin_title);
    }
    response
}

pub fn pathbuf_to_string(pathbuf: PathBuf)->String{
    pathbuf.into_os_string().into_string().unwrap()
}

pub fn home_dir()->String{
    pathbuf_to_string(dirs::home_dir().unwrap())
}


pub fn default_ppk_path() -> String{
    format!("{}/.ssh/id_rsa", home_dir())
}


pub fn backup_bucket_name(node_id: &str, node: WebmanNode) -> String{
    format!("{}-icitifysms-{}", node.domainName.unwrap().replace(".", "-"), node_id)
}