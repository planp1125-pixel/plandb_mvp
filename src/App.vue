<template>
  <div id="app" class="app-container">
    <!-- Header -->
    <header class="app-header">
      <div class="header-content">
        <div>
          <h1>SQLCipher Schema Comparison Tool</h1>
          <div class="subtitle">Desktop application for SQLCipher database management</div>
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
              :class="{
                'nav-icon-active': activeNavItem === 'settings',
                'nav-icon-inactive': activeNavItem !== 'settings'
              }"
              @click="activeNavItem = 'settings'"
              title="Settings"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 15.5A3.5 3.5 0 0 1 8.5 12A3.5 3.5 0 0 1 12 8.5a3.5 3.5 0 0 1 3.5 3.5a3.5 3.5 0 0 1-3.5 3.5m7.43-2.53c.04-.32.07-.64.07-.97c0-.33-.03-.66-.07-1l2.11-1.63c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.31-.61-.22l-2.49 1c-.52-.39-1.06-.73-1.69-.98l-.37-2.65A.506.506 0 0 0 14 2h-4c-.25 0-.46.18-.5.42l-.37 2.65c-.63.25-1.17.59-1.69.98l-2.49-1c-.22-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64L4.57 11c-.04.34-.07.67-.07 1c0 .33.03.65.07.97l-2.11 1.66c-.19.15-.25.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1.01c.52.4 1.06.74 1.69.99l.37 2.65c.04.24.25.42.5.42h4c.25 0 .46-.18.5-.42l.37-2.65c.63-.26 1.17-.59 1.69-.99l2.49 1.01c.22.08.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.66Z"/>
              </svg>
            </button>
            
            <button 
              class="nav-icon"
              :class="{
                'nav-icon-active': activeNavItem === 'subscription',
                'nav-icon-inactive': activeNavItem !== 'subscription'
              }"
              @click="activeNavItem = 'subscription'"
              title="Subscription"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2A10 10 0 0 0 2 12a10 10 0 0 0 10 10a10 10 0 0 0 10-10A10 10 0 0 0 12 2m0 18a8 8 0 0 1-8-8a8 8 0 0 1 8-8a8 8 0 0 1 8 8a8 8 0 0 1-8 8m.5-13H11v6l5.25 3.15l.75-1.23l-4.5-2.67V7Z"/>
              </svg>
            </button>
            
            <button 
              class="nav-icon"
              :class="{
                'nav-icon-active': activeNavItem === 'profile',
                'nav-icon-inactive': activeNavItem !== 'profile'
              }"
              @click="activeNavItem = 'profile'"
              title="Profile"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4a4 4 0 0 1 4 4a4 4 0 0 1-4 4a4 4 0 0 1-4-4a4 4 0 0 1 4-4m0 10c4.42 0 8 1.79 8 4v2H4v-2c0-2.21 3.58-4 8-4Z"/>
              </svg>
            </button>
          </div>
        </nav>

        <!-- Sidebar toggle button -->
        <button 
          v-show="activeNavItem === 'database'"
          @click="toggleSidebar"
          class="sidebar-toggle"
        >
          {{ isSidebarVisible ? '◀' : '▶' }}
        </button>

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

          <!-- Settings Content -->
          <div v-show="activeNavItem === 'settings'" class="h-full">
            <Settings />
          </div>

          <!-- Subscription Content -->
          <div v-show="activeNavItem === 'subscription'" class="h-full">
            <Subscription />
          </div>

          <!-- Profile Content -->
          <div v-show="activeNavItem === 'profile'" class="h-full">
            <Profile />
          </div>
        </div>
      </div>
    </main>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import DatabaseConnection from './components/DatabaseConnection.vue';
import Database from './components/Database.vue';
import Settings from './components/Settings.vue';
import Subscription from './components/Subscription.vue';
import Profile from './components/Profile.vue';
import type { DatabaseInfo, SchemaComparison as SchemaComparisonType } from './services/databaseService';

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

// License management
onMounted(() => {
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
})

// App state
const connectedDatabases = ref<DatabaseInfo[]>([]);
const selectedDatabase = ref<DatabaseInfo | null>(null);
const activeTab = ref('explorer');
const isSidebarVisible = ref(true);
const activeNavItem = ref('database');

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
  padding: 1rem 1.5rem;
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

.nav-icon-active {
  background-color: var(--blue-500);
  color: white;
}

.nav-icon-active:hover {
  background-color: var(--blue-600);
  transform: translateX(0.125rem);
}

/* Sidebar Toggle Button */
.sidebar-toggle {
  position: absolute;
  top: 0.5rem;
  left: 4.75rem;
  z-index: 30;
  background-color: var(--blue-500);
  color: white;
  border: none;
  border-radius: 0.5rem;
  width: 2.5rem;
  height: 2.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--shadow-lg);
  transition: all 0.2s;
}

.sidebar-toggle:hover {
  background-color: var(--blue-600);
  transform: translateY(-0.125rem);
  box-shadow: var(--shadow-xl);
}

.sidebar-toggle:active {
  transform: translateY(0);
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

  .sidebar-toggle {
    left: 4.25rem;
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

  .sidebar-toggle {
    left: 3.75rem;
    width: 2rem;
    height: 2rem;
    font-size: 0.75rem;
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
  .sidebar-toggle,
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
