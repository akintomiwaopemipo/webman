use indexmap::IndexMap;
use sea_query::Iden;
use serde_json::json;
use sqlx::{mysql::MySqlRow , SqlitePool, Row, Column};
use utils::{random_characters, random_digits};
use eyre::Result;

pub mod generate_models;
pub mod tables;
pub mod entities;



#[derive(sqlx::FromRow, Clone, Debug)]
pub struct ColumnStructure {
    pub cid: i64,
    pub name: String,
    pub r#type: String,
    pub notnull: i64,
    pub dflt_value: Option<String>,
    pub pk: i64,
}


// pub mod table;
// pub mod entity;


#[derive(PartialEq)]
pub enum RandomFromDbContext{
    Digits,
    Characters
}


pub mod sea_query_custom_functions {
    
    use sea_query::{Iden, Write};

    pub struct Length;

    impl Iden for Length {
        fn unquoted(&self, s: &mut dyn Write) {
            write!(s, "LENGTH").unwrap();
        }
    }
}



pub struct DbConnectionParams {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub name: String
}






pub fn row_to_json_value(row: MySqlRow) -> serde_json::Value{
    let mut json_row = json!({});
    for column in row.columns(){
        let column_name = column.name();

        if let Ok(value) = row.try_get::<i32,_>(column_name){
            json_row[column_name] = value.into();
        }else{
            if let Ok(value) = row.try_get::<f64,_>(column_name){
                json_row[column_name] = value.into();
            }else{
                if let Ok(value) = row.try_get::<String,_>(column_name){
                    json_row[column_name] = value.into();
                }else{
                    json_row[column_name] = "".into();
                }
            }
        }

    }
    json_row
}


pub fn rows_to_json_values(row: Vec<MySqlRow>) -> Vec<serde_json::Value>{
    row.into_iter().map(row_to_json_value).collect()
}




pub async fn unique_from_db<T: Iden>(table_name: T, column_name: T, length: usize, pool: &SqlitePool, context: RandomFromDbContext)->String{

    loop{

        let content = match context{
            RandomFromDbContext::Digits => random_digits(length),
            RandomFromDbContext::Characters => random_characters(length)
        };


        let rows = sqlx::query(&format!("SELECT * FROM `{}` WHERE `{}` = ?", table_name.to_string(), column_name.to_string()))
            .bind(content.clone())
            .fetch_all(pool)
            .await.unwrap();

        if rows.len() == 0{
            return content;
        }
    }
    
}



pub async fn unique_digits<T: Iden>(table_name: T, column_name: T, length: usize, pool: &SqlitePool)->String{
    unique_from_db(table_name, column_name, length, pool, RandomFromDbContext::Digits).await
}



pub async fn unique_characters<T: Iden>(table_name: T, column_name: T, length: usize, pool: &SqlitePool)->String{
    unique_from_db(table_name, column_name, length, pool, RandomFromDbContext::Characters).await
}



pub async fn tables(pool: &SqlitePool) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar::<_, String>("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .fetch_all(pool)
        .await?)
}



pub async fn drop_all_tables(pool: &SqlitePool) -> Result<()> {
    for table in tables(pool).await?{
        sqlx::query(&format!("DROP TABLE `{table}`"))
            .execute(pool)
            .await?;
    }

    Ok(())
}


pub async fn columns<T: Iden>(table: T, pool: &SqlitePool) -> Vec<String>{
    columns_from_str(&table.to_string(), pool).await
}



pub async fn column_structures(table: &str, pool: &SqlitePool) -> Result<IndexMap<String, ColumnStructure>, sqlx::Error> {
    let sql = format!("PRAGMA table_info(`{}`)", table);

    let columns: Vec<ColumnStructure> =
        sqlx::query_as(&sql)
            .fetch_all(pool)
            .await?;

    Ok(columns
        .into_iter()
        .map(|column| (column.name.clone(), column))
        .collect())
}

pub async fn column_structure(column: &str, table: &str, pool: &SqlitePool) -> Result<ColumnStructure, sqlx::Error> {
    let columns = column_structures(table, pool).await?;

    Ok(columns.get(column).cloned().unwrap())
}



pub async fn columns_from_str(table: &str, pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar::<_, String>(&format!(
        "SELECT name FROM pragma_table_info('{table}') ORDER BY cid"
    ))
    .fetch_all(pool)
    .await
    .unwrap()
}


pub async fn column_exists<T: Iden>(column: T, table: T, pool: &SqlitePool) -> bool{
    columns(table, pool).await.contains(&column.to_string())
}


pub async fn add_column<T: Iden + Clone>(column: T, table: T, mut column_definitions: IndexMap<&str, &str>, pool: &SqlitePool){
    
    if !column_definitions.contains_key("type"){
        column_definitions.insert("type", "VARCHAR");
    }

    if !column_definitions.contains_key("length"){
        column_definitions.insert("length", "50");
    }

    if !column_definitions.contains_key("null"){
        column_definitions.insert("null", "true");
    }

    let mut _type = format!("{}({})", column_definitions["type"], column_definitions["length"]);

    let _after = if column_definitions.contains_key("after"){
        format!("AFTER `{}`", column_definitions["after"])
    }else{
        "".to_string()
    };


    let _null = if column_definitions.contains_key("null"){
        "".to_string()
    }else{
        format!("NOT  NULL")
    };


    let _default = if column_definitions.contains_key("default"){
        format!("DEFAULT {}", column_definitions["default"])
    }else{
        "".to_string()
    };    


    let sql_type = sql_character(column_definitions["type"]);


    if sql_type.contains("TEXT") || sql_type.contains("JSON") || sql_type.contains("BLOB"){
        _type = column_definitions["type"].to_string();
    }

    if !column_exists(column.clone(), table.clone(), pool).await{
        sqlx::query(&format!(
            "ALTER TABLE {table} ADD `{column}` {_type} {_null} {_default} {_after}", 
            table=table.to_string(), column=column.to_string(), _type=_type, _null=_null, _default=_default, _after=_after
        ))
        .execute(pool).await.unwrap();
    }



}


pub fn sql_character(character: &str) -> String{
    return character.to_string().trim().to_uppercase()
}




pub async fn last_insert_id<T: Iden>(table_name: T, pool: &SqlitePool) -> i32{
    let key = "id";
    if let Ok(row) = sqlx::query(&format!("SELECT {key} FROM `{table_name}` ORDER BY {key} DESC", table_name=table_name.to_string())).fetch_one(pool).await{
        row.get::<i32,_>(key)
    }else{
        i32::default()
    }
}


pub async fn last_insert_uid<T: Iden>(table_name: T, pool: &SqlitePool) -> i32{
    let key = "uid";
    if let Ok(row) = sqlx::query(&format!("SELECT {key} FROM `{table_name}` ORDER BY {key} DESC", table_name=table_name.to_string())).fetch_one(pool).await{
        row.get::<i32,_>(key)
    }else{
        i32::default()
    }
}