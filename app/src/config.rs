use std::path::Path;

use crud::sqlite::Crud;
use db::entities::{Metadata, Nodes, Servers};
pub use indexmap::IndexMap;
use prelude::SerdeJsonSerialize;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use eyre::Result;
use db::tables::{ Nodes as DbNodes, Servers as DbServers };
use utils::ssh::Session;


pub use sqlx::SqlitePool as Pool;

pub const SSL_ROOT_PARENT: &'static str = "/etc/ssl/icitifysms";



#[derive(Serialize, Deserialize, Clone)]
pub struct NodeSsh {
    
    pub username: String,
    
    pub password: Option<String>,
    
    pub private_key: Option<String>
}


#[derive(Serialize, Deserialize, Clone)]
pub struct NodeMySql {
    
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
    
    pub ssh: NodeSsh,
    
    pub mysql: Option<NodeMySql>,

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


    pub async fn active_nodes(pool: &SqlitePool) -> Result<IndexMap<String, NodeData>> {
        let mut nodes = IndexMap::new();
        for row in Crud::new(Nodes::Table, pool)
            .set_column(Nodes::Active, 1.into())
            .fetch().await?{
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








pub struct CustomDomainData {
    pub domain_name: String,
    pub node_id: String,
    pub active: bool
}



pub struct CustomDomain {
    pub domain_name: String,
    pub node_id: String,
}



impl CustomDomain {

    pub fn new(domain_name: &str, node_id: &str) -> Self {
        Self {
            domain_name: domain_name.into(),
            node_id: node_id.into()
        }
    }

    pub fn ssl_dir(&self) -> String {
        let ssl_key = &self.node_id[..3];
        let domain_name = &self.domain_name;
        format!("{SSL_ROOT_PARENT}/{ssl_key}/{domain_name}")
    }

    pub fn ssl_key_path(&self) -> Result<String> {
        let ssl_dir = self.ssl_dir();
        Ok(format!("{ssl_dir}/key.pem"))
    }


    pub fn ssl_cert_path(&self) -> Result<String> {
        let ssl_dir = self.ssl_dir();
        Ok(format!("{ssl_dir}/cert.pem"))
    }



    pub fn active_domains_from_cache(nodes: &IndexMap<String, NodeData>) -> Result<Vec<CustomDomainData>> {
        
        let mut custom_domains = Vec::new();
        
        for domain in Self::list_from_cache(nodes)? {

            let custom_domain = CustomDomain::new(&domain.domain_name, &domain.node_id);

            if domain.active && Path::new(&custom_domain.ssl_key_path()?).exists() && Path::new(&custom_domain.ssl_cert_path()?).exists() {
                custom_domains.push(domain);
            }
        }

        Ok(custom_domains)
    }


    
    pub fn list_from_cache(nodes: &IndexMap<String, NodeData>) -> Result<Vec<CustomDomainData>> {
        
        let mut custom_domains = Vec::new();
        
        for node in nodes.values() {

            if let Some(domain_name) = &node.custom_domain {
                custom_domains.push(CustomDomainData{
                    domain_name: domain_name.clone(),
                    node_id: node.node_id.clone(),
                    active: node.active
                });
            }
        }

        Ok(custom_domains)


    }



    pub async fn list(pool: &SqlitePool) -> Result<Vec<CustomDomainData>> {
        let nodes = Node::list(pool).await?;
        Ok(Self::list_from_cache(&nodes)?)
    }


    pub async fn active_domains(pool: &SqlitePool) -> Result<Vec<CustomDomainData>> {
        let nodes = Node::list(pool).await?;
        Ok(Self::active_domains_from_cache(&nodes)?)
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


}