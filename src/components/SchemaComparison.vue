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

    <!-- Loading indicator -->
    <div v-if="isLoadingSchemas" class="loading-indicator">
      <div class="spinner"></div>
      <p>Loading detailed table schemas...</p>
    </div>

    <!-- Comparison Results - Side by Side -->
    <div v-if="comparisonResult && !isLoadingSchemas" class="comparison-results">
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
                  <div v-if="table.columns.length > 0" class="columns-list">
                    <h6>Columns ({{ table.columns.length }}):</h6>
                    <div 
                      v-for="(column, index) in table.columns" 
                      :key="'added_col_' + column.name"
                      class="column-item added"
                    >
                      <div class="column-number">#{{ index + 1 }}</div>
                      <div class="column-info">
                        <span class="column-name">{{ column.name }}</span>
                        <span class="column-type">{{ column.data_type }}</span>
                        <div class="column-attributes">
                          <span v-if="column.is_primary_key" class="attribute-badge pk">PK</span>
                          <span v-if="!column.is_nullable" class="attribute-badge nn">NOT NULL</span>
                          <span v-if="column.default_value" class="attribute-badge def">Default: {{ column.default_value }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div v-else class="empty-columns-message">
                    No columns found
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
                  <div v-if="table.columns.length > 0" class="columns-list">
                    <h6>Columns ({{ table.columns.length }}):</h6>
                    <div 
                      v-for="(column, index) in table.columns" 
                      :key="'removed_col_' + column.name"
                      class="column-item removed"
                    >
                      <div class="column-number">#{{ index + 1 }}</div>
                      <div class="column-info">
                        <span class="column-name">{{ column.name }}</span>
                        <span class="column-type">{{ column.data_type }}</span>
                        <div class="column-attributes">
                          <span v-if="column.is_primary_key" class="attribute-badge pk">PK</span>
                          <span v-if="!column.is_nullable" class="attribute-badge nn">NOT NULL</span>
                          <span v-if="column.default_value" class="attribute-badge def">Default: {{ column.default_value }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div v-else class="empty-columns-message">
                    No columns found
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

  <!-- Modified Tables Section with ALL columns side-by-side -->
  <div v-if="modifiedTables.length > 0" class="status-section">
    <div class="status-section-header modified">
      <span class="status-icon">🔄</span>
      <h4>Modified Tables ({{ modifiedTables.length }})</h4>
      <p>Tables with structural differences - showing ALL columns with changes highlighted</p>
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
    <div v-for="modifiedTable in modifiedTables" :key="'modified_pair_' + modifiedTable.tableName" class="table-pair-container">
      <div class="side-by-side-container">
        
        <!-- Source Side -->
        <div class="database-column-no-header">
          <div class="column-content">
            <div class="table-card">
              <div 
                class="table-header-card modified"
                @click="toggleTableDetails(modifiedTable.tableName)"
              >
                <div class="table-info">
                  <span class="table-name">{{ modifiedTable.tableName }}</span>
                  <span class="table-status-badge modified">Modified</span>
                </div>
                <span class="toggle-icon">
                  {{ expandedTables.has(modifiedTable.tableName) ? '▼' : '▶' }}
                </span>
              </div>
              <div v-if="expandedTables.has(modifiedTable.tableName)" class="table-details">
                <div v-if="modifiedTable.sourceColumns.length > 0" class="columns-list">
                  <h6>Columns ({{ modifiedTable.sourceColumns.length }}) - Before:</h6>
                  <div 
                    v-for="(column, index) in modifiedTable.sourceColumns" 
                    :key="'source_mod_col_' + column.name"
                    class="column-item"
                    :class="column.status"
                  >
                    <div class="column-number">#{{ index + 1 }}</div>
                    <div class="column-info">
                      <span class="column-name">{{ column.name }}</span>
                      <span class="column-type">{{ column.data_type }}</span>
                      <div class="column-attributes">
                        <span v-if="column.is_primary_key" class="attribute-badge pk">PK</span>
                        <span v-if="!column.is_nullable" class="attribute-badge nn">NOT NULL</span>
                        <span v-if="column.default_value" class="attribute-badge def">Default: {{ column.default_value }}</span>
                      </div>
                      <span v-if="column.changes && column.changes.length > 0" class="column-changes">
                        Changes: {{ column.changes.join(', ') }}
                      </span>
                      <span v-if="column.status" class="status-label">{{ getStatusLabel(column.status) }}</span>
                    </div>
                  </div>
                </div>
                <div v-else class="empty-columns-message">
                  No columns found
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Target Side -->
        <div class="database-column-no-header">
          <div class="column-content">
            <div class="table-card">
              <div 
                class="table-header-card modified"
                @click="toggleTableDetails(modifiedTable.tableName)"
              >
                <div class="table-info">
                  <span class="table-name">{{ modifiedTable.tableName }}</span>
                  <span class="table-status-badge modified">Modified</span>
                </div>
                <span class="toggle-icon">
                  {{ expandedTables.has(modifiedTable.tableName) ? '▼' : '▶' }}
                </span>
              </div>
              <div v-if="expandedTables.has(modifiedTable.tableName)" class="table-details">
                <div v-if="modifiedTable.targetColumns.length > 0" class="columns-list">
                  <h6>Columns ({{ modifiedTable.targetColumns.length }}) - After:</h6>
                  <div 
                    v-for="(column, index) in modifiedTable.targetColumns" 
                    :key="'target_mod_col_' + column.name"
                    class="column-item"
                    :class="column.status"
                  >
                    <div class="column-number">#{{ index + 1 }}</div>
                    <div class="column-info">
                      <span class="column-name">{{ column.name }}</span>
                      <span class="column-type">{{ column.data_type }}</span>
                      <div class="column-attributes">
                        <span v-if="column.is_primary_key" class="attribute-badge pk">PK</span>
                        <span v-if="!column.is_nullable" class="attribute-badge nn">NOT NULL</span>
                        <span v-if="column.default_value" class="attribute-badge def">Default: {{ column.default_value }}</span>
                      </div>
                      <span v-if="column.changes && column.changes.length > 0" class="column-changes">
                        Changes: {{ column.changes.join(', ') }}
                      </span>
                      <span v-if="column.status" class="status-label">{{ getStatusLabel(column.status) }}</span>
                    </div>
                  </div>
                </div>
                <div v-else class="empty-columns-message">
                  No columns found
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
                  <div v-if="table.columns.length > 0" class="columns-list">
                    <h6>Columns ({{ table.columns.length }}):</h6>
                    <div 
                      v-for="(column, index) in table.columns" 
                      :key="'unchanged_col_' + column.name"
                      class="column-item unchanged"
                    >
                      <div class="column-number">#{{ index + 1 }}</div>
                      <div class="column-info">
                        <span class="column-name">{{ column.name }}</span>
                        <span class="column-type">{{ column.data_type }}</span>
                        <div class="column-attributes">
                          <span v-if="column.is_primary_key" class="attribute-badge pk">PK</span>
                          <span v-if="!column.is_nullable" class="attribute-badge nn">NOT NULL</span>
                          <span v-if="column.default_value" class="attribute-badge def">Default: {{ column.default_value }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div v-else class="empty-columns-message">
                    No columns found
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
                  <div v-if="table.columns.length > 0" class="columns-list">
                    <h6>Columns ({{ table.columns.length }}):</h6>
                    <div 
                      v-for="(column, index) in table.columns" 
                      :key="'unchanged_col_' + column.name"
                      class="column-item unchanged"
                    >
                      <div class="column-number">#{{ index + 1 }}</div>
                      <div class="column-info">
                        <span class="column-name">{{ column.name }}</span>
                        <span class="column-type">{{ column.data_type }}</span>
                        <div class="column-attributes">
                          <span v-if="column.is_primary_key" class="attribute-badge pk">PK</span>
                          <span v-if="!column.is_nullable" class="attribute-badge nn">NOT NULL</span>
                          <span v-if="column.default_value" class="attribute-badge def">Default: {{ column.default_value }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div v-else class="empty-columns-message">
                    No columns found
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <!-- SQL Patch Modal -->
    <!-- Patch Preview Modal -->
    <div v-if="showPatchModal" class="patch-preview-overlay" @click="closePatchModal">
      <div class="patch-preview-modal" @click.stop>
        <div class="patch-preview-header">
          <h3>🔧 Generated SQL Patch Preview</h3>
          <button @click="closePatchModal" class="close-btn">✕</button>
        </div>

        <div class="patch-preview-info">
          <div class="info-item">
            <strong>File:</strong> {{ patchFilename }}
          </div>
          <div class="info-item">
            <strong>Size:</strong> {{ formatBytes(generatedPatchSQL.length) }}
          </div>
          <div class="info-item warning">
            ⚠️ <strong>Important:</strong> Review carefully before executing on your database!
          </div>
        </div>

        <div class="patch-preview-content">
          <pre><code>{{ generatedPatchSQL }}</code></pre>
        </div>

        <div class="patch-preview-actions">
          <button @click="copyPatchToClipboard" class="action-btn copy-btn" :disabled="isCopied">
            {{ isCopied ? '✅ Copied!' : '📋 Copy to Clipboard' }}
          </button>
          <button @click="downloadPatchFromModal" class="action-btn download-btn" :disabled="isDownloaded">
            {{ isDownloaded ? '✅ Downloaded!' : '💾 Download SQL File' }}
          </button>
          <button @click="closePatchModal" class="action-btn cancel-btn">
            Close
          </button>
        </div>
      </div>
    </div>
  </div>

  
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue"
import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';
import { DatabaseService, type DatabaseInfo, type SchemaComparison, type ColumnInfo, type TableInfo } from '../services/databaseService';

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
const isLoadingSchemas = ref(false);
const error = ref('');
const comparisonResult = ref<SchemaComparison | null>(null);
const expandedTables = ref<Set<string>>(new Set());

// Store full table schemas
const db1FullSchemas = ref<Map<string, ColumnInfo[]>>(new Map());
const db2FullSchemas = ref<Map<string, ColumnInfo[]>>(new Map());

// Persistent state key for this component
const STORAGE_KEY = 'schema_comparison_state';

// Interface for table display
interface TableDisplay {
  name: string;
  status: 'added' | 'removed' | 'modified' | 'unchanged';
  columns: ColumnDisplayExtended[];
}

interface ColumnDisplayExtended extends ColumnInfo {
  status?: 'added' | 'removed' | 'modified' | 'unchanged';
  changes?: string[];
}

interface ModifiedTableDisplay {
  tableName: string;
  sourceColumns: ColumnDisplayExtended[];
  targetColumns: ColumnDisplayExtended[];
}

// Fetch full table schemas for both databases
const fetchFullSchemas = async () => {
  if (!database1.value || !database2.value) return;
  
  isLoadingSchemas.value = true;
  try {
    // Fetch all tables from both databases
    const [db1Tables, db2Tables] = await Promise.all([
      databaseService.getDatabaseTables(database1.value),
      databaseService.getDatabaseTables(database2.value)
    ]);
    
    // Store in maps for quick lookup
    db1FullSchemas.value.clear();
    db2FullSchemas.value.clear();
    
    db1Tables.forEach((table: TableInfo) => {
      db1FullSchemas.value.set(table.name, table.columns);
    });
    
    db2Tables.forEach((table: TableInfo) => {
      db2FullSchemas.value.set(table.name, table.columns);
    });
    
  //  error.value = ''; // Clear any previous errors on success
    
  } catch (err) {
    console.error('Error fetching full schemas:', err);
    // DON'T show error if databases aren't connected yet
    // Only show error if we're actively comparing
    // if (isComparing.value) {
       error.value = `Failed to load full table schemas: ${err}`;
    // }
  } finally {
    isLoadingSchemas.value = false;
  }
}

// Computed properties for grouped display by status
const addedTables = computed((): TableDisplay[] => {
  if (!comparisonResult.value) return [];
  
  return comparisonResult.value.added_tables.map(tableName => {
    const columns = db2FullSchemas.value.get(tableName) || [];
    return {
      name: tableName,
      status: 'added' as const,
      columns: columns.map(col => ({
        ...col,
        status: 'added' as const
      }))
    };
  }).sort((a, b) => a.name.localeCompare(b.name));
});

const removedTables = computed((): TableDisplay[] => {
  if (!comparisonResult.value) return [];
  
  return comparisonResult.value.removed_tables.map(tableName => {
    const columns = db1FullSchemas.value.get(tableName) || [];
    return {
      name: tableName,
      status: 'removed' as const,
      columns: columns.map(col => ({
        ...col,
        status: 'removed' as const
      }))
    };
  }).sort((a, b) => a.name.localeCompare(b.name));
});

const modifiedTables = computed((): ModifiedTableDisplay[] => {
  if (!comparisonResult.value) return [];
  
  return comparisonResult.value.modified_tables.map(modifiedTable => {
    // Get full schemas for both databases
    const sourceFullColumns = db1FullSchemas.value.get(modifiedTable.table_name) || [];
    const targetFullColumns = db2FullSchemas.value.get(modifiedTable.table_name) || [];
    
    // Create sets for quick lookup
    const addedColumnNames = new Set(modifiedTable.added_columns.map(c => c.name));
    const removedColumnNames = new Set(modifiedTable.removed_columns);
    const modifiedColumnMap = new Map(
      modifiedTable.modified_columns.map(c => [c.column_name, c])
    );
    
    // Process source columns (mark removed and modified)
    const sourceColumns: ColumnDisplayExtended[] = sourceFullColumns.map(col => {
      if (removedColumnNames.has(col.name)) {
        return { ...col, status: 'removed' as const };
      } else if (modifiedColumnMap.has(col.name)) {
        const modInfo = modifiedColumnMap.get(col.name)!;
        return {
          ...col,
          data_type: modInfo.old_type,
          status: 'modified' as const,
          changes: modInfo.changes
        };
      } else {
        return { ...col, status: 'unchanged' as const };
      }
    });
    
    // Process target columns (mark added and modified)
    const targetColumns: ColumnDisplayExtended[] = targetFullColumns.map(col => {
      if (addedColumnNames.has(col.name)) {
        return { ...col, status: 'added' as const };
      } else if (modifiedColumnMap.has(col.name)) {
        const modInfo = modifiedColumnMap.get(col.name)!;
        return {
          ...col,
          data_type: modInfo.new_type,
          status: 'modified' as const,
          changes: modInfo.changes
        };
      } else {
        return { ...col, status: 'unchanged' as const };
      }
    });
    
    return {
      tableName: modifiedTable.table_name,
      sourceColumns,
      targetColumns
    };
  }).sort((a, b) => a.tableName.localeCompare(b.tableName));
});

const unchangedTables = computed((): TableDisplay[] => {
  if (!comparisonResult.value) return [];
  
  return comparisonResult.value.identical_tables.map(tableName => {
    const columns = db1FullSchemas.value.get(tableName) || [];
    return {
      name: tableName,
      status: 'unchanged' as const,
      columns: columns.map(col => ({
        ...col,
        status: 'unchanged' as const
      }))
    };
  }).sort((a, b) => a.name.localeCompare(b.name));
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
    
    // Fetch full schemas for all tables
    await fetchFullSchemas();
    
    emit('comparison-complete', result);
    saveState();
  } catch (err) {
    error.value = `Comparison failed: ${err}`;
    await message(String(err), { title: 'Comparison Failed', kind: 'error' });
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

const getStatusLabel = (status: string): string => {
  switch (status) {
    case 'added': return '✚ Added';
    case 'removed': return '✖ Removed';
    case 'modified': return '✎ Modified';
    case 'unchanged': return '✓ Unchanged';
    default: return '';
  }
};

const exportComparison = () => {
  if (!comparisonResult.value) return;

  error.value = '';

  const report = generateComparisonReport(comparisonResult.value);
  const filename = `schema-comparison-${new Date().toISOString().slice(0, 10)}.txt`;
  downloadReport(report, filename);

  // Export successful - file downloaded, no notification needed
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

  // Added Tables with columns
  if (comparison.added_tables.length > 0) {
    report += `ADDED TABLES\n`;
    report += `------------\n`;
    comparison.added_tables.forEach(tableName => {
      report += `+ ${tableName}\n`;
      const columns = db2FullSchemas.value.get(tableName) || [];
      if (columns.length > 0) {
        columns.forEach((col, idx) => {
          report += `  ${idx + 1}. ${col.name} (${col.data_type})`;
          if (col.is_primary_key) report += ` PK`;
          if (!col.is_nullable) report += ` NOT NULL`;
          report += `\n`;
        });
      }
      report += `\n`;
    });
  }

  // Removed Tables with columns
  if (comparison.removed_tables.length > 0) {
    report += `REMOVED TABLES\n`;
    report += `--------------\n`;
    comparison.removed_tables.forEach(tableName => {
      report += `- ${tableName}\n`;
      const columns = db1FullSchemas.value.get(tableName) || [];
      if (columns.length > 0) {
        columns.forEach((col, idx) => {
          report += `  ${idx + 1}. ${col.name} (${col.data_type})`;
          if (col.is_primary_key) report += ` PK`;
          if (!col.is_nullable) report += ` NOT NULL`;
          report += `\n`;
        });
      }
      report += `\n`;
    });
  }

  // Modified Tables with full column details
  if (comparison.modified_tables.length > 0) {
    report += `MODIFIED TABLES\n`;
    report += `---------------\n`;
    comparison.modified_tables.forEach(table => {
      report += `~ ${table.table_name}\n`;
      
      const sourceColumns = db1FullSchemas.value.get(table.table_name) || [];
      const targetColumns = db2FullSchemas.value.get(table.table_name) || [];
      
      report += `  Source Columns (${sourceColumns.length}):\n`;
      sourceColumns.forEach((col, idx) => {
        const isRemoved = table.removed_columns.includes(col.name);
        const isModified = table.modified_columns.find(m => m.column_name === col.name);
        let status = '  ';
        if (isRemoved) status = '- ';
        if (isModified) status = '~ ';
        report += `    ${status}${idx + 1}. ${col.name} (${col.data_type})`;
        if (col.is_primary_key) report += ` PK`;
        if (!col.is_nullable) report += ` NOT NULL`;
        report += `\n`;
      });
      
      report += `  Target Columns (${targetColumns.length}):\n`;
      targetColumns.forEach((col, idx) => {
        const isAdded = table.added_columns.find(a => a.name === col.name);
        const isModified = table.modified_columns.find(m => m.column_name === col.name);
        let status = '  ';
        if (isAdded) status = '+ ';
        if (isModified) status = '~ ';
        report += `    ${status}${idx + 1}. ${col.name} (${col.data_type})`;
        if (col.is_primary_key) report += ` PK`;
        if (!col.is_nullable) report += ` NOT NULL`;
        report += `\n`;
      });

      if (table.modified_columns.length > 0) {
        report += `  Changes:\n`;
        table.modified_columns.forEach(col => {
          report += `    ~ ${col.column_name}: ${col.old_type} → ${col.new_type}\n`;
          if (col.changes.length > 0) {
            report += `      ${col.changes.join(', ')}\n`;
          }
        });
      }
      report += `\n`;
    });
  }

  // Unchanged Tables
  if (comparison.identical_tables.length > 0) {
    report += `UNCHANGED TABLES\n`;
    report += `----------------\n`;
    comparison.identical_tables.forEach(tableName => {
      report += `✓ ${tableName}\n`;
      const columns = db1FullSchemas.value.get(tableName) || [];
      if (columns.length > 0) {
        report += `  Columns (${columns.length}):\n`;
        columns.forEach((col, idx) => {
          report += `    ${idx + 1}. ${col.name} (${col.data_type})`;
          if (col.is_primary_key) report += ` PK`;
          if (!col.is_nullable) report += ` NOT NULL`;
          report += `\n`;
        });
      }
      report += `\n`;
    });
  }

  return report;
};

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
        
        // // Reload full schemas if we have a comparison result
        // if (comparisonResult.value && database1.value && database2.value) {
        //   fetchFullSchemas();
        // }
      }
    }
  } catch (e) {
    console.warn('Failed to load schema comparison state:', e);
  }
};

const showPatchModal = ref(false);
const generatedPatchSQL = ref('');
const patchFilename = ref('');
const isCopied = ref(false);
const isDownloaded = ref(false);

// const generatePatch = async () => {
//   if (!database1.value || !database2.value) return;
  
//   error.value = '';
//   successMessage.value = '';

//   try {
//     const patchSql = await invoke<string>('generate_schema_patch', {
//       db1Path: database1.value,
//       db2Path: database2.value,
//     });
    
//     const db1Name = getDatabaseName(database1.value).replace(/[^a-z0-9]/gi, '_');
//     const db2Name = getDatabaseName(database2.value).replace(/[^a-z0-9]/gi, '_');
//     const timestamp = new Date().toISOString().slice(0, 19).replace(/[:.]/g, '-');
//     const filename = `patch_${db1Name}_to_${db2Name}_${timestamp}.sql`;

//     downloadPatch(patchSql, filename);
    
//     successMessage.value = `✅ SQL patch generated successfully!\nFile: ${filename}\nLocation:Downloads folder`;
    
//     setTimeout(() => {
//       successMessage.value = '';
//     }, 5000);
//   } catch (err) {
//     error.value = `Failed to generate patch: ${err}`;
//   }
// };

const generatePatch = async () => {
  if (!database1.value || !database2.value) return;
  
  error.value = '';

  try {
    const patchSql = await invoke<string>('generate_schema_patch', {
      db1Path: database1.value,
      db2Path: database2.value,
    });
    
    const db1Name = getDatabaseName(database1.value).replace(/[^a-z0-9]/gi, '_');
    const db2Name = getDatabaseName(database2.value).replace(/[^a-z0-9]/gi, '_');
    const timestamp = new Date().toISOString().slice(0, 19).replace(/[:.]/g, '-');
    const filename = `patch_${db1Name}_to_${db2Name}_${timestamp}.sql`;

    // Check file size - if larger than 5MB, skip preview and auto-download
    const fileSizeBytes = new Blob([patchSql]).size;
    const SIZE_LIMIT_MB = 5;
    const SIZE_LIMIT_BYTES = SIZE_LIMIT_MB * 1024 * 1024;

    if (fileSizeBytes > SIZE_LIMIT_BYTES) {
      // Large file - skip preview, directly download
      const fileSizeMB = (fileSizeBytes / (1024 * 1024)).toFixed(2);
      await message(
        `Patch size: ${fileSizeMB}MB\n\nFile is too large to preview and will be downloaded directly.\n\nFile: ${filename}`,
        { title: 'Large Patch Generated', kind: 'info' }
      );
      downloadPatch(patchSql, filename);
    } else {
      // Small file - show preview modal
      generatedPatchSQL.value = patchSql;
      patchFilename.value = filename;
      showPatchModal.value = true;
    }

  } catch (err) {
    error.value = `Failed to generate patch: ${err}`;
    await message(String(err), { title: 'Patch Generation Failed', kind: 'error' });
  }
};
const copyPatchToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(generatedPatchSQL.value);
    // Show "Copied!" feedback on button
    isCopied.value = true;
    setTimeout(() => {
      isCopied.value = false;
    }, 2000);
  } catch (err) {
    error.value = `Failed to copy to clipboard: ${err}`;
    await message('Could not copy to clipboard', { title: 'Copy Failed', kind: 'error' });
  }
};

const downloadPatchFromModal = () => {
  downloadPatch(generatedPatchSQL.value, patchFilename.value);
  // Show "Downloaded!" feedback on button
  isDownloaded.value = true;
  setTimeout(() => {
    isDownloaded.value = false;
  }, 2000);
};

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
};

const closePatchModal = () => {
  showPatchModal.value = false;
};

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

.schema-comparison {
  max-width: 100%;
  height: 100%;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.comparison-header {
  margin-bottom: 30px;
  text-align: center;
}

.comparison-header h3 {
  margin: 0 0 10px 0;
  color: var(--text-primary);
}

.comparison-header p {
  color: var(--text-secondary);
  margin: 0;
}

.database-selection {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 20px;
  align-items: end;
  margin-bottom: 30px;
  padding: 20px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.db-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.db-selector label {
  font-weight: 600;
  color: var(--text-primary);
}

.db-selector select {
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
  color: var(--text-primary);
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

.loading-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin: 20px 0;
  border: 1px solid var(--border-color);
}

.spinner {
  border: 4px solid var(--border-light);
  border-top: 4px solid #007bff;
  border-radius: 50%;
  width: 50px;
  height: 50px;
  animation: spin 1s linear infinite;
  margin-bottom: 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-indicator p {
  color: var(--text-secondary);
  font-size: 1.1em;
  margin: 0;
}

.comparison-results {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
}

.results-summary {
  padding: 25px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.results-summary h4 {
  margin: 0 0 20px 0;
  color: var(--text-primary);
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
  /* border: 1px solid #dee2e6; */
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}




.stat-item.added {
  background: var(--status-added-bg);
  border-left: 4px solid var(--status-added-border);
}

.stat-item.removed {
  background: var(--status-removed-bg);
  border-left: 4px solid var(--status-removed-border);
}

.stat-item.modified {
  background: var(--status-modified-bg);
  border-left: 4px solid var(--status-modified-border);
}

.stat-item.unchanged {
  background: var(--status-unchanged-bg);
  border-left: 4px solid var(--status-unchanged-border);
}

.stat-item .count {
  display: block;
  font-size: 2em;
  font-weight: bold;
  line-height: 1;
  margin-bottom: 5px;
  color: var(--text-primary);
}

.stat-item .label {
  font-size: 0.9em;
  color: var(--text-secondary);
}
/* Status Sections */
.status-section {
  margin-bottom: 30px;
  border: 1px solid var(--border-color);
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
  background: var(--status-added-bg);
  color: var(--status-added-text);
  border-left: 4px solid var(--status-added-border);
}

.status-section-header.removed {
  background: var(--status-removed-bg);
  color: var(--status-removed-text);
  border-left: 4px solid var(--status-removed-border);
}

.status-section-header.modified {
  background: var(--status-modified-bg);
  color: var(--status-modified-text);
  border-left: 4px solid var(--status-modified-border);
}

.status-section-header.unchanged {
  background: var(--status-unchanged-bg);
  color: var(--status-unchanged-text);
  border-left: 4px solid var(--status-unchanged-border);
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
  background: var(--border-color);
}

/* Side by Side Layout - Equal Heights */
.side-by-side-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1px;
  background: var(--border-color);
  align-items: stretch;
}

.database-column {
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
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
  padding: 20px;
  display: flex;
  flex-direction: column;
}

.column-content.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
}

.empty-state .empty-message {
  text-align: center;
  color: var(--text-secondary);
  font-style: italic;
  padding: 40px 20px;
}

.empty-columns-message {
  text-align: center;
  color: var(--text-secondary);
  font-style: italic;
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: 4px;
}

.table-card {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-primary);
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
  background-color: var(--bg-secondary);
}

.table-header-card.added {
  background: var(--status-added-bg);
  border-left: 4px solid var(--status-added-border);
}

.table-header-card.removed {
  background: var(--status-removed-bg);
  border-left: 4px solid var(--status-removed-border);
}

.table-header-card.modified {
  background: var(--status-modified-bg);
  border-left: 4px solid var(--status-modified-border);
}

.table-header-card.unchanged {
  background: var(--status-unchanged-bg);
  border-left: 4px solid var(--status-unchanged-border);
}

.table-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.table-name {
  font-family: monospace;
  font-weight: 600;
  font-size: 1em;
  color: var(--text-primary);
}

.table-status-badge {
  font-size: 0.75em;
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 500;
  text-transform: uppercase;
}

.table-status-badge.added {
  background: var(--status-added-badge-bg);
  color: var(--status-added-badge-text);
}

.table-status-badge.removed {
  background: var(--status-removed-badge-bg);
  color: var(--status-removed-badge-text);
}

.table-status-badge.modified {
  background: var(--status-modified-badge-bg);
  color: var(--status-modified-badge-text);
}

.table-status-badge.unchanged {
  background: var(--status-unchanged-badge-bg);
  color: var(--status-unchanged-badge-text);
}

.toggle-icon {
  font-size: 0.8em;
  color: var(--text-primary);
}

.table-details {
  padding: 15px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
}

.columns-list h6 {
  margin: 0 0 10px 0;
  color: var(--text-primary);
  font-size: 0.9em;
}

.column-item {
  display: flex;
  gap: 12px;
  padding: 10px 12px;
  margin-bottom: 8px;
  border-radius: 4px;
  border-left: 3px solid transparent;
  align-items: flex-start;
}

.column-item.added {
  background: var(--status-added-bg);
  border-left-color: var(--status-added-border);
}

.column-item.removed {
  background: var(--status-removed-bg);
  border-left-color: var(--status-removed-border);
}

.column-item.modified {
  background: var(--status-modified-bg);
  border-left-color: var(--status-modified-border);
}

.column-item.unchanged {
  background: var(--status-unchanged-bg);
  border-left-color: var(--status-unchanged-border);
}

.column-number {
  font-family: monospace;
  font-weight: bold;
  font-size: 0.85em;
  color: var(--text-secondary);
  min-width: 30px;
  flex-shrink: 0;
}

.column-info {
  display: flex;
  flex-direction: column;
  color: var(--text-primary);
  gap: 4px;
  flex: 1;
}

.column-name {
  font-family: monospace;
  font-weight: 600;
  font-size: 0.9em;
  color: var(--text-primary);
}

.column-type {
  font-family: monospace;
  font-size: 0.8em;
  color: var(--text-secondary);
}

.column-attributes {
  display: flex;
  gap: 6px;
  color: var(--text-secondary);
  flex-wrap: wrap;
}

.attribute-badge {
  font-size: 0.7em;
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: 600;
  text-transform: uppercase;
}

.attribute-badge.pk {
  background: #007bff;
  color: white;
}

.attribute-badge.nn {
  background: #6c757d;
  color: white;
}

.attribute-badge.def {
  background: #17a2b8;
  color: white;
}

.column-changes {
  font-size: 0.75em;
  color: #856404;
  font-style: italic;
  padding: 4px 0;
}

.status-label {
  font-size: 0.75em;
  font-weight: 600;
  padding: 2px 0;
  color: var(--text-primary);
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

.side-by-side-container.header-only {
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
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
}

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

/* Modal Styles */
/* Patch Preview Modal Styles */
.patch-preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(4px);
}

.patch-preview-modal {
  background: var(--bg-primary);
  border-radius: 12px;
  width: 90%;
  max-width: 900px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
}

.patch-preview-header {
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-secondary);
  border-radius: 12px 12px 0 0;
}

.patch-preview-header h3 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.3em;
}

.patch-preview-header .close-btn {
  background: none;
  border: none;
  font-size: 1.5em;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.patch-preview-header .close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.patch-preview-info {
  padding: 16px 24px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-item {
  color: var(--text-primary);
  font-size: 0.9em;
}

.info-item strong {
  color: var(--text-primary);
  margin-right: 8px;
}

.info-item.warning {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
  padding: 12px;
  border-radius: 6px;
  border-left: 3px solid #f59e0b;
}

.patch-preview-content {
  flex: 1;
  overflow: auto;
  padding: 0;
  background: #1e1e1e;
}

.patch-preview-content pre {
  margin: 0;
  padding: 20px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 0.85em;
  line-height: 1.6;
  color: #d4d4d4;
  overflow: auto;
}

.patch-preview-content code {
  font-family: 'Courier New', Courier, monospace;
  color: #d4d4d4;
}

.patch-preview-actions {
  padding: 20px 24px;
  border-top: 1px solid var(--border-color);
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  background: var(--bg-secondary);
  border-radius: 0 0 12px 12px;
}

.action-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 0.95em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.copy-btn {
  background: #3b82f6;
  color: white;
}

.copy-btn:hover:not(:disabled) {
  background: #2563eb;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
}

.copy-btn:disabled {
  background: #10b981;
  cursor: default;
  opacity: 1;
}

.download-btn {
  background: #10b981;
  color: white;
}

.download-btn:hover:not(:disabled) {
  background: #059669;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);
}

.download-btn:disabled {
  background: #10b981;
  cursor: default;
  opacity: 0.8;
}

.cancel-btn {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.cancel-btn:hover {
  background: var(--bg-hover);
}
</style>
