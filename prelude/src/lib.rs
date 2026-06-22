use std::{env, fs, path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};
use mysql::*;
use indexmap::IndexMap;
use subprocess::Exec;
use eyre::{ Report, eyre };



#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeSSH{
    
    pub username: String,
    
    pub password: Option<String>,
    
    pub privateKey: Option<String>
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeMySQL{
    
    pub username: Option<String>,
    
    pub password: Option<String>,

    pub database: Option<String>,
    
    pub databases: Option<Vec<String>>,

    pub phpmyadminAuthKey: Option<String>
}


#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebmanNodeBackup{
    pub bucket: String,

    pub regulationRange: u32
}



#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebmanNode{

    pub name: Option<String>,
    
    pub host: String,
    
    pub hostname: Option<String>,
    
    pub home: Option<String>,

    pub relDirname: Option<String>,

    pub remoteHomeDir: Option<String>,
    
    pub domainName: Option<String>,

    pub baseUrl: Option<String>,
    
    pub nodeUrl: Option<String>,
    
    pub ssh: NodeSSH,
    
    pub mysql: Option<NodeMySQL>,

    pub backup: Option<WebmanNodeBackup>,

    pub mimics: Option<String>,

    pub active: Option<bool>,

    pub devMode: Option<bool>
}



#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct  WebmanRoot{

    pub username: String,

    pub password: String,

    pub privateKey: Option<String>,

    pub mysql: Option<NodeMySQL>
}



#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebmanTest{
    
    nodeId: String,
    
    active: bool
}



#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebmanGitConfigUser{

    pub name: String,

    pub email: String
}


#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebmanGitConfig{
    
    pub user: WebmanGitConfigUser

}


#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebmanGit{

    pub config: WebmanGitConfig

}



#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebmanConfig{
    pub nodes: IndexMap<String, WebmanNode>,

    pub roots: IndexMap<String, WebmanRoot>,

    pub applicationType: Option<String>,

    pub test: Option<WebmanTest>,

    pub nodeConfigName: Option<String>,

    pub remotes: Option<Vec<String>>,

    pub git: Option<WebmanGit>

}


fn shell_exec(command: &str){
    Exec::shell(command).join().unwrap();  
}

pub fn shell_exec_to_string(command: &str)->String{
    
    std::str::from_utf8(&Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process").stdout).unwrap()
        .trim_end()
        .to_string()
}


pub struct JsonQueryCommand{
    command: String,
    tmp_file: String
}


pub trait SerdeJsonSerialize<T>
    where T: serde::Serialize
{
    fn stringify(&self) -> String;

    fn php_stringify(&self) -> String;

    fn stringify_pretty(&self) -> String;

    fn print(&self);

    fn pretty_print(&self);

    fn json_value(&self) -> serde_json::Value;

    fn query_command(&self, query: &str) -> JsonQueryCommand;

    fn query_to_string(&self, query: &str) -> String;

    fn query(&self, query: &str);

}


impl <T>SerdeJsonSerialize<T> for T
    where T: serde::Serialize
{
    fn stringify(&self) -> String {
        let string_value = serde_json::to_string(self).unwrap();
        json::stringify(json::parse(&string_value).unwrap())
    }

    fn php_stringify(&self) -> String {
        self.stringify().stringify()
    }

    fn stringify_pretty(&self) -> String {
        let string_value = serde_json::to_string(self).unwrap();
        json::stringify_pretty(json::parse(&string_value).unwrap(), 4)
    }


    fn print(&self) {
        println!("{}", self.stringify());
    }

    fn pretty_print(&self) {
        println!("{}", self.stringify_pretty());
    }


    fn json_value(&self) -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(&self.stringify()).unwrap()
    }


    fn query_command(&self, query: &str) -> JsonQueryCommand{
    
        let tmp_file =  shell_exec_to_string("mktemp");

        fs::write(&tmp_file, self.stringify()).unwrap();

        let debug_mode = debug_mode();
        
        if debug_mode{
            println!("tmp file: {tmp_file}");
        }

        if query.contains("'"){
            Err::<(), Report>(eyre!("Query must not contain single quote")).unwrap();
        }
        
        let command = format!(r#"cat "{tmp_file}" | jq '.[] | select({query})' | jq -s '.'"#);
        
        if debug_mode{
            println!("{command}");
        }
        JsonQueryCommand { command, tmp_file }
    }

    fn query_to_string(&self, query: &str) -> String {
        let query_command = self.query_command(query);
        let execution_result = shell_exec_to_string(&query_command.command);
        if !debug_mode(){
            fs::remove_file(&query_command.tmp_file).unwrap();
        }
        execution_result
    }

    fn query(&self, query: &str){
        let query_command = self.query_command(query);
        shell_exec(&query_command.command);
        if !debug_mode(){
            std::fs::remove_file(&query_command.tmp_file).unwrap();
        }
    }


}



pub trait SerdeJsonValueSerialize{
    
    fn as_string(&self) -> String;

    fn as_minified_string(&self) -> String;

    fn as_boolean(&self) -> bool;

    fn merge(&mut self, value: &serde_json::Value);

}


impl SerdeJsonValueSerialize for serde_json::Value{


    fn as_string(&self) -> String {
        
        type Value = serde_json::Value;
        
        match self{
            Value::Array(array) => array.stringify_pretty(),
            Value::Object(object) => object.stringify_pretty(),
            Value::String(string) => string.clone(),
            Value::Number(number) => number.to_string(),
            Value::Bool(boolean) => boolean.to_string(),
            Value::Null => String::default()
        }

    }

    fn as_minified_string(&self) -> String {
        
        type Value = serde_json::Value;
        
        match self{
            Value::Array(array) => array.stringify(),
            Value::Object(object) => object.stringify(),
            Value::String(string) => string.clone(),
            Value::Number(number) => number.to_string(),
            Value::Bool(boolean) => boolean.to_string(),
            Value::Null => String::default()
        }
    }

    
    fn as_boolean(&self) -> bool {
        type Value = serde_json::Value;
        
        match self{
            Value::String(string) => string == "1" || string == "true",
            Value::Number(number) => number.to_string() == "1",
            Value::Bool(boolean) => *boolean,
            _ => false
        }
    }


    fn merge(&mut self, value: &serde_json::Value){
        type Value = serde_json::Value;
        let a = self;
        let b = value;

        match (a, b) {
            (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
                for (k, v) in b {
                    a.entry(k.clone()).or_insert(Value::Null).merge(v);
                }
            }
            (a, b) => {
                *a = b.clone();
            }
        }
    }
}



pub trait PregReplace{
    fn preg_replace(&self, pattern: &str, replace_with: &str) -> String;
}


impl PregReplace for String{
    fn preg_replace(&self, pattern: &str, replace_with: &str) -> String {
        regex::Regex::new(pattern).unwrap().replace_all(self, replace_with).into_owned()
    }
}


pub fn debug_mode() -> bool{
    env::var("WEBMAN_DEBUG")
        .map(|env_var| env_var == "1")
        .unwrap_or(false)
}



pub fn print_header_block(title: &str){
    println!();
    println!("### ----------------------------------------------------------");
    println!("###              {title}");
    println!("### ----------------------------------------------------------");
}


pub trait ToPathString {
    fn to_path_string(&self) -> String;
}

impl ToPathString for PathBuf{
    fn to_path_string(&self) -> String {
        self.clone().into_os_string().into_string().unwrap()
    }
}