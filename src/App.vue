/* Responsive Design */
@media (max-width: 1024px) {
  .nav-sidebar {
    width: 3rem;
  }
  
  .nav-icon {
    width: 2.5rem;
    height: 2.5rem;
  }
  
  .main-content-with-nav {
    margin-left: 3rem;
  }
  
  .sidebar-toggle {
    left: 3.625rem;
  }
  
  .sidebar {
    width: 18.75rem;
    position: absolute;
    z-index: 999;
    background: var(--white);
    box-shadow: 2px 0 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
    height: 100%;
  }
  
  .sidebar-hidden {
    transform: translateX(-100%);
    width: 0;
  }
  
  .tab {
    padding: 0.75rem 1.25rem;
  }
}

@media (max-width: 768px) {
  .nav-sidebar {
    width: 3rem;
    padding: 0.75rem 0;
  }
  
  .nav-icon {
    width: 2.25rem;
    height: 2.25rem;
  }
  
  .main-content-with-nav {
    margin-left: 3rem;
  }
  
  .sidebar-toggle {
    left: 3.625rem;
    width: 2rem;
    height: 2rem;
    font-size: 0.75rem;
  }
  
  .sidebar {
    width: 18.75rem;
    position: absolute;
    z-index: 999;
    background: var(--white);
    box-shadow: 2px 0 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  }
  
  .sidebar-hidden {
    transform: translateX(-100%);
  }
  
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--white);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    width: 100%;
  }

  .main-content-fullwidth {
    margin-left: 0;
    width: 100%;
  }
  
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
  
  .content-panel {
    padding: 1rem;
  }
  
  .setting-item,
  .profile-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }
  
  .setting-input,
  .profile-input {
    width: 100%;
    min-width: auto;
  }
}

/* Focus States for Accessibility */
.sidebar-toggle:focus,
.nav-icon:focus {
  outline: 2px solid var(--primary-500);
  outline-offset: 2px;
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

.content-panel {
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
  
  .main-content-with-nav {
    margin-left: 0;
  }
  
  .app-header {
    background: var(--white);
    color: var(--gray-900);
    box-shadow: none;
  }
}
    <template>
  <div id="app">
    <!-- <header class="app-header">
      <div class="header-content">
        <h1>SQLCipher Differ Tool</h1>
        <div class="subtitle">Desktop application for SQLCipher database management</div>
      </div>
    </header> -->

    <header class="app-header">
  <div class="header-content flex items-center justify-between">
    <div>
      <h1>SQLCipher Schema Comparison Tool</h1>
      <div class="subtitle">Desktop application for SQLCipher database management</div>
    </div>

    <!-- Theme toggle -->
    <button
      class="px-3 py-1.5 text-sm rounded-lg border border-black/10 dark:border-white/10
             bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-100
             hover:bg-slate-50 dark:hover:bg-slate-700 transition"
      @click="toggleTheme"
      :title="isDark ? 'Switch to Light' : 'Switch to Dark'"
    >
      {{ isDark ? '🌙 Dark' : '☀️ Light' }}
    </button>
  </div>
</header>
 


    <main class="app-main">
      <div class="container">
        <!-- Left Navigation Icons -->
        <nav class="nav-sidebar">
          <div class="nav-icons">
            <button 
              class="nav-icon"
              :class="{ 'nav-icon-active': activeNavItem === 'database' }"
              @click="activeNavItem = 'database'"
              title="Database Connections"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 3C7.58 3 4 4.79 4 7v10c0 2.21 3.58 4 8 4s8-1.79 8-4V7c0-2.21-3.58-4-8-4M12 5c3.31 0 6 1.34 6 3s-2.69 3-6 3-6-1.34-6-3 2.69-3 6-3m-6 5.15c1.36.72 3.54 1.15 6 1.15s4.64-.43 6-1.15V12c0 1.66-2.69 3-6 3s-6-1.34-6-3v-1.85m0 3.7c1.36.72 3.54 1.15 6 1.15s4.64-.43 6-1.15V17c0 1.66-2.69 3-6 3s-6-1.34-6-3v-1.15Z"/>
              </svg>
            </button>
            
            <button 
              class="nav-icon"
              :class="{ 'nav-icon-active': activeNavItem === 'settings' }"
              @click="activeNavItem = 'settings'"
              title="Settings"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 15.5A3.5 3.5 0 0 1 8.5 12A3.5 3.5 0 0 1 12 8.5a3.5 3.5 0 0 1 3.5 3.5a3.5 3.5 0 0 1-3.5 3.5m7.43-2.53c.04-.32.07-.64.07-.97c0-.33-.03-.66-.07-1l2.11-1.63c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.31-.61-.22l-2.49 1c-.52-.39-1.06-.73-1.69-.98l-.37-2.65A.506.506 0 0 0 14 2h-4c-.25 0-.46.18-.5.42l-.37 2.65c-.63.25-1.17.59-1.69.98l-2.49-1c-.22-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64L4.57 11c-.04.34-.07.67-.07 1c0 .33.03.65.07.97l-2.11 1.66c-.19.15-.25.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1.01c.52.4 1.06.74 1.69.99l.37 2.65c.04.24.25.42.5.42h4c.25 0 .46-.18.5-.42l.37-2.65c.63-.26 1.17-.59 1.69-.99l2.49 1.01c.22.08.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.66Z"/>
              </svg>
            </button>
            
            <button 
              class="nav-icon"
              :class="{ 'nav-icon-active': activeNavItem === 'subscription' }"
              @click="activeNavItem = 'subscription'"
              title="Subscription"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2A10 10 0 0 0 2 12a10 10 0 0 0 10 10a10 10 0 0 0 10-10A10 10 0 0 0 12 2m0 18a8 8 0 0 1-8-8a8 8 0 0 1 8-8a8 8 0 0 1 8 8a8 8 0 0 1-8 8m.5-13H11v6l5.25 3.15l.75-1.23l-4.5-2.67V7Z"/>
              </svg>
            </button>
            
            <button 
              class="nav-icon"
              :class="{ 'nav-icon-active': activeNavItem === 'profile' }"
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
        <button class="sidebar-toggle" @click="toggleSidebar" v-show="activeNavItem === 'database'">
          {{ isSidebarVisible ? '◀' : '▶' }}
        </button>

        <!-- Sidebar for database connections -->
        <aside 
          class="sidebar" 
          :class="{ 'sidebar-hidden': !isSidebarVisible || activeNavItem !== 'database' }"
          v-show="activeNavItem === 'database'"
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
            'main-content-fullwidth': !isSidebarVisible || activeNavItem !== 'database',
            'main-content-with-nav': true 
          }"
        >
          <!-- Database Content -->
          <div v-if="activeNavItem === 'database'">
            <Database 
              :connected-databases="connectedDatabases"
              :selected-database="selectedDatabase"
              @table-selected="handleTableSelected"
              @comparison-complete="handleComparisonComplete"
            />
          </div>

          <!-- Settings Content -->
          <Settings v-else-if="activeNavItem === 'settings'" />

          <!-- Subscription Content -->
          <Subscription v-else-if="activeNavItem === 'subscription'" />

          <!-- Profile Content -->
          <Profile v-else-if="activeNavItem === 'profile'" />
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
// import { ref } from 'vue';
import { ref, onMounted } from 'vue'
import DatabaseConnection from './components/DatabaseConnection.vue';
import Database from './components/Database.vue';
import Settings from './components/Settings.vue';
import Subscription from './components/Subscription.vue';
import Profile from './components/Profile.vue';
import type { DatabaseInfo, SchemaComparison as SchemaComparisonType } from './services/databaseService';


const isDark = ref(false)

function applyTheme(dark: boolean) {
  const root = document.documentElement // <html>
  root.classList.toggle('dark', dark)
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
  // keep in sync with OS changes (optional)
  const mq = window.matchMedia?.('(prefers-color-scheme: dark)')
  mq?.addEventListener('change', e => {
    // only auto-switch if user hasn't explicitly chosen
    if (!localStorage.getItem('theme')) applyTheme(e.matches)
  })
})
// State
const connectedDatabases = ref<DatabaseInfo[]>([]);
const selectedDatabase = ref<DatabaseInfo | null>(null);
const activeTab = ref('explorer');
const isSidebarVisible = ref(true);
const activeNavItem = ref('database'); // Add navigation state


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
/* Tailwind-inspired utility classes */

/* Color Variables */
/*:root {
  --primary-500: #6366f1;
  --primary-600: #4f46e5;
  --primary-700: #3730a3;
  --gray-50: #f8fafc;
  --gray-100: #f1f5f9;
  --gray-200: #e2e8f0;
  --gray-300: #cbd5e1;
  --gray-500: #64748b;
  --gray-700: #334155;
  --gray-800: #1e293b;
  --gray-900: #0f172a;
  --white: #ffffff;
  --blue-500: #3b82f6;
  --blue-600: #2563eb;
  --blue-700: #1d4ed8;
} 

:root {
  --primary-500: #6366f1;
  --primary-600: #4f46e5;
  --gray-50: #f8fafc;
  --gray-100: #f1f5f9;
  --gray-800: #1e293b;
  --gray-900: #0f172a;
  --white: #ffffff;
} 


:root.dark {
  --primary-500: #818cf8;
  --primary-600: #6366f1;
  --gray-50: #0f172a;     
  --gray-100: #1e293b;    
  --gray-800: #f1f5f9;    
  --gray-900: #f8fafc;     
  --white: #0f172a;      
}
*/

/* Reset and Base */
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

/* App Container */
#app {
  font-family: 'Inter', 'Segoe UI', -apple-system, BlinkMacSystemFont, sans-serif;
  height: 100vh;
  background-color: var(--gray-50);
  color: var(--gray-900);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  line-height: 1.6;
}

/* Header */
.app-header {
  background: linear-gradient(135deg, var(--primary-500) 0%, var(--primary-600) 100%);
  
  color: var(--white);
  padding: 1rem 1.5rem;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  position: relative;
  z-index: 50;
}

.header-content {
  max-width: none;
}

.app-header h1 {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0 0 0.25rem 0;
  letter-spacing: -0.025em;
}

.subtitle {
  font-size: 0.875rem;
  opacity: 0.8;
  font-weight: 400;
}

/* Main Layout */
.app-main {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.container {
  display: flex;
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
}

/* Left Navigation Sidebar */
.nav-sidebar {
  width: 4rem;
  background: var(--gray-800);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1rem 0;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.1);
  z-index: 200;
}

.nav-icons {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  width: 100%;
}

.nav-icon {
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--gray-300);
  cursor: pointer;
  border-radius: 0.75rem;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  margin: 0 0.5rem;
}

.nav-icon:hover {
  background: var(--gray-700);
  color: var(--white);
  transform: translateX(2px);
}

.nav-icon-active {
  background: var(--primary-500);
  color: var(--white);
}

.nav-icon-active:hover {
  background: var(--primary-600);
  transform: translateX(2px);
}

/* Sidebar Toggle Button */
.sidebar-toggle {
  position: absolute;
  top: 0.625rem;
  left: 4.625rem; /* Adjust for nav sidebar width */
  z-index: 1000;
  background: var(--primary-500);
  color: var(--white);
  border: none;
  border-radius: 0.5rem;
  width: 2.5rem;
  height: 2.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.sidebar-toggle:hover {
  background: var(--primary-600);
  transform: translateY(-1px);
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
}

.sidebar-toggle:active {
  transform: translateY(0);
}

/* Sidebar */
.sidebar {
  width: 25rem;
  flex-shrink: 0;
  background: var(--white);
  border-right: 1px solid var(--gray-200);
  overflow-y: auto;
  height: 100%;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  z-index: 100;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1);
}

.sidebar-hidden {
  transform: translateX(-100%);
  width: 0;
  border-right: none;
}

/* Custom Scrollbar */
.sidebar::-webkit-scrollbar {
  width: 6px;
}

.sidebar::-webkit-scrollbar-track {
  background: var(--gray-100);
}

.sidebar::-webkit-scrollbar-thumb {
  background: var(--gray-300);
  border-radius: 3px;
}

.sidebar::-webkit-scrollbar-thumb:hover {
  background: var(--primary-500);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--white);
  transition: margin-left 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  height: 100%; /* Add this */
}

/* Ensure database content gets full height */
.main-content > div {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* Make sure individual components can scroll */
.main-content :deep(.content-panel) {
  height: 100%;
  overflow-y: auto;
  flex: 1;
}
.main-content-fullwidth {
  margin-left: 0;
}

.main-content-with-nav {
  margin-left: 4rem; /* Account for nav sidebar */
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

/* Responsive Design */
@media (max-width: 1024px) {
  .sidebar {
    width: 18.75rem;
    position: absolute;
    z-index: 999;
    background: var(--white);
    box-shadow: 2px 0 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
    height: 100%;
  }
  
  .sidebar-hidden {
    transform: translateX(-100%);
    width: 0;
  }
  
  .tab {
    padding: 0.75rem 1.25rem;
  }
}

@media (max-width: 768px) {
  .sidebar {
    width: 18.75rem;
    position: absolute;
    z-index: 999;
    background: var(--white);
    box-shadow: 2px 0 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  }
  
  .sidebar-hidden {
    transform: translateX(-100%);
  }
  
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--white);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    width: 100%;
  }

  .main-content-fullwidth {
    margin-left: 0;
    width: 100%;
  }
  
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

/* Focus States for Accessibility */
.sidebar-toggle:focus {
  outline: 2px solid var(--primary-500);
  outline-offset: 2px;
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
  .sidebar-toggle,
  .sidebar {
    display: none;
  }
  
  .main-content {
    width: 100%;
    margin: 0;
  }
  
  .app-header {
    background: var(--white);
    color: var(--gray-900);
    box-shadow: none;
  }
}

/*html.dark {
  color-scheme: dark;
}

html.dark #app {
  background-color: #0f172a;
  color: #f8fafc;
}

html.dark .app-header {
  background: linear-gradient(135deg, #4338ca 0%, #3730a3 100%);
}

html.dark .sidebar,
html.dark .main-content {
  background: #1e293b;
  border-color: #334155;
}

html.dark .nav-sidebar {
  background: #0f172a;
}

html.dark .welcome-content {
  background: #1e293b;
  border-color: #334155;
  color: #f8fafc;
}

html.dark .tab-panel {
  background: #1e293b;
}*/

</style>