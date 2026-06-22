use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeSSH{
    
    pub username: String,
    
    pub password: Option<String>,
    
    pub private_key: Option<String>
}


#[derive(Serialize, Deserialize, Clone)]
pub struct NodeMySQL{
    
    pub username: Option<String>,
    
    pub password: Option<String>,

    pub database: Option<String>,
    
    pub databases: Option<Vec<String>>,

    pub phpmyadmin_auth_key: Option<String>
}



#[derive(Serialize, Deserialize, Clone)]
pub struct NodeBackup{
    pub bucket: String,

    pub regulation_range: u32
}




#[derive(Serialize, Deserialize, Clone)]
pub struct NodeData {

    pub node_id: String,

    pub app_id: String,

    pub name: String,
    
    pub host: String,
    
    pub hostname: Option<String>,
    
    pub home: Option<String>,

    pub rel_dirname: Option<String>,

    pub remote_home_dir: Option<String>,
    
    pub domain_name: String,

    pub custom_domain: Option<String>,

    pub base_url: Option<String>,
    
    pub node_url: String,
    
    pub ssh: NodeSSH,
    
    pub mysql: Option<NodeMySQL>,

    pub backup: NodeBackup,

    pub timezone_offset: Option<i64>,

    pub mimics: Option<String>,

    pub active: bool,

    pub dev_mode: bool
}




#[derive(Serialize, Deserialize, Clone)]
pub struct ServerData {

    pub root_ip: String,

    pub username: String,

    pub password: String,

    pub hostname: String,

    pub provider: String,
}




#[derive(Serialize, Deserialize, Clone)]
pub struct Test {
    
    pub node_id: String,
    
    pub active: bool
}




#[derive(Serialize, Deserialize, Clone)]
pub struct GitConfigUser{

    pub name: String,

    pub email: String
}



#[derive(Serialize, Deserialize, Clone)]
pub struct GitConfig{
    
    pub user: GitConfigUser

}



#[derive(Serialize, Deserialize, Clone)]
pub struct Git {

    pub config: GitConfig

}




#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigData {
    pub nodes: IndexMap<String, NodeData>,

    pub servers: IndexMap<String, ServerData>,

    pub application_type: Option<String>,

    pub test: Option<Test>,

    pub nodes_config_name: String,

    pub remotes: Option<Vec<String>>,

    pub git: Option<Git>

}