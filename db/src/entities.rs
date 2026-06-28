use sea_query::Iden;
use serde::{ Serialize, Deserialize };
use strum::{EnumString, Display};
use enum_iterator::Sequence;

#[derive(Ord, Eq, PartialEq, PartialOrd, Iden, Clone, Copy, Hash, Debug, Serialize, Deserialize, EnumString, Display, Sequence)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum _SqlxMigrations {
    Table,
    Version,
    Description,
    InstalledOn,
    Success,
    Checksum,
    ExecutionTime,
}


#[derive(Ord, Eq, PartialEq, PartialOrd, Iden, Clone, Copy, Hash, Debug, Serialize, Deserialize, EnumString, Display, Sequence)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Metadata {
    Table,
    Id,
    Property,
    Value,
}


#[derive(Ord, Eq, PartialEq, PartialOrd, Iden, Clone, Copy, Hash, Debug, Serialize, Deserialize, EnumString, Display, Sequence)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Nodes {
    Table,
    Id,
    NodeId,
    AppId,
    Name,
    Host,
    Hostname,
    RelDirname,
    RemoteHomeDir,
    DomainName,
    CustomDomain,
    NodeUrl,
    Ssh,
    Mysql,
    Backup,
    TimezoneOffset,
    Mimics,
    Active,
    DevMode,
}


#[derive(Ord, Eq, PartialEq, PartialOrd, Iden, Clone, Copy, Hash, Debug, Serialize, Deserialize, EnumString, Display, Sequence)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Servers {
    Table,
    Id,
    Ip,
    Username,
    Password,
    Hostname,
    Provider,
    KeyPath,
}


