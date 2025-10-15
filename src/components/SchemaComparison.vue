<template>
  <div class="schema-comparison">
    <div class="comparison-header">
      <h3>Database Schema Comparison</h3>
      <p>Compare table structures between two SQLCipher databases</p>
    </div>

      <!-- Error message -->
      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <!-- Success message -->
      <div v-if="successMessage" class="success-message">
        <pre>{{ successMessage }}</pre>
      </div>


    <div class="database-selection">
      <div class="db-selector">
        <label>Database 1 (Source):</label>
        <select v-model="database1" :disabled="databases.length === 0">
          <option value="">Select first database...</option>
          <option v-for="db in databases" :key="db.path + '_1'" :value="db.path">
            {{ db.name }}
          </option>
        </select>
      </div>

      <div class="vs-indicator">VS</div>

      <div class="db-selector">
        <label>Database 2 (Target):</label>
        <select v-model="database2" :disabled="databases.length === 0">
          <option value="">Select second database...</option>
          <option v-for="db in databases" :key="db.path + '_2'" :value="db.path">
            {{ db.name }}
          </option>
        </select>
      </div>
    </div>

    <div class="comparison-actions">
      <button 
        @click="compareSchemas"
        :disabled="!database1 || !database2 || database1 === database2 || isComparing"
        class="compare-btn"
      >
        {{ isComparing ? 'Comparing...' : 'Compare Schemas' }}
      </button>
      
      <button 
        v-if="comparisonResult"
        @click="exportComparison"
        class="export-btn"
      >
        Export Report
      </button>
      
      <button 
        v-if="comparisonResult"
        @click="generatePatch"
        class="patch-btn"
        >
        Generate SQL Patch
      </button>
    </div>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>

    <!-- Comparison Results - Side by Side -->
    <div v-if="comparisonResult" class="comparison-results">
      <div class="results-summary">
        <h4>Comparison Summary</h4>
        <div class="summary-stats">
          <div class="stat-item added">
            <span class="count">{{ comparisonResult.added_tables.length }}</span>
            <span class="label">Added Tables</span>
          </div>
          <div class="stat-item removed">
            <span class="count">{{ comparisonResult.removed_tables.length }}</span>
            <span class="label">Removed Tables</span>
          </div>
          <div class="stat-item modified">
            <span class="count">{{ comparisonResult.modified_tables.length }}</span>
            <span class="label">Modified Tables</span>
          </div>
          <div class="stat-item unchanged">
            <span class="count">{{ comparisonResult.identical_tables.length }}</span>
            <span class="label">Unchanged Tables</span>
          </div>
        </div>
      </div>

      <!-- Side by Side Comparison - Grouped by Status -->
      
      <!-- Added Tables Section -->
      <div v-if="addedTables.length > 0" class="status-section">
        <div class="status-section-header added">
          <span class="status-icon">➕</span>
          <h4>Added Tables ({{ addedTables.length }})</h4>
          <p>Tables that exist only in the target database</p>
        </div>
        <div class="side-by-side-container">
          <div class="database-column">
            <div class="column-header">
              <h4>{{ getDatabaseName(database1) }} (Source)</h4>
            </div>
            <div class="column-content empty-state">
              <div class="empty-message">
                <span>These tables don't exist in the source database</span>
              </div>
            </div>
          </div>
          <div class="database-column">
            <div class="column-header">
              <h4>{{ getDatabaseName(database2) }} (Target)</h4>
            </div>
            <div class="column-content">
              <div v-for="table in addedTables" :key="'added_' + table.name" class="table-card">
                <div 
                  class="table-header-card added"
                  @click="toggleTableDetails(table.name)"
                >
                  <div class="table-info">
                    <span class="table-name">{{ table.name }}</span>
                    <span class="table-status-badge added">Added</span>
                  </div>
                  <span class="toggle-icon">
                    {{ expandedTables.has(table.name) ? '▼' : '▶' }}
                  </span>
                </div>
                <div v-if="expandedTables.has(table.name)" class="table-details">
                  <div class="empty-columns-message">
                    Column details not available for added tables
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Removed Tables Section -->
      <div v-if="removedTables.length > 0" class="status-section">
        <div class="status-section-header removed">
          <span class="status-icon">➖</span>
          <h4>Removed Tables ({{ removedTables.length }})</h4>
          <p>Tables that exist only in the source database</p>
        </div>
        <div class="side-by-side-container">
          <div class="database-column">
            <div class="column-header">
              <h4>{{ getDatabaseName(database1) }} (Source)</h4>
            </div>
            <div class="column-content">
              <div v-for="table in removedTables" :key="'removed_' + table.name" class="table-card">
                <div 
                  class="table-header-card removed"
                  @click="toggleTableDetails(table.name)"
                >
                  <div class="table-info">
                    <span class="table-name">{{ table.name }}</span>
                    <span class="table-status-badge removed">Removed</span>
                  </div>
                  <span class="toggle-icon">
                    {{ expandedTables.has(table.name) ? '▼' : '▶' }}
                  </span>
                </div>
                <div v-if="expandedTables.has(table.name)" class="table-details">
                  <div class="empty-columns-message">
                    Column details not available for removed tables
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="database-column">
            <div class="column-header">
              <h4>{{ getDatabaseName(database2) }} (Target)</h4>
            </div>
            <div class="column-content empty-state">
              <div class="empty-message">
                <span>These tables don't exist in the target database</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modified Tables Section
      <div v-if="modifiedTables.source.length > 0" class="status-section">
        <div class="status-section-header modified">
          <span class="status-icon">🔄</span>
          <h4>Modified Tables ({{ modifiedTables.source.length }})</h4>
          <p>Tables that have structural differences between databases</p>
        </div>
        <div class="side-by-side-container">
          <div class="database-column">
            <div class="column-header">
              <h4>{{ getDatabaseName(database1) }} (Source)</h4>
            </div>
            <div class="column-content">
              <div v-for="table in modifiedTables.source" :key="'modified_source_' + table.name" class="table-card">
                <div 
                  class="table-header-card modified"
                  @click="toggleTableDetails(table.name)"
                >
                  <div class="table-info">
                    <span class="table-name">{{ table.name }}</span>
                    <span class="table-status-badge modified">Modified</span>
                  </div>
                  <span class="toggle-icon">
                    {{ expandedTables.has(table.name) ? '▼' : '▶' }}
                  </span>
                </div>
                <div v-if="expandedTables.has(table.name)" class="table-details">
                  <div v-if="table.columns.length > 0" class="columns-list">
                    <h6>Columns (Before):</h6>
                    <div 
                      v-for="column in table.columns" 
                      :key="'source_mod_col_' + column.name"
                      class="column-item"
                      :class="column.status"
                    >
                      <span class="column-name">{{ column.name }}</span>
                      <span class="column-type">{{ column.data_type }}</span>
                      <span v-if="column.changes" class="column-changes">
                        {{ column.changes.join(', ') }}
                      </span>
                    </div>
                  </div>
                  <div v-else class="empty-columns-message">
                    No column changes detected in source
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="database-column">
            <div class="column-header">
              <h4>{{ getDatabaseName(database2) }} (Target)</h4>
            </div>
            <div class="column-content">
              <div v-for="table in modifiedTables.target" :key="'modified_target_' + table.name" class="table-card">
                <div 
                  class="table-header-card modified"
                  @click="toggleTableDetails(table.name)"
                >
                  <div class="table-info">
                    <span class="table-name">{{ table.name }}</span>
                    <span class="table-status-badge modified">Modified</span>
                  </div>
                  <span class="toggle-icon">
                    {{ expandedTables.has(table.name) ? '▼' : '▶' }}
                  </span>
                </div>
                <div v-if="expandedTables.has(table.name)" class="table-details">
                  <div v-if="table.columns.length > 0" class="columns-list">
                    <h6>Columns (After):</h6>
                    <div 
                      v-for="column in table.columns" 
                      :key="'target_mod_col_' + column.name"
                      class="column-item"
                      :class="column.status"
                    >
                      <span class="column-name">{{ column.name }}</span>
                      <span class="column-type">{{ column.data_type }}</span>
                      <span v-if="column.changes" class="column-changes">
                        {{ column.changes.join(', ') }}
                      </span>
                    </div>
                  </div>
                  <div v-else class="empty-columns-message">
                    No column changes detected in target
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div> -->

  <!-- Replace the Modified Tables section -->
  <div v-if="modifiedTables.source.length > 0" class="status-section">
    <div class="status-section-header modified">
      <span class="status-icon">🔄</span>
      <h4>Modified Tables ({{ modifiedTables.source.length }})</h4>
      <p>Tables that have structural differences between databases</p>
    </div>
  
  <!-- SINGLE header row at the top -->
  <div class="side-by-side-container header-only">
    <div class="column-header">
      <h4>{{ getDatabaseName(database1) }} (Source)</h4>
    </div>
    <div class="column-header">
      <h4>{{ getDatabaseName(database2) }} (Target)</h4>
    </div>
  </div>
  
  <!-- Table pairs WITHOUT headers -->
  <div v-for="(sourceTable, index) in modifiedTables.source" :key="'modified_pair_' + sourceTable.name" class="table-pair-container">
    <div class="side-by-side-container">
      
      <!-- Source Side - NO HEADER -->
      <div class="database-column-no-header">
        <div class="column-content">
          <div class="table-card">
            <div 
              class="table-header-card modified"
              @click="toggleTableDetails(sourceTable.name)"
            >
              <div class="table-info">
                <span class="table-name">{{ sourceTable.name }}</span>
                <span class="table-status-badge modified">Modified</span>
              </div>
              <span class="toggle-icon">
                {{ expandedTables.has(sourceTable.name) ? '▼' : '▶' }}
              </span>
            </div>
            <div v-if="expandedTables.has(sourceTable.name)" class="table-details">
              <div v-if="sourceTable.columns.length > 0" class="columns-list">
                <h6>Columns (Before):</h6>
                <div 
                  v-for="column in sourceTable.columns" 
                  :key="'source_mod_col_' + column.name"
                  class="column-item"
                  :class="column.status"
                >
                  <span class="column-name">{{ column.name }}</span>
                  <span class="column-type">{{ column.data_type }}</span>
                  <span v-if="column.changes" class="column-changes">
                    {{ column.changes.join(', ') }}
                  </span>
                </div>
              </div>
              <div v-else class="empty-columns-message">
                No column changes detected in source
              </div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Target Side - NO HEADER -->
      <div class="database-column-no-header">
        <div class="column-content">
          <div class="table-card">
            <div 
              class="table-header-card modified"
              @click="toggleTableDetails(modifiedTables.target[index].name)"
            >
              <div class="table-info">
                <span class="table-name">{{ modifiedTables.target[index].name }}</span>
                <span class="table-status-badge modified">Modified</span>
              </div>
              <span class="toggle-icon">
                {{ expandedTables.has(modifiedTables.target[index].name) ? '▼' : '▶' }}
              </span>
            </div>
            <div v-if="expandedTables.has(modifiedTables.target[index].name)" class="table-details">
              <div v-if="modifiedTables.target[index].columns.length > 0" class="columns-list">
                <h6>Columns (After):</h6>
                <div 
                  v-for="column in modifiedTables.target[index].columns" 
                  :key="'target_mod_col_' + column.name"
                  class="column-item"
                  :class="column.status"
                >
                  <span class="column-name">{{ column.name }}</span>
                  <span class="column-type">{{ column.data_type }}</span>
                  <span v-if="column.changes" class="column-changes">
                    {{ column.changes.join(', ') }}
                  </span>
                </div>
              </div>
              <div v-else class="empty-columns-message">
                No column changes detected in target
              </div>
            </div>
          </div>
        </div>
      </div>
      
    </div>
  </div>
</div>

      <!-- Unchanged Tables Section -->
      <div v-if="unchangedTables.length > 0" class="status-section">
        <div class="status-section-header unchanged">
          <span class="status-icon">✓</span>
          <h4>Unchanged Tables ({{ unchangedTables.length }})</h4>
          <p>Tables that are identical in both databases</p>
        </div>
        <div class="side-by-side-container">
          <div class="database-column">
            <div class="column-header">
              <h4>{{ getDatabaseName(database1) }} (Source)</h4>
            </div>
            <div class="column-content">
              <div v-for="table in unchangedTables" :key="'unchanged_source_' + table.name" class="table-card">
                <div 
                  class="table-header-card unchanged"
                  @click="toggleTableDetails(table.name)"
                >
                  <div class="table-info">
                    <span class="table-name">{{ table.name }}</span>
                    <span class="table-status-badge unchanged">Unchanged</span>
                  </div>
                  <span class="toggle-icon">
                    {{ expandedTables.has(table.name) ? '▼' : '▶' }}
                  </span>
                </div>
                <div v-if="expandedTables.has(table.name)" class="table-details">
                  <div class="empty-columns-message">
                    Column details not available for unchanged tables
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="database-column">
            <div class="column-header">
              <h4>{{ getDatabaseName(database2) }} (Target)</h4>
            </div>
            <div class="column-content">
              <div v-for="table in unchangedTables" :key="'unchanged_target_' + table.name" class="table-card">
                <div 
                  class="table-header-card unchanged"
                  @click="toggleTableDetails(table.name)"
                >
                  <div class="table-info">
                    <span class="table-name">{{ table.name }}</span>
                    <span class="table-status-badge unchanged">Unchanged</span>
                  </div>
                  <span class="toggle-icon">
                    {{ expandedTables.has(table.name) ? '▼' : '▶' }}
                  </span>
                </div>
                <div v-if="expandedTables.has(table.name)" class="table-details">
                  <div class="empty-columns-message">
                    Column details not available for unchanged tables
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue"
import { invoke } from '@tauri-apps/api/core'; 
import { DatabaseService, type DatabaseInfo, type SchemaComparison } from '../services/databaseService';

// Props
const props = defineProps<{
  databases: DatabaseInfo[];
}>();

const emit = defineEmits<{
  'comparison-complete': [comparison: SchemaComparison];
}>();

// Reactive state
const databaseService = new DatabaseService();
const database1 = ref('');
const database2 = ref('');
const isComparing = ref(false);
const error = ref('');
const comparisonResult = ref<SchemaComparison | null>(null);
const expandedTables = ref<Set<string>>(new Set());

// Persistent state key for this component
const STORAGE_KEY = 'schema_comparison_state';

// Interface for table display
interface TableDisplay {
  name: string;
  status: 'added' | 'removed' | 'modified' | 'unchanged';
  columns: ColumnDisplay[];
}

interface ColumnDisplay {
  name: string;
  data_type: string;
  status: 'added' | 'removed' | 'modified' | 'unchanged';
  changes?: string[];
}


// Computed properties for grouped display by status
const addedTables = computed((): TableDisplay[] => {
  if (!comparisonResult.value) return [];
  
  return comparisonResult.value.added_tables.map(tableName => ({
    name: tableName,
    status: 'added' as const,
    columns: []
  })).sort((a, b) => a.name.localeCompare(b.name));
});

const removedTables = computed((): TableDisplay[] => {
  if (!comparisonResult.value) return [];
  
  return comparisonResult.value.removed_tables.map(tableName => ({
    name: tableName,
    status: 'removed' as const,
    columns: []
  })).sort((a, b) => a.name.localeCompare(b.name));
});

const modifiedTables = computed(() => {
  if (!comparisonResult.value) return { source: [], target: [] };
  
  const sourceModified: TableDisplay[] = [];
  const targetModified: TableDisplay[] = [];
  
  comparisonResult.value.modified_tables.forEach(modifiedTable => {
    // Source table (showing removed and old versions of modified columns)
    const sourceColumns: ColumnDisplay[] = [];
    
    modifiedTable.removed_columns.forEach(colName => {
      sourceColumns.push({
        name: colName,
        data_type: 'Unknown',
        status: 'removed' as const
      });
    });
    
    modifiedTable.modified_columns.forEach(modCol => {
      sourceColumns.push({
        name: modCol.column_name,
        data_type: modCol.old_type,
        status: 'modified' as const,
        changes: modCol.changes
      });
    });
    
    sourceModified.push({
      name: modifiedTable.table_name,
      status: 'modified' as const,
      columns: sourceColumns
    });
    
    // Target table (showing added and new versions of modified columns)
    const targetColumns: ColumnDisplay[] = [];
    
    modifiedTable.added_columns.forEach(col => {
      targetColumns.push({
        name: col.name,
        data_type: col.data_type,
        status: 'added' as const
      });
    });
    
    modifiedTable.modified_columns.forEach(modCol => {
      targetColumns.push({
        name: modCol.column_name,
        data_type: modCol.new_type,
        status: 'modified' as const,
        changes: modCol.changes
      });
    });
    
    targetModified.push({
      name: modifiedTable.table_name,
      status: 'modified' as const,
      columns: targetColumns
    });
  });
  
  return {
    source: sourceModified.sort((a, b) => a.name.localeCompare(b.name)),
    target: targetModified.sort((a, b) => a.name.localeCompare(b.name))
  };
});

const unchangedTables = computed((): TableDisplay[] => {
  if (!comparisonResult.value) return [];
  
  return comparisonResult.value.identical_tables.map(tableName => ({
    name: tableName,
    status: 'unchanged' as const,
    columns: []
  })).sort((a, b) => a.name.localeCompare(b.name));
});

// Methods
const compareSchemas = async () => {
  if (!database1.value || !database2.value || database1.value === database2.value) {
    error.value = 'Please select two different databases to compare';
    return;
  }

  isComparing.value = true;
  error.value = '';
  comparisonResult.value = null;
  expandedTables.value.clear();

  try {
    const result = await databaseService.compareDatabaseSchemas(database1.value, database2.value);
    comparisonResult.value = result;
    emit('comparison-complete', result);
    saveState();
  } catch (err) {
    error.value = `Comparison failed: ${err}`;
  } finally {
    isComparing.value = false;
  }
};

const toggleTableDetails = (tableName: string) => {
  const s = new Set(expandedTables.value);
  if (s.has(tableName)) {
    s.delete(tableName);
  } else {
    s.add(tableName);
  }
  expandedTables.value = s;
  saveState();
};

const getDatabaseName = (path: string): string => {
  const db = props.databases.find(d => d.path === path);
  return db ? db.name : path;
};



// const exportComparison = () => {
//   if (!comparisonResult.value) return;

//   const report = generateComparisonReport(comparisonResult.value);
//   downloadReport(report);
// };

const exportComparison = () => {
  if (!comparisonResult.value) return;

  error.value = '';
  successMessage.value = '';
  
  const report = generateComparisonReport(comparisonResult.value);
  const filename = `schema-comparison-${new Date().toISOString().slice(0, 10)}.txt`;
  downloadReport(report, filename);
  
  successMessage.value = `✅ Comparison report exported successfully!\nFile: ${filename}\nLocation: Your browser's download folder`;
  
  setTimeout(() => {
    successMessage.value = '';
  }, 5000);
};
const generateComparisonReport = (comparison: SchemaComparison): string => {
  let report = `Database Schema Comparison Report\n`;
  report += `=====================================\n\n`;
  report += `Source Database: ${comparison.database1}\n`;
  report += `Target Database: ${comparison.database2}\n`;
  report += `Generated: ${new Date().toLocaleString()}\n\n`;

  // Summary
  report += `SUMMARY\n`;
  report += `-------\n`;
  report += `Added Tables: ${comparison.added_tables.length}\n`;
  report += `Removed Tables: ${comparison.removed_tables.length}\n`;
  report += `Modified Tables: ${comparison.modified_tables.length}\n`;
  report += `Unchanged Tables: ${comparison.identical_tables.length}\n\n`;

  // Added Tables
  if (comparison.added_tables.length > 0) {
    report += `ADDED TABLES\n`;
    report += `------------\n`;
    comparison.added_tables.forEach(table => {
      report += `+ ${table}\n`;
    });
    report += `\n`;
  }

  // Removed Tables
  if (comparison.removed_tables.length > 0) {
    report += `REMOVED TABLES\n`;
    report += `--------------\n`;
    comparison.removed_tables.forEach(table => {
      report += `- ${table}\n`;
    });
    report += `\n`;
  }

  // Modified Tables
  if (comparison.modified_tables.length > 0) {
    report += `MODIFIED TABLES\n`;
    report += `---------------\n`;
    comparison.modified_tables.forEach(table => {
      report += `~ ${table.table_name}\n`;
      
      if (table.added_columns.length > 0) {
        report += `  Added Columns:\n`;
        table.added_columns.forEach(col => {
          report += `    + ${col.name} (${col.data_type})\n`;
        });
      }

      if (table.removed_columns.length > 0) {
        report += `  Removed Columns:\n`;
        table.removed_columns.forEach(col => {
          report += `    - ${col}\n`;
        });
      }

      if (table.modified_columns.length > 0) {
        report += `  Modified Columns:\n`;
        table.modified_columns.forEach(col => {
          report += `    ~ ${col.column_name}: ${col.old_type} → ${col.new_type}\n`;
          if (col.changes.length > 0) {
            report += `      Changes: ${col.changes.join(', ')}\n`;
          }
        });
      }
      report += `\n`;
    });
  }

  return report;
};

// const downloadReport = (content: string) => {
//   const blob = new Blob([content], { type: 'text/plain' });
//   const url = URL.createObjectURL(blob);
//   const a = document.createElement('a');
//   a.href = url;
//   a.download = `schema-comparison-${new Date().toISOString().slice(0, 10)}.txt`;
//   document.body.appendChild(a);
//   a.click();
//   document.body.removeChild(a);
//   URL.revokeObjectURL(url);
// };

const downloadReport = (content: string, filename: string) => {
  const blob = new Blob([content], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

// State persistence
const saveState = () => {
  const state = {
    database1: database1.value,
    database2: database2.value,
    comparisonResult: comparisonResult.value,
    expandedTables: Array.from(expandedTables.value),
    timestamp: Date.now()
  };
  
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (e) {
    console.warn('Failed to save schema comparison state:', e);
  }
};

const loadState = () => {
  try {
    const savedState = localStorage.getItem(STORAGE_KEY);
    if (savedState) {
      const state = JSON.parse(savedState);
      
      // Only load state if it's recent (within 1 hour)
      if (Date.now() - state.timestamp < 3600000) {
        database1.value = state.database1 || '';
        database2.value = state.database2 || '';
        comparisonResult.value = state.comparisonResult || null;
        expandedTables.value = new Set(state.expandedTables || []);
      }
    }
  } catch (e) {
    console.warn('Failed to load schema comparison state:', e);
  }
};
const successMessage = ref('');
const generatePatch = async () => {
  if (!database1.value || !database2.value) return;
  
   error.value = ''; // Clear previous errors
  successMessage.value = ''; // Clear previous success

  try {
    const patchSql = await invoke<string>('generate_schema_patch', {
      db1Path: database1.value,
      db2Path: database2.value,
      // db2Password: db2Password,
    });
    
   // const filename = `schema-patch-${new Date().toISOString().slice(0, 10)}.sql`;
    // const now = new Date();
    // const timestamp = `${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}-${String(now.getDate()).padStart(2,'0')}_${String(now.getHours()).padStart(2,'0')}-${String(now.getMinutes()).padStart(2,'0')}-${String(now.getSeconds()).padStart(2,'0')}`;
    // const filename = `schema-patch-${timestamp}.sql`;

    const db1Name = getDatabaseName(database1.value).replace(/[^a-z0-9]/gi, '_');
    const db2Name = getDatabaseName(database2.value).replace(/[^a-z0-9]/gi, '_');
    const timestamp = new Date().toISOString().slice(0, 19).replace(/[:.]/g, '-');
    const filename = `patch_${db1Name}_to_${db2Name}_${timestamp}.sql`;

    downloadPatch(patchSql, filename);
    
    //downloadPatch(patchSql);
     // Show success message
    successMessage.value = `✅ SQL patch generated successfully!\nFile: ${filename}\nLocation:Downloads folder`;
    
    // Auto-clear after 5 seconds
    setTimeout(() => {
      successMessage.value = '';
    }, 5000);
  } catch (err) {
    error.value = `Failed to generate patch: ${err}`;
  }
};


// const generatePatch = async () => {
//   if (!database1.value || !database2.value) return;
  
//   // Find the database info to get the password
//   const db2Info = props.databases.find(db => db.path === database2.value);
  
//   if (!db2Info) {
//     error.value = 'Target database information not found. Please reconnect the database.';
//     return;
//   }

//   // Get the password - use the stored password or prompt user if not available
//   let db2Password = db2Info.password || '';
  
//   // If password is not stored, you might want to prompt the user
//   if (!db2Password) {
//     // Option 1: Use a prompt (simple but not secure)
//     db2Password = prompt(`Please enter the password for database: ${db2Info.name}`) || '';
    
//     // Option 2: Or show an error and return
//     // error.value = `Password not available for ${db2Info.name}. Please reconnect the database with the password.`;
//     // return;
//   }

//   if (!db2Password) {
//     error.value = 'Password is required to generate the patch.';
//     return;
//   }

//   try {
//     const patchSql = await invoke<string>('generate_schema_patch', {
//       db1Path: database1.value,
//       db2Path: database2.value,
//       db2Password: db2Password, // Pass the password from frontend
//     });
    
//     downloadPatch(patchSql);
//     error.value = ''; // Clear any previous errors
//   } catch (err) {
//     error.value = `Failed to generate patch: ${err}`;
//   }
// };
// const downloadPatch = (sql: string) => {
//   const blob = new Blob([sql], { type: 'text/plain' });
//   const url = URL.createObjectURL(blob);
//   const a = document.createElement('a');
//   a.href = url;
//   a.download = `schema-patch-${new Date().toISOString().slice(0, 10)}.sql`;
//   document.body.appendChild(a);
//   a.click();
//   document.body.removeChild(a);
//   URL.revokeObjectURL(url);
// };
const downloadPatch = (sql: string, filename: string) => {
  const blob = new Blob([sql], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

// Watchers to save state on changes
watch([database1, database2], saveState);
watch(comparisonResult, saveState, { deep: true });

// Lifecycle hooks
onMounted(() => {
  loadState();
});

onBeforeUnmount(() => {
  saveState();
});
</script>

<style scoped>
.patch-btn {
  padding: 12px 30px;
  background: #6610f2;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 1em;
  cursor: pointer;
  transition: background-color 0.2s;
}

.patch-btn:hover {
  background: #520dc2;
}

.schema-comparison {
  max-width: 100%;
  height: 100%;
}

.comparison-header {
  margin-bottom: 30px;
  text-align: center;
}

.comparison-header h3 {
  margin: 0 0 10px 0;
  color: #343a40;
}

.comparison-header p {
  color: #6c757d;
  margin: 0;
}

.database-selection {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 20px;
  align-items: end;
  margin-bottom: 30px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
}

.db-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.db-selector label {
  font-weight: 600;
  color: #495057;
}

.db-selector select {
  padding: 10px 15px;
  border: 1px solid #ced4da;
  border-radius: 6px;
  background: white;
  font-size: 1em;
}

.vs-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: #007bff;
  color: white;
  border-radius: 50%;
  font-weight: bold;
  font-size: 0.9em;
}

.comparison-actions {
  display: flex;
  justify-content: center;
  gap: 15px;
  margin-bottom: 30px;
}

.compare-btn {
  padding: 12px 30px;
  background: #28a745;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 1em;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}

.compare-btn:hover:not(:disabled) {
  background: #218838;
}

.compare-btn:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.export-btn {
  padding: 12px 30px;
  background: #17a2b8;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 1em;
  cursor: pointer;
  transition: background-color 0.2s;
}

.export-btn:hover {
  background: #138496;
}

.error-message {
  padding: 15px;
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
  border-radius: 4px;
  margin-bottom: 20px;
  text-align: center;
}

.success-message {
  padding: 15px;
  background: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
  border-radius: 4px;
  margin-bottom: 20px;
  text-align: center;
  animation: slideIn 0.3s ease-out;
}

.success-message pre {
  margin: 0;
  font-family: inherit;
  white-space: pre-line;
  font-size: 0.95em;
}

.comparison-results {
  background: white;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  overflow: hidden;
}

.results-summary {
  padding: 25px;
  background: #f8f9fa;
  border-bottom: 1px solid #dee2e6;
}

.results-summary h4 {
  margin: 0 0 20px 0;
  color: #343a40;
  text-align: center;
}

.summary-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 15px;
}

.stat-item {
  text-align: center;
  padding: 15px;
  border-radius: 6px;
  background: white;
  border: 1px solid #dee2e6;
}

.stat-item.added {
  border-left: 4px solid #28a745;
}

.stat-item.removed {
  border-left: 4px solid #dc3545;
}

.stat-item.modified {
  border-left: 4px solid #ffc107;
}

.stat-item.unchanged {
  border-left: 4px solid #6c757d;
}

.stat-item .count {
  display: block;
  font-size: 2em;
  font-weight: bold;
  line-height: 1;
  margin-bottom: 5px;
}

.stat-item .label {
  font-size: 0.9em;
  color: #6c757d;
}

/* Status Sections */
.status-section {
  margin-bottom: 30px;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  overflow: hidden;
}

.status-section-header {
  padding: 20px 25px;
  display: flex;
  align-items: center;
  gap: 15px;
  font-weight: 600;
}

.status-section-header.added {
  background: linear-gradient(135deg, #d4edda 0%, #c3e6cb 100%);
  color: #155724;
  border-bottom: 1px solid #c3e6cb;
}

.status-section-header.removed {
  background: linear-gradient(135deg, #f8d7da 0%, #f5c6cb 100%);
  color: #721c24;
  border-bottom: 1px solid #f5c6cb;
}

.status-section-header.modified {
  background: linear-gradient(135deg, #fff3cd 0%, #ffeaa7 100%);
  color: #856404;
  border-bottom: 1px solid #ffeaa7;
}

.status-section-header.unchanged {
  background: linear-gradient(135deg, #e2e3e5 0%, #d6d8db 100%);
  color: #495057;
  border-bottom: 1px solid #d6d8db;
}

.status-section-header h4 {
  margin: 0;
  font-size: 1.2em;
}

.status-section-header p {
  margin: 0;
  font-size: 0.9em;
  opacity: 0.8;
  font-weight: normal;
}

.status-icon {
  font-size: 1.5em;
  width: 30px;
  text-align: center;
}

/* Table Pair Container - For Modified Tables */
.table-pair-container {
  /* margin-bottom: 1px; */
  background: #dee2e6;
}

/* Side by Side Layout - Equal Heights */
.side-by-side-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1px;
  background: #dee2e6;
  align-items: stretch;
}

.database-column {
  background: white;
  display: flex;
  flex-direction: column;
  /* min-height: 100%; */
}

.column-header {
  padding: 15px 20px;
  background: #343a40;
  color: white;
  text-align: center;
}

.column-header h4 {
  margin: 0;
  font-size: 1em;
}

.column-content {
  /* flex: 1; */
  padding: 20px;
  display: flex;
  flex-direction: column;
}

.column-content.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f8f9fa;
}

.empty-state .empty-message {
  text-align: center;
  color: #6c757d;
  font-style: italic;
  padding: 40px 20px;
}

.empty-columns-message {
  text-align: center;
  color: #6c757d;
  font-style: italic;
  padding: 15px;
  background: #f8f9fa;
  border-radius: 4px;
}

.table-card {
  /* flex: 1; */
  border: 1px solid #dee2e6;
  border-radius: 6px;
  overflow: hidden;
}

.table-header-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s;
}

.table-header-card:hover {
  background-color: #f8f9fa;
}

.table-header-card.added {
  background: #d4edda;
  border-left: 4px solid #28a745;
}

.table-header-card.removed {
  background: #f8d7da;
  border-left: 4px solid #dc3545;
}

.table-header-card.modified {
  background: #fff3cd;
  border-left: 4px solid #ffc107;
}

.table-header-card.unchanged {
  background: #e2e3e5;
  border-left: 4px solid #6c757d;
}

.table-info {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.table-name {
  font-family: monospace;
  font-weight: 600;
  font-size: 1em;
}

.table-status-badge {
  font-size: 0.75em;
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 500;
  text-transform: uppercase;
}

.table-status-badge.added {
  background: #28a745;
  color: white;
}

.table-status-badge.removed {
  background: #dc3545;
  color: white;
}

.table-status-badge.modified {
  background: #ffc107;
  color: #212529;
}

.table-status-badge.unchanged {
  background: #6c757d;
  color: white;
}

.toggle-icon {
  font-size: 0.8em;
  color: #495057;
}

.table-details {
  padding: 15px;
  background: #f8f9fa;
  border-top: 1px solid #dee2e6;
  /* min-height: 100px; */
}

.columns-list h6 {
  margin: 0 0 10px 0;
  color: #495057;
  font-size: 0.9em;
}

.column-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 12px;
  margin-bottom: 8px;
  border-radius: 4px;
  border-left: 3px solid transparent;
}

.column-item.added {
  background: #d4edda;
  border-left-color: #28a745;
}

.column-item.removed {
  background: #f8d7da;
  border-left-color: #dc3545;
}

.column-item.modified {
  background: #fff3cd;
  border-left-color: #ffc107;
}

.column-item.unchanged {
  background: #e2e3e5;
  border-left-color: #6c757d;
}

.column-name {
  font-family: monospace;
  font-weight: 600;
  font-size: 0.9em;
}

.column-type {
  font-family: monospace;
  font-size: 0.8em;
  color: #6c757d;
}

.column-changes {
  font-size: 0.75em;
  color: #856404;
  font-style: italic;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 768px) {
  .database-selection {
    grid-template-columns: 1fr;
    gap: 15px;
    text-align: center;
  }

  .vs-indicator {
    justify-self: center;
    order: 2;
  }

  .summary-stats {
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  .side-by-side-container {
    grid-template-columns: 1fr;
  }

  .status-section-header {
    flex-direction: column;
    text-align: center;
    gap: 10px;
  }

  .status-section-header h4 {
    font-size: 1.1em;
  }
}


side-by-side-container.header-only {
  margin-bottom: 0;
}

.side-by-side-container.header-only .column-header {
  padding: 15px 20px;
  background: #343a40;
  color: white;
  text-align: center;
}

/* Column without header */
.database-column-no-header {
  background: white;
  display: flex;
  flex-direction: column;
}
</style>