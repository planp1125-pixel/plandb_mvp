<template>
  <div id="app" class="app-container">
    <!-- Header -->
    <header class="app-header">
      <div class="header-content">
        <div class="flex items-center gap-3">
          <img src="./assets/logo1.png" alt="planDB Logo" class="w-20 h-20" />
          <div>
            <h1 class="text-xl font-bold">planDB</h1>
            <div class="subtitle">Desktop application for SQLCipher database management</div>
          </div>
        </div>

        <!-- Theme toggle -->
        <button
          class="theme-toggle-btn"
          @click="toggleTheme"
          :title="isDark ? 'Switch to Light' : 'Switch to Dark'"
        >
          {{ isDark ? '🌙' : '☀️' }}
        </button>
      </div>
    </header>

    <!-- Main Content -->
    <main class="main-wrapper">
      <div class="content-container">
        <!-- Left Navigation Sidebar -->
        <nav class="nav-sidebar">
          <div class="nav-buttons-container">
            <!-- Sidebar toggle button (only shown when database is active) -->
            <button
              v-show="activeNavItem === 'database'"
              @click="toggleSidebar"
              class="nav-icon sidebar-toggle-icon"
              :title="isSidebarVisible ? 'Hide Database Panel' : 'Show Database Panel'"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path v-if="isSidebarVisible" d="M15.41,16.58L10.83,12L15.41,7.41L14,6L8,12L14,18L15.41,16.58Z"/>
                <path v-else d="M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z"/>
              </svg>
            </button>

            <button
              class="nav-icon"
              :class="{
                'nav-icon-active': activeNavItem === 'database',
                'nav-icon-inactive': activeNavItem !== 'database'
              }"
              @click="activeNavItem = 'database'"
              title="Database Connections"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 3C7.58 3 4 4.79 4 7v10c0 2.21 3.58 4 8 4s8-1.79 8-4V7c0-2.21-3.58-4-8-4M12 5c3.31 0 6 1.34 6 3s-2.69 3-6 3-6-1.34-6-3 2.69-3 6-3m-6 5.15c1.36.72 3.54 1.15 6 1.15s4.64-.43 6-1.15V12c0 1.66-2.69 3-6 3s-6-1.34-6-3v-1.85m0 3.7c1.36.72 3.54 1.15 6 1.15s4.64-.43 6-1.15V17c0 1.66-2.69 3-6 3s-6-1.34-6-3v-1.15Z"/>
              </svg>
            </button>
            <button
              class="nav-icon"
              @click="reportBugEmail"
              title="Report Bug via Email"
            >
              <span class="text-xl">✉️</span>
            </button>

            <button
              class="nav-icon"
              @click="reportBugGithub"
              title="Report Bug on GitHub"
            >
              <span class="text-xl">🐙</span>
            </button>
          </div>

          <!-- Trial Info -->
          <div class="trial-info">
            <div class="version-tag">Beta v{{ trialInfo.version }}</div>
            <div class="trial-days" :class="{ 'expired': trialInfo.is_expired }">
              {{ trialInfo.is_expired ? 'Trial Expired' : `Trial: ${trialInfo.remaining_days} days left` }}
            </div>
          </div>
        </nav>

        <!-- Sidebar for database connections -->
        <aside 
          v-show="activeNavItem === 'database'"
          class="sidebar"
          :class="{
            'sidebar-visible': isSidebarVisible && activeNavItem === 'database',
            'sidebar-hidden': !isSidebarVisible || activeNavItem !== 'database'
          }"
        >
          <DatabaseConnection 
            @database-connected="handleDatabaseConnected"
            @select-database="handleDatabaseSelected"
          />
        </aside>

        <!-- Main content area -->
        <div 
          class="main-content"
          :class="{
            'main-content-full': !isSidebarVisible || activeNavItem !== 'database',
            'main-content-with-sidebar': isSidebarVisible && activeNavItem === 'database'
          }"
        >
          <!-- Database Content -->
          <div v-show="activeNavItem === 'database'" class="h-full">
            <Database
              :connected-databases="connectedDatabases"
              :selected-database="selectedDatabase"
              @table-selected="handleTableSelected"
              @comparison-complete="handleComparisonComplete"
            />
          </div>
        </div>
      </div>
    </main>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { open } from '@tauri-apps/plugin-shell';
import DatabaseConnection from "./components/DatabaseConnection.vue";
import Database from './components/Database.vue';
import { DatabaseService, type DatabaseInfo, type SchemaComparison as SchemaComparisonType, type TrialInfo } from './services/databaseService';

// Theme management
const isDark = ref(false)

function applyTheme(dark: boolean) {
  const root = document.documentElement
  root.classList.toggle('dark', dark)
  root.setAttribute('data-theme', dark ? 'dark' : 'light')
  localStorage.setItem('theme', dark ? 'dark' : 'light')
  isDark.value = dark
}

function toggleTheme() {
  applyTheme(!isDark.value)
}

// App state
const connectedDatabases = ref<DatabaseInfo[]>([]);
const selectedDatabase = ref<DatabaseInfo | null>(null);
const activeTab = ref('explorer');
const isSidebarVisible = ref(true);
const activeNavItem = ref('database');
const databaseService = new DatabaseService();

const trialInfo = ref<TrialInfo>({
  is_expired: false,
  remaining_days: 90,
  version: '0.5.2'
});

// License management and initial setup
onMounted(async () => {
  // Theme initialization
  const ls = localStorage.getItem('theme')
  if (ls) {
    applyTheme(ls === 'dark')
  } else {
    const prefersDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches
    applyTheme(!!prefersDark)
  }

  const mq = window.matchMedia?.('(prefers-color-scheme: dark)')
  mq?.addEventListener('change', e => {
    if (!localStorage.getItem('theme')) applyTheme(e.matches)
  })

  // Fetch trial info
  try {
    trialInfo.value = await databaseService.getTrialInfo();
  } catch (e) {
    console.error('Failed to fetch trial info:', e);
  }
});

const reportBugEmail = async () => {
  const subject = encodeURIComponent(`Bug Report - planDB v${trialInfo.value.version}`);
  const body = encodeURIComponent(`Please describe the issue you are facing:\n\n\n\nOS: Linux\nVersion: ${trialInfo.value.version}`);
  const mailtoLink = `mailto:planptechsup@gmail.com?subject=${subject}&body=${body}`;
  
  try {
    await open(mailtoLink);
  } catch (e) {
    console.error('Failed to open email client:', e);
  }
};

const reportBugGithub = async () => {
  // Replace with your actual repo URL
    const openGitHubReport = async () => {
      await open('https://github.com/planp1125-pixel/planDB/issues/new');
    }; 
  try {
    await openGitHubReport();
  } catch (e) {
    console.error('Failed to open GitHub:', e);
  }
};

// Toggle sidebar visibility
const toggleSidebar = () => {
  isSidebarVisible.value = !isSidebarVisible.value;
};

// Event handlers
const handleDatabaseConnected = (database: DatabaseInfo) => {
  console.log('Database connected:', database);
  
  const existing = connectedDatabases.value.find(db => db.path === database.path);
  if (!existing) {
    connectedDatabases.value.push(database);
  }

  selectedDatabase.value = database;
  activeTab.value = 'explorer';
};

const handleDatabaseSelected = (database: DatabaseInfo) => {
  console.log('Database selected:', database);
  selectedDatabase.value = database;
};

const handleComparisonComplete = (comparison: SchemaComparisonType) => {
  console.log('Schema comparison completed:', comparison);
};

const handleTableSelected = (tableName: string) => {
  console.log('Table selected:', tableName);
  activeTab.value = 'viewer';
};
</script>

<style scoped>
/* App Container */
.app-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  font-family: sans-serif;
  overflow: hidden;
}

/* Header */
.app-header {
  background: var(--gradient-header);
  color: var(--text-inverse);
  box-shadow: var(--shadow-lg);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  position: relative;
  z-index: 50;
}

.header-content {
  padding: 0.1rem 1.5rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.app-header h1 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.25rem;
  letter-spacing: -0.025em;
}

.app-header .subtitle {
  font-size: 0.875rem;
  opacity: 0.8;
  font-weight: 400;
}

/* Main Layout */
.main-wrapper {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.content-container {
  display: flex;
  width: 100%;
  height: 100%;
  position: relative;
}

/* Navigation Sidebar */
.nav-sidebar {
  width: 3.5rem;
  background-color: var(--bg-nav);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1rem 0;
  box-shadow: var(--shadow-lg);
  z-index: 40;
  flex-shrink: 0;
}

.nav-buttons-container {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  width: 100%;
}

/* Theme Toggle Button */
.theme-toggle-btn {
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
  border-radius: 0.5rem;
  border: 1px solid rgba(0, 0, 0, 0.1);
  background: white;
  color: var(--gray-800);
  transition: all 0.2s;
  cursor: pointer;
}

.theme-toggle-btn:hover {
  background: var(--gray-50);
}

/* Navigation Icons */
.nav-icon {
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  border: none;
  cursor: pointer;
  border-radius: 0.5rem;
  transition: all 0.2s;
  margin: 0 0.25rem;
}

.nav-icon-inactive {
  color: var(--gray-300);
}

.nav-icon-inactive:hover {
  background-color: var(--gray-700);
  color: white;
  transform: translateX(0.125rem);
}

.nav-icon:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.nav-icon-active {
  background: var(--bg-tertiary);
  color: var(--primary-500);
}

.nav-icon-inactive {
  color: var(--text-secondary);
}

.trial-info {
  margin-top: auto;
  padding: 1rem 0.5rem;
  text-align: center;
  font-size: 0.75rem;
  border-top: 1px solid var(--border-color);
}

.version-tag {
  color: var(--text-secondary);
  font-weight: 500;
  margin-bottom: 0.25rem;
}

.trial-days {
  color: var(--primary-500);
  font-weight: 600;
}

.trial-days.expired {
  color: #ef4444;
  font-weight: 600;
}

/* Sidebar Toggle Icon in Navigation */
.sidebar-toggle-icon {
  color: var(--gray-400);
  margin-bottom: 0.5rem;
  border-bottom: 1px solid var(--gray-700);
  border-radius: 0.375rem 0.375rem 0 0;
  padding-bottom: 0.5rem;
}

.sidebar-toggle-icon:hover {
  background-color: var(--gray-700);
  color: var(--blue-400);
  transform: translateX(0.125rem);
}

/* Sidebar */
.sidebar {
  width: 24rem;
  flex-shrink: 0;
  background-color: var(--bg-sidebar);
  border-right: 1px solid var(--border-primary);
  overflow-y: auto;
  height: 100%;
  transition: transform 0.2s;
  position: relative;
  z-index: 20;
  box-shadow: var(--shadow-lg);
}

.sidebar-visible {
  transform: translateX(0);
}

.sidebar-hidden {
  transform: translateX(-100%);
  width: 0;
  border-right: 0;
}

/* Main Content */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: auto;
  background-color: var(--bg-card);
  min-height: 100%;
}

.main-content-with-sidebar {
  margin-left: 0;
}

.main-content-full {
  margin-left: 0;
}

/* Custom scrollbar for sidebar */
.sidebar::-webkit-scrollbar {
  width: 6px;
}

.sidebar::-webkit-scrollbar-track {
  background-color: var(--gray-100);
}

.sidebar::-webkit-scrollbar-thumb {
  background-color: var(--gray-300);
  border-radius: 3px;
}

.sidebar::-webkit-scrollbar-thumb:hover {
  background-color: var(--blue-500);
}

/* Responsive Design */
@media (max-width: 1024px) {
  .nav-sidebar {
    width: 4rem;
  }

  .nav-icon {
    width: 2.25rem;
    height: 2.25rem;
  }

  .sidebar {
    width: 22rem;
    position: absolute;
    z-index: 30;
    background-color: var(--bg-sidebar);
    box-shadow: var(--shadow-xl);
    height: 100%;
  }
}

@media (max-width: 768px) {
  .nav-sidebar {
    width: 3.5rem;
    padding-top: 0.75rem;
    padding-bottom: 0.75rem;
  }

  .nav-icon {
    width: 2rem;
    height: 2rem;
  }

  .sidebar {
    width: 18rem;
  }

  .app-header h1 {
    font-size: 1.125rem;
  }

  .app-header .subtitle {
    font-size: 0.75rem;
  }
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

/* Print Styles */
@media print {
  .nav-sidebar,
  .sidebar {
    display: none;
  }

  .main-content {
    width: 100%;
    margin: 0;
  }

  .app-header {
    background: white;
    color: var(--gray-900);
    box-shadow: none;
  }
}

/* Ensure child components use full height */
.main-content > div {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
