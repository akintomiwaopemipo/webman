use eyre::Result;
use sqlx::SqlitePool;

use crate::app::App;

pub struct ConfigDb;

impl ConfigDb {
    
    pub fn connection_url() -> String {
        let document_root = App::document_root();
        format!("sqlite://{document_root}/.webman/config.db")
    }


    pub async fn connection_pool() -> Result<SqlitePool> {
        Ok(SqlitePool::connect(&Self::connection_url()).await?)
    }

}