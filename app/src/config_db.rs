use eyre::Result;
use sqlx::SqlitePool;

pub struct ConfigDb;

impl ConfigDb {
    
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

}