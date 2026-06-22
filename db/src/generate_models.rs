use std::fs;

use eyre::Result;
use convert_case::{Case, Casing};
use sea_query::Alias;
use sqlx::SqlitePool;



const TABLES_PATH: &str = "/opt/webman/db/src/tables.rs";
const ENTITIES_PATH: &str = "/opt/webman/db/src/entities.rs";


pub async fn generate_models(pool: &SqlitePool) -> Result<()> {
    generate_tables(pool).await?;
    generate_entities(pool).await?;

    Ok(())
}


pub async fn generate_tables(pool: &SqlitePool) -> Result<()> {
    let mut output = String::new();

    output.push_str("#![allow(non_snake_case)]\n\n");
    output.push_str("use serde::{Deserialize, Serialize};\n");
    output.push_str("use chrono::{DateTime, Utc};\n");
    output.push_str("use utoipa::ToSchema;\n");
    output.push_str("use sqlx::FromRow;\n\n");

    let tables = crate::tables(&pool).await?;

    for table in tables {
        let mut model_def = String::new();

        let mut table_struct_name = table
            .replace('-', "_")
            .to_case(Case::Pascal);

        if table.starts_with("_") {
            table_struct_name = format!("_{table_struct_name}");
        }

        model_def.push_str("#[allow(non_snake_case)]\n");
        model_def.push_str("#[derive(Serialize, Deserialize, ToSchema, FromRow, Clone, Default)]\n");
        model_def.push_str(&format!("pub struct {} {{\n", table_struct_name));

        let columns = crate::columns(Alias::new(&table), &pool).await;

        for column in columns {
            let mut field_name = column.clone();

            if field_name == "type" || field_name == "_type" {
                field_name = "r#type".to_string();
            }

            let column_structure = crate::column_structure(&column, &table, &pool).await?;

            let column_type = column_structure.r#type.to_lowercase();
            let nullable = column_structure.notnull == 0;

            let is_unsigned = column_type.contains("unsigned");

            let rust_type = if column_type == "tinyint(1)" {
                "bool".to_string()
            } else if column_type.starts_with("tinyint") {
                if is_unsigned { "u8" } else { "i8" }.to_string()
            } else if column_type.starts_with("smallint") {
                if is_unsigned { "u16" } else { "i16" }.to_string()
            } else if column_type.starts_with("mediumint") {
                if is_unsigned { "u32" } else { "i32" }.to_string()
            } else if column_type.starts_with("int") {
                if is_unsigned { "u32" } else { "i32" }.to_string()
            } else if column_type.starts_with("bigint") {
                if is_unsigned { "u64" } else { "i64" }.to_string()
            } else if column_type.starts_with("float")
                || column_type.starts_with("double")
                || column_type.starts_with("decimal")
            {
                "f64".to_string()
            } else if matches!(
                column_type.as_str(),
                "blob" | "mediumblob" | "longblob"
            ) {
                "Vec<u8>".to_string()
            } else if column_type == "datetime" {
                "DateTime<Utc>".to_string()
            } else if column_type == "timestamp" {
                "DateTime<Utc>".to_string()
            } else if column_type == "json" {
                "serde_json::Value".to_string()
            } else {
                "String".to_string()
            };

            let field_type = if nullable {
                format!("Option<{}>", rust_type)
            } else {
                rust_type
            };

            model_def.push_str(&format!(
                "    pub {}: {},\n",
                field_name, field_type
            ));
        }

        model_def.push_str("}\n\n");
        output.push_str(&model_def);
    }

    fs::write(TABLES_PATH, &output)?;

    Ok(())

}



pub async fn generate_entities(pool: &SqlitePool) -> Result<()> {
    let mut output = String::new();

    output.push_str("use sea_query::Iden;\n");
    output.push_str("use serde::{ Serialize, Deserialize };\n");
    output.push_str("use strum::{EnumString, Display};\n");
    output.push_str("use enum_iterator::Sequence;\n\n");

    let tables = crate::tables(&pool).await?;

    for table in tables {
        let mut table_enum_name = table
            .replace('-', "_")
            .to_case(Case::Pascal);

        if table.starts_with("_") {
            table_enum_name = format!("_{table_enum_name}");
        }

        output.push_str(
            "#[derive(Ord, Eq, PartialEq, PartialOrd, Iden, Clone, Copy, Hash, Debug, Serialize, Deserialize, EnumString, Display, Sequence)]\n"
        );
        output.push_str("#[serde(rename_all = \"snake_case\")]\n");
        output.push_str("#[strum(serialize_all = \"snake_case\")]\n");
        output.push_str(&format!("pub enum {} {{\n", table_enum_name));

        output.push_str("    Table,\n");

        let columns = crate::columns(Alias::new(&table), &pool).await;

        for column in columns {
            let column_enum_name = column
                .replace('-', "_")
                .to_case(Case::Pascal);

            if column == "type" {
                output.push_str("    #[iden = \"type\"]\n");
            }

            output.push_str(&format!("    {},\n", column_enum_name));
        }

        output.push_str("}\n\n\n");
    }

    fs::write(ENTITIES_PATH, &output)?;

    Ok(())
}