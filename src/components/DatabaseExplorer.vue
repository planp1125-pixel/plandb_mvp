<template>

  <div class="database-explorer">
    <div class="explorer-header">
      <h3>Database Structure</h3>
      <div v-if="selectedDatabase" class="database-info">
        <div class="current-db">
          <strong>{{ currentDatabase?.name || 'No database selected' }}</strong>
          <span v-if="currentDatabase" class="db-path">{{ currentDatabase.path }}</span>
        </div>
        <div class="db-selector">
          <select v-model="currentDbPath" @change="loadTables">
            <option value="">Select database...</option>
            <option v-for="db in databases" :key="db.path" :value="db.path">
              {{ db.name }} ({{ db.table_count }} tables)
            </option>
          </select>
        </div>
      </div>
    </div>

    <div class="toolbar">
      <div class="table-count" v-if="tables.length > 0">
        {{ tables.length }} tables found
      </div>
      <div class="actions">
        <button @click="refreshTables" :disabled="loading" class="action-btn">
          <span class="icon">🔄</span>
          Refresh
        </button>
        <button @click="expandAll" class="action-btn">
          <span class="icon">📂</span>
          Expand All
        </button>
        <button @click="collapseAll" class="action-btn">
          <span class="icon">📁</span>
          Collapse All
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading">
      <div class="loading-spinner"></div>
      <p>Loading database structure...</p>
    </div>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>

    <!-- Tree Structure -->
    <div v-if="tables.length > 0" class="database-tree">
      <div 
        v-for="table in tables" 
        :key="table.name" 
        class="table-node"
        :class="{ expanded: expandedTables.has(table.name) }"
      >
        <!-- Table Header Row -->
        <div class="table-row" @click="toggleTable(table.name)">
          <div class="expand-icon">
            <span v-if="expandedTables.has(table.name)">▼</span>
            <span v-else>▶</span>
          </div>
          <div class="table-icon">
            <span class="icon">🗃️</span>
          </div>
          <div class="table-info">
            <span class="table-name">{{ table.name }}</span>
            <span class="table-meta">{{ table.row_count.toLocaleString() }} rows, {{ table.columns.length }} columns</span>
          </div>
          <div class="table-actions">
            <button 
              @click.stop="viewTableData(table)" 
              class="action-icon"
              title="View table data"
            >
              👁️
            </button>
            <button 
              @click.stop="exportTableData(table)" 
              class="action-icon"
              title="Export table data"
            >
              💾
            </button>
          </div>
        </div>

        <!-- Expanded Table Schema -->
        <div v-if="expandedTables.has(table.name)" class="table-schema">
          <div class="schema-header">
            <h5>Columns ({{ table.columns.length }})</h5>
            <div class="schema-stats">
              <span class="stat primary-keys" v-if="primaryKeyColumns(table).length > 0">
                Primary Keys: {{ primaryKeyColumns(table).join(', ') }}
              </span>
            </div>
          </div>

          <div class="columns-list">
            <div 
              v-for="column in table.columns" 
              :key="column.name"
              class="column-row"
              :class="{ 'primary-key': column.is_primary_key }"
            >
              <div class="column-icon">
                <span v-if="column.is_primary_key" class="icon key-icon" title="Primary Key">🔑</span>
                <span v-else class="icon">📄</span>
              </div>
              <div class="column-info">
                <span class="column-name">{{ column.name }}</span>
                <span class="column-details">
                  {{ column.data_type }}
                  <span v-if="!column.is_nullable" class="not-null">NOT NULL</span>
                  <span v-if="column.default_value" class="default-value">DEFAULT {{ column.default_value }}</span>
                </span>
              </div>
              <div class="column-badges">
                <span v-if="column.is_primary_key" class="badge primary">PK</span>
                <span v-if="!column.is_nullable" class="badge not-null">NOT NULL</span>
                <span v-if="column.default_value" class="badge default">DEFAULT</span>
              </div>
            </div>
          </div>

          <div class="schema-footer">
            <button @click="copyCreateStatement(table)" class="copy-sql-btn">
              📋 Copy CREATE TABLE
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-else-if="currentDbPath && !loading" class="no-tables">
      <div class="empty-state">
        <span class="icon">📭</span>
        <h4>No Tables Found</h4>
        <p>This database appears to be empty or all tables are system tables.</p>
      </div>
    </div>

    <div v-else-if="!currentDbPath && !loading" class="no-database">
      <div class="empty-state">
        <span class="icon">🗄️</span>
        <h4>No Database Selected</h4>
        <p>Select a database from the dropdown above to view its structure.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue';
import { DatabaseService, type DatabaseInfo, type TableInfo } from '../services/databaseService';

const props = defineProps<{
  databases: DatabaseInfo[];
  selectedDatabase?: DatabaseInfo | null;
}>();

const emit = defineEmits<{
  'table-selected': [tableName: string, dbPath: string];
  'view-data': [table: TableInfo];
}>();

// State
const databaseService = new DatabaseService();
const currentDbPath = ref('');
const tables = ref<TableInfo[]>([]);
const loading = ref(false);
const error = ref('');
const expandedTables = reactive(new Set<string>());

// Computed
const currentDatabase = computed(() => {
  return props.databases.find(db => db.path === currentDbPath.value);
});

const primaryKeyColumns = (table: TableInfo) => {
  return table.columns.filter(col => col.is_primary_key).map(col => col.name);
};

// Methods
const loadTables = async () => {
  if (!currentDbPath.value) {
    tables.value = [];
    expandedTables.clear();
    return;
  }

  loading.value = true;
  error.value = '';
  expandedTables.clear();

  try {
    console.log('Loading tables for database:', currentDbPath.value);
    const tableList = await databaseService.getDatabaseTables(currentDbPath.value);
    console.log('Loaded tables:', tableList);
    tables.value = tableList;
  } catch (err) {
    error.value = `Failed to load tables: ${err}`;
    tables.value = [];
    console.error('Error loading tables:', err);
  } finally {
    loading.value = false;
  }
};

const refreshTables = () => {
  loadTables();
};

const toggleTable = (tableName: string) => {
  if (expandedTables.has(tableName)) {
    expandedTables.delete(tableName);
  } else {
    expandedTables.add(tableName);
  }
};

const expandAll = () => {
  tables.value.forEach(table => {
    expandedTables.add(table.name);
  });
};

const collapseAll = () => {
  expandedTables.clear();
};

const viewTableData = (table: TableInfo) => {
  console.log('View data for table:', table.name);
  emit('table-selected', table.name, currentDbPath.value);
  emit('view-data', table);
};

const exportTableData = async (table: TableInfo) => {
  console.log('Export data for table:', table.name);
  // You can implement export logic here or emit an event
  alert(`Export functionality for ${table.name} - coming soon!`);
};

const copyCreateStatement = (table: TableInfo) => {
  const createStatement = generateCreateStatement(table);
  navigator.clipboard.writeText(createStatement).then(() => {
    console.log('CREATE statement copied to clipboard');
    // You could show a toast notification here
  });
};

const generateCreateStatement = (table: TableInfo): string => {
  const columns = table.columns.map(col => {
    let colDef = `  "${col.name}" ${col.data_type}`;
    if (!col.is_nullable) colDef += ' NOT NULL';
    if (col.default_value) colDef += ` DEFAULT ${col.default_value}`;
    return colDef;
  });

  const primaryKeys = table.columns.filter(col => col.is_primary_key).map(col => col.name);
  if (primaryKeys.length > 0) {
    columns.push(`  PRIMARY KEY ("${primaryKeys.join('", "')}")`);
  }

  return `CREATE TABLE "${table.name}" (\n${columns.join(',\n')}\n);`;
};

// Watchers
watch(() => props.selectedDatabase, (newDb) => {
  if (newDb) {
    currentDbPath.value = newDb.path;
    loadTables();
  }
}, { immediate: true });

watch(currentDbPath, () => {
  loadTables();
});
</script>

<style scoped>

/* Component uses global CSS variables from theme.css */
.database-explorer {
  --border-light: var(--border-secondary);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.explorer-header {
  padding: 20px;
  border-bottom: 1px solid #dee2e6;
   background: var(--bg-secondary);
  /* background: #f8f9fa; */
}

.explorer-header h3 {
  margin: 0 0 15px 0;
  color: var(--text-primary);
  /* color: #343a40; */
  display: flex;
  align-items: center;
  gap: 10px;
}

.database-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}

.current-db {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.db-path {
  font-family: monospace;
  font-size: 0.8em;
  color: var(--text-secondary);
  /* color: #6c757d; */
  background: #e9ecef;
  padding: 2px 6px;
  border-radius: 3px;
}

.db-selector select {
  padding: 8px 12px;
  border: 1px solid #ced4da;
  border-radius: 6px;
  background: var(--bg-primary);
  /* background: white; */
  min-width: 250px;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: var(--bg-primary);
  /* background: white; */
  border-bottom: 1px solid #dee2e6;
}

.table-count {
  font-weight: 500;
  color: #495057;
}

.actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--bg-secondary);
  /* background: #f8f9fa; */
  border: 1px solid var(--border-color);
  /* border: 1px solid #dee2e6; */
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
  transition: all 0.2s;
}

.action-btn:hover:not(:disabled) {
  background: #e9ecef;
  border-color: #adb5bd;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 60px 20px;
  color: var(--text-secondary);
  /* color: #6c757d; */
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 15px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error-message {
  padding: 15px 20px;
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
  margin: 15px 20px;
  border-radius: 6px;
}

.database-tree {
  flex: 1;
  overflow-y: auto;
  padding: 10px 0;
}

.table-node {
  border-bottom: 1px solid #f1f3f4;
}

.table-node.expanded {
  background: #f8f9ff;
}

.table-row {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  cursor: pointer;
  transition: background-color 0.2s;
  user-select: none;
  position: sticky;        /* Add this */
  top: 0;                  /* Add this */
  z-index: 10;            /* Add this */
  background: var(--bg-primary);
  /* background: white;       */
}

.table-row:hover {
  background: #f5f5f5;
}

.table-node.expanded .table-row {
  background: #e3f2fd;
  border-bottom: 1px solid #bbdefb;
  position: sticky;        /* Add this */
  top: 0;                  /* Add this */
  z-index: 10;            /* Add this */
}
.expand-icon {
  width: 20px;
  display: flex;
  justify-content: center;
  color: var(--text-secondary);
  /* color: #6c757d; */
  font-size: 0.9em;
}

.table-icon {
  margin: 0 10px;
  font-size: 1.1em;
}

.table-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.table-name {
  font-weight: 600;
  font-family: monospace;
  color: var(--text-primary);
  /* color: #343a40; */
  font-size: 1em;
}

.table-meta {
  font-size: 0.85em;
  color: var(--text-secondary);
  /* color: #6c757d; */
}

.table-actions {
  display: flex;
  gap: 8px;
  opacity: 0;
  transition: opacity 0.2s;
}

.table-row:hover .table-actions {
  opacity: 1;
}

.action-icon {
  background: none;
  border: none;
  cursor: pointer;
  padding: 6px;
  border-radius: 4px;
  transition: background-color 0.2s;
  font-size: 1em;
}

.action-icon:hover {
  background: rgba(0, 0, 0, 0.1);
}

.table-schema {
  padding: 0 20px 20px 60px;
  background: #f8f9ff;
  border-top: 1px solid #e3f2fd;
}

.schema-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 0 10px 0;
  border-bottom: 1px solid #e3f2fd;
  margin-bottom: 15px;
}

.schema-header h5 {
  margin: 0;
  color: #1976d2;
  font-weight: 600;
}

.schema-stats {
  font-size: 0.85em;
}

.primary-keys {
  color: #2e7d32;
  font-weight: 500;
}

.columns-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.column-row {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  background: var(--bg-primary);
  /* background: white; */
  border-radius: 4px;
  transition: all 0.2s;
}

.column-row:hover {
  background: #f5f5f5;
}

.column-row.primary-key {
  background: #fff8e1;
  border-left: 3px solid #ff9800;
}

.column-icon {
  width: 24px;
  text-align: center;
  margin-right: 10px;
}

.key-icon {
  color: #ff9800;
}

.column-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.column-name {
  font-weight: 600;
  font-family: monospace;
  color: var(--text-primary);
  /* color: #343a40; */
}

.column-details {
  font-size: 0.8em;
  color: var(--text-secondary);
  /* color: #6c757d; */
  font-family: monospace;
}

.column-badges {
  display: flex;
  gap: 4px;
}

.badge {
  padding: 2px 6px;
  border-radius: 10px;
  font-size: 0.65em;
  font-weight: 600;
  text-transform: uppercase;
}

.badge.primary {
  background: #fff3e0;
  color: #e65100;
}

.badge.not-null {
  background: #fce4ec;
  color: #c2185b;
}

.badge.default {
  background: #e8f5e8;
  color: #2e7d32;
}

.schema-footer {
  margin-top: 15px;
  padding-top: 15px;
  border-top: 1px solid #e3f2fd;
}

.copy-sql-btn {
  padding: 8px 16px;
  background: #2196f3;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
  transition: background-color 0.2s;
}

.copy-sql-btn:hover {
  background: #1976d2;
}

.no-tables,
.no-database {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-state {
  text-align: center;
  padding: 40px;
}

.empty-state .icon {
  font-size: 4em;
  margin-bottom: 20px;
  display: block;
}

.empty-state h4 {
  color: var(--text-secondary);
  /* color: #6c757d; */
  margin-bottom: 10px;
}

.empty-state p {
  color: var(--text-secondary);
  /* color: #6c757d; */
  margin: 0;
}

/* Responsive Design */
@media (max-width: 768px) {
  .database-info {
    flex-direction: column;
    align-items: stretch;
  }

  .toolbar {
    flex-direction: column;
    gap: 10px;
    align-items: stretch;
  }

  .actions {
    justify-content: center;
    flex-wrap: wrap;
  }

  .table-schema {
    padding: 0 10px 20px 40px;
  }

  .schema-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
}
</style>