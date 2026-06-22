use std::path::PathBuf;
use crud::sqlite::Crud;
use db::entities::{Metadata, Nodes, Servers};
use indexmap::IndexMap;
use prelude::SerdeJsonSerialize;
use sqlx::SqlitePool;
use util::unique_characters_from_fs;
use eyre::Result;
use db::tables::{ Nodes as DbNodes, Servers as DbServers };
use utils::ssh::Session;
use crate::config_structs::{ConfigData, NodeData, ServerData};


pub struct Node {
    pub node_id: String,
    pool: SqlitePool,
}


impl Node{


    pub fn new(node_id: &str, pool: &SqlitePool) -> Self {
        Self {
            node_id: node_id.into(),
            pool: pool.clone()
        }
    }


    pub async fn data(&self) -> Result<NodeData> {  
        let db_row = Crud::new(Nodes::Table, &self.pool)
            .set_column(Nodes::NodeId, self.node_id.clone().into())
            .fetch_one::<DbNodes>().await?;
        Ok(Self::from_db_row(db_row)?)
    }
        


    pub fn from_db_row(row: DbNodes) -> Result<NodeData> {
        let node = NodeData {
            node_id: row.node_id,
            app_id: row.app_id,
            name: row.name,
            host: row.host,
            hostname: row.hostname,
            home: None,
            rel_dirname: row.rel_dirname,
            remote_home_dir: row.remote_home_dir,
            domain_name: row.domain_name,
            custom_domain: row.custom_domain,
            base_url: None,
            node_url: row.node_url,
            ssh: serde_json::from_str(&row.ssh)?,
            mysql: row.mysql.map(|mysql| serde_json::from_str(&mysql)).transpose()?,
            backup: serde_json::from_str(&row.backup)?,
            timezone_offset: row.timezone_offset.map(|offset| offset as i64),
            mimics: row.mimics,
            active: row.active.unwrap_or(0) == 1,
            dev_mode: row.dev_mode.unwrap_or(0) == 1
        };
        Ok(node)
    }



    pub async fn list(pool: &SqlitePool) -> Result<IndexMap<String, NodeData>> {
        let mut nodes = IndexMap::new();
        for row in Crud::new(Nodes::Table, pool).fetch().await?{
            let node = Self::from_db_row(row)?;
            nodes.insert(node.node_id.clone(), node);
        }
        Ok(nodes)
    }



    pub async fn add(data: NodeData, pool: &SqlitePool) -> Result<()>{
        Crud::new(Nodes::Table, pool)
            .set_column(Nodes::NodeId, data.node_id.clone().into())
            .set_column(Nodes::AppId, data.app_id.into())
            .set_column(Nodes::Name, data.name.into())
            .set_column(Nodes::Host, data.host.into())
            .set_column(Nodes::Hostname, data.hostname.into())
            .set_column(Nodes::RelDirname, data.rel_dirname.into())
            .set_column(Nodes::RemoteHomeDir, data.remote_home_dir.into())
            .set_column(Nodes::DomainName, data.domain_name.into())
            .set_column(Nodes::CustomDomain, data.custom_domain.into())
            .set_column(Nodes::NodeUrl, data.node_url.into())
            .set_column(Nodes::Ssh, serde_json::to_string(&data.ssh)?.into())
            .set_column(Nodes::Mysql, data.mysql.map(|mysql| mysql.stringify()).into())
            .set_column(Nodes::Backup, serde_json::to_string(&data.backup)?.into())
            .set_column(Nodes::Mimics, data.mimics.into())
            .set_column(Nodes::Active, if data.active { 1 } else { 0 }.into())
            .set_column(Nodes::DevMode, if data.dev_mode { 1 } else { 0 }.into())
            .insert().await?;
        Ok(())
    }




    pub async fn update(data: NodeData, pool: &SqlitePool) -> Result<()>{
        Crud::new(Nodes::Table, pool)

            .set_column(Nodes::NodeId, data.node_id.clone().into())
            
            .set_update(Nodes::AppId, data.app_id.into())
            .set_update(Nodes::Name, data.name.into())
            .set_update(Nodes::Host, data.host.into())
            .set_update(Nodes::Hostname, data.hostname.into())
            .set_update(Nodes::RelDirname, data.rel_dirname.into())
            .set_update(Nodes::RemoteHomeDir, data.remote_home_dir.into())
            .set_update(Nodes::DomainName, data.domain_name.into())
            .set_update(Nodes::CustomDomain, data.custom_domain.into())
            .set_update(Nodes::NodeUrl, data.node_url.into())
            .set_update(Nodes::Ssh, serde_json::to_string(&data.ssh)?.into())
            .set_update(Nodes::Mysql, data.mysql.map(|mysql| mysql.stringify()).into())
            .set_update(Nodes::Backup, serde_json::to_string(&data.backup)?.into())
            .set_update(Nodes::Mimics, data.mimics.into())
            .set_update(Nodes::Active, if data.active { 1 } else { 0 }.into())
            .set_update(Nodes::DevMode, if data.dev_mode { 1 } else { 0 }.into())
            
            .update().await?;
        Ok(())
    }

    
    
    pub async  fn hostname(&self) -> Result<String> {
        let node = self.data().await?;
        if node.hostname.is_some(){
            Ok(node.hostname.clone().unwrap())
        }else{
            Ok(format!("{}_{}", node.ssh.username, node.host))
        }
    }


    pub async fn remote_dir(&self) -> Result<String> {
        let node = self.data().await?;
        Ok(format!("/home/{}", node.ssh.username))
    }
    
    pub async fn remote_node_dir(&self) -> Result<String> {
        let node = self.data().await?;
        Ok(node.home.clone().unwrap_or(format!("{}/public_html", self.remote_dir().await?)))
    }


    pub async fn ssh(&self) -> Result<Session> {
        let node = self.data().await?;
        Session::connect(
            &node.host,
            &node.ssh.username,
            &node.ssh.password.unwrap_or_default(),
        ).await
    }


}


pub struct Server {
    pub root_ip: String,
    pool: SqlitePool,
}


impl Server {

     pub fn new(root_ip: &str, pool: &SqlitePool) -> Self {
        Self {
            root_ip: root_ip.into(),
            pool: pool.clone()
        }
    }



    pub async fn data(&self) -> Result<ServerData> {
        let db_row = Crud::new(Servers::Table, &self.pool)
            .set_column(Servers::RootIp, self.root_ip.clone().into())
            .fetch_one::<DbServers>().await?;
        Ok(Self::from_db_row(db_row)?)
    }
        


    pub fn from_db_row(row: DbServers) -> Result<ServerData> {
        Ok(ServerData {
            root_ip: row.root_ip,
            username: row.username,
            password: row.password,
            hostname: row.hostname.unwrap_or_default(),
            provider: row.provider.unwrap_or_default(),
        })
    }


    pub async fn list(pool: &SqlitePool) -> Result<IndexMap<String, ServerData>> {
        let mut servers = IndexMap::new();
        for row in Crud::new(Servers::Table, pool).fetch().await?{
            let server = Self::from_db_row(row)?;
            servers.insert(server.root_ip.clone(), server);
        }
        Ok(servers)
    }


    pub async fn ssh(&self) -> Result<Session> {
        let server = self.data().await?;
        Session::connect(
            &server.root_ip,
            &server.username,
            &server.password,
        ).await
    }
}



pub struct Config {
    pool: SqlitePool,
}


impl Config {

    pub fn new(pool: &SqlitePool) -> Self {
        Self {
            pool: pool.clone()
        }
    }

    pub async fn data(&self) -> Result<ConfigData> {

        let nodes = Node::list(&self.pool).await?;
        let servers = Server::list(&self.pool).await?;


        let application_type = None;

        let test = None;

        let nodes_config_name = self.nodes_config_name().await.unwrap_or("webman.config".into());

        let remotes = None;

        let git = None;

        
        Ok(ConfigData {
            nodes,
            servers,
            application_type,
            test,
            nodes_config_name,
            remotes,
            git,
        })
    }
    

    pub async fn nodes(&self) -> Result<IndexMap<String, NodeData>> {
        let config = self.data().await?;
        Ok(config.nodes)
    }


    pub async fn metadata(&self, property: &str) -> Result<String> {
        Ok(Crud::new(Metadata::Table, &self.pool)
            .set_column(Metadata::Property, property.into())
            .select_column(Metadata::Value)
            .fetch_one::<(String,)>().await?.0)
    }


    pub async fn nodes_config_name(&self) -> Result<String> {
        self.metadata("nodes_config_name").await
    }
    


    pub fn connection_url() -> String {
        let document_root = Self::document_root();
        format!("sqlite://{document_root}/.webman/config.db")
    }


    pub async fn connection_pool() -> Result<SqlitePool> {
        Ok(SqlitePool::connect(&Self::connection_url()).await?)
    }


    pub fn document_root()->String{
        let mut _dirname = util::current_dir();
    
        loop{
            if util::file_exists(&format!(r#"{}/.webman/config.db"#,&_dirname)){
                return _dirname;
            }else{
                _dirname = util::dirname(&_dirname);
            }
    
            if _dirname == "/"{
                return String::from(""); 
            }
        }
    }
    
    
    
    pub async fn node_ids(&self) -> Result<Vec<String>>{
        Ok(util::indexmap_keys(self.data().await?.nodes))
    }
    
    
    
    pub async fn active_node_ids(&self) -> Result<Vec<String>>{
        let mut accumulator: Vec<String> = vec![];
    
        for node_id in self.node_ids().await?{
            let node = Node::new(&node_id, &self.pool).data().await?;
            
            if node.active{
                accumulator.push(node_id);
            }
            
        }
        
        Ok(accumulator)
    }




    pub fn tmp_directory() -> String{
        let _tmp_directory = PathBuf::from_iter([
            &Config::document_root(),
            "tmp"
        ]).into_os_string().into_string().unwrap();
        if !util::directory_exists(&_tmp_directory){
            Self::mkdir(&_tmp_directory);
        }
        String::from(_tmp_directory)
    }


    pub fn new_tmp_file(file_extension: &str, length: usize) -> String{
        
        let mut extension = format!("{}", file_extension);

        if !extension.is_empty(){
            extension = format!(".{}", file_extension);
        }
    
        let _tmp_directory = Self::tmp_directory();
    
        let file_name = format!("{}", unique_characters_from_fs(&_tmp_directory, length.try_into().unwrap(), Some(&extension)));
    
        format!("{}/{}", _tmp_directory, file_name)
    
    
    }



    /// Make directory recursively
    pub fn mkdir(path: &str){
        std::fs::create_dir_all(path).expect("Error occured while creating directory");
    }


    pub fn copy_directory(from: &str, to: &str){
        Self::mkdir(from);
        Self::mkdir(to);
        fs_extra::dir::copy(
            from, 
            to, 
            &fs_extra::dir::CopyOptions { ..Default::default() }
        ).unwrap();
    }


    pub fn config_file() -> String{
        let document_root = Self::document_root();
        format!("{document_root}/.webman/config.db")
    }


}