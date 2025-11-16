<template>
  <div class="data-comparison">

    <!-- Loading Overlay for Comparison -->
    <div v-if="isComparing" class="loading-overlay">
      <div class="loading-content">
        <div class="spinner-large"></div>
        <h3>{{ comparisonProgress.message }}</h3>
        <div class="progress-bar-large">
          <div class="progress-fill-large" :style="{ width: comparisonProgress.percentage + '%' }"></div>
        </div>
        <div class="progress-stats">
          <p><strong>{{ comparisonProgress.percentage }}%</strong> Complete</p>
          <p>Table {{ comparisonProgress.currentTable }} of {{ comparisonProgress.totalTables }}</p>
          <p v-if="comparisonProgress.currentRows">Processing: {{ comparisonProgress.currentRows.toLocaleString() }} / {{ comparisonProgress.totalRows.toLocaleString() }} rows</p>
          <p v-if="comparisonProgress.speed">Speed: <strong>{{ comparisonProgress.speed.toLocaleString() }}</strong> rows/sec</p>
        </div>
      </div>
    </div>

    <!-- Loading Overlay for Patch Generation -->
    <!-- <div v-if="isGeneratingPatch" class="loading-overlay">
      <div class="loading-content">
        <div class="spinner-large"></div>
        <h3>Generating SQL Patch...</h3>
        <p class="loading-message">This may take a moment for large datasets</p>
        <div class="spinner-dots">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>
    </div> -->

    <!-- Loading Overlay for Patch Generation -->
    <div v-if="isGeneratingPatch" class="loading-overlay">
      <div class="loading-content">
        <h3>Generating SQL Patch...</h3>
        <p class="loading-message">This may take a moment for large datasets</p>
        <div class="spinner-dots">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>
    </div>

    <div class="database-selection">
      <div class="db-selector">
        <label>Database 1 (Source):</label>
        <select v-model="database1" :disabled="databases.length === 0 || isComparing">
          <option value="">Select first database...</option>
          <option v-for="db in databases" :key="`${db.path}_1`" :value="db.path">
            {{ db.name }}
          </option>
        </select>
      </div>

      <div class="vs-indicator">VS</div>

      <div class="db-selector">
        <label>Database 2 (Target):</label>
        <select v-model="database2" :disabled="databases.length === 0 || isComparing">
          <option value="">Select second database...</option>
          <option v-for="db in databases" :key="`${db.path}_2`" :value="db.path">
            {{ db.name }}
          </option>
        </select>
      </div>
    </div>

    <!-- Comparison Options -->
    <div class="comparison-options" v-if="database1 && database2 && !isComparing">
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
        <label>Chunk size for large tables:</label>
        <select v-model="options.chunkSize">
          <option :value="5000">5,000 rows (slower, less memory)</option>
          <option :value="10000">10,000 rows (balanced)</option>
          <option :value="25000">25,000 rows (faster, more memory)</option>
          <option :value="50000">50,000 rows (very fast, high memory)</option>
        </select>
        <small>Tables with &gt;50K rows will use chunked processing</small>
      </div>
    </div>

    <!-- Enhanced Table Selection -->
    <div v-if="database1 && database2 && commonTables.length > 0 && !isComparing" class="tables-selection">
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
          <div v-for="table in commonTablesWithInfo" :key="table.name" class="table-grid-item">
            <label class="table-checkbox">
              <input type="checkbox" v-model="selectedTables" :value="table.name" />
              <span class="table-name">
                {{ table.name }}
                <small class="row-info">({{ table.rowCount?.toLocaleString() || '?' }} rows)</small>
              </span>
            </label>
            <select 
              v-if="selectedTables.includes(table.name)" 
              v-model="keyColumns[table.name]" 
              class="per-table-key-select"
            >
              <option value="">Select key for {{ table.name }}</option>
              <option v-for="col in tableColumns[table.name]" :key="col" :value="col">
                {{ col }}
              </option>
            </select>
          </div>
        </div>
      </div>
      
      <div class="key-note" v-if="selectedTables.length > 0">
        <small>⚠️ Select a unique key column (e.g., 'id') for each table. Required for accurate comparison.</small>
      </div>
    </div>

    <div class="comparison-actions">
      <button 
        @click="compareTables"
        :disabled="!canCompare"
        class="compare-btn"
      >
        {{ isComparing ? '⏳ Comparing...' : '🔍 Compare Data' }}
      </button>
      
      <button 
        v-if="hasResults && !isComparing"
        @click="exportAllDifferences"
        class="export-btn"
      >
        📥 Export Report
      </button>

      <button 
        v-if="hasResults && !isComparing"
        @click="generateDataPatch"
        class="patch-btn"
        :disabled="isGeneratingPatch"
      >
        {{ isGeneratingPatch ? '⏳ Generating...' : '🔧 Generate Patch' }}
      </button>
    </div>

    <!-- Toast Notification -->
    <div v-if="toast.show" class="toast" :class="toast.type">
      <div class="toast-icon">{{ toast.type === 'success' ? '✅' : toast.type === 'info' ? 'ℹ️' : '❌' }}</div>
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
    <div v-if="hasResults && !isComparing" class="comparison-results">
      <div class="results-header">
        <h4>📊 Data Comparison Results</h4>
        <div class="results-controls">
          <div class="filter-controls">
            <label>Filter:</label>
            <select v-model="currentFilter">
              <option value="all">All Tables</option>
              <option value="identical">✅ Unchanged Only</option>
              <option value="different">⚠️ With Differences</option>
              <option value="missing">🔴 Removed Rows</option>
              <option value="extra">🟢 Added Rows</option>
            </select>
          </div>
          <div class="view-mode-controls">
            <label>View:</label>
            <button 
              @click="viewMode = 'sideBySide'" 
              :class="{ active: viewMode === 'sideBySide' }"
              class="view-btn"
            >
              Side by Side
            </button>
            <button 
              @click="viewMode = 'single'" 
              :class="{ active: viewMode === 'single' }"
              class="view-btn"
            >
              Single View
            </button>
          </div>
        </div>
      </div>

      <div class="results-summary">
        <h4>Summary ({{ currentFilterDisplay }})</h4>
        <div class="summary-stats">
          <div class="stat-item added" v-if="filteredTotalExtra > 0">
            <span class="count">{{ filteredTotalExtra.toLocaleString() }}</span>
            <span class="label">Added to Target</span>
          </div>
          <div class="stat-item removed" v-if="filteredTotalMissing > 0">
            <span class="count">{{ filteredTotalMissing.toLocaleString() }}</span>
            <span class="label">Removed from Target</span>
          </div>
          <div class="stat-item modified" v-if="filteredTotalDifferent > 0">
            <span class="count">{{ filteredTotalDifferent.toLocaleString() }}</span>
            <span class="label">Modified Rows</span>
          </div>
          <div class="stat-item unchanged" v-if="filteredTotalIdentical > 0">
            <span class="count">{{ filteredTotalIdentical.toLocaleString() }}</span>
            <span class="label">Unchanged Rows</span>
          </div>
        </div>
      </div>

      <div v-if="filteredIdenticalTables.length === 0 && filteredDifferentTables.length === 0 && filteredMissingTables.length === 0 && filteredExtraTables.length === 0" class="no-results">
        No results matching the current filter.
      </div>

      <!-- Added Rows Section -->
      <div v-if="showExtraSection" class="status-section">
        <div class="status-section-header added">
          <span class="status-icon">+</span>
          <h4>Added Rows ({{ filteredTotalExtra }})</h4>
          <p>Rows that exist only in the target database</p>
        </div>

        <div v-for="result in filteredExtraTables" :key="'extra_' + result.tableName" class="table-card">
          <div
            class="table-header-card added"
            @click="toggleCard('extra', result.tableName)"
          >
            <div class="table-info">
              <span class="table-name">{{ result.tableName }}</span>
              <span class="table-status-badge added">{{ result.summary.extraInTarget }} added rows</span>
            </div>
            <div class="header-controls">
              <button
                @click.stop="generateSingleTablePatch(result.tableName)"
                class="mini-patch-btn"
                title="Generate patch for this table only"
              >
                📄 Patch
              </button>

              <span class="toggle-icon">
                {{ expandedCards.has(`extra_${result.tableName}`) ? '▼' : '▶' }}
              </span>
            </div>
          </div>
          <div v-if="expandedCards.has(`extra_${result.tableName}`)" class="table-details">
            <div class="table-view-container">
              <!-- Pagination info and controls -->
              <div v-if="result.comparison.extraInTarget.length > ROWS_PER_PAGE" class="pagination-info">
                <span class="pagination-text">
                  Showing {{ ((getCurrentPage(`extra_${result.tableName}`) - 1) * ROWS_PER_PAGE) + 1 }}
                  to {{ Math.min(getCurrentPage(`extra_${result.tableName}`) * ROWS_PER_PAGE, result.comparison.extraInTarget.length) }}
                  of {{ result.comparison.extraInTarget.length.toLocaleString() }} rows
                </span>
                <div class="pagination-controls">
                  <button
                    @click="setPage(`extra_${result.tableName}`, getCurrentPage(`extra_${result.tableName}`) - 1)"
                    :disabled="getCurrentPage(`extra_${result.tableName}`) === 1"
                    class="pagination-btn"
                  >
                    ← Previous
                  </button>
                  <span class="page-indicator">
                    Page {{ getCurrentPage(`extra_${result.tableName}`) }} of {{ getTotalPages(result.comparison.extraInTarget.length) }}
                  </span>
                  <button
                    @click="setPage(`extra_${result.tableName}`, getCurrentPage(`extra_${result.tableName}`) + 1)"
                    :disabled="getCurrentPage(`extra_${result.tableName}`) >= getTotalPages(result.comparison.extraInTarget.length)"
                    class="pagination-btn"
                  >
                    Next →
                  </button>
                </div>
              </div>

              <div class="scrollable-table-container">
                <table class="data-table">
                  <thead>
                    <tr>
                      <th v-for="column in result.comparison.commonColumns" :key="'extra_' + column">
                        {{ column }}
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(row, index) in getPaginatedRows(result.comparison.extraInTarget, `extra_${result.tableName}`)" :key="'extra_row_' + index" class="row-extra">
                      <td v-for="column in result.comparison.commonColumns" :key="'extra_' + column">
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

      <!-- Removed Rows Section -->
      <div v-if="showMissingSection" class="status-section">
        <div class="status-section-header removed">
          <span class="status-icon">−</span>
          <h4>Removed Rows ({{ filteredTotalMissing }})</h4>
          <p>Rows that exist only in the source database</p>
        </div>

        <div v-for="result in filteredMissingTables" :key="'missing_' + result.tableName" class="table-card">
          <div
            class="table-header-card removed"
            @click="toggleCard('missing', result.tableName)"
          >
            <div class="table-info">
              <span class="table-name">{{ result.tableName }}</span>
              <span class="table-status-badge removed">{{ result.summary.missingInTarget }} removed rows</span>
            </div>
            <div class="header-controls">
              <button
                @click.stop="generateSingleTablePatch(result.tableName)"
                class="mini-patch-btn"
                title="Generate patch for this table only"
              >
                📄 Patch
              </button>

              <span class="toggle-icon">
                {{ expandedCards.has(`missing_${result.tableName}`) ? '▼' : '▶' }}
              </span>
            </div>
          </div>
          <div v-if="expandedCards.has(`missing_${result.tableName}`)" class="table-details">
            <div class="table-view-container">
              <!-- Pagination info and controls -->
              <div v-if="result.comparison.missingInTarget.length > ROWS_PER_PAGE" class="pagination-info">
                <span class="pagination-text">
                  Showing {{ ((getCurrentPage(`missing_${result.tableName}`) - 1) * ROWS_PER_PAGE) + 1 }}
                  to {{ Math.min(getCurrentPage(`missing_${result.tableName}`) * ROWS_PER_PAGE, result.comparison.missingInTarget.length) }}
                  of {{ result.comparison.missingInTarget.length.toLocaleString() }} rows
                </span>
                <div class="pagination-controls">
                  <button
                    @click="setPage(`missing_${result.tableName}`, getCurrentPage(`missing_${result.tableName}`) - 1)"
                    :disabled="getCurrentPage(`missing_${result.tableName}`) === 1"
                    class="pagination-btn"
                  >
                    ← Previous
                  </button>
                  <span class="page-indicator">
                    Page {{ getCurrentPage(`missing_${result.tableName}`) }} of {{ getTotalPages(result.comparison.missingInTarget.length) }}
                  </span>
                  <button
                    @click="setPage(`missing_${result.tableName}`, getCurrentPage(`missing_${result.tableName}`) + 1)"
                    :disabled="getCurrentPage(`missing_${result.tableName}`) >= getTotalPages(result.comparison.missingInTarget.length)"
                    class="pagination-btn"
                  >
                    Next →
                  </button>
                </div>
              </div>

              <div class="scrollable-table-container">
                <table class="data-table">
                  <thead>
                    <tr>
                      <th v-for="column in result.comparison.commonColumns" :key="'miss_' + column">
                        {{ column }}
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(row, index) in getPaginatedRows(result.comparison.missingInTarget, `missing_${result.tableName}`)" :key="'miss_row_' + index" class="row-missing">
                      <td v-for="column in result.comparison.commonColumns" :key="'miss_' + column">
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

      <!-- Modified Rows Section -->
      <div v-if="showDifferentSection" class="status-section">
        <div class="status-section-header modified">
          <span class="status-icon">≠</span>
          <h4>Tables with Modified Rows ({{ filteredTotalDifferent }})</h4>
          <p>Rows exist in both but have different values</p>
        </div>

        <div v-for="result in filteredDifferentTables" :key="'different_' + result.tableName" class="table-card">
          <div
            class="table-header-card modified"
            @click="toggleCard('different', result.tableName)"
          >
            <div class="table-info">
              <span class="table-name">{{ result.tableName }}</span>
              <span class="table-status-badge modified">{{ result.summary.differentRows }} modified rows</span>
            </div>
            <div class="header-controls">
              <!-- Per-table view toggle -->
              <div class="per-table-view-toggle" @click.stop>
                <button
                  @click="setTableViewMode(result.tableName, 'sideBySide')"
                  :class="{ active: getTableViewMode(result.tableName) === 'sideBySide' }"
                  class="mini-view-btn"
                  title="Side by Side View"
                >
                  ⚏
                </button>
                <button
                  @click="setTableViewMode(result.tableName, 'single')"
                  :class="{ active: getTableViewMode(result.tableName) === 'single' }"
                  class="mini-view-btn"
                  title="Single Row View"
                >
                  ☰
                </button>
              </div>

              <!-- Per-table patch button -->
              <button
                @click.stop="generateSingleTablePatch(result.tableName)"
                class="mini-patch-btn"
                title="Generate patch for this table only"
              >
                📄 Patch
              </button>

              <span class="toggle-icon">
                {{ expandedCards.has(`different_${result.tableName}`) ? '▼' : '▶' }}
              </span>
            </div>
          </div>

          <div v-if="expandedCards.has(`different_${result.tableName}`)" class="table-details">
            <div class="table-view-container">
              <!-- Pagination info and controls for Modified rows -->
              <div v-if="result.comparison.differentRows.length > ROWS_PER_PAGE && getTableViewMode(result.tableName) === 'sideBySide'" class="pagination-info">
                <span class="pagination-text">
                  Showing {{ ((getCurrentPage(`different_${result.tableName}`) - 1) * ROWS_PER_PAGE) + 1 }}
                  to {{ Math.min(getCurrentPage(`different_${result.tableName}`) * ROWS_PER_PAGE, result.comparison.differentRows.length) }}
                  of {{ result.comparison.differentRows.length.toLocaleString() }} rows
                </span>
                <div class="pagination-controls">
                  <button
                    @click="setPage(`different_${result.tableName}`, getCurrentPage(`different_${result.tableName}`) - 1)"
                    :disabled="getCurrentPage(`different_${result.tableName}`) === 1"
                    class="pagination-btn"
                  >
                    ← Previous
                  </button>
                  <span class="page-indicator">
                    Page {{ getCurrentPage(`different_${result.tableName}`) }} of {{ getTotalPages(result.comparison.differentRows.length) }}
                  </span>
                  <button
                    @click="setPage(`different_${result.tableName}`, getCurrentPage(`different_${result.tableName}`) + 1)"
                    :disabled="getCurrentPage(`different_${result.tableName}`) >= getTotalPages(result.comparison.differentRows.length)"
                    class="pagination-btn"
                  >
                    Next →
                  </button>
                </div>
              </div>

              <div v-if="getTableViewMode(result.tableName) === 'sideBySide'" class="side-by-side-view">
                <!-- Source Table -->
                <div class="source-section">
                  <div class="section-title source-title">
                    <h6>{{ getDatabaseName(database1) }} (Source)</h6>
                  </div>
                  <div class="scrollable-table-container">
                    <table class="data-table">
                      <thead>
                        <tr>
                          <th v-for="column in result.comparison.commonColumns" :key="'src_' + column">
                            {{ column }}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr v-for="(diff, index) in getPaginatedRows(result.comparison.differentRows, `different_${result.tableName}`)" :key="'src_row_' + index" class="row-different">
                          <td v-for="column in result.comparison.commonColumns" :key="'src_' + column"
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
                          <th v-for="column in result.comparison.commonColumns" :key="'tgt_' + column">
                            {{ column }}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr v-for="(diff, index) in getPaginatedRows(result.comparison.differentRows, `different_${result.tableName}`)" :key="'tgt_row_' + index" class="row-different">
                          <td v-for="column in result.comparison.commonColumns" :key="'tgt_' + column"
                              :class="{ 'cell-different': diff.differentColumns.includes(column) }">
                            {{ formatCellValue(diff.targetRow[column]) }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>

              <div v-else class="single-view">
                <!-- Pagination for single view -->
                <div v-if="result.comparison.differentRows.length > ROWS_PER_PAGE" class="pagination-info">
                  <span class="pagination-text">
                    Showing {{ ((getCurrentPage(`different_${result.tableName}`) - 1) * ROWS_PER_PAGE) + 1 }}
                    to {{ Math.min(getCurrentPage(`different_${result.tableName}`) * ROWS_PER_PAGE, result.comparison.differentRows.length) }}
                    of {{ result.comparison.differentRows.length.toLocaleString() }} rows
                  </span>
                  <div class="pagination-controls">
                    <button
                      @click="setPage(`different_${result.tableName}`, getCurrentPage(`different_${result.tableName}`) - 1)"
                      :disabled="getCurrentPage(`different_${result.tableName}`) === 1"
                      class="pagination-btn"
                    >
                      ← Previous
                    </button>
                    <span class="page-indicator">
                      Page {{ getCurrentPage(`different_${result.tableName}`) }} of {{ getTotalPages(result.comparison.differentRows.length) }}
                    </span>
                    <button
                      @click="setPage(`different_${result.tableName}`, getCurrentPage(`different_${result.tableName}`) + 1)"
                      :disabled="getCurrentPage(`different_${result.tableName}`) >= getTotalPages(result.comparison.differentRows.length)"
                      class="pagination-btn"
                    >
                      Next →
                    </button>
                  </div>
                </div>

                <div v-for="(diff, diffIndex) in getPaginatedRows(result.comparison.differentRows, `different_${result.tableName}`)" :key="'single_diff_' + diffIndex" class="diff-comparison">
                  <div class="diff-key">
                    <strong>{{ result.keyColumn }}:</strong> {{ formatCellValue(diff.sourceRow[result.keyColumn]) }}
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

      <!-- Unchanged Tables Section -->
      <div v-if="showIdenticalSection" class="status-section">
        <div class="status-section-header unchanged">
          <span class="status-icon">✓</span>
          <h4>Unchanged Tables ({{ filteredTotalIdentical }})</h4>
          <p>Tables with no differences</p>
        </div>

        <div v-for="result in filteredIdenticalTables" :key="'identical_' + result.tableName" class="table-card">
          <div
            class="table-header-card unchanged"
            @click="toggleCard('identical', result.tableName)"
          >
            <div class="table-info">
              <span class="table-name">{{ result.tableName }}</span>
              <span class="table-status-badge unchanged">{{ result.summary.identicalRows }} unchanged rows</span>
            </div>
            <span class="toggle-icon">
              {{ expandedCards.has(`identical_${result.tableName}`) ? '▼' : '▶' }}
            </span>
          </div>
          <div v-if="expandedCards.has(`identical_${result.tableName}`)" class="table-details">
            <div class="table-view-container">
              <!-- Pagination info and controls -->
              <div v-if="result.comparison.identicalRows.length > ROWS_PER_PAGE" class="pagination-info">
                <span class="pagination-text">
                  Showing {{ ((getCurrentPage(`identical_${result.tableName}`) - 1) * ROWS_PER_PAGE) + 1 }}
                  to {{ Math.min(getCurrentPage(`identical_${result.tableName}`) * ROWS_PER_PAGE, result.comparison.identicalRows.length) }}
                  of {{ result.comparison.identicalRows.length.toLocaleString() }} rows
                </span>
                <div class="pagination-controls">
                  <button
                    @click="setPage(`identical_${result.tableName}`, getCurrentPage(`identical_${result.tableName}`) - 1)"
                    :disabled="getCurrentPage(`identical_${result.tableName}`) === 1"
                    class="pagination-btn"
                  >
                    ← Previous
                  </button>
                  <span class="page-indicator">
                    Page {{ getCurrentPage(`identical_${result.tableName}`) }} of {{ getTotalPages(result.comparison.identicalRows.length) }}
                  </span>
                  <button
                    @click="setPage(`identical_${result.tableName}`, getCurrentPage(`identical_${result.tableName}`) + 1)"
                    :disabled="getCurrentPage(`identical_${result.tableName}`) >= getTotalPages(result.comparison.identicalRows.length)"
                    class="pagination-btn"
                  >
                    Next →
                  </button>
                </div>
              </div>

              <div class="scrollable-table-container">
                <table class="data-table">
                  <thead>
                    <tr>
                      <th v-for="column in result.comparison.commonColumns" :key="'identical_' + column">
                        {{ column }}
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(row, index) in getPaginatedRows(result.comparison.identicalRows, `identical_${result.tableName}`)" :key="'identical_row_' + index">
                      <td v-for="column in result.comparison.commonColumns" :key="'identical_' + column">
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
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
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
const tableRowCounts = ref<Record<string, number>>({});
const isComparing = ref(false);
const error = ref('');
const tableComparisons = ref<TableComparisonResult[]>([]);
const expandedCards = ref(new Set<string>());
const viewMode = ref<'sideBySide' | 'single'>('sideBySide');
const perTableViewMode = ref<Record<string, 'sideBySide' | 'single'>>({}); // Per-table view mode
const options = ref({
  ignoreCase: false,
  ignoreWhitespace: false,
  chunkSize: 10000
});

const currentFilter = ref<'all' | 'identical' | 'different' | 'missing' | 'extra'>('all');
const isGeneratingPatch = ref(false);
const toast = ref({
  show: false,
  type: 'success' as 'success' | 'error' | 'info',
  title: '',
  message: ''
});

const comparisonProgress = ref({
  message: 'Initializing...',
  percentage: 0,
  currentTable: 0,
  totalTables: 0,
  currentRows: 0,
  totalRows: 0,
  speed: 0
});

// Pagination for large datasets
const ROWS_PER_PAGE = 1000; // Show 1000 rows per page max
const pagination = ref<Record<string, number>>({}); // Track current page for each expanded section

// Helper to get current page for a section
const getCurrentPage = (key: string): number => {
  return pagination.value[key] || 1;
};

// Helper to set page for a section
const setPage = (key: string, page: number) => {
  pagination.value[key] = page;
};

// Helper to get paginated rows
const getPaginatedRows = (rows: any[], key: string): any[] => {
  const currentPage = getCurrentPage(key);
  const startIndex = (currentPage - 1) * ROWS_PER_PAGE;
  const endIndex = startIndex + ROWS_PER_PAGE;
  return rows.slice(startIndex, endIndex);
};

// Helper to get total pages
const getTotalPages = (totalRows: number): number => {
  return Math.ceil(totalRows / ROWS_PER_PAGE);
};

// Toast helper
const showToast = (type: 'success' | 'error' | 'info', title: string, message: string) => {
  toast.value = { show: true, type, title, message };
  setTimeout(() => {
    toast.value.show = false;
  }, type === 'info' ? 8000 : 5000);
};

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
    all: 'All Tables',
    identical: 'Unchanged Only',
    different: 'With Differences',
    missing: 'Removed Rows',
    extra: 'Added Rows'
  };
  return map[currentFilter.value] || 'All Tables';
});

const filteredIdenticalTables = computed(() => {
  if (currentFilter.value !== 'all' && currentFilter.value !== 'identical') return [];
  return tableComparisons.value.filter(r => 
    r.summary.identicalRows > 0 && 
    r.summary.differentRows === 0 && 
    r.summary.missingInTarget === 0 && 
    r.summary.extraInTarget === 0
  );
});

const filteredDifferentTables = computed(() => {
  if (currentFilter.value !== 'all' && currentFilter.value !== 'different') return [];
  return tableComparisons.value.filter(r => r.summary.differentRows > 0);
});

const filteredMissingTables = computed(() => {
  if (currentFilter.value !== 'all' && currentFilter.value !== 'missing') return [];
  return tableComparisons.value.filter(r => r.summary.missingInTarget > 0);
});

const filteredExtraTables = computed(() => {
  if (currentFilter.value !== 'all' && currentFilter.value !== 'extra') return [];
  return tableComparisons.value.filter(r => r.summary.extraInTarget > 0);
});

const filteredTotalIdentical = computed(() => {
  return tableComparisons.value.reduce((sum, r) => sum + r.summary.identicalRows, 0);
});

const filteredTotalDifferent = computed(() => {
  return tableComparisons.value.reduce((sum, r) => sum + r.summary.differentRows, 0);
});

const filteredTotalMissing = computed(() => {
  return tableComparisons.value.reduce((sum, r) => sum + r.summary.missingInTarget, 0);
});

const filteredTotalExtra = computed(() => {
  return tableComparisons.value.reduce((sum, r) => sum + r.summary.extraInTarget, 0);
});

const showIdenticalSection = computed(() => filteredIdenticalTables.value.length > 0);
const showDifferentSection = computed(() => filteredDifferentTables.value.length > 0);
const showMissingSection = computed(() => filteredMissingTables.value.length > 0);
const showExtraSection = computed(() => filteredExtraTables.value.length > 0);

const commonTablesWithInfo = computed(() => {
  return commonTables.value.map(name => ({
    name,
    rowCount: tableRowCounts.value[name]
  }));
});

// Watch for database changes
watch([database1, database2], async () => {
  await loadCommonTables();
});

// Methods
const loadCommonTables = async () => {
  if (!database1.value || !database2.value) {
    commonTables.value = [];
    selectedTables.value = [];
    return;
  }

  try {
    const [tables1, tables2] = await Promise.all([
      invoke<any[]>('get_database_tables', { dbPath: database1.value }),
      invoke<any[]>('get_database_tables', { dbPath: database2.value })
    ]);

    const tableNames1 = new Set(tables1.map((t: any) => t.name));
    const tableNames2 = new Set(tables2.map((t: any) => t.name));
    
    commonTables.value = [...tableNames1].filter(name => tableNames2.has(name));
    
    // Store row counts
    tableRowCounts.value = {};
    tables1.forEach((t: any) => {
      if (commonTables.value.includes(t.name)) {
        tableRowCounts.value[t.name] = t.row_count || 0;
      }
    });
    
    // Load columns for common tables
    for (const tableName of commonTables.value) {
      const tableData = await invoke<TableData>('get_table_data', {
        dbPath: database1.value,
        tableName: tableName,
        limit: 1,
        offset: 0
      });
      tableColumns.value[tableName] = tableData.columns;
      
      // Auto-select common key columns
      const cols = tableData.columns;
      if (cols.includes('id')) {
        keyColumns.value[tableName] = 'id';
      } else if (cols.includes('ID')) {
        keyColumns.value[tableName] = 'ID';
      } else if (cols.includes('uuid')) {
        keyColumns.value[tableName] = 'uuid';
      } else if (cols.includes('UUID')) {
        keyColumns.value[tableName] = 'UUID';
      } else if (cols[0]) {
        keyColumns.value[tableName] = cols[0];
      }
    }
    
    showToast('success', 'Tables Loaded', `Found ${commonTables.value.length} common tables`);
  } catch (err) {
    error.value = `Failed to load tables: ${err}`;
    showToast('error', 'Error', error.value);
  }
};

const selectAllTables = () => {
  selectedTables.value = [...commonTables.value];
};

const deselectAllTables = () => {
  selectedTables.value = [];
};

const compareTables = async () => {
  if (!canCompare.value) return;
  
  isComparing.value = true;
  error.value = '';
  tableComparisons.value = [];
  
  comparisonProgress.value = {
    message: 'Starting comparison...',
    percentage: 0,
    currentTable: 0,
    totalTables: selectedTables.value.length,
    currentRows: 0,
    totalRows: 0,
    speed: 0
  };
  
  const startTime = Date.now();
  
  try {
    for (let i = 0; i < selectedTables.value.length; i++) {
      const tableName = selectedTables.value[i];
      const tableKey = keyColumns.value[tableName];
      
      comparisonProgress.value.currentTable = i + 1;
      comparisonProgress.value.message = `Comparing table ${i + 1}/${selectedTables.value.length}: ${tableName}`;
      
      const result = await compareSingleTable(tableName, tableKey);
      tableComparisons.value.push(result);
      
      comparisonProgress.value.percentage = Math.round(((i + 1) / selectedTables.value.length) * 100);
      
      // Allow UI to update
      await new Promise(resolve => setTimeout(resolve, 0));
    }
    
    const elapsed = (Date.now() - startTime) / 1000;
    showToast('success', 'Comparison Complete', 
      `Compared ${selectedTables.value.length} tables in ${elapsed.toFixed(1)}s`);
    
  } catch (err) {
    error.value = `Comparison failed: ${err}`;
    showToast('error', 'Comparison Failed', String(err));
    console.error('Comparison error:', err);
  } finally {
    isComparing.value = false;
  }
};

const compareSingleTable = async (tableName: string, tableKey: string): Promise<TableComparisonResult> => {
  // Get row counts
  const [sourceCount, targetCount] = await Promise.all([
    invoke<TableData>('get_table_data', {
      dbPath: database1.value,
      tableName,
      limit: 1,
      offset: 0
    }),
    invoke<TableData>('get_table_data', {
      dbPath: database2.value,
      tableName,
      limit: 1,
      offset: 0
    })
  ]);

  const sourceTotal = sourceCount.total_count;
  const targetTotal = targetCount.total_count;
  const maxRows = Math.max(sourceTotal, targetTotal);

  let comparison: ComparisonResult;

  // Use chunking for large tables (>50K rows)
  if (maxRows > 50000) {
    console.log(`📊 Using chunked comparison for ${tableName} (${maxRows.toLocaleString()} rows)`);
    showToast('info', 'Large Table', 
      `Table "${tableName}" has ${maxRows.toLocaleString()} rows. Using optimized chunked processing.`);
    
    comparison = await compareTableRowsChunked(
      tableName,
      database1.value,
      database2.value,
      tableKey,
      options.value.chunkSize
    );
  } else {
    console.log(`📊 Using regular comparison for ${tableName} (${maxRows.toLocaleString()} rows)`);
    
    // Regular comparison for smaller tables
    const [source, target] = await Promise.all([
      invoke<TableData>('get_table_data', {
        dbPath: database1.value,
        tableName,
        limit: null,
        offset: null
      }),
      invoke<TableData>('get_table_data', {
        dbPath: database2.value,
        tableName,
        limit: null,
        offset: null
      })
    ]);

    const sourceData = source.rows || [];
    const targetData = target.rows || [];
    
    comparison = performDataComparison(
      sourceData, 
      targetData, 
      source.columns || [], 
      target.columns || [], 
      tableKey, 
      tableName
    );
  }

  return {
    tableName,
    keyColumn: tableKey,
    sourceData: [],
    targetData: [],
    comparison,
    summary: {
      identicalRows: comparison.identicalRows.length,
      differentRows: comparison.differentRows.length,
      missingInTarget: comparison.missingInTarget.length,
      extraInTarget: comparison.extraInTarget.length
    }
  };
};

/**
 * FIXED: Proper key-based chunked comparison
 * This loads ALL data from both databases in chunks and compares by keys
 * Not by offset position (which was the bug in the old version)
 */
const compareTableRowsChunked = async (
  tableName: string,
  db1Path: string,
  db2Path: string,
  keyColumn: string,
  chunkSize: number = 10000
): Promise<ComparisonResult> => {
  console.log(`🚀 Starting chunked comparison for ${tableName}`);
  
  // Get total row counts
  const [sourceCountData, targetCountData] = await Promise.all([
    invoke<TableData>('get_table_data', { dbPath: db1Path, tableName, limit: 1, offset: 0 }),
    invoke<TableData>('get_table_data', { dbPath: db2Path, tableName, limit: 1, offset: 0 })
  ]);
  
  const sourceTotal = sourceCountData.total_count;
  const targetTotal = targetCountData.total_count;
  
  console.log(`📊 Source rows: ${sourceTotal}, Target rows: ${targetTotal}`);
  
  comparisonProgress.value.totalRows = sourceTotal + targetTotal;
  comparisonProgress.value.currentRows = 0;
  
  // Load ALL source data in chunks
  const sourceMap = new Map<string, DataRow>();
  let sourceColumns: string[] = [];
  
  let offset = 0;
  const sourceStartTime = Date.now();
  
  while (offset < sourceTotal) {
    const chunk = await invoke<TableData>('get_table_data', {
      dbPath: db1Path,
      tableName,
      limit: chunkSize,
      offset
    });
    
    if (offset === 0) {
      sourceColumns = chunk.columns;
    }
    
    // Convert rows to objects and store in map
    const rowObjects = convertToObject(chunk.rows, chunk.columns);
    for (const row of rowObjects) {
      const key = normalizeValue(row[keyColumn]);
      if (key && key !== 'NULL') {
        sourceMap.set(key, row);
      }
    }
    
    offset += chunkSize;
    comparisonProgress.value.currentRows = offset;
    
    const elapsed = (Date.now() - sourceStartTime) / 1000;
    comparisonProgress.value.speed = Math.round(offset / elapsed);
    
    console.log(`   Source progress: ${offset}/${sourceTotal} (${Math.round(offset/sourceTotal*100)}%)`);
    
    // Allow UI to update
    await new Promise(resolve => setTimeout(resolve, 0));
  }
  
  console.log(`✅ Loaded ${sourceMap.size} source rows into memory`);
  
  // Load ALL target data in chunks
  const targetMap = new Map<string, DataRow>();
  let targetColumns: string[] = [];
  
  offset = 0;
  const targetStartTime = Date.now();
  
  while (offset < targetTotal) {
    const chunk = await invoke<TableData>('get_table_data', {
      dbPath: db2Path,
      tableName,
      limit: chunkSize,
      offset
    });
    
    if (offset === 0) {
      targetColumns = chunk.columns;
    }
    
    // Convert rows to objects and store in map
    const rowObjects = convertToObject(chunk.rows, chunk.columns);
    for (const row of rowObjects) {
      const key = normalizeValue(row[keyColumn]);
      if (key && key !== 'NULL') {
        targetMap.set(key, row);
      }
    }
    
    offset += chunkSize;
    comparisonProgress.value.currentRows = sourceTotal + offset;
    
    const elapsed = (Date.now() - targetStartTime) / 1000;
    comparisonProgress.value.speed = Math.round(offset / elapsed);
    
    console.log(`   Target progress: ${offset}/${targetTotal} (${Math.round(offset/targetTotal*100)}%)`);
    
    // Allow UI to update
    await new Promise(resolve => setTimeout(resolve, 0));
  }
  
  console.log(`✅ Loaded ${targetMap.size} target rows into memory`);
  console.log(`🔍 Starting comparison of ${sourceMap.size} vs ${targetMap.size} rows`);
  
  // Now compare the two maps
  const commonColumns = sourceColumns.filter(col => targetColumns.includes(col));
  const sourceOnlyColumns = sourceColumns.filter(col => !targetColumns.includes(col));
  const targetOnlyColumns = targetColumns.filter(col => !sourceColumns.includes(col));
  
  const identical: DataRow[] = [];
  const different: DifferentRow[] = [];
  const missingInTarget: DataRow[] = [];
  const extraInTarget: DataRow[] = [];
  
  // Compare rows by key
  for (const [key, sourceRow] of sourceMap) {
    const targetRow = targetMap.get(key);
    
    if (targetRow) {
      const diffColumns = findDifferentColumns(sourceRow, targetRow, commonColumns, keyColumn);
      if (diffColumns.length > 0) {
        different.push({ sourceRow, targetRow, differentColumns: diffColumns });
      } else {
        identical.push(sourceRow);
      }
      targetMap.delete(key); // Remove from target map
    } else {
      missingInTarget.push(sourceRow);
    }
  }
  
  // Remaining rows in targetMap are extra
  extraInTarget.push(...targetMap.values());
  
  console.log(`✅ Comparison complete:`);
  console.log(`   Identical: ${identical.length}`);
  console.log(`   Different: ${different.length}`);
  console.log(`   Missing: ${missingInTarget.length}`);
  console.log(`   Extra: ${extraInTarget.length}`);
  
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
  if (expandedCards.value.has(key)) {
    expandedCards.value.delete(key);
  } else {
    expandedCards.value.add(key);
  }
};

const formatCellValue = (value: any): string => {
  if (value === null || value === undefined) return 'NULL';
  if (typeof value === 'object') return JSON.stringify(value);
  if (typeof value === 'string' && value.length > 100) {
    return value.substring(0, 100) + '...';
  }
  return String(value);
};

const getDatabaseName = (path: string): string => {
  const db = props.databases.find(d => d.path === path);
  return db ? db.name : path.split('/').pop() || path;
};

// Helper function to get view mode for a specific table
const getTableViewMode = (tableName: string): 'sideBySide' | 'single' => {
  return perTableViewMode.value[tableName] || 'sideBySide';
};

// Helper function to set view mode for a specific table
const setTableViewMode = (tableName: string, mode: 'sideBySide' | 'single') => {
  perTableViewMode.value[tableName] = mode;
};

const exportAllDifferences = async () => {
  try {
    const exportData = {
      timestamp: new Date().toISOString(),
      databases: {
        source: database1.value,
        target: database2.value
      },
      summary: {
        totalTables: tableComparisons.value.length,
        totalIdentical: filteredTotalIdentical.value,
        totalDifferent: filteredTotalDifferent.value,
        totalMissing: filteredTotalMissing.value,
        totalExtra: filteredTotalExtra.value
      },
      tables: tableComparisons.value.map(result => ({
        tableName: result.tableName,
        summary: result.summary,
        differences: {
          differentRows: result.comparison.differentRows,
          missingRows: result.comparison.missingInTarget,
          extraRows: result.comparison.extraInTarget
        }
      }))
    };
    
    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `database-comparison-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    
    showToast('success', 'Exported', 'Comparison report exported successfully');
  } catch (err) {
    showToast('error', 'Export Failed', String(err));
  }
};

const generateDataPatch = async () => {
  isGeneratingPatch.value = true;
  await nextTick(); // Wait for Vue to update
  await new Promise(resolve => setTimeout(resolve, 50)); // Allow browser to paint
  
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
      tableComparisons: comparisonData  // camelCase - Tauri converts to snake_case
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
  
  isGeneratingPatch.value = true;
  await nextTick(); // Wait for Vue to update
  await new Promise(resolve => setTimeout(resolve, 50)); // Allow browser to paint
  
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
      tableComparisons: comparisonData  // camelCase - Tauri converts to snake_case
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
  } finally {
    isGeneratingPatch.value = false;
  }
};
</script>

<style scoped>
/* Same styles as the original file - keeping all existing CSS */
/* I'm preserving all your existing styles to maintain the same look and feel */

/* Component uses global CSS variables from theme.css */
.data-comparison {
  /* Map local variable names to global theme variables */
  --bg-surface: var(--bg-tertiary);
  --bg-muted: var(--bg-tertiary);
  --table-row-hover: var(--bg-hover);
  --border-light: var(--border-secondary);
}
.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(4px);
}

.loading-content {
  /* background: white; */
  background: var(--bg-card);
  padding: 40px;
  border-radius: 16px;
  max-width: 600px;
  width: 90%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  text-align: center;
}

.spinner-large {
  width: 80px;
  height: 80px;
  border: 6px solid #f3f3f3;
  border-top: 6px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 30px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-content h3 {
  color: var(--text-primary);
  /* color: #2c3e50; */
  margin-bottom: 25px;
  font-size: 1.4em;
}

.progress-bar-large {
  width: 100%;
  height: 30px;
  background: #ecf0f1;
  border-radius: 15px;
  overflow: hidden;
  margin-bottom: 20px;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
}

.progress-fill-large {
  height: 100%;
  background: linear-gradient(90deg, #3498db, #2ecc71);
  transition: width 0.3s ease;
  border-radius: 15px;
}

.progress-stats {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.progress-stats p {
  margin: 0;
  color: var(--text-secondary);
  /* color: #555; */
  font-size: 1em;
}

.progress-stats strong {
  color: var(--text-primary);
  /* color: #2c3e50; */
  font-size: 1.1em;
}

/* Loading Message for Patch Generation */
.loading-message {
  color: var(--text-secondary);
  font-size: 1.1em;
  margin: 15px 0 25px 0;
}

/* Animated Dots for Patch Generation */
.spinner-dots {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 10px;
  margin-top: 20px;
}

.spinner-dots span {
  width: 15px;
  height: 15px;
  background: #3498db;
  border-radius: 50%;
  animation: bounce 1.4s infinite ease-in-out both;
}

.spinner-dots span:nth-child(1) {
  animation-delay: -0.32s;
}

.spinner-dots span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0.6);
    opacity: 0.5;
  }
  40% {
    transform: scale(1);
    opacity: 1;
  }
}

/* Main Container */
.data-comparison {
  padding: 20px;
  /* background: #f5f7fa; */
  background: var(--bg-secondary);
  min-height: 100vh;
  color: var(--text-primary);
}

.comparison-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 30px;
  border-radius: 12px;
  margin-bottom: 30px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
}

.comparison-header h3 {
  margin: 0 0 10px 0;
  font-size: 2em;
}

.comparison-header p {
  margin: 0;
  opacity: 0.95;
  font-size: 1.1em;
}

/* Database Selection */
.database-selection {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 20px;
  align-items: center;
  margin-bottom: 30px;
  /* background: white; */
  background: var(--bg-card);
  padding: 30px;
  border-radius: 12px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
}

.db-selector {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.db-selector label {
  font-weight: 600;
  /* color: #2c3e50; */
  color: var(--text-primary);
  font-size: 1.05em;
}

.db-selector select {
  padding: 12px 15px;
  border: 2px solid var(--border-color);
  /* border: 2px solid #e0e0e0; */
  border-radius: 8px;
  font-size: 1em;
  transition: all 0.3s ease;
  /* background: white; */
  background: var(--bg-card);
}

.db-selector select:hover:not(:disabled) {
  border-color: #3498db;
}

.db-selector select:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

/* .vs-indicator {
  font-size: 1.8em;
  font-weight: bold;
  color: #3498db;
  background: #e3f2fd;
  width: 60px;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
} */

/* Comparison Options */
.comparison-options {
  /* background: white; */
  background: var(--bg-card);
  padding: 25px;
  border-radius: 12px;
  margin-bottom: 30px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
}

.option-group {
  display: flex;
  gap: 30px;
  margin-bottom: 20px;
}

.option-group label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  font-size: 1em;
  /* color: #2c3e50; */
  color: var(--text-primary);
}

.option-group input[type="checkbox"] {
  width: 20px;
  height: 20px;
  cursor: pointer;
}

.limit-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.limit-options label {
  font-weight: 600;
  /* color: #2c3e50; */
  color: var(--text-primary);
  font-size: 0.95em;
}

.limit-options select {
  padding: 10px 12px;
  border: 2px solid var(--border-color);
  /* border: 2px solid #e0e0e0; */
  border-radius: 6px;
  font-size: 0.95em;
  max-width: 400px;
}

.limit-options small {
  color: var(--text-secondary);
  font-size: 0.85em;
}

/* Tables Selection */
.tables-selection {
  /* background: white; */
  background: var(--bg-card);
  padding: 25px;
  border-radius: 12px;
  margin-bottom: 30px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
}

.selection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 15px;
}

.selection-header h4 {
  margin: 0;
  color: var(--text-primary);
  /* color: #2c3e50; */
  font-size: 1.3em;
}

.bulk-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.bulk-btn {
  padding: 10px 18px;
  background: #3498db;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.95em;
  transition: all 0.3s ease;
}

.bulk-btn:hover:not(:disabled) {
  background: #2980b9;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(52, 152, 219, 0.3);
}

.bulk-btn:disabled {
  background: #bdc3c7;
  cursor: not-allowed;
}

.selection-count {
  font-size: 0.9em;
  color: var(--text-secondary);
  font-weight: 500;
}

.table-grid-container {
  max-height: 500px;
  overflow-y: auto;
  border: 2px solid var(--border-color);
  border-radius: 8px;
  padding: 15px;
}

.table-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 15px;
}

.table-grid-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 15px;
  background: var(--bg-surface);
  border-radius: 8px;
  border: 2px solid var(--border-color);
  /* border: 2px solid #e0e0e0; */
  transition: all 0.2s;
}

.table-grid-item:hover {
  border-color: #3498db;
  box-shadow: 0 2px 8px rgba(52, 152, 219, 0.2);
}

.table-checkbox {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.table-checkbox input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.table-name {
  font-weight: 600;
  color: var(--text-primary);
  /* color: #2c3e50; */
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.row-info {
  color: var(--text-secondary);
  font-weight: normal;
  font-size: 0.85em;
}

.per-table-key-select {
  padding: 8px 10px;
  border: 2px solid var(--border-color);
  /* border: 2px solid #e0e0e0; */
  border-radius: 6px;
  font-size: 0.9em;
}

.key-note {
  margin-top: 15px;
  padding: 12px;
  background: #fff3cd;
  border-left: 4px solid #ffc107;
  border-radius: 4px;
}

/* Action Buttons */
.comparison-actions {
  display: flex;
  gap: 15px;
  margin-bottom: 30px;
  flex-wrap: wrap;
}


.compare-btn,
.export-btn,
.patch-btn {
  padding: 15px 30px;
  font-size: 1.1em;
  font-weight: 600;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

/*.compare-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.compare-btn:hover:not(:disabled) {
  transform: translateY(-3px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
} */

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
  background: #bdc3c7;
  cursor: not-allowed;
}

/* .export-btn {
  background: #27ae60;
  color: white;
}

.export-btn:hover {
  background: #229954;
  transform: translateY(-3px);
} */

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

/* .patch-btn {
  background: #f39c12;
  color: white;
}

.patch-btn:hover:not(:disabled) {
  background: #e67e22;
  transform: translateY(-3px);
} */
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

.patch-btn:hover:not(:disabled) {
  background: #520dc2;
}
/* Toast Notification */
.toast {
  position: fixed;
  top: 20px;
  right: 20px;
  /* background: white; */
  background: var(--bg-card);
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
  display: flex;
  gap: 15px;
  align-items: start;
  max-width: 400px;
  z-index: 10001;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    transform: translateX(400px);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.toast.success {
  border-left: 4px solid #27ae60;
}

.toast.error {
  border-left: 4px solid #e74c3c;
}

.toast.info {
  border-left: 4px solid #3498db;
}

.toast-icon {
  font-size: 1.5em;
}

.toast-content {
  flex: 1;
}

.toast-title {
  font-weight: 600;
  margin-bottom: 5px;
  color: var(--text-primary);
  /* color: #2c3e50; */
}

.toast-message {
  font-size: 0.95em;
  color: var(--text-secondary);
  /* color: #555; */
}

.toast-close {
  background: none;
  border: none;
  font-size: 1.5em;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 0;
}

.toast-close:hover {
  color: var(--text-primary);
  /* color: #2c3e50; */
}

/* Error Message */
.error-message {
  background: #ffebee;
  color: #c62828;
  padding: 15px 20px;
  border-radius: 8px;
  margin-bottom: 20px;
  border-left: 4px solid #e74c3c;
  font-weight: 500;
}

/* Results Section */
.comparison-results {
  /* background: white; */
  background: var(--bg-card);
  padding: 30px;
  border-radius: 12px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
}

.results-header {
  margin-bottom: 25px;
}

.results-header h4 {
  margin: 0 0 20px 0;
  color: var(--text-primary);
  /* color: #2c3e50; */
  font-size: 1.6em;
}

.results-controls {
  display: flex;
  gap: 20px;
  align-items: center;
  flex-wrap: wrap;
}

.filter-controls,
.view-mode-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.filter-controls label,
.view-mode-controls label {
  font-weight: 600;
  color: var(--text-primary);
  /* color: #2c3e50; */
}

.filter-controls select {
  padding: 10px 15px;
  border: 2px solid var(--border-color);
  /* border: 2px solid #e0e0e0; */
  border-radius: 6px;
  font-size: 1em;
}

.view-btn {
  padding: 8px 16px;
  background: var(--bg-muted);
  border: 2px solid var(--border-color);
  /* border: 2px solid #e0e0e0; */
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s;
  font-size: 0.95em;
  color: var(--text-primary);
  /* color: #2c3e50; */
}

.view-btn:hover {
  background: var(--bg-hover);
}

.view-btn.active {
  background: #3498db;
  color: white;
  border-color: #3498db;
}

/* Results Summary */
.results-summary {
  background: var(--bg-surface);
  padding: 25px;
  border-radius: 8px;
  margin-bottom: 30px;
}

.results-summary h4 {
  margin: 0 0 20px 0;
  color: var(--text-primary);
  /* color: #2c3e50; */
  font-size: 1.3em;
}

.summary-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
}

.stat-item {
  padding: 15px;
  border-radius: 6px;
  text-align: center;
  /* box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05); */
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

/* .stat-item .count {
  display: block;
  font-size: 2.5em;
  font-weight: bold;
  margin-bottom: 8px;
  color: var(--text-primary);
 
}

.stat-item .label {
  display: block;
  font-size: 1em;
  color: var(--text-secondary);
  
  font-weight: 500;
} */

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
}

.status-section-header {
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 15px;
}

.status-section-header.added {
  background: var(--status-added-bg);
  border-left: 4px solid var(--status-added-border);
}

.status-section-header.removed {
  background: var(--status-removed-bg);
  border-left: 4px solid var(--status-removed-border);
}

.status-section-header.modified {
  background: var(--status-modified-bg);
  border-left: 4px solid var(--status-modified-border);
}

.status-section-header.unchanged {
  background: var(--status-unchanged-bg);
  border-left: 4px solid var(--status-unchanged-border);
}

.status-icon {
  font-size: 2em;
  font-weight: bold;
}

.status-section-header h4 {
  margin: 0;
  color: var(--text-primary);
  /* color: #2c3e50; */
  font-size: 1.3em;
}

.status-section-header p {
  margin: 5px 0 0 0;
  color: var(--text-secondary);
  font-size: 0.95em;
}

/* Table Cards */
.table-card {
  /* background: white; */
  background: var(--bg-card);
  border: 2px solid var(--border-color);
  /* border: 2px solid #e0e0e0; */
  border-radius: 10px;
  margin-bottom: 15px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.table-card:hover {
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
}

.table-header-card {
  padding: 20px 25px;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: all 0.2s;
}

.table-header-card:hover {
  background: #f8f9fa;
}

.table-header-card.added {
  background: var(--status-added-bg);
}

.table-header-card.removed {
  background: var(--status-removed-bg);
}

.table-header-card.modified {
  background: var(--status-modified-bg);
}

.table-header-card.unchanged {
  background: var(--status-unchanged-bg);
}

.table-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.table-info .table-name {
  font-weight: 600;
  font-size: 1.2em;
  color: var(--text-primary);
  /* color: #2c3e50; */
}

.table-status-badge {
  font-size: 0.9em;
  color: var(--text-secondary);
  font-weight: 500;
}

.toggle-icon {
  font-size: 1.4em;
  color: var(--text-secondary);
}

/* Table Details */
.table-details {
  padding: 25px;
  border-top: 2px solid var(--border-color);
  background: var(--bg-surface);
}

.side-by-side-view {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.source-section,
.target-section {
  /* background: white; */
  background: var(--bg-card);
  border-radius: 8px;
  overflow: hidden;
}

.section-title {
  padding: 12px 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.source-title {
  background: var(--bg-muted);
}

.target-title {
  background: var(--bg-muted);
}

.section-title h6 {
  margin: 0;
  font-size: 1em;
}

.scrollable-table-container {
  overflow-x: auto !important;
  overflow-y: auto;
  max-height: 500px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  /* background: white; */
  background: var(--bg-card);
  /* Force horizontal scroll to appear when needed */
  -webkit-overflow-scrolling: touch;
  scrollbar-color: var(--border-color) var(--bg-secondary);
  scrollbar-width: auto;
}

.data-table {
  width: max-content;
  min-width: 100%;  /* Ensure table takes full width but allows horizontal scroll when wide */
  border-collapse: collapse;
  font-size: 0.95em;
  /* Prevent table from shrinking below content width */
  table-layout: auto;
  color: var(--text-primary);
}

.data-table thead {
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--bg-surface);
}

.data-table th {
  padding: 14px;
  text-align: left;
  color: var(--text-primary);
  font-weight: 600;
  border-bottom: 2px solid var(--border-color);
  white-space: nowrap;
  /* Ensure headers don't wrap */
  min-width: 100px;
}

.data-table td {
  padding: 12px 14px;
  border-bottom: 1px solid var(--border-color);
  /* Prevent cell content from wrapping too early */
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
  color: var(--text-primary);
}

.data-table tbody tr {
  background: var(--bg-card);
}

.data-table tbody tr:nth-child(even) {
  background: var(--bg-surface);
}

.data-table tbody tr:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.row-different {
  background: rgba(255, 181, 71, 0.15);
  color: var(--text-primary);
}

.row-missing {
  background: rgba(239, 68, 68, 0.12);
  color: var(--text-primary);
}

.row-extra {
  background: rgba(59, 130, 246, 0.12);
  color: var(--text-primary);
}

.cell-different {
  background: #fed7aa !important;
  font-weight: 600 !important;
}

.single-view {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.diff-comparison {
  /* background: white; */
  background: var(--bg-card);
  padding: 20px;
  border-radius: 8px;
  border: 2px solid var(--border-color);
  /* border: 2px solid #e0e0e0; */
}

.diff-key {
  margin-bottom: 15px;
  padding: 10px;
  background: #e3f2fd;
  border-radius: 4px;
  font-size: 1.05em;
}

.no-results {
  text-align: center;
  padding: 80px 20px;
  color: var(--text-secondary);
  font-style: italic;
  font-size: 1.2em;
}

/* Responsive Design */
@media (max-width: 768px) {
  .database-selection {
    grid-template-columns: 1fr;
  }

  .vs-indicator {
    justify-self: center;
    order: 2;
  }

  .side-by-side-view {
    grid-template-columns: 1fr;
  }

  .comparison-options {
    padding: 20px;
  }

  .option-group {
    flex-direction: column;
    gap: 15px;
  }

  .table-grid {
    grid-template-columns: 1fr;
  }

  .summary-stats {
    grid-template-columns: 1fr;
  }

  .comparison-actions {
    flex-direction: column;
  }

  .compare-btn,
  .export-btn,
  .patch-btn {
    width: 100%;
  }
}

/* Scrollbar Styling */
::-webkit-scrollbar {
  width: 12px;
  height: 12px;
}

::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 6px;
}

::-webkit-scrollbar-thumb {
  background: #888;
  border-radius: 6px;
}

::-webkit-scrollbar-thumb:hover {
  background: #555;
}

/* Header Controls for Per-Table Actions */
.header-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.mini-patch-btn {
  padding: 6px 12px;
  background: #f39c12;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
  transition: all 0.2s;
  font-weight: 500;
}

.mini-patch-btn:hover {
  background: #e67e22;
  transform: translateY(-1px);
  box-shadow: 0 2px 5px rgba(243, 156, 18, 0.3);
}

.mini-patch-btn:active {
  transform: translateY(0);
}

.per-table-view-toggle {
  display: flex;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
  /* background: white; */
  background: var(--bg-card);
}

.mini-view-btn {
  padding: 4px 8px;
  border: none;
  /* background: white; */
  background: var(--bg-card);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 0.75em;
  transition: all 0.2s;
  min-width: 45px;
}

.mini-view-btn:hover {
  background: var(--bg-surface);
}

.mini-view-btn.active {
  background: #007bff;
  color: white;
}
.loading-content .spinner-dots {
  margin-top: 30px;
}
.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000; /* Make sure this is high enough */
  backdrop-filter: blur(4px);
}

/* Pagination Styles */
.pagination-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: var(--bg-surface);
  border-radius: 8px;
  margin-bottom: 15px;
  border: 1px solid var(--border-primary);
  flex-wrap: wrap;
  gap: 15px;
}

.pagination-text {
  font-size: 0.95em;
  color: var(--text-secondary);
  font-weight: 500;
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.pagination-btn {
  padding: 8px 16px;
  background: var(--primary-500);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9em;
  font-weight: 500;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.pagination-btn:hover:not(:disabled) {
  background: var(--primary-600);
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(99, 102, 241, 0.3);
}

.pagination-btn:disabled {
  background: var(--gray-400);
  cursor: not-allowed;
  opacity: 0.5;
}

.page-indicator {
  font-size: 0.9em;
  color: var(--text-primary);
  font-weight: 600;
  padding: 0 8px;
  white-space: nowrap;
}

@media (max-width: 768px) {
  .pagination-info {
    flex-direction: column;
    align-items: stretch;
  }

  .pagination-controls {
    justify-content: center;
  }
}
</style>
