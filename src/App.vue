<template>
  <div id="app" class="min-h-screen flex flex-col bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 font-sans overflow-hidden">
    <!-- Header -->
    <header class="bg-gradient-to-r from-blue-500 to-blue-600 dark:from-blue-600 dark:to-blue-700 text-white shadow-lg border-b border-white/10 relative z-50">
      <div class="px-6 py-4 flex items-center justify-between">
        <div>
          <h1 class="text-xl font-semibold mb-1 tracking-tight">SQLCipher Schema Comparison Tool</h1>
          <div class="text-sm opacity-80 font-normal">Desktop application for SQLCipher database management</div>
        </div>

        <!-- Theme toggle -->
        <button
          class="px-3 py-1.5 text-sm rounded-lg border border-black/10 dark:border-white/10 bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-100 hover:bg-slate-50 dark:hover:bg-slate-700 transition-all duration-200"
          @click="toggleTheme"
          :title="isDark ? 'Switch to Light' : 'Switch to Dark'"
        >
          {{ isDark ? '🌙' : '☀️' }}
        </button>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-1 flex overflow-hidden">
      <div class="flex w-full h-full relative">
        <!-- Left Navigation Sidebar -->
        <nav class="w-30 bg-gray-800 dark:bg-gray-900 flex flex-col items-center py-4 shadow-lg z-40 flex-shrink-0">
          <div class="flex flex-col gap-2 w-full">
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
          <div v-if="activeNavItem === 'database'" class="h-full">
            <Database 
              :connected-databases="connectedDatabases"
              :selected-database="selectedDatabase"
              @table-selected="handleTableSelected"
              @comparison-complete="handleComparisonComplete"
            />
          </div>

          <!-- Settings Content -->
          <div v-else-if="activeNavItem === 'settings'" class="h-full">
            <Settings />
          </div>

          <!-- Subscription Content -->
          <div v-else-if="activeNavItem === 'subscription'" class="h-full">
            <Subscription />
          </div>

          <!-- Profile Content -->
          <div v-else-if="activeNavItem === 'profile'" class="h-full">
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
/* Navigation Icons */
.nav-icon {
  @apply w-12 h-12 flex items-center justify-center bg-transparent border-none cursor-pointer rounded-xl transition-all duration-200 mx-2;
}

.nav-icon-inactive {
  @apply text-gray-300 hover:bg-gray-700 hover:text-white hover:translate-x-0.5;
}

.nav-icon-active {
  @apply bg-blue-500 text-white hover:bg-blue-600 hover:translate-x-0.5;
}

/* Sidebar Toggle Button */
.sidebar-toggle {
  @apply absolute top-2 left-[4.5rem] z-30 bg-blue-500 text-white border-none rounded-lg w-10 h-10 cursor-pointer text-sm flex items-center justify-center shadow-lg transition-all duration-200 hover:bg-blue-600 hover:-translate-y-0.5 hover:shadow-xl active:translate-y-0;
}

/* Sidebar */
.sidebar {
  @apply w-80 flex-shrink-0 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 overflow-y-auto h-full transition-transform duration-200 relative z-20 shadow-lg;
}

.sidebar-visible {
  @apply translate-x-0;
}

.sidebar-hidden {
  @apply -translate-x-full w-0 border-r-0;
}

/* Main Content */
.main-content {
  @apply flex-1 flex flex-col overflow-auto bg-white dark:bg-gray-800 min-h-full;
}

.main-content-with-sidebar {
  @apply ml-0;
}

.main-content-full {
  @apply ml-0;
}

/* Custom scrollbar for sidebar */
.sidebar::-webkit-scrollbar {
  width: 6px;
}

.sidebar::-webkit-scrollbar-track {
  @apply bg-gray-100 dark:bg-gray-700;
}

.sidebar::-webkit-scrollbar-thumb {
  @apply bg-gray-300 dark:bg-gray-600 rounded;
}

.sidebar::-webkit-scrollbar-thumb:hover {
  @apply bg-blue-500;
}

/* Responsive Design */
@media (max-width: 1024px) {
  .nav-sidebar {
    width: 3rem;
  }
  
  .nav-icon {
    @apply w-10 h-10;
  }
  
  .sidebar-toggle {
    @apply left-[3.5rem];
  }
  
  .sidebar {
    @apply w-72 absolute z-30 bg-white dark:bg-gray-800 shadow-xl h-full;
  }
}

@media (max-width: 768px) {
  .nav-sidebar {
    @apply w-12 py-3;
  }
  
  .nav-icon {
    @apply w-9 h-9;
  }
  
  .sidebar-toggle {
    @apply left-[3.5rem] w-8 h-8 text-xs;
  }
  
  .sidebar {
    @apply w-64;
  }
  
  .app-header h1 {
    @apply text-lg;
  }
  
  .app-header .subtitle {
    @apply text-xs;
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
    @apply hidden;
  }
  
  .main-content {
    @apply w-full m-0;
  }
  
  .app-header {
    @apply bg-white text-gray-900 shadow-none;
  }
}

/* Ensure child components use full height */
.main-content > div {
  @apply h-full flex flex-col;
}
</style>
