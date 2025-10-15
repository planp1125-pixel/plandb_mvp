use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub path: String,
    pub name: String,
    pub table_count: i32,
    pub is_connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub password: Option<String>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub row_count: i64,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

// Existing array-based TableData (for browse/schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_count: i64,
}

// New object-based TableData (for data comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDataObject {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Value>,  // JSON objects { "col": value }
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaComparison {
    pub database1: String,
    pub database2: String,
    pub added_tables: Vec<String>,
    pub removed_tables: Vec<String>,
    pub modified_tables: Vec<TableDiff>,
    pub identical_tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDiff {
    pub table_name: String,
    pub added_columns: Vec<ColumnInfo>,
    pub removed_columns: Vec<String>,
    pub modified_columns: Vec<ColumnDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDiff {
    pub column_name: String,
    pub old_type: String,
    pub new_type: String,
    pub changes: Vec<String>,
}