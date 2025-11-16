<template>
  <div class="table-viewer">
    <div class="table-selection">
      <div class="selection-controls">
        <div class="database-selector">
          <label>Database:</label>
          <select v-model="selectedDbPath" @change="loadDatabaseTables">
            <option value="">Select database...</option>
            <option v-for="db in databases" :key="db.path" :value="db.path">
              {{ db.name }}
            </option>
          </select>
        </div>

        <div class="table-selector" v-if="availableTables.length > 0">
          <label>Table:</label>
          <select v-model="selectedTable" @change="handleTableChange">
            <option value="">Select table...</option>
            <option v-for="table in availableTables" :key="table.name" :value="table.name">
              {{ table.name }} ({{ table.row_count.toLocaleString() }} rows)
            </option>
          </select>
        </div>

        <div class="limit-selector" v-if="selectedTable">
          <label>Rows per page:</label>
          <select v-model="rowLimit" @change="handleLimitChange">
            <option :value="100">100 rows</option>
            <option :value="500">500 rows</option>
            <option :value="1000">1,000 rows</option>
            <option :value="2000">2,000 rows</option>
            <option :value="5000">5,000 rows</option>
          </select>
        </div>
      </div>

      <div class="viewer-actions" v-if="selectedTable">
        <button @click="refreshData" :disabled="isLoading" class="refresh-btn">
          {{ isLoading ? 'Loading...' : 'Refresh' }}
        </button>
        <button @click="exportData" :disabled="!tableData" class="export-btn">
          Export CSV
        </button>
      </div>
    </div>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>

    <div v-if="isLoading" class="loading">
      <div class="loading-spinner"></div>
      <p>Loading table data...</p>
    </div>

    <!-- Table Data Display -->
    <div v-if="tableData && !isLoading" class="table-data">
      <div class="data-info">
        <div class="info-stats">
          <span class="stat">
            <strong>{{ tableData.rows.length.toLocaleString() }}</strong>
            rows on this page
          </span>
          <span class="stat">
            of <strong>{{ tableData.total_count.toLocaleString() }}</strong> total
          </span>
          <span class="stat">
            <strong>{{ tableData.columns.length }}</strong> columns
          </span>
        </div>

        <div class="search-filter">
          <input
            v-model="searchTerm"
            type="text"
            placeholder="Search in current page..."
            class="search-input"
          />
          <button
            v-if="searchTerm"
            @click="searchTerm = ''"
            class="clear-search-btn"
            title="Clear search"
          >
            ✕
          </button>
        </div>
      </div>

      <div class="table-container" v-if="filteredRows.length > 0">
        <table class="data-table">
          <thead>
            <tr>
              <th class="row-number">#</th>
              <th v-for="column in tableData.columns" :key="column" class="data-header">
                {{ column }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(row, index) in filteredRows" :key="index">
              <td class="row-number">{{ index + 1 }}</td>
              <td v-for="(cell, cellIndex) in row" :key="cellIndex" class="data-cell">
                <div class="cell-content" :title="formatCellValue(cell)">
                  {{ formatCellValue(cell) }}
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="filteredRows.length === 0 && searchTerm" class="no-results">
        <p>No rows match "{{ searchTerm }}"</p>
        <button @click="searchTerm = ''" class="clear-filter-btn">Clear Search</button>
      </div>

      <!-- Pagination -->
      <div v-if="totalPages > 1" class="pagination">
        <div class="pagination-info">
          <span>
            Showing {{ (currentPage - 1) * pageSize + 1 }} to
            {{ Math.min((currentPage - 1) * pageSize + tableData.rows.length, totalCount) }} of
            {{ totalCount.toLocaleString() }} rows
          </span>
          <span v-if="isPaging" class="pagination-loading">
            <span class="mini-spinner"></span> Loading page...
          </span>
        </div>
        
        <div class="pagination-controls">
          <button
            @click="loadTableData(1, false)"
            :disabled="currentPage === 1"
            class="page-btn"
          >
            First
          </button>
          <button 
            @click="loadTableData(currentPage - 1, false)" 
            :disabled="currentPage === 1"
            class="page-btn"
          >
            Previous
          </button>
          
          <span class="page-info">
            Page {{ currentPage }} of {{ totalPages }}
          </span>
          
          <button 
            @click="loadTableData(currentPage + 1, false)" 
            :disabled="currentPage === totalPages"
            class="page-btn"
          >
            Next
          </button>
          <button 
            @click="loadTableData(totalPages, false)" 
            :disabled="currentPage === totalPages"
            class="page-btn"
          >
            Last
          </button>
        </div>
      </div>
    </div>

    <div v-else-if="selectedDbPath && !isLoading" class="no-selection">
      <p>Select a table to view its data</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { DatabaseService, type DatabaseInfo, type TableInfo, type TableData } from '../services/databaseService';

const props = defineProps<{
  databases: DatabaseInfo[];
  selectedDatabase?: DatabaseInfo | null;
  initialTable?: string;
  initialDbPath?: string;
}>();

const emit = defineEmits<{
  'table-opened': [tableName: string, dbPath: string];
}>();

// State
const databaseService = new DatabaseService();
const selectedDbPath = ref('');
const availableTables = ref<TableInfo[]>([]);
const selectedTable = ref('');
const tableData = ref<TableData | null>(null);
const rowLimit = ref<number | null>(1000); // default to 1000 rows
const isLoading = ref(false);
const isPaging = ref(false);
const error = ref('');

// Pagination state
const currentPage = ref(1);
const searchTerm = ref('');

// Computed
const pageSize = computed(() => (rowLimit.value === null ? 1000 : rowLimit.value || 1000));
const totalCount = computed(() => tableData.value?.total_count || 0);
const totalPages = computed(() => Math.max(1, Math.ceil(totalCount.value / pageSize.value)));

// Filter rows based on search term (client-side filtering on current page only)
const filteredRows = computed(() => {
  if (!tableData.value || !searchTerm.value.trim()) {
    return tableData.value?.rows || [];
  }

  const search = searchTerm.value.toLowerCase();
  return tableData.value.rows.filter(row =>
    row.some(cell =>
      String(cell || '').toLowerCase().includes(search)
    )
  );
});

// Methods
const loadDatabaseTables = async () => {
  if (!selectedDbPath.value) {
    availableTables.value = [];
    selectedTable.value = '';
    tableData.value = null;
    return;
  }

  isLoading.value = true;
  error.value = '';

  try {
    const tables = await databaseService.getDatabaseTables(selectedDbPath.value);
    availableTables.value = tables;
    selectedTable.value = '';
    tableData.value = null;
  } catch (err) {
    error.value = `Failed to load tables: ${err}`;
    availableTables.value = [];
  } finally {
    isLoading.value = false;
  }
};

const loadTableData = async (targetPage: number = 1, shouldEmit: boolean = true) => {
  if (!selectedDbPath.value || !selectedTable.value) {
    tableData.value = null;
    return;
  }

  // Show full overlay only on first load; use lightweight paging flag otherwise
  if (!tableData.value || targetPage === 1) {
    isLoading.value = true;
  } else {
    isPaging.value = true;
  }
  error.value = '';
  currentPage.value = targetPage;

  try {
    const offset = (currentPage.value - 1) * pageSize.value;
    const data = await databaseService.getTableData(
      selectedDbPath.value,
      selectedTable.value,
      pageSize.value,
      offset
    );
    tableData.value = data;

    // Emit event when table is loaded manually in the main Browse Data tab
    // Don't emit if this is a dynamic tab (has initialTable prop) or if explicitly disabled
    if (!props.initialTable && shouldEmit) {
      emit('table-opened', selectedTable.value, selectedDbPath.value);
    }
  } catch (err) {
    error.value = `Failed to load table data: ${err}`;
    tableData.value = null;
  } finally {
    isLoading.value = false;
    isPaging.value = false;
  }
};

const handleTableChange = () => {
  // When user manually selects a table, load it and emit event
  loadTableData(1, true);
};

const handleLimitChange = () => {
  // When changing row limit, reload data but don't create new tab
  loadTableData(1, false);
};

const refreshData = () => {
  loadTableData(currentPage.value, false);
};

const exportData = () => {
  if (!tableData.value) return;

  const csvContent = generateCSV(tableData.value);
  downloadCSV(csvContent, `${selectedTable.value}.csv`);
};

const formatCellValue = (value: any): string => {
  if (value === null || value === undefined) {
    return 'NULL';
  }
  
  if (typeof value === 'string' && value.startsWith('<BLOB')) {
    return value;
  }

  return String(value);
};

const generateCSV = (data: TableData): string => {
  const headers = data.columns.join(',');
  const rows = data.rows.map(row => 
    row.map(cell => {
      const value = formatCellValue(cell);
      // Escape quotes and wrap in quotes if contains comma or quote
      if (value.includes(',') || value.includes('"') || value.includes('\n')) {
        return `"${value.replace(/"/g, '""')}"`;
      }
      return value;
    }).join(',')
  );
  
  return [headers, ...rows].join('\n');
};

const downloadCSV = (content: string, filename: string) => {
  const blob = new Blob([content], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

// Watchers
watch(() => props.selectedDatabase, (newDb) => {
  if (newDb && !props.initialDbPath) {
    selectedDbPath.value = newDb.path;
    loadDatabaseTables();
  }
}, { immediate: true });

// Watch for initialDbPath and initialTable props changes
watch(() => [props.initialDbPath, props.initialTable] as const, ([dbPath, tableName]) => {
  if (dbPath && tableName) {
    selectedDbPath.value = dbPath;
    loadDatabaseTables().then(() => {
      selectedTable.value = tableName;
      loadTableData(1);
    });
  }
}, { immediate: true });
</script>

<style scoped>

/* Component uses global CSS variables from theme.css */
.table-viewer {
  --text-label: var(--text-primary);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.viewer-header {
  margin-bottom: 25px;
  text-align: center;
}

.viewer-header h3 {
  margin: 0 0 8px 0;
  color: var(--text-primary);
  /* color: #343a40; */
}

.viewer-header p {
  color: var(--text-secondary);
  /* color: #6c757d; */
  margin: 0;
}

.table-selection {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 20px;
  margin-bottom: 25px;
  padding: 20px;
  background: var(--bg-secondary);
  /* background: #f8f9fa; */
  border-radius: 8px;
  flex-wrap: wrap;
}

.selection-controls {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
  align-items: flex-end;
}

.database-selector,
.table-selector,
.limit-selector {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 180px;
}

.database-selector label,
.table-selector label,
.limit-selector label {
  font-weight: 600;
  color: var(--text-label);
  /* color: #495057; */
  font-size: 0.9em;
}

.database-selector select,
.table-selector select,
.limit-selector select {
  padding: 8px 12px;
  border: 1px solid var(--border-secondary);
  /* border: 1px solid #ced4da; */
  border-radius: 4px;
  background: var(--bg-primary);
  /* background: white; */
  font-size: 0.95em;
}

.viewer-actions {
  display: flex;
  gap: 10px;
}

.refresh-btn,
.export-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9em;
  font-weight: 500;
}

.refresh-btn {
  background: #6c757d;
  color: white;
}

.refresh-btn:hover:not(:disabled) {
  background: #545b62;
}

.refresh-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.export-btn {
  background: #17a2b8;
  color: white;
}

.export-btn:hover:not(:disabled) {
  background: #138496;
}

.error-message {
  padding: 15px;
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
  border-radius: 4px;
  margin-bottom: 20px;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px;
  color: var(--text-secondary);
  /* color: #6c757d; */
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 15px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.table-data {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  /* background: white; */
  border: 1px solid var(--border-color);
  /* border: 1px solid #dee2e6; */
  border-radius: 8px;
  overflow: hidden;
}

.data-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: var(--bg-secondary);
  /* background: #f8f9fa; */
  border-bottom: 1px solid #dee2e6;
  flex-wrap: wrap;
  gap: 15px;
}

.info-stats {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.stat {
  font-size: 0.9em;
  color: var(--text-label);
  /* color: #495057; */
}

.search-filter {
  display: flex;
  align-items: center;
  position: relative;
}

.search-input {
  padding: 6px 30px 6px 12px;
  border: 1px solid var(--border-secondary);
  /* border: 1px solid #ced4da; */
  border-radius: 4px;
  width: 250px;
  font-size: 0.9em;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.search-input:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.clear-search-btn {
  position: absolute;
  right: 8px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 1.2em;
  padding: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s;
}

.clear-search-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.table-container {
  flex: 1;
  overflow: auto;
  max-height: 600px;
  /* Enable hardware acceleration for smoother scrolling */
  transform: translateZ(0);
  -webkit-overflow-scrolling: touch;
  will-change: scroll-position;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9em;
  /* Optimize rendering */
  contain: layout style;
}

.data-table th,
.data-table td {
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid #dee2e6;
  border-right: 1px solid #f1f3f4;
}

.data-table th {
  background: var(--bg-secondary);
  /* background: #f8f9fa; */
  font-weight: 600;
  color: var(--text-label);
  /* color: #495057; */
  position: sticky;
  top: 0;
  z-index: 10;
}

.row-number {
  width: 60px;
  background: var(--bg-tertiary);
  /* background: #f1f3f4 !important; */
  font-weight: 500;
  text-align: center;
  font-family: monospace;
}

.data-header {
  font-family: monospace;
  font-weight: 600;
  white-space: nowrap;
  min-width: 100px;
}

.data-cell {
  max-width: 200px;
  position: relative;
}

.cell-content {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: monospace;
  line-height: 1.4;
}

.data-cell:hover .cell-content {
  background: var(--bg-secondary);
  /* background: #f8f9fa; */
}

.no-results {
  padding: 40px 20px;
  text-align: center;
  color: var(--text-secondary);
  /* color: #6c757d; */
}

.clear-filter-btn {
  margin-left: 10px;
  padding: 4px 12px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
}

.clear-filter-btn:hover {
  background: #0056b3;
}

.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: var(--bg-secondary);
  /* background: #f8f9fa; */
  border-top: 1px solid #dee2e6;
  flex-wrap: wrap;
  gap: 15px;
}

.pagination-info {
  font-size: 0.9em;
  color: var(--text-label);
  /* color: #495057; */
  display: flex;
  align-items: center;
  gap: 12px;
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.page-btn {
  padding: 6px 12px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
}

.page-btn:hover:not(:disabled) {
  background: #0056b3;
}

.page-btn:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.pagination-loading {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-size: 0.9em;
}

.mini-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--border-secondary);
  border-top: 2px solid var(--primary-500, #3b82f6);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.page-info {
  font-weight: 500;
  color: var(--text-label);
  /* color: #495057; */
  margin: 0 10px;
  font-size: 0.9em;
}

.no-selection {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  /* background: white; */
  border: 1px solid var(--border-color);
  /* border: 1px solid #dee2e6; */
  border-radius: 8px;
  color: var(--text-secondary);
  /* color: #6c757d; */
  min-height: 300px;
}

/* Responsive Design */
@media (max-width: 768px) {
  .table-selection {
    flex-direction: column;
    align-items: stretch;
  }

  .selection-controls {
    flex-direction: column;
    align-items: stretch;
  }

  .database-selector,
  .table-selector,
  .limit-selector {
    min-width: auto;
  }

  .viewer-actions {
    justify-content: center;
  }

  .data-info {
    flex-direction: column;
    align-items: stretch;
    text-align: center;
  }

  .info-stats {
    justify-content: center;
  }

  .search-input {
    width: 100%;
  }

  .table-container {
    max-height: 400px;
  }

  .data-table {
    font-size: 0.8em;
  }

  .data-table th,
  .data-table td {
    padding: 6px 8px;
  }

  .cell-content {
    max-width: 120px;
  }

  .pagination {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .pagination-controls {
    flex-wrap: wrap;
    justify-content: center;
  }
}

/* Very small screens */
@media (max-width: 480px) {
  .data-table th,
  .data-table td {
    padding: 4px 6px;
    font-size: 0.75em;
  }

  .cell-content {
    max-width: 80px;
  }

  .page-btn {
    padding: 4px 8px;
    font-size: 0.8em;
  }
}
</style>
