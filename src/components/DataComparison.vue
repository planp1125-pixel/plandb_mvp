<template>
  <div class="data-comparison">
    <div class="comparison-header">
      <h3>Database Data Comparison</h3>
      <p>Compare data between two SQLCipher databases</p>
    </div>

    <div class="database-selection">
      <div class="db-selector">
        <label>Database 1 (Source):</label>
        <select v-model="database1" :disabled="databases.length === 0">
          <option value="">Select first database...</option>
          <option v-for="db in databases" :key="`${db.path}_1`" :value="db.path">
            {{ db.name }}
          </option>
        </select>
      </div>

      <div class="vs-indicator">VS</div>

      <div class="db-selector">
        <label>Database 2 (Target):</label>
        <select v-model="database2" :disabled="databases.length === 0">
          <option value="">Select second database...</option>
          <option v-for="db in databases" :key="`${db.path}_2`" :value="db.path">
            {{ db.name }}
          </option>
        </select>
      </div>
    </div>

    <!-- Comparison Options -->
    <div class="comparison-options" v-if="database1 && database2">
      <div class="option-group">
        <label>
          <input type="checkbox" v-model="options.ignoreCase"> 
          Ignore case
        </label>
        <label>
          <input type="checkbox" v-model="options.ignoreWhitespace"> 
          Ignore whitespace
        </label>
      </div>
      
      <div class="limit-options">
        <label>Sample size:</label>
        <select v-model="options.rowLimit">
          <option :value="100">100 rows</option>
          <option :value="500">500 rows</option>
          <option :value="1000">1000 rows</option>
          <option :value="-1">All rows</option>
        </select>
      </div>
    </div>

    <!-- Enhanced Table Selection -->
    <div v-if="database1 && database2 && commonTables.length > 0" class="tables-selection">
      <div class="selection-header">
        <h4>Select Tables to Compare</h4>
        <div class="bulk-actions">
          <button @click="selectAllTables" class="bulk-btn" :disabled="selectedTables.length === commonTables.length">Select All</button>
          <button @click="deselectAllTables" class="bulk-btn" :disabled="selectedTables.length === 0">Deselect All</button>
          <span class="selection-count">{{ selectedTables.length }} of {{ commonTables.length }} selected</span>
        </div>
      </div>
      
      <div class="table-grid-container">
        <div class="table-grid">
          <div v-for="table in commonTables" :key="table" class="table-grid-item">
            <label class="table-checkbox">
              <input type="checkbox" v-model="selectedTables" :value="table" />
              <span class="table-name">{{ table }}</span>
            </label>
            <select 
              v-if="selectedTables.includes(table)" 
              v-model="keyColumns[table]" 
              class="per-table-key-select"
            >
              <option value="">Select key for {{ table }}</option>
              <option v-for="col in tableColumns[table]" :key="col" :value="col">
                {{ col }}
              </option>
            </select>
          </div>
        </div>
      </div>
      
      <div class="key-note" v-if="selectedTables.length > 0">
        <small>Per-table key mapping enabled. Select a unique key (e.g., 'id') for each table. If not selected, defaults to 'id' or first column.</small>
      </div>
    </div>

    <div class="comparison-actions">
      <button 
        @click="compareTables"
        :disabled="!canCompare"
        class="compare-btn"
      >
        {{ isComparing ? 'Comparing...' : 'Compare Data' }}
      </button>
      
      <button 
        v-if="hasResults"
        @click="exportAllDifferences"
        class="export-btn"
      >
        Export Report
      </button>

      <button 
      v-if="hasResults"
      @click="generateDataPatch"
      class="patch-btn"
      :disabled="isGeneratingPatch"
    >
      {{ isGeneratingPatch ? 'Generating...' : 'Generate Data Patch' }}
    </button>
    </div>

    <!-- Toast Notification -->
  <div v-if="toast.show" class="toast" :class="toast.type">
    <div class="toast-icon">{{ toast.type === 'success' ? '✅' : '❌' }}</div>
    <div class="toast-content">
      <div class="toast-title">{{ toast.title }}</div>
      <div class="toast-message">{{ toast.message }}</div>
    </div>
    <button @click="toast.show = false" class="toast-close">×</button>
  </div>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>

    <!-- Comparison Results -->
    <div v-if="hasResults" class="comparison-results">
      <div class="results-header">
        <h4>Data Comparison Results</h4>
        <div class="results-controls">
         
          
          <!-- Filter Controls -->
          <div class="filter-controls">
            <label>Filter:</label>
            <select v-model="currentFilter">
              <option value="all">All</option>
              <option value="identical">Identical</option>
              <option value="different">Different</option>
              <option value="missing">Only in Source</option>
              <option value="extra">Only in Target</option>
            </select>
          </div>
        </div>
      </div>

      <div class="results-summary">
        <h4>Summary ({{ currentFilterDisplay }})</h4>
        <div class="summary-stats">
          <div class="stat-item unchanged" v-if="filteredTotalIdentical > 0">
            <span class="count">{{ filteredTotalIdentical }}</span>
            <span class="label">Identical Rows</span>
          </div>
          <div class="stat-item modified" v-if="filteredTotalDifferent > 0">
            <span class="count">{{ filteredTotalDifferent }}</span>
            <span class="label">Different Rows</span>
          </div>
          <div class="stat-item removed" v-if="filteredTotalMissing > 0">
            <span class="count">{{ filteredTotalMissing }}</span>
            <span class="label">Missing Rows</span>
          </div>
          <div class="stat-item added" v-if="filteredTotalExtra > 0">
            <span class="count">{{ filteredTotalExtra }}</span>
            <span class="label">Extra Rows</span>
          </div>
        </div>
      </div>

      <div v-if="filteredTables.length === 0" class="no-results">
        No results matching the current filter.
      </div>

      <!-- Identical Rows Section -->
      <div v-if="showIdenticalSection" class="status-section">
        <div class="status-section-header unchanged">
          <span class="status-icon">✓</span>
          <h4>Identical Rows ({{ filteredTotalIdentical }})</h4>
          <p>Rows that are unchanged between databases</p>
        </div>
        
        <div v-for="result in filteredIdenticalTables" :key="'identical_' + result.tableName" class="table-card">
          <div 
            class="table-header-card unchanged"
            @click="toggleCard('identical', result.tableName)"
          >
            <div class="table-info">
              <span class="table-name">{{ result.tableName }}</span>
              <span class="table-status-badge unchanged">{{ result.summary.identicalRows }} identical rows</span>
            </div>
            <span class="toggle-icon">
              {{ expandedCards.has(`identical_${result.tableName}`) ? '▼' : '▶' }}
            </span>
          </div>
          <div v-if="expandedCards.has(`identical_${result.tableName}`)" class="table-details">
            <div class="table-view-container">
              <div v-if="viewMode === 'sideBySide'" class="side-by-side-view">
                <div class="database-column">
                  <div class="column-header">
                    <h5>{{ getDatabaseName(database1) }} (Source)</h5>
                  </div>
                  <div class="scrollable-table-container">
                    <table class="data-table">
                      <thead>
                        <tr>
                          <th v-for="column in result.comparison.commonColumns" :key="'src_ident_' + column">
                            {{ column }}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr v-for="(row, index) in result.comparison.identicalRows" :key="'src_ident_row_' + index">
                          <td v-for="column in result.comparison.commonColumns" :key="'src_ident_' + column">
                            {{ formatCellValue(row[column]) }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
              
              <div v-else class="single-view">
                <div class="scrollable-table-container">
                  <table class="data-table">
                    <thead>
                      <tr>
                        <th v-for="column in result.comparison.commonColumns" :key="'single_ident_' + column">
                          {{ column }}
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="(row, index) in result.comparison.identicalRows" :key="'single_ident_row_' + index">
                        <td v-for="column in result.comparison.commonColumns" :key="'single_ident_' + column">
                          {{ formatCellValue(row[column]) }}
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

<!-- Replace your Different Rows section template with this: -->

      <!-- Different Rows Section - Updated with proper side-by-side structure -->
      <div v-if="showDifferentSection" class="status-section">
        <div class="status-section-header modified">
          <span class="status-icon">📄</span>
          <h4>Different Rows ({{ filteredTotalDifferent }})</h4>
          <p>Rows with the same key but different values</p>
        </div>
        
        <div v-for="result in filteredDifferentTables" :key="'different_' + result.tableName" class="table-card">
          <div class="table-header-card modified" @click="toggleCard('different', result.tableName)">
            <div class="table-info">
              <span class="table-name">{{ result.tableName }}</span>
              <span class="table-status-badge modified">{{ result.summary.differentRows }} different rows</span>
            </div>
            <div class="header-controls">
              <!-- Per-table view toggle -->
              <div class="per-table-view-toggle" @click.stop>
                <button 
                  @click="setTableViewMode(result.tableName, 'sideBySide')"
                  :class="{ active: getTableViewMode(result.tableName) === 'sideBySide' }"
                  class="mini-view-btn"
                >
                  Side
                </button>
                <button 
                  @click="setTableViewMode(result.tableName, 'single')"
                  :class="{ active: getTableViewMode(result.tableName) === 'single' }"
                  class="mini-view-btn"
                >
                  Single
                </button>
              </div>

              <!-- NEW: Per-table patch button -->
              <button 
                @click.stop="generateSingleTablePatch(result.tableName)"
                class="mini-patch-btn"
                title="Generate patch for this table only"
              >
                📄
              </button>

              <span class="toggle-icon">
                {{ expandedCards.has(`different_${result.tableName}`) ? '▼' : '▶' }}
              </span>
            </div>
          </div>
          
          <div v-if="expandedCards.has(`different_${result.tableName}`)" class="table-details">
            <div class="table-view-container">
              
              <!-- Side by Side View - Two separate tables horizontally -->
              <div v-if="getTableViewMode(result.tableName) === 'sideBySide'" class="side-by-side-different-view">
                
                <!-- Source Side -->
                <div class="source-side">
                  <div class="side-header">
                    <h6>{{ getDatabaseName(database1) }} (Source)</h6>
                  </div>
                  <div class="scrollable-table-container">
                    <table class="data-table">
                      <thead>
                        <tr>
                          <th>Key</th>
                          <th v-for="column in result.comparison.commonColumns" :key="'src_side_' + column">
                            {{ column }}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr v-for="(diff, index) in result.comparison.differentRows" :key="'src_side_' + index">
                          <td class="key-cell">{{ formatCellValue(diff.sourceRow[result.keyColumn]) }}</td>
                          <td v-for="column in result.comparison.commonColumns" :key="'src_side_val_' + column"
                              :class="{ 'cell-different': diff.differentColumns.includes(column) }">
                            {{ formatCellValue(diff.sourceRow[column]) }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
                
                <!-- Target Side -->
                <div class="target-side">
                  <div class="side-header">
                    <h6>{{ getDatabaseName(database2) }} (Target)</h6>
                  </div>
                  <div class="scrollable-table-container">
                    <table class="data-table">
                      <thead>
                        <tr>
                          <th>Key</th>
                          <th v-for="column in result.comparison.commonColumns" :key="'tgt_side_' + column">
                            {{ column }}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr v-for="(diff, index) in result.comparison.differentRows" :key="'tgt_side_' + index">
                          <td class="key-cell">{{ formatCellValue(diff.targetRow[result.keyColumn]) }}</td>
                          <td v-for="column in result.comparison.commonColumns" :key="'tgt_side_val_' + column"
                              :class="{ 'cell-different': diff.differentColumns.includes(column) }">
                            {{ formatCellValue(diff.targetRow[column]) }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
                
              </div>
              
              <!-- Single View - Stacked vertically (keep existing) -->
              <div v-else class="single-different-view">
                <div v-for="(diff, index) in result.comparison.differentRows" :key="'single_diff_' + index" class="single-diff-container">
                  <div class="diff-key-header">
                    <strong>Key: {{ formatCellValue(diff.sourceRow[result.keyColumn]) }}</strong>
                  </div>
                  
                  <!-- Source Table -->
                  <div class="source-section">
                    <div class="section-title source-title">
                      <h6>{{ getDatabaseName(database1) }} (Source)</h6>
                    </div>
                    <div class="scrollable-table-container">
                      <table class="data-table">
                        <thead>
                          <tr>
                            <th v-for="column in result.comparison.commonColumns" :key="'single_src_' + column">
                              {{ column }}
                            </th>
                          </tr>
                        </thead>
                        <tbody>
                          <tr>
                            <td v-for="column in result.comparison.commonColumns" :key="'single_src_val_' + column"
                                :class="{ 'cell-different': diff.differentColumns.includes(column) }">
                              {{ formatCellValue(diff.sourceRow[column]) }}
                            </td>
                          </tr>
                        </tbody>
                      </table>
                    </div>
                  </div>
                  
                  <!-- Target Table -->
                  <div class="target-section">
                    <div class="section-title target-title">
                      <h6>{{ getDatabaseName(database2) }} (Target)</h6>
                    </div>
                    <div class="scrollable-table-container">
                      <table class="data-table">
                        <thead>
                          <tr>
                            <th v-for="column in result.comparison.commonColumns" :key="'single_tgt_' + column">
                              {{ column }}
                            </th>
                          </tr>
                        </thead>
                        <tbody>
                          <tr>
                            <td v-for="column in result.comparison.commonColumns" :key="'single_tgt_val_' + column"
                                :class="{ 'cell-different': diff.differentColumns.includes(column) }">
                              {{ formatCellValue(diff.targetRow[column]) }}
                            </td>
                          </tr>
                        </tbody>
                      </table>
                    </div>
                  </div>
                </div>
              </div>
              
            </div>
          </div>
        </div>
      </div>

      <!-- Missing Rows Section -->
      <div v-if="showMissingSection" class="status-section">
        <div class="status-section-header removed">
          <span class="status-icon">−</span>
          <h4>Missing Rows ({{ filteredTotalMissing }})</h4>
          <p>Rows that exist only in the source database</p>
        </div>
        
        <div v-for="result in filteredMissingTables" :key="'missing_' + result.tableName" class="table-card">
          <div 
            class="table-header-card removed"
            @click="toggleCard('missing', result.tableName)"
          >
            <div class="table-info">
              <span class="table-name">{{ result.tableName }}</span>
              <span class="table-status-badge removed">{{ result.summary.missingInTarget }} missing rows</span>
            </div>
            <span class="toggle-icon">
              {{ expandedCards.has(`missing_${result.tableName}`) ? '▼' : '▶' }}
            </span>
          </div>
          <div v-if="expandedCards.has(`missing_${result.tableName}`)" class="table-details">
            <div class="table-view-container">
              <div v-if="viewMode === 'sideBySide'" class="side-by-side-view">
                <div class="database-column">
                  <div class="column-header">
                    <h5>{{ getDatabaseName(database1) }} (Source Only)</h5>
                  </div>
                  <div class="scrollable-table-container">
                    <table class="data-table">
                      <thead>
                        <tr>
                          <th v-for="column in result.comparison.commonColumns" :key="'src_miss_' + column">
                            {{ column }}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr v-for="(row, index) in result.comparison.missingInTarget" :key="'src_miss_row_' + index" class="row-missing">
                          <td v-for="column in result.comparison.commonColumns" :key="'src_miss_' + column">
                            {{ formatCellValue(row[column]) }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
              
              <div v-else class="single-view">
                <div class="scrollable-table-container">
                  <table class="data-table">
                    <thead>
                      <tr>
                        <th v-for="column in result.comparison.commonColumns" :key="'single_miss_' + column">
                          {{ column }}
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="(row, index) in result.comparison.missingInTarget" :key="'single_miss_row_' + index" class="row-missing">
                        <td v-for="column in result.comparison.commonColumns" :key="'single_miss_' + column">
                          {{ formatCellValue(row[column]) }}
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Extra Rows Section -->
      <div v-if="showExtraSection" class="status-section">
        <div class="status-section-header added">
          <span class="status-icon">+</span>
          <h4>Extra Rows ({{ filteredTotalExtra }})</h4>
          <p>Rows that exist only in the target database</p>
        </div>
        
        <div v-for="result in filteredExtraTables" :key="'extra_' + result.tableName" class="table-card">
          <div 
            class="table-header-card added"
            @click="toggleCard('extra', result.tableName)"
          >
            <div class="table-info">
              <span class="table-name">{{ result.tableName }}</span>
              <span class="table-status-badge added">{{ result.summary.extraInTarget }} extra rows</span>
            </div>
            <span class="toggle-icon">
              {{ expandedCards.has(`extra_${result.tableName}`) ? '▼' : '▶' }}
            </span>
          </div>
          <div v-if="expandedCards.has(`extra_${result.tableName}`)" class="table-details">
            <div class="table-view-container">
              <div v-if="viewMode === 'sideBySide'" class="side-by-side-view">
                <div class="database-column">
                  <div class="column-header">
                    <h5>{{ getDatabaseName(database2) }} (Target Only)</h5>
                  </div>
                  <div class="scrollable-table-container">
                    <table class="data-table">
                      <thead>
                        <tr>
                          <th v-for="column in result.comparison.commonColumns" :key="'tgt_extra_' + column">
                            {{ column }}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr v-for="(row, index) in result.comparison.extraInTarget" :key="'tgt_extra_row_' + index" class="row-extra">
                          <td v-for="column in result.comparison.commonColumns" :key="'tgt_extra_' + column">
                            {{ formatCellValue(row[column]) }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
              
              <div v-else class="single-view">
                <div class="scrollable-table-container">
                  <table class="data-table">
                    <thead>
                      <tr>
                        <th v-for="column in result.comparison.commonColumns" :key="'single_extra_' + column">
                          {{ column }}
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="(row, index) in result.comparison.extraInTarget" :key="'single_extra_row_' + index" class="row-extra">
                        <td v-for="column in result.comparison.commonColumns" :key="'single_extra_' + column">
                          {{ formatCellValue(row[column]) }}
                        </td>
                      </tr>
                    </tbody>
                  </table>
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
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { DatabaseInfo } from '../services/databaseService';

interface Props {
  databases: DatabaseInfo[];
}

const props = defineProps<Props>();

interface DataRow {
  [key: string]: any;
}

interface TableData {
  rows: any[];
  columns: string[];
  total_count: number;
}

interface DifferentRow {
  sourceRow: DataRow;
  targetRow: DataRow;
  differentColumns: string[];
}

interface ComparisonResult {
  commonColumns: string[];
  sourceOnlyColumns: string[];
  targetOnlyColumns: string[];
  differentRows: DifferentRow[];
  missingInTarget: DataRow[];
  extraInTarget: DataRow[];
  identicalRows: DataRow[];
}

interface TableComparisonResult {
  tableName: string;
  keyColumn: string;
  sourceData: DataRow[];
  targetData: DataRow[];
  comparison: ComparisonResult;
  summary: {
    identicalRows: number;
    differentRows: number;
    missingInTarget: number;
    extraInTarget: number;
  };
}

// State
const database1 = ref('');
const database2 = ref('');
const commonTables = ref<string[]>([]);
const selectedTables = ref<string[]>([]);
const keyColumns = ref<Record<string, string>>({});
const tableColumns = ref<Record<string, string[]>>({});
const isComparing = ref(false);
const error = ref('');
const tableComparisons = ref<TableComparisonResult[]>([]);
const expandedCards = ref(new Set<string>());
const viewMode = ref<'sideBySide' | 'single'>('sideBySide');
const options = ref({
  ignoreCase: false,
  ignoreWhitespace: false,
  rowLimit: -1
});


const tableViewModes = ref<Record<string, 'sideBySide' | 'single'>>({});

const setTableViewMode = (tableName: string, mode: 'sideBySide' | 'single') => {
  tableViewModes.value[tableName] = mode;
};
const getTableViewMode = (tableName: string): 'sideBySide' | 'single' => {
  return tableViewModes.value[tableName] || 'sideBySide';
};

const currentFilter = ref<'all' | 'identical' | 'different' | 'missing' | 'extra'>('all');

// Computed properties
const canCompare = computed(() => {
  if (isComparing.value || !database1.value || !database2.value) return false;
  if (database1.value === database2.value) return false;
  if (selectedTables.value.length === 0) return false;
  return selectedTables.value.every(tableName => 
    keyColumns.value[tableName] && keyColumns.value[tableName].length > 0
  );
});

const hasResults = computed(() => tableComparisons.value.length > 0);

const currentFilterDisplay = computed(() => {
  const map: Record<string, string> = {
    all: 'All Results',
    identical: 'Identical Only',
    different: 'Different Only',
    missing: 'Only in Source',
    extra: 'Only in Target'
  };
  return map[currentFilter.value] || 'All Results';
});

const filteredTables = computed(() => {
  if (currentFilter.value === 'all') return tableComparisons.value;
  if (currentFilter.value === 'identical') return identicalTables.value;
  if (currentFilter.value === 'different') return differentTables.value;
  if (currentFilter.value === 'missing') return missingTables.value;
  if (currentFilter.value === 'extra') return extraTables.value;
  return [];
});

const filteredIdenticalTables = computed(() => 
  (currentFilter.value === 'all' || currentFilter.value === 'identical') 
    ? identicalTables.value 
    : []
);

const filteredDifferentTables = computed(() => 
  (currentFilter.value === 'all' || currentFilter.value === 'different') 
    ? differentTables.value 
    : []
);

const filteredMissingTables = computed(() => 
  (currentFilter.value === 'all' || currentFilter.value === 'missing') 
    ? missingTables.value 
    : []
);

const filteredExtraTables = computed(() => 
  (currentFilter.value === 'all' || currentFilter.value === 'extra') 
    ? extraTables.value 
    : []
);

const filteredTotalIdentical = computed(() => 
  filteredIdenticalTables.value.reduce((acc, t) => acc + t.summary.identicalRows, 0)
);

const filteredTotalDifferent = computed(() => 
  filteredDifferentTables.value.reduce((acc, t) => acc + t.summary.differentRows, 0)
);

const filteredTotalMissing = computed(() => 
  filteredMissingTables.value.reduce((acc, t) => acc + t.summary.missingInTarget, 0)
);

const filteredTotalExtra = computed(() => 
  filteredExtraTables.value.reduce((acc, t) => acc + t.summary.extraInTarget, 0)
);

const showIdenticalSection = computed(() => 
  (currentFilter.value === 'all' || currentFilter.value === 'identical') && 
  filteredTotalIdentical.value > 0
);

const showDifferentSection = computed(() => 
  (currentFilter.value === 'all' || currentFilter.value === 'different') && 
  filteredTotalDifferent.value > 0
);

const showMissingSection = computed(() => 
  (currentFilter.value === 'all' || currentFilter.value === 'missing') && 
  filteredTotalMissing.value > 0
);

const showExtraSection = computed(() => 
  (currentFilter.value === 'all' || currentFilter.value === 'extra') && 
  filteredTotalExtra.value > 0
);

const identicalTables = computed(() => 
  tableComparisons.value
    .filter(t => t.summary.identicalRows > 0)
    .sort((a, b) => a.tableName.localeCompare(b.tableName))
);

const differentTables = computed(() => 
  tableComparisons.value
    .filter(t => t.summary.differentRows > 0)
    .sort((a, b) => a.tableName.localeCompare(b.tableName))
);

const missingTables = computed(() => 
  tableComparisons.value
    .filter(t => t.summary.missingInTarget > 0)
    .sort((a, b) => a.tableName.localeCompare(b.tableName))
);

const extraTables = computed(() => 
  tableComparisons.value
    .filter(t => t.summary.extraInTarget > 0)
    .sort((a, b) => a.tableName.localeCompare(b.tableName))
);

// Methods
const getDatabaseName = (path: string): string => {
  return props.databases.find(d => d.path === path)?.name || path;
};

const selectAllTables = () => {
  selectedTables.value = [...commonTables.value];
};

const deselectAllTables = () => {
  selectedTables.value = [];
  keyColumns.value = {};
};

const compareTables = async () => {
  if (!canCompare.value) return;
  
  isComparing.value = true;
  error.value = '';
  tableComparisons.value = [];
  expandedCards.value.clear();

  try {
    for (const tableName of selectedTables.value) {
      try {
        const result = await compareTable(tableName);
        tableComparisons.value.push(result);
      } catch (err) {
        console.error(`Table ${tableName} comparison failed:`, err);
        error.value += `Table ${tableName}: ${err instanceof Error ? err.message : String(err)}\n`;
      }
    }
    
    if (tableComparisons.value.length === 0) {
      error.value = 'No data found to compare in selected tables.';
    }
  } catch (err) {
    error.value = `Comparison failed: ${err instanceof Error ? err.message : String(err)}`;
  } finally {
    isComparing.value = false;
  }
};

const getCommonTables = async (db1Path: string, db2Path: string): Promise<string[]> => {
  try {
    const [tables1, tables2] = await Promise.all([
      invoke<{ name: string }[]>('get_database_tables', { dbPath: db1Path }),
      invoke<{ name: string }[]>('get_database_tables', { dbPath: db2Path })
    ]);
    
    const tableNames1 = tables1.map(t => t.name);
    const tableNames2 = tables2.map(t => t.name);
    
    return tableNames1.filter(name => tableNames2.includes(name)).sort();
  } catch (err) {
    throw new Error(`Failed to get table list: ${err}`);
  }
};

const loadTableColumns = async (tableName: string) => {
  try {
    const data = await invoke<TableData>('get_table_data', {
      dbPath: database1.value,
      tableName,
      limit: 1
    });
    tableColumns.value[tableName] = data.columns || [];
  } catch (err) {
    console.error(`Failed to load columns for ${tableName}:`, err);
  }
};

const compareTable = async (tableName: string): Promise<TableComparisonResult> => {
  const limitValue = options.value.rowLimit === -1 ? null : options.value.rowLimit;
  const tableKey = keyColumns.value[tableName] || '';
  if (!tableKey) {
    throw new Error(`No key column specified for table: ${tableName}`);
  }
  
  const [source, target] = await Promise.all([
    invoke<TableData>('get_table_data', {
      dbPath: database1.value,
      tableName,
      limit: limitValue
    }),
    invoke<TableData>('get_table_data', {
      dbPath: database2.value,
      tableName,
      limit: limitValue
    })
  ]);

  const sourceData = source.rows || [];
  const targetData = target.rows || [];
  
  const comparison = performDataComparison(
    sourceData, 
    targetData, 
    source.columns || [], 
    target.columns || [], 
    tableKey, 
    tableName
  );

  return {
    tableName,
    keyColumn: tableKey,
    sourceData,
    targetData,
    comparison,
    summary: {
      identicalRows: comparison.identicalRows.length,
      differentRows: comparison.differentRows.length,
      missingInTarget: comparison.missingInTarget.length,
      extraInTarget: comparison.extraInTarget.length
    }
  };
};

const performDataComparison = (
  sourceData: any[],
  targetData: any[],
  sourceColumns: string[],
  targetColumns: string[],
  keyColumn: string,
  tableName: string
): ComparisonResult => {
  const commonColumns = sourceColumns.filter(col => targetColumns.includes(col));
  const sourceOnlyColumns = sourceColumns.filter(col => !targetColumns.includes(col));
  const targetOnlyColumns = targetColumns.filter(col => !sourceColumns.includes(col));

  const convertToObject = (rows: any[], cols: string[]): DataRow[] => {
    return rows.map(row => {
      if (typeof row === 'object' && row !== null && !Array.isArray(row)) return row;
      const obj: DataRow = {};
      cols.forEach((col, i) => {
        obj[col] = Array.isArray(row) ? row[i] : row;
      });
      return obj;
    });
  };

  const sourceRows = convertToObject(sourceData, sourceColumns);
  const targetRows = convertToObject(targetData, targetColumns);

  const identical: DataRow[] = [];
  const different: DifferentRow[] = [];
  const missingInTarget: DataRow[] = [];
  const extraInTarget: DataRow[] = [];

  if (!keyColumn || !commonColumns.includes(keyColumn)) {
    throw new Error(`Invalid key column (${keyColumn}) for table ${tableName}`);
  }

  const sourceMap = new Map<string, DataRow>();
  for (const row of sourceRows) {
    const key = normalizeValue(row[keyColumn]);
    if (key && key !== 'NULL') {
      sourceMap.set(key, row);
    }
  }

  const targetMap = new Map<string, DataRow>();
  for (const row of targetRows) {
    const key = normalizeValue(row[keyColumn]);
    if (key && key !== 'NULL') {
      targetMap.set(key, row);
    }
  }

  for (const [key, sourceRow] of sourceMap) {
    const targetRow = targetMap.get(key);
    if (targetRow) {
      const diffColumns = findDifferentColumns(sourceRow, targetRow, commonColumns, keyColumn);
      if (diffColumns.length > 0) {
        different.push({ sourceRow, targetRow, differentColumns: diffColumns });
      } else {
        identical.push(sourceRow);
      }
      targetMap.delete(key);
    } else {
      missingInTarget.push(sourceRow);
    }
  }

  extraInTarget.push(...targetMap.values());

  return {
    commonColumns,
    sourceOnlyColumns,
    targetOnlyColumns,
    differentRows: different,
    missingInTarget,
    extraInTarget,
    identicalRows: identical
  };
};

const normalizeValue = (value: any): string => {
  if (value == null) return 'NULL';
  if (typeof value === 'string') {
    if (options.value.ignoreCase) value = value.toLowerCase();
    if (options.value.ignoreWhitespace) value = value.trim();
  }
  return String(value);
};

const findDifferentColumns = (sourceRow: DataRow, targetRow: DataRow, columns: string[], keyColumn: string): string[] => {
  return columns.filter(col => {
    if (col === keyColumn) return false;
    let sourceVal = sourceRow[col];
    let targetVal = targetRow[col];

    if (sourceVal === targetVal) return false;
    if (sourceVal == null && targetVal == null) return false;

    if (typeof sourceVal === 'string' && typeof targetVal === 'string') {
      if (options.value.ignoreCase) {
        sourceVal = sourceVal.toLowerCase();
        targetVal = targetVal.toLowerCase();
      }
      if (options.value.ignoreWhitespace) {
        sourceVal = sourceVal.trim();
        targetVal = targetVal.trim();
      }
    }

    return sourceVal !== targetVal;
  });
};

const toggleCard = (section: string, tableName: string) => {
  const key = `${section}_${tableName}`;
  const newSet = new Set(expandedCards.value);
  if (newSet.has(key)) {
    newSet.delete(key);
  } else {
    newSet.add(key);
  }
  expandedCards.value = newSet;
};

const formatCellValue = (value: any): string => {
  if (value === null || value === undefined) return 'NULL';
  if (typeof value === 'string' && value === '') return '(empty)';
  if (typeof value === 'string' && value.length > 50) return value.substring(0, 47) + '...';
  return String(value);
};

const exportAllDifferences = () => {
  if (!hasResults.value) return;
  
  const report = generateAllDifferencesReport();
  downloadReport(report, 'data-comparison-report.txt');
};

const generateAllDifferencesReport = (): string => {
  let report = `Data Comparison Report\n`;
  report += `=======================\n\n`;
  report += `Generated: ${new Date().toLocaleString()}\n`;
  report += `Source: ${getDatabaseName(database1.value)}\n`;
  report += `Target: ${getDatabaseName(database2.value)}\n\n`;
  
  tableComparisons.value.forEach(result => {
    report += `Table: ${result.tableName} (Key: ${result.keyColumn})\n`;
    report += `${'-'.repeat(50)}\n`;
    report += `Summary:\n`;
    report += `- Identical Rows: ${result.summary.identicalRows}\n`;
    report += `- Different Rows: ${result.summary.differentRows}\n`;
    report += `- Missing in Target: ${result.summary.missingInTarget}\n`;
    report += `- Extra in Target: ${result.summary.extraInTarget}\n\n`;
    
    if (result.comparison.differentRows.length > 0) {
      report += `Different Rows:\n`;
      result.comparison.differentRows.forEach((diff, index) => {
        report += `  ${index + 1}. Key: ${normalizeValue(diff.sourceRow[result.keyColumn])}\n`;
        diff.differentColumns.forEach(col => {
          report += `      ${col}: "${diff.sourceRow[col]}" -> "${diff.targetRow[col]}"\n`;
        });
      });
      report += `\n`;
    }
  });
  
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

// Watchers
// watch([database1, database2], async () => {
//   tableComparisons.value = [];
//   error.value = '';
//   expandedCards.value.clear();
//   commonTables.value = [];
//   selectedTables.value = [];
//   keyColumns.value = {};
//   tableColumns.value = {};
  
//   if (database1.value && database2.value && database1.value !== database2.value) {
//     try {
//       commonTables.value = await getCommonTables(database1.value, database2.value);
//       selectedTables.value = commonTables.value.slice();
//       for (const table of commonTables.value) {
//         await loadTableColumns(table);
//       }
//     } catch (err) {
//       error.value = `Failed to load tables: ${err}`;
//     }
//   }
// });

watch([database1, database2], async () => {
  tableComparisons.value = [];
  error.value = '';
  expandedCards.value.clear();
  commonTables.value = [];
  selectedTables.value = [];
  keyColumns.value = {};
  tableColumns.value = {};
  tableViewModes.value = {}; // Add this line
  
  if (database1.value && database2.value && database1.value !== database2.value) {
    try {
      commonTables.value = await getCommonTables(database1.value, database2.value);
      selectedTables.value = commonTables.value.slice();
      for (const table of commonTables.value) {
        await loadTableColumns(table);
      }
    } catch (err) {
      error.value = `Failed to load tables: ${err}`;
    }
  }
});

watch(selectedTables, async (newTables, oldTables) => {
  for (const table of newTables.filter(t => !oldTables.includes(t))) {
    await loadTableColumns(table);
    const cols = tableColumns.value[table] || [];
    const defaultKey = cols.find(col => col.toLowerCase().includes('id')) || cols[0] || '';
    keyColumns.value[table] = defaultKey;
  }
  
  for (const table of oldTables.filter(t => !newTables.includes(t))) {
    delete keyColumns.value[table];
    delete tableColumns.value[table];
  }
});




const isGeneratingPatch = ref(false);
const toast = ref({
  show: false,
  type: 'success' as 'success' | 'error',
  title: '',
  message: ''
});

// Add toast helper
const showToast = (type: 'success' | 'error', title: string, message: string) => {
  toast.value = { show: true, type, title, message };
  setTimeout(() => {
    toast.value.show = false;
  }, 5000);
};

// Add data patch generation method
const generateDataPatch = async () => {
  if (!hasResults.value || !database1.value || !database2.value) return;
  
  isGeneratingPatch.value = true;
  
  try {
    // Prepare comparison data for backend
    const comparisonData = tableComparisons.value.map(result => ({
      tableName: result.tableName,
      keyColumn: result.keyColumn,
      comparison: {
        missingInTarget: result.comparison.missingInTarget,
        differentRows: result.comparison.differentRows.map(diff => ({
          sourceRow: diff.sourceRow,
          targetRow: diff.targetRow,
          differentColumns: diff.differentColumns
        })),
        extraInTarget: result.comparison.extraInTarget,
        commonColumns: result.comparison.commonColumns
      }
    }));

    const patchSql = await invoke<string>('generate_data_patch', {
      db1Path: database1.value,
      db2Path: database2.value,
      tableComparisons: comparisonData
    });
    
    // Generate filename with database names and timestamp
    const db1Name = getDatabaseName(database1.value)
      .replace(/\.[^/.]+$/, '')
      .replace(/[^a-z0-9]/gi, '_');
    
    const db2Name = getDatabaseName(database2.value)
      .replace(/\.[^/.]+$/, '')
      .replace(/[^a-z0-9]/gi, '_');
    
    const timestamp = new Date().toISOString()
      .slice(0, 19)
      .replace('T', '_')
      .replace(/:/g, '-');
    
    const filename = `data_patch_${db1Name}_to_${db2Name}_${timestamp}.sql`;
    
    downloadPatch(patchSql, filename);
    
    showToast('success', 'Data Patch Generated', 
      `File: ${filename}\nSaved to Downloads folder\n\nIMPORTANT: Review before executing!`);
    
  } catch (err) {
    showToast('error', 'Patch Generation Failed', String(err));
  } finally {
    isGeneratingPatch.value = false;
  }
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


const generateSingleTablePatch = async (tableName: string) => {
  const tableResult = tableComparisons.value.find(t => t.tableName === tableName);
  if (!tableResult) return;
  
  try {
    // Reuse the same backend command with single table
    const comparisonData = [{
      tableName: tableResult.tableName,
      keyColumn: tableResult.keyColumn,
      comparison: {
        missingInTarget: tableResult.comparison.missingInTarget,
        differentRows: tableResult.comparison.differentRows.map(diff => ({
          sourceRow: diff.sourceRow,
          targetRow: diff.targetRow,
          differentColumns: diff.differentColumns
        })),
        extraInTarget: tableResult.comparison.extraInTarget,
        commonColumns: tableResult.comparison.commonColumns
      }
    }];

    const patchSql = await invoke<string>('generate_data_patch', {
      db1Path: database1.value,
      db2Path: database2.value,
      tableComparisons: comparisonData
    });
    
    const timestamp = new Date().toISOString()
      .slice(0, 19)
      .replace('T', '_')
      .replace(/:/g, '-');
    
    const filename = `data_patch_${tableName}_${timestamp}.sql`;
    downloadPatch(patchSql, filename);
    
    showToast('success', 'Table Patch Generated', 
      `Table: ${tableName}\nFile: ${filename}`);
    
  } catch (err) {
    showToast('error', 'Failed', String(err));
  }
};

</script>


<style scoped>
.data-comparison {
  max-width: 100%;
  height: 100%;
  padding: 20px;
  margin: 0 auto;
}

.comparison-header {
  text-align: center;
  margin-bottom: 30px;
}

.comparison-header h3 {
  margin: 0;
  font-size: 1.5em;
  color: #2c3e50;
}

.comparison-header p {
  margin: 5px 0 0;
  color: #6c757d;
}

/* Per-table view toggle styles */
.header-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.per-table-view-toggle {
  display: flex;
  border: 1px solid #ced4da;
  border-radius: 4px;
  overflow: hidden;
  background: white;
}

.mini-view-btn {
  padding: 4px 8px;
  border: none;
  background: white;
  color: #495057;
  cursor: pointer;
  font-size: 0.75em;
  transition: all 0.2s;
  min-width: 45px;
}

.mini-view-btn:hover {
  background: #f8f9fa;
}

.mini-view-btn.active {
  background: #007bff;
  color: white;
}

/* Fixed single view table scrolling */
.single-view-table {
  max-height: 300px !important;
  overflow-x: auto !important;
  overflow-y: auto !important;
  width: 100%;
}

/* Enhanced scrollbar for single view tables */

.single-view-table::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 10px;
  border: 2px solid #e0e0e0;
}

.single-view-table::-webkit-scrollbar-thumb {
  background: #666;
  border-radius: 10px;
  border: 3px solid #f1f1f1;
  min-height: 40px;
  min-width: 40px;
}

.single-view-table::-webkit-scrollbar-thumb:hover {
  background: #444;
}

.single-view-table::-webkit-scrollbar-corner {
  background: #f1f1f1;
}


.database-selection {
  display: grid;
  grid-template-columns: 2fr auto 2fr;
  align-items: center;
  gap: 20px;
  margin-bottom: 30px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #dee2e6;
}

.db-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.db-selector label {
  font-weight: 500;
  color: #495057;
}

.db-selector select {
  padding: 10px;
  border: 1px solid #ced4da;
  border-radius: 6px;
  font-size: 1em;
  background: white;
}

.vs-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 50px;
  height: 50px;
  background: #007bff;
  color: white;
  border-radius: 50%;
  font-weight: bold;
  font-size: 1.1em;
}

.comparison-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  margin-bottom: 30px;
  padding: 15px;
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 6px;
}

.option-group {
  display: flex;
  gap: 20px;
}

.option-group label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9em;
  cursor: pointer;
}

.limit-options {
  display: flex;
  align-items: center;
  gap: 10px;
}

.limit-options label {
  font-weight: 500;
  color: #495057;
}

.limit-options select {
  padding: 6px;
  border: 1px solid #ced4da;
  border-radius: 4px;
}

.tables-selection {
  margin-bottom: 30px;
}

.selection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.selection-header h4 {
  margin: 0;
  font-size: 1.2em;
  color: #2c3e50;
}

.bulk-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.bulk-btn {
  padding: 8px 16px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9em;
  transition: background 0.2s;
}

.bulk-btn:hover:not(:disabled) {
  background: #0056b3;
}

.bulk-btn:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.selection-count {
  font-size: 0.9em;
  color: #6c757d;
}

.table-grid-container {
  max-height: 250px;
  overflow-y: auto;
  padding: 15px;
  background: white;
  border: 1px solid #dee2e6;
  border-radius: 6px;
}

.table-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
}

.table-grid-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.table-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 4px;
  font-size: 0.9em;
  cursor: pointer;
  transition: background 0.2s;
}

.table-checkbox:hover {
  background: #e9ecef;
}


.per-table-key-select {
  padding: 6px;
  border: 1px solid #ced4da;
  border-radius: 4px;
  font-size: 0.85em;
  width: 100%;
  background: white;
}

.key-note {
  margin-top: 10px;
  font-style: italic;
  color: #6c757d;
  font-size: 0.85em;
}

.comparison-actions {
  display: flex;
  gap: 15px;
  justify-content: center;
  margin-bottom: 30px;
}

.compare-btn,
.export-btn {
  padding: 12px 24px;
  border: none;
  border-radius: 6px;
  font-size: 1em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.compare-btn {
  background: #28a745;
  color: white;
}

.compare-btn:hover:not(:disabled) {
  background: #218838;
  transform: translateY(-1px);
}

.compare-btn:disabled {
  background: #6c757d;
  cursor: not-allowed;
  transform: none;
}

.export-btn {
  background: #007bff;
  color: white;
}

.export-btn:hover {
  background: #0056b3;
  transform: translateY(-1px);
}

.error-message {
  color: #dc3545;
  background: #f8d7da;
  border: 1px solid #f5c6cb;
  padding: 12px;
  border-radius: 6px;
  margin-bottom: 20px;
  white-space: pre-wrap;
}

.comparison-results {
  margin-top: 30px;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 20px;
  background: white;
  border: 1px solid #dee2e6;
  border-radius: 6px;
}

.results-header h4 {
  margin: 0;
  font-size: 1.3em;
  color: #2c3e50;
}

.results-controls {
  display: flex;
  gap: 20px;
  align-items: center;
}

.view-toggle {
  display: flex;
  border: 1px solid #ced4da;
  border-radius: 6px;
  overflow: hidden;
}

.view-btn {
  padding: 8px 16px;
  border: none;
  background: white;
  color: #495057;
  cursor: pointer;
  font-size: 0.9em;
  transition: all 0.2s;
}

.view-btn:hover {
  background: #f8f9fa;
}

.view-btn.active {
  background: #007bff;
  color: white;
}

.filter-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.filter-controls label {
  font-weight: 500;
  color: #495057;
}

.filter-controls select {
  padding: 8px;
  border: 1px solid #ced4da;
  border-radius: 4px;
  background: white;
}

.results-summary {
  margin-bottom: 30px;
  padding: 20px;
  background: white;
  border: 1px solid #dee2e6;
  border-radius: 6px;
}

.results-summary h4 {
  margin: 0 0 20px;
  font-size: 1.2em;
  color: #2c3e50;
  text-align: center;
}

.summary-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 20px;
}

.stat-item {
  text-align: center;
  padding: 20px;
  border-radius: 8px;
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  transition: transform 0.2s;
}

.stat-item:hover {
  transform: translateY(-2px);
}

.stat-item.added {
  border-left: 5px solid #28a745;
  background: linear-gradient(135deg, #d4edda, #c3e6cb);
}

.stat-item.removed {
  border-left: 5px solid #dc3545;
  background: linear-gradient(135deg, #f8d7da, #f5c6cb);
}

.stat-item.modified {
  border-left: 5px solid #ffc107;
  background: linear-gradient(135deg, #fff3cd, #ffeaa7);
}

.stat-item.unchanged {
  border-left: 5px solid #6c757d;
  background: linear-gradient(135deg, #e2e3e5, #d6d8db);
}

.stat-item .count {
  display: block;
  font-size: 2.5em;
  font-weight: bold;
  line-height: 1;
  margin-bottom: 8px;
  color: #2c3e50;
}

.stat-item .label {
  font-size: 0.9em;
  color: #495057;
  font-weight: 500;
}

.status-section {
  margin-bottom: 30px;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  overflow: hidden;
  background: white;
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
  font-size: 1.3em;
}

.status-section-header p {
  margin: 0;
  font-size: 0.9em;
  opacity: 0.9;
  font-weight: normal;
}

.status-icon {
  font-size: 1.8em;
  width: 40px;
  text-align: center;
}

.table-card {
  margin: 15px;
  border: 1px solid #dee2e6;
  border-radius: 6px;
  overflow: hidden;
  background: white;
}

.table-header-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
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
  gap: 8px;
}

.table-name {
  font-family: 'Courier New', monospace;
  font-weight: 600;
  font-size: 1.1em;
  color: #2c3e50;
}

.table-status-badge {
  font-size: 0.75em;
  padding: 4px 10px;
  border-radius: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
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
  font-size: 1em;
  color: #495057;
  transition: transform 0.2s;
}

.table-details {
  border-top: 1px solid #dee2e6;
  background: #f8f9fa;
}

.table-view-container {
  padding: 20px;
}


.scrollable-table-container {
  max-height: 400px;
  overflow-x: auto;
  overflow-y: auto;
  border: 2px solid #dee2e6;
  border-radius: 6px;
  background: white;
  scroll-behavior: smooth;
  scrollbar-gutter: stable;
  padding-top: 10px;
}


.scrollable-table-container::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
}

.scrollable-table-container::-webkit-scrollbar-thumb {
  background: #888;
  border-radius: 8px;
  border: 2px solid #f1f1f1;
  min-height: 30px;
  min-width: 30px;
}

.scrollable-table-container::-webkit-scrollbar-thumb:hover {
  background: #555;
}

.scrollable-table-container::-webkit-scrollbar-corner {
  background: #f1f1f1;
}

.scrollable-table-container {
  scrollbar-width: thin;
  scrollbar-color: #888 #f1f1f1;
}


/* /* Firefox Support */
.scrollable-table-container {
  scrollbar-width: thick;
  scrollbar-color: #666 #f1f1f1;
}
/* Ensure missing and extra sections have proper scrollbars */
.status-section .scrollable-table-container {
  max-height: 400px !important;
  overflow: auto !important;
}
/* Base Data Table Styles */
.data-table {
  width: 100%;
  min-width: max-content;
  border-collapse: collapse;
  font-size: 0.85em;
  background: white;
  table-layout: auto;
}

/* Ensure all table cells maintain minimum width */
.data-table th,
.data-table td {
  min-width: 120px;
  max-width: 200px;
  padding: 12px 15px;
  text-align: left;
  border-bottom: 1px solid #dee2e6;
  border-right: 1px solid #f0f0f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.data-table th {
  background: #f8f9fa;
  font-weight: 600;
  color: #495057;
  position: sticky;
  top: 0;
  z-index: 10;
  border-bottom: 2px solid #dee2e6;
}

.data-table td {
  font-family: 'Courier New', monospace;
  font-size: 0.8em;
  color: #2c3e50;
}
/* Force horizontal scroll when content exceeds container */
.scrollable-table-container .data-table {
  width: max-content;
  min-width: 100%;
}
/* Side by Side View Styles */
.side-by-side-view {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.database-column {
  border: 1px solid #dee2e6;
  border-radius: 6px;
  overflow: hidden;
  background: white;
}

.column-header {
  padding: 12px 15px;
  background: #343a40;
  color: white;
  text-align: center;
}

.column-header h5 {
  margin: 0;
  font-size: 1em;
  font-weight: 500;
}

/* Single View Styles */
.single-view {
  background: white;
  border-radius: 6px;
  overflow: hidden;
}

/* DIFFERENT ROWS - Side by Side Specific Styles */
.side-by-side-different-view {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  background: white;
  border-radius: 6px;
  overflow: hidden;
}

.side-by-side-different-view .diff-comparison-container {
  display: contents; /* This allows grid items to be direct children */
}

.side-by-side-different-view .source-side,
.side-by-side-different-view .target-side {
  border: 2px solid #dee2e6;
  border-radius: 8px;
  overflow: hidden;
  background: white;
}

.side-by-side-different-view .source-side {
  border-left: 4px solid #4caf50;
}

.side-by-side-different-view .target-side {
  border-left: 4px solid #ff9800;
}

.side-by-side-different-view .side-header {
  padding: 12px 15px;
  font-weight: 600;
  text-align: center;
  font-size: 0.9em;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.side-by-side-different-view .source-side .side-header {
  background: #e8f5e8;
  color: #2e7d32;
  border-bottom: 2px solid #4caf50;
}

.side-by-side-different-view .target-side .side-header {
  background: #fff3e0;
  color: #ef6c00;
  border-bottom: 2px solid #ff9800;
}

.side-by-side-different-view .scrollable-table-container {
  max-height: 400px;
  overflow-x: auto;
  overflow-y: auto;
  border: none;
  border-radius: 0;
}

.side-by-side-different-view .source-side .data-table {
  background: #f8fff8;
}

.side-by-side-different-view .target-side .data-table {
  background: #fffcf5;
}

.side-by-side-different-view .source-side .data-table th {
  background: #e8f5e8 !important;
  color: #2e7d32 !important;
}

.side-by-side-different-view .target-side .data-table th {
  background: #fff3e0 !important;
  color: #ef6c00 !important;
}

.side-by-side-different-view .source-side .cell-different {
  background: #c8e6c9 !important;
  color: #1b5e20 !important;
  font-weight: bold !important;
}

.side-by-side-different-view .target-side .cell-different {
  background: #ffcc80 !important;
  color: #e65100 !important;
  font-weight: bold !important;
}

/* Add scrollbar styles specifically for side-by-side view */

.side-by-side-different-view .scrollable-table-container::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 10px;
  border: 2px solid #e0e0e0;
}

.side-by-side-different-view .scrollable-table-container::-webkit-scrollbar-thumb {
  background: #666;
  border-radius: 10px;
  border: 3px solid #f1f1f1;
  min-height: 40px;
  min-width: 40px;
}

.side-by-side-different-view .scrollable-table-container::-webkit-scrollbar-thumb:hover {
  background: #444;
}

.side-by-side-different-view .scrollable-table-container::-webkit-scrollbar-corner {
  background: #f1f1f1;
}

/* DIFFERENT ROWS - Single View Specific Styles */
.single-different-view {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.single-diff-container {
  border: 1px solid #dee2e6;
  border-radius: 8px;
  overflow: hidden;
  background: white;
}

.diff-key-header {
  padding: 12px 20px;
  background: #e3f2fd;
  border-bottom: 2px solid #2196f3;
  font-size: 1em;
  color: #1976d2;
  text-align: center;
  font-weight: bold;
}

.source-section,
.target-section {
  margin-bottom: 15px;
}

.source-section:last-child,
.target-section:last-child {
  margin-bottom: 0;
}

.section-title {
  padding: 10px 15px;
  font-weight: 600;
  text-align: center;
  font-size: 0.9em;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.source-title {
  background: #e8f5e8;
  color: #2e7d32;
  border-bottom: 2px solid #4caf50;
}

.target-title {
  background: #fff3e0;
  color: #ef6c00;
  border-bottom: 2px solid #ff9800;
}

.source-section .scrollable-table-container {
  border-left: 4px solid #4caf50;
  border-right: 4px solid #4caf50;
  border-bottom: 4px solid #4caf50;
  border-top: 1px solid #dee2e6;
  border-radius: 0 0 6px 6px;
  max-height: 250px;
}

.target-section .scrollable-table-container {
  border-left: 4px solid #ff9800;
  border-right: 4px solid #ff9800;
  border-bottom: 4px solid #ff9800;
  border-top: 1px solid #dee2e6;
  border-radius: 0 0 6px 6px;
  max-height: 250px;
}

.source-section .data-table {
  background: #f8fff8;
}

.target-section .data-table {
  background: #fffcf5;
}

.source-section .data-table th {
  background: #e8f5e8 !important;
  color: #2e7d32 !important;
}

.target-section .data-table th {
  background: #fff3e0 !important;
  color: #ef6c00 !important;
}

.source-section .cell-different {
  background: #c8e6c9 !important;
  color: #1b5e20 !important;
  font-weight: bold !important;
}

.target-section .cell-different {
  background: #ffcc80 !important;
  color: #e65100 !important;
  font-weight: bold !important;
}

/* Key Cell Styling */
.key-cell {
  background: #e3f2fd !important;
  font-weight: bold !important;
  color: #1976d2 !important;
  position: sticky;
  left: 0;
  z-index: 15;
  border-right: 2px solid #2196f3 !important;
}

/* Row Type Styling */
.row-different {
  background: #fef3c7;
}

.row-missing {
  background: #fee2e2;
}

.row-extra {
  background: #e0e7ff;
}

.cell-different {
  background: #fed7aa !important;
  font-weight: 600 !important;
  color: #d97706 !important;
}

/* No Results */
.no-results {
  text-align: center;
  padding: 60px 20px;
  color: #6c757d;
  font-style: italic;
  font-size: 1.1em;
}

/* Responsive Design */
@media (max-width: 1200px) {
  .results-controls {
    flex-direction: column;
    gap: 15px;
  }
}

@media (max-width: 768px) {
  .data-comparison {
    padding: 15px;
  }
  
  .database-selection {
    grid-template-columns: 1fr;
    gap: 15px;
    text-align: center;
  }

  .vs-indicator {
    order: 2;
    justify-self: center;
  }

  .comparison-options {
    flex-direction: column;
    gap: 15px;
  }

  .option-group {
    flex-direction: column;
    gap: 10px;
  }

  .results-header {
    flex-direction: column;
    gap: 15px;
  }

  .results-controls {
    width: 100%;
    justify-content: center;
  }

  .summary-stats {
    grid-template-columns: repeat(2, 1fr);
    gap: 15px;
  }

  .selection-header {
    flex-direction: column;
    gap: 10px;
  }

  .bulk-actions {
    flex-wrap: wrap;
    justify-content: center;
  }

  .table-grid {
    grid-template-columns: 1fr;
  }

  .view-toggle {
    width: 100%;
  }

  .view-btn {
    flex: 1;
  }

  .status-section-header {
    flex-direction: column;
    text-align: center;
    gap: 10px;
  }

  .table-info {
    align-items: center;
    text-align: center;
  }

  /* Mobile scrollbar adjustments */
  .scrollable-table-container::-webkit-scrollbar {
    width: 14px;
    height: 14px;
  }
  
  .side-by-side-different-view .key-header {
    position: static;
  }
  
  .source-section .scrollable-table-container,
  .target-section .scrollable-table-container {
    max-height: 200px;
  }

  /* Single diff container height fix */
.single-diff-container .scrollable-table-container {
  max-height: 300px;
}

.source-section .scrollable-table-container,
.target-section .scrollable-table-container {
  max-height: 300px !important;
}
}



.patch-btn {
  padding: 12px 24px;
  background: #6610f2;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 1em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.patch-btn:hover:not(:disabled) {
  background: #520dc2;
  transform: translateY(-1px);
}

.patch-btn:disabled {
  background: #6c757d;
  cursor: not-allowed;
  transform: none;
}

/* Toast notification styles */
.toast {
  position: fixed;
  bottom: 20px;
  right: 20px;
  min-width: 350px;
  max-width: 500px;
  padding: 16px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: start;
  gap: 12px;
  z-index: 9999;
  animation: slideInRight 0.3s ease-out;
}

.toast.success {
  background: #d4edda;
  border-left: 4px solid #28a745;
  color: #155724;
}

.toast.error {
  background: #f8d7da;
  border-left: 4px solid #dc3545;
  color: #721c24;
}

.toast-icon {
  font-size: 24px;
  line-height: 1;
}

.toast-content {
  flex: 1;
}

.toast-title {
  font-weight: 600;
  margin-bottom: 4px;
}

.toast-message {
  font-size: 0.9em;
  white-space: pre-line;
}

.toast-close {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  padding: 0;
  line-height: 1;
  opacity: 0.5;
}

.toast-close:hover {
  opacity: 1;
}

@keyframes slideInRight {
  from {
    transform: translateX(400px);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.mini-patch-btn {
  padding: 4px 8px;
  border: none;
  background: #6610f2;
  color: white;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
  transition: background 0.2s;
}

.mini-patch-btn:hover {
  background: #520dc2;
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>