import { invoke } from '@tauri-apps/api/core';

export interface DatabaseInfo {
  path: string;
  name: string;
  table_count: number;
  is_connected: boolean;
  alias?: string;  // Add this line
}

export interface TableInfo {
  name: string;
  row_count: number;
  columns: ColumnInfo[];
}

export interface ColumnInfo {
  name: string;
  data_type: string;
  is_nullable: boolean;
  default_value?: string;
  is_primary_key: boolean;
}

export interface TableData {
  columns: string[];
  rows: any[][];
  total_count: number;
}

export interface SchemaComparison {
  database1: string;
  database2: string;
  added_tables: string[];
  removed_tables: string[];
  modified_tables: TableDiff[];
  identical_tables: string[];
}

export interface TableDiff {
  table_name: string;
  added_columns: ColumnInfo[];
  removed_columns: string[];
  modified_columns: ColumnDiff[];
}

export interface ColumnDiff {
  column_name: string;
  old_type: string;
  new_type: string;
  changes: string[];
}

export interface DataComparisonResult {
  table_name: string;
  total_rows_db1: number;
  total_rows_db2: number;
  rows_inserted: number;
  rows_deleted: number;
  rows_potentially_modified: number;
  identical: boolean;
}


export class DatabaseService {
  async testConnection(): Promise<string> {
    return await invoke('test_connection');
  }

  async connectDatabase(path: string, password: string): Promise<DatabaseInfo> {
    console.log('Connecting to database:', path);
    return await invoke('connect_database', { path, password });
  }

  async getDatabaseTables(dbPath: string): Promise<TableInfo[]> {
    console.log('Getting tables for database:', dbPath);
    const result = await invoke('get_database_tables', { dbPath });
    console.log('Tables received from backend:', result);
    return result as TableInfo[];
  }

  // Add this to your DatabaseService class
async getTableColumns(databasePath: string, tableName: string): Promise<any[]> {
  try {
    return await invoke('get_table_columns', {
      dbPath: databasePath,
      tableName: tableName
    });
  } catch (err) {
    console.error(`Failed to get columns for table ${tableName}:`, err);
    return [];
  }
}

  // async getTableData(dbPath: string, tableName: string, limit?: number): Promise<TableData> {
  //   console.log('Getting table data:', tableName, 'from', dbPath);
  //   return await invoke('get_table_data', { 
  //     dbPath, 
  //     tableName, 
  //     limit: limit || 100 
  //   });
  // }

  // REPLACE the existing getTableData method in databaseService.ts (around line 70)
// with this version that supports OFFSET
// REPLACE the existing getTableData method in databaseService.ts (around line 70)
// with this version that supports OFFSET

async getTableData(
  dbPath: string, 
  tableName: string, 
  limit?: number, 
  offset?: number  // ← NEW PARAMETER
): Promise<TableData> {
  console.log('Getting table data:', tableName, 'from', dbPath, 'limit:', limit, 'offset:', offset);
  return await invoke('get_table_data', { 
    dbPath, 
    tableName, 
    limit: limit || 100,
    offset: offset || 0  // ← Pass offset
  });
}
// async getTableData(
//   dbPath: string, 
//   tableName: string, 
//   limit?: number, 
//   offset?: number  // ← NEW PARAMETER
// ): Promise<TableData> {
//   console.log('Getting table data:', tableName, 'from', dbPath, 'limit:', limit, 'offset:', offset);
//   return await invoke('get_table_data', { 
//     dbPath, 
//     tableName, 
//     limit: limit || 100,
//     offset: offset || 0  // ← Pass offset
//   });
// }


  async compareDatabaseSchemas(db1Path: string, db2Path: string): Promise<SchemaComparison> {
    console.log('Comparing schemas:', db1Path, 'vs', db2Path);
    return await invoke('compare_database_schemas', { db1Path, db2Path });
  }

  async compareTableDataFast(
  db1Path: string, 
  db2Path: string, 
  tableName: string, 
  primaryKey: string
): Promise<DataComparisonResult> {
  console.log('Fast comparing table:', tableName);
  return await invoke('compare_table_data_fast', { 
    db1Path, 
    db2Path, 
    tableName, 
    primaryKey 
  });
}
}



