<template>
  <div class="table-viewer">
    <div class="viewer-header">
      <h3>Table Data Viewer</h3>
      <p>Browse and inspect table contents from your SQLCipher databases</p>
    </div>

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
          <select v-model="selectedTable" @change="loadTableData">
            <option value="">Select table...</option>
            <option v-for="table in availableTables" :key="table.name" :value="table.name">
              {{ table.name }} ({{ table.row_count.toLocaleString() }} rows)
            </option>
          </select>
        </div>

        <div class="limit-selector" v-if="selectedTable">
          <label>Rows to show:</label>
          <select v-model="rowLimit" @change="loadTableData">
            <option :value="50">50 rows</option>
            <option :value="100">100 rows</option>
            <option :value="500">500 rows</option>
            <option :value="1000">1000 rows</option>
            <option :value="null">All rows</option>
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
            rows displayed
          </span>
          <span class="stat" v-if="tableData.total_count > tableData.rows.length">
            of <strong>{{ tableData.total_count.toLocaleString() }}</strong> total
          </span>
          <span class="stat">
            <strong>{{ tableData.columns.length }}</strong> columns
          </span>
        </div>

        <div class="search-filter" v-if="tableData.rows.length > 0">
          <input 
            v-model="searchTerm" 
            type="text" 
            placeholder="Filter data..."
            class="search-input"
          />
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
            <tr v-for="(row, index) in paginatedRows" :key="index">
              <td class="row-number">{{ (currentPage - 1) * pageSize + index + 1 }}</td>
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
        No rows match your filter: "{{ searchTerm }}"
        <button @click="searchTerm = ''" class="clear-filter-btn">Clear Filter</button>
      </div>

      <!-- Pagination -->
      <div v-if="totalPages > 1" class="pagination">
        <div class="pagination-info">
          Showing {{ (currentPage - 1) * pageSize + 1 }} to 
          {{ Math.min(currentPage * pageSize, filteredRows.length) }} of 
          {{ filteredRows.length }} filtered rows
        </div>
        
        <div class="pagination-controls">
          <button 
            @click="currentPage = 1" 
            :disabled="currentPage === 1"
            class="page-btn"
          >
            First
          </button>
          <button 
            @click="currentPage--" 
            :disabled="currentPage === 1"
            class="page-btn"
          >
            Previous
          </button>
          
          <span class="page-info">
            Page {{ currentPage }} of {{ totalPages }}
          </span>
          
          <button 
            @click="currentPage++" 
            :disabled="currentPage === totalPages"
            class="page-btn"
          >
            Next
          </button>
          <button 
            @click="currentPage = totalPages" 
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
}>();

// State
const databaseService = new DatabaseService();
const selectedDbPath = ref('');
const availableTables = ref<TableInfo[]>([]);
const selectedTable = ref('');
const tableData = ref<TableData | null>(null);
const rowLimit = ref<number | null>(100);
const isLoading = ref(false);
const error = ref('');

// Pagination and filtering
const searchTerm = ref('');
const currentPage = ref(1);
const pageSize = ref(25);

// Computed
const filteredRows = computed(() => {
  if (!tableData.value || !searchTerm.value) {
    return tableData.value?.rows || [];
  }

  return tableData.value.rows.filter(row => 
    row.some(cell => 
      String(cell || '').toLowerCase().includes(searchTerm.value.toLowerCase())
    )
  );
});

const totalPages = computed(() => {
  return Math.ceil(filteredRows.value.length / pageSize.value);
});

const paginatedRows = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return filteredRows.value.slice(start, end);
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

const loadTableData = async () => {
  if (!selectedDbPath.value || !selectedTable.value) {
    tableData.value = null;
    return;
  }

  isLoading.value = true;
  error.value = '';
  currentPage.value = 1;

  try {
    const data = await databaseService.getTableData(
      selectedDbPath.value,
      selectedTable.value,
      rowLimit.value || undefined
    );
    tableData.value = data;
  } catch (err) {
    error.value = `Failed to load table data: ${err}`;
    tableData.value = null;
  } finally {
    isLoading.value = false;
  }
};

const refreshData = () => {
  loadTableData();
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
  if (newDb) {
    selectedDbPath.value = newDb.path;
    loadDatabaseTables();
  }
}, { immediate: true });

watch(searchTerm, () => {
  currentPage.value = 1;
});
</script>

<style scoped>

:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f8f9fa;
  --bg-tertiary: #f1f3f4;
  --text-primary: #343a40;
  --text-secondary: #6c757d;
  --text-label: #495057;
  --border-color: #dee2e6;
  --border-secondary: #ced4da;
}

/* Dark mode CSS variables */
:global(html.dark) .table-viewer {
  --bg-primary: #1a1d21;
  --bg-secondary: #24292e;
  --bg-tertiary: #2d333b;
  --text-primary: #f0f6fc;
  --text-secondary: #c9d1d9;
  --text-label: #e6edf3;
  --border-color: #30363d;
  --border-secondary: #3c444d;
}

.table-viewer {
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
}

.search-input {
  padding: 6px 12px;
  border: 1px solid var(--border-secondary);
  /* border: 1px solid #ced4da; */
  border-radius: 4px;
  width: 200px;
  font-size: 0.9em;
}

.table-container {
  flex: 1;
  overflow: auto;
  max-height: 500px;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9em;
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