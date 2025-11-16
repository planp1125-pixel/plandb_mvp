<template>
  <div class="database-content">
    <div v-if="connectedDatabases.length === 0" class="welcome-screen">
      <div class="welcome-content">
        <h2>Welcome to SQLCipher/SQLite Differ Tool</h2>
        <p>Connect to your SQLCipher/SQLite  databases to get started:</p>
        <ul>
          <li>View and compare database schemas</li>
          <li>View and compare datas</li>
          <li>Browse table contents</li>
          <li>Export comparison reports</li>
        </ul>
      </div>
    </div>

    <!-- Tabs for different functionalities -->
    <div v-else class="tabs-container">
      <div class="tabs">
        <button
          v-for="tab in allTabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          :class="['tab', { 'tab-active': activeTab === tab.id }]"
        >
          <span class="tab-label">{{ tab.name }}</span>
          <!-- Close button for table tabs -->
          <button
            v-if="tab.id.startsWith('table_')"
            @click="closeTableTab(tab.id, $event)"
            class="tab-close-btn"
            title="Close tab"
          >
            ×
          </button>
        </button>
      </div>

      <div class="tab-content">
        <!-- Keep all components mounted but hide/show them -->
        <div v-show="activeTab === 'explorer'" class="tab-panel">
          <DatabaseExplorer
            :databases="connectedDatabases"
            :selected-database="selectedDatabase"
            @table-selected="handleTableSelected"
          />
        </div>

        <div v-show="activeTab === 'comparison'" class="tab-panel">
          <SchemaComparison
            :databases="connectedDatabases"
            @comparison-complete="handleComparisonComplete"
          />
        </div>

        <div v-show="activeTab === 'data-comparison'" class="tab-panel">
          <DataComparison
            :databases="connectedDatabases"
          />
        </div>

        <div v-show="activeTab === 'viewer'" class="tab-panel">
          <TableViewer
            :databases="connectedDatabases"
            :selected-database="selectedDatabase"
            @table-opened="handleTableSelected"
          />
        </div>

        <!-- Render TableViewer for each opened table tab -->
        <div
          v-for="tableTab in openedTableTabs"
          :key="tableTab.id"
          v-show="activeTab === tableTab.id"
          class="tab-panel"
        >
          <TableViewer
            :databases="connectedDatabases"
            :selected-database="connectedDatabases.find(d => d.path === tableTab.dbPath) || null"
            :initial-table="tableTab.tableName"
            :initial-db-path="tableTab.dbPath"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import DatabaseExplorer from './DatabaseExplorer.vue';
import SchemaComparison from './SchemaComparison.vue';
import DataComparison from './DataComparison.vue';
import TableViewer from './TableViewer.vue';
import type { DatabaseInfo, SchemaComparison as SchemaComparisonType } from '../services/databaseService';

interface Props {
  connectedDatabases: DatabaseInfo[];
  selectedDatabase: DatabaseInfo | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'table-selected': [tableName: string];
  'comparison-complete': [comparison: SchemaComparisonType];
}>();

const activeTab = ref('explorer');

// Manage opened table tabs
interface TableTab {
  id: string;
  tableName: string;
  dbPath: string;
  dbName: string;
}

const openedTableTabs = ref<TableTab[]>([]);
const activeTableTabId = ref<string | null>(null);

const staticTabs = [
  { id: 'explorer', name: 'Database Structure' },
  { id: 'comparison', name: 'Schema Comparison' },
  { id: 'data-comparison', name: 'Data Comparison' },
  { id: 'viewer', name: 'Browse Data' },
];

// Combine static tabs with dynamic table tabs
const allTabs = computed(() => {
  const tabs = [...staticTabs];

  // Add opened table tabs
  openedTableTabs.value.forEach(tableTab => {
    tabs.push({
      id: tableTab.id,
      name: `${tableTab.tableName} (${tableTab.dbName})`
    });
  });

  return tabs;
});

const handleTableSelected = (tableName: string, dbPath: string) => {
  // Find the database name
  const db = props.connectedDatabases.find(d => d.path === dbPath);
  const dbName = db?.name || dbPath.split('/').pop() || 'Unknown DB';

  // Create unique ID for this table tab (encode to handle special characters)
  // Use a simpler approach: hash the path and table name
  const pathHash = btoa(dbPath).replace(/[^a-zA-Z0-9]/g, '').substring(0, 20);
  const tabId = `table_${pathHash}_${tableName.replace(/[^a-zA-Z0-9]/g, '_')}`;

  // Check if tab already exists
  const existingTab = openedTableTabs.value.find(t => t.id === tabId);

  if (!existingTab) {
    // Add new table tab
    openedTableTabs.value.push({
      id: tabId,
      tableName,
      dbPath,
      dbName
    });
  }

  // Switch to this table tab
  activeTab.value = tabId;
  activeTableTabId.value = tabId;

  emit('table-selected', tableName);
};

const closeTableTab = (tabId: string, event: Event) => {
  event.stopPropagation();

  const index = openedTableTabs.value.findIndex(t => t.id === tabId);
  if (index === -1) return;

  openedTableTabs.value.splice(index, 1);

  // If closing the active tab, switch to another tab
  if (activeTab.value === tabId) {
    if (openedTableTabs.value.length > 0) {
      // Switch to the previous table tab or the last one
      const newIndex = Math.max(0, index - 1);
      activeTab.value = openedTableTabs.value[newIndex]?.id || 'explorer';
    } else {
      activeTab.value = 'explorer';
    }
  }
};

const handleComparisonComplete = (comparison: SchemaComparisonType) => {
  emit('comparison-complete', comparison);
};
</script>

<style scoped>
.database-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* Welcome Screen */
.welcome-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: var(--white);
  padding: 2rem;
}

.welcome-content {
  text-align: center;
  max-width: 28rem;
  padding: 2.5rem;
  background: var(--gray-50);
  border-radius: 1rem;
  border: 1px solid var(--gray-200);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1);
}

.welcome-content h2 {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--gray-900);
  margin-bottom: 1rem;
}

.welcome-content p {
  color: var(--gray-700);
  margin-bottom: 1.5rem;
}

.welcome-content ul {
  text-align: left;
  color: var(--gray-600);
  padding-left: 1.25rem;
}

.welcome-content li {
  margin-bottom: 0.5rem;
}

/* Tabs Container */
.tabs-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Tabs Navigation */
.tabs {
  display: flex;
  background: var(--white);
  border-bottom: 1px solid var(--gray-200);
  flex-shrink: 0;
  padding: 0 1.25rem;
  box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
  overflow-x: auto;
  overflow-y: hidden;
}

/* Custom scrollbar for tabs */
.tabs::-webkit-scrollbar {
  height: 6px;
}

.tabs::-webkit-scrollbar-track {
  background: var(--gray-100);
}

.tabs::-webkit-scrollbar-thumb {
  background: var(--gray-300);
  border-radius: 3px;
}

.tabs::-webkit-scrollbar-thumb:hover {
  background: var(--primary-500);
}

.tab {
  padding: 1rem 1.5rem;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--gray-500);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border-bottom: 3px solid transparent;
  white-space: nowrap;
  border-radius: 0.5rem 0.5rem 0 0;
  position: relative;
  margin: 0 0.25rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.tab-label {
  flex: 1;
}

.tab-close-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.25rem;
  line-height: 1;
  padding: 0 0.25rem;
  color: var(--gray-400);
  transition: all 0.2s;
  border-radius: 0.25rem;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tab-close-btn:hover {
  background: var(--gray-200);
  color: var(--gray-700);
}

.tab:hover {
  background: var(--gray-100);
  color: var(--primary-600);
}

.tab-active {
  color: var(--primary-600);
  background: var(--white);
  border-bottom-color: var(--primary-500);
  box-shadow: 0 -2px 0 var(--primary-500);
}

.tab:focus {
  outline: 2px solid var(--primary-500);
  outline-offset: -2px;
}

/* Tab Content */
.tab-content {
  flex: 1;
  overflow: hidden;
}

.tab-panel {
  height: 100%;
  overflow: auto;
  padding: 1.5rem;
  background: var(--gray-50);
}

/* Custom scrollbar for tab panels */
.tab-panel::-webkit-scrollbar {
  width: 8px;
}

.tab-panel::-webkit-scrollbar-track {
  background: var(--gray-100);
}

.tab-panel::-webkit-scrollbar-thumb {
  background: var(--gray-300);
  border-radius: 4px;
}

.tab-panel::-webkit-scrollbar-thumb:hover {
  background: var(--primary-500);
}

/* Deep selectors for child components */
.tab-panel :deep(.database-explorer),
.tab-panel :deep(.schema-comparison),
.tab-panel :deep(.table-viewer) {
  max-width: 100%;
  margin: 0;
  padding: 0.625rem;
}

/* Animation for smooth transitions */
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

.welcome-content {
  animation: slideIn 0.5s ease-out;
}

.tabs {
  animation: slideIn 0.3s ease-out;
}

/* Responsive Design */
@media (max-width: 768px) {
  .tabs {
    padding: 0 0.625rem;
    flex-wrap: wrap;
  }
  
  .tab {
    padding: 0.625rem 0.9375rem;
    font-size: 0.8125rem;
    flex: 1;
    min-width: 6.25rem;
  }
  
  .tab-panel {
    padding: 0.9375rem;
  }
}
</style>