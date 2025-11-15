<template>
  <div class="database-connection">
    <h3>Database Connections</h3>
    <p class="subtitle">Connect to SQLCipher databases for comparison</p>
    
    <!-- Connection Form -->
    <div class="connection-form">
      <h4>Add New Database Connection</h4>
      
      <div class="form-group">
        <label for="db-path">Database File Path:</label>
        <div class="file-input-group">
          <input 
            id="db-path"
            v-model="dbPath" 
            type="text" 
            placeholder="e.g., /home/user/database.db or ~/Downloads/data.db"
            class="file-path-input"
          />
          <button @click="selectFile" class="select-file-btn">Browse</button>
        </div>
      </div>

      <!-- <div class="form-group">
        <label for="password">Password (Optional for regular SQLite):</label>
        <input 
          id="password"
          v-model="password" 
          type="password" 
          placeholder="Leave empty for unencrypted SQLite, or enter SQLCipher password"
          class="password-input"
        />
      </div> -->

      <div class="form-group">
        <label for="db-alias">Database Alias (Optional):</label>
        <input 
          id="db-alias"
          v-model="dbAlias" 
          type="text" 
          placeholder="e.g., Production DB, Test DB, Backup DB"
          class="alias-input"
        />
        <small class="help-text">Give this database a friendly name for easier identification</small>
      </div>

      <div class="form-actions">
        <button 
          @click="handleConnect" 
          :disabled="!dbPath || isConnecting"
          class="connect-btn"
        >
          {{ isConnecting ? 'Connecting...' : 'Add Database' }}
        </button>
      </div>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <div v-if="successMessage" class="success-message">
        {{ successMessage }}
      </div>
    </div>

    <!-- NEW: Migration Tools Section
    <div v-if="connectedDatabases.length > 0" class="migration-tools-section">
      <h4>🔐 Migration & Encryption Tools</h4>
      <p class="section-subtitle">Convert SQLite databases to encrypted SQLCipher or change encryption keys</p>
      
      <div class="migration-tools-table">
        <div class="migration-table-header">
          <div class="col-database">Database</div>
          <div class="col-type">Type</div>
          <div class="col-action">Action</div>
        </div>
        
        <div 
          v-for="db in connectedDatabases" 
          :key="db.path" 
          class="migration-table-row"
        >
          <div class="col-database">
            <div class="db-name-compact">
              <strong>{{ db.alias || db.name }}</strong>
              <span class="db-path-compact">{{ db.path }}</span>
            </div>
          </div>
          
          <div class="col-type">
            <span 
              class="type-badge" 
              :class="getDatabaseType(db.path) === 'sqlite' ? 'badge-sqlite' : 'badge-sqlcipher'"
            >
              {{ getDatabaseType(db.path) === 'sqlite' ? '📄 SQLite' : '🔒 SQLCipher' }}
            </span>
          </div>
          
          <div class="col-action">
            <button 
              v-if="getDatabaseType(db.path) === 'sqlite'" 
              @click="openMigrationModal(db)" 
              class="action-btn migrate-btn-new"
              title="Convert to encrypted SQLCipher"
            >
              <span class="btn-icon">🔒</span>
              <span class="btn-text">Migrate to SQLCipher</span>
            </button>
            
            <button 
              v-if="getDatabaseType(db.path) === 'sqlcipher'" 
              @click="openRekeyModal(db)" 
              class="action-btn rekey-btn-new"
              title="Change encryption password"
            >
              <span class="btn-icon">🔐</span>
              <span class="btn-text">Change Password</span>
            </button>

            <span 
              v-if="getDatabaseType(db.path) === 'unknown'" 
              class="no-action"
            >
              Unknown type
            </span>
          </div>
        </div>
      </div>
    </div> -->

    <!-- REPLACE THE ENTIRE MIGRATION TOOLS SECTION WITH THIS -->

    <!-- NEW: Compact Migration Tools Section -->
    <div v-if="connectedDatabases.length > 0" class="migration-tools-section">
      <h4>🔐 Migration & Encryption Tools</h4>
      <p class="section-subtitle">Convert SQLite to encrypted SQLCipher or change passwords</p>
      
      <div class="migration-list-compact">
        <div 
          v-for="db in connectedDatabases" 
          :key="db.path" 
          class="migration-row-compact"
        >
          <!-- Left: Database Info -->
          <div class="db-info-compact">
            <div class="db-name-row">
              <strong>{{ db.alias || db.name }}</strong>
              <span 
                class="type-badge-inline" 
                :class="getDatabaseType(db.path) === 'sqlite' ? 'badge-sqlite' : 'badge-sqlcipher'"
              >
                {{ getDatabaseType(db.path) === 'sqlite' ? '📄 SQLite' : '🔒 SQLCipher' }}
              </span>
            </div>
          </div>
          
          <!-- Right: Action Button -->
          <div class="action-compact">
            <button 
              v-if="getDatabaseType(db.path) === 'sqlite'" 
              @click="openMigrationModal(db)" 
              class="compact-btn migrate-btn-compact"
              title="Convert to encrypted SQLCipher"
            >
              🔒 Migrate
            </button>
            
            <button 
              v-if="getDatabaseType(db.path) === 'sqlcipher'" 
              @click="openRekeyModal(db)" 
              class="compact-btn rekey-btn-compact"
              title="Change encryption password"
            >
              🔐 Rekey
            </button>

            <span 
              v-if="getDatabaseType(db.path) === 'unknown'" 
              class="no-action"
            >
              Unknown
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Connected Databases List -->
    <div v-if="connectedDatabases.length > 0" class="connected-databases">
      <h4>Connected Databases ({{ connectedDatabases.length }})</h4>
      
     
      
      <!-- Database List -->
      <div class="database-list">
        <div v-for="(db, index) in connectedDatabases" :key="db.path" class="db-card">
          <div class="db-header">
            <div class="db-title">
              <span class="db-number">{{ index + 1 }}</span>
              <div class="db-info">
                <strong>{{ db.alias || db.name }}</strong>
                <span class="table-count">{{ db.table_count }} tables</span>
              </div>
            </div>
            
          </div>
          <div class="db-path">{{ db.path }}</div>
          <div class="db-actions">
              
              <button @click="removeDatabase(db.path)" class="remove-btn">Remove</button>
            </div>
        </div>
      </div>

     
    </div>

    <div v-else class="no-databases">
      <div class="empty-state">
        <h4>No Databases Connected</h4>
        <p>Connect at least 2 SQLCipher databases to start comparing schemas.</p>
      </div>
    </div>

    <!-- Migration Modal -->
    <!-- <MigrationModal
      v-if="selectedDatabaseForMigration"
      :is-open="showMigrationModal"
      :database-path="selectedDatabaseForMigration.path"
      :database-name="selectedDatabaseForMigration.alias || selectedDatabaseForMigration.name"
      :is-rekey="false"
      @close="showMigrationModal = false"
      @success="handleMigrationSuccess"
    /> -->

    <!-- Rekey Modal -->
    <!-- <MigrationModal
      v-if="selectedDatabaseForMigration"
      :is-open="showRekeyModal"
      :database-path="selectedDatabaseForMigration.path"
      :database-name="selectedDatabaseForMigration.alias || selectedDatabaseForMigration.name"
      :is-rekey="true"
      @close="showRekeyModal = false"
      @success="handleRekeySuccess"
    /> -->

     <!-- Add ConnectionModal to template -->
    <Teleport to="body">
      <ConnectionModal
        ref="connectionModalRef"
        :is-open="showConnectionModal"
        :database-path="pendingConnectionPath"
        @close="showConnectionModal = false"
        @connect="handleConnectionModalConnect"
      />
    </Teleport>
    <!-- Migration Modal with Teleport -->
    <Teleport to="body">
      <MigrationModal
        v-if="selectedDatabaseForMigration"
        :is-open="showMigrationModal"
        :database-path="selectedDatabaseForMigration.path"
        :database-name="selectedDatabaseForMigration.alias || selectedDatabaseForMigration.name"
        :is-rekey="false"
        @close="showMigrationModal = false"
        @success="handleMigrationSuccess"
      />
    </Teleport>

    <!-- Rekey Modal with Teleport -->
    <Teleport to="body">
      <MigrationModal
        v-if="selectedDatabaseForMigration"
        :is-open="showRekeyModal"
        :database-path="selectedDatabaseForMigration.path"
        :database-name="selectedDatabaseForMigration.alias || selectedDatabaseForMigration.name"
        :is-rekey="true"
        @close="showRekeyModal = false"
        @success="handleRekeySuccess"
    />
</Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import * as dialog from '@tauri-apps/plugin-dialog';
import MigrationModal from './MigrationModal.vue';
import ConnectionModal from './ConnectionModal.vue';

interface DatabaseInfo {
  path: string;
  name: string;
  table_count: number;
  is_connected: boolean;
  alias?: string;
  password?: string; 
}

const emit = defineEmits<{
  'database-connected': [db: DatabaseInfo];
  'select-database': [db: DatabaseInfo];
  'databases-ready': [databases: DatabaseInfo[]];
  'start-comparison': [databases: DatabaseInfo[]];
  'view-tables': [database: DatabaseInfo];
}>();

// State
// const dbPath = ref('');
// const password = ref('');
// const dbAlias = ref('');

const dbPath = ref<string>('');
//const password = ref<string>('');  // Even if input is commented, keep this
const dbAlias = ref<string>('');
const isConnecting = ref(false);
const error = ref('');
const successMessage = ref('');
const connectedDatabases = ref<DatabaseInfo[]>([]);

// Connection modal state
const showConnectionModal = ref(false);
const pendingConnectionPath = ref('');
const connectionModalRef = ref<any>(null);

// Migration state
const selectedDatabaseForMigration = ref<DatabaseInfo | null>(null);
const showMigrationModal = ref(false);
const showRekeyModal = ref(false);
const databaseTypes = ref<Map<string, string>>(new Map());

// Computed
const canCompare = computed(() => connectedDatabases.value.length >= 2);

// Database type detection helpers
const getDatabaseType = (dbPath: string): string => {
  const type = databaseTypes.value.get(dbPath) || 'unknown';
  return type;
};

const checkDatabaseType = async (dbPath: string) => {
  try {
    console.log('🔍 Checking database type for:', dbPath);
    const type = await invoke<string>('check_database_type', { dbPath });
    databaseTypes.value.set(dbPath, type);
    console.log('✅ Database type detected:', type, 'for', dbPath);
  } catch (error) {
    console.error('❌ Failed to check database type:', error);
    databaseTypes.value.set(dbPath, 'unknown');
  }
};

// Methods
const selectFile = async () => {
  try {
    const selected = await dialog.open({
      multiple: false,
      filters: [
        {
          name: 'Database Files',
          extensions: ['db', 'sqlite', 'sqlite3', 'db3']
        }
      ]
    });

    if (selected && typeof selected === 'string') {
      dbPath.value = selected;
    }
  } catch (err) {
    error.value = `Failed to open file dialog: ${err}`;
  }
};

// const handleConnect = async () => {
//   if (!dbPath.value) {
//     error.value = 'Please enter database path';
//     return;
//   }

//   // Auto-fix common path issues
//   let fixedPath = dbPath.value.trim();
  
//   // Handle tilde expansion
//   if (fixedPath.startsWith('~/')) {
//     const homeDir = await invoke<string>('get_home_directory').catch(() => '/home/user');
//     fixedPath = fixedPath.replace('~', homeDir);
//   }

//   // Clean up path
//   fixedPath = fixedPath.replace(/\\/g, '/');
  
//   isConnecting.value = true;
//   error.value = '';
//   successMessage.value = '';

//   try {
//     const response = await invoke<DatabaseInfo>('connect_database', {
//       path: fixedPath,  // Fixed: was dbPath
//       password: password.value || ''
//     });

//     const dbInfo: DatabaseInfo = {
//       ...response,
//       path: fixedPath,
//       is_connected: true
//     };
    
//     // Store password if provided
//     dbInfo.password = password.value || '';
//     // Add alias if provided
//     if (dbAlias.value.trim()) {
//       dbInfo.alias = dbAlias.value.trim();
//     }
    
//     connectedDatabases.value.push(dbInfo);
    
//     // Check database type (SQLite vs SQLCipher)
//     await checkDatabaseType(fixedPath);
    
//     const dbType = password.value ? 'SQLCipher (encrypted)' : 'SQLite (unencrypted)';
//     successMessage.value = `Successfully connected to ${dbInfo.alias || dbInfo.name} as ${dbType} (${dbInfo.table_count} tables)`;
//     emit('database-connected', dbInfo);
    
//     // Emit ready signal if we have enough databases
//     if (canCompare.value) {
//       emit('databases-ready', connectedDatabases.value);
//     }
    
//     // Clear form
//     dbPath.value = '';
//     password.value = '';
//     dbAlias.value = '';
    
//   } catch (err: any) {
//     error.value = `Failed to connect: ${err}`;
//   } finally {
//     isConnecting.value = false;
//   }
// };


const handleConnect = async () => {
  if (!dbPath.value) {
    error.value = 'Please enter database path';
    return;
  }

  // Auto-fix common path issues
  let fixedPath = dbPath.value.trim();
  
  // Handle tilde expansion
  if (fixedPath.startsWith('~/')) {
    const homeDir = await invoke<string>('get_home_directory').catch(() => '/home/user');
    fixedPath = fixedPath.replace('~', homeDir);
  }

  // Clean up path
  fixedPath = fixedPath.replace(/\\/g, '/');
  
  error.value = '';
  successMessage.value = '';

  try {
    // Check database type FIRST
    console.log('🔍 Checking database type for:', fixedPath);
    const dbType = await invoke<string>('check_database_type', { dbPath: fixedPath });
    console.log('📊 Database type:', dbType);

    if (dbType === 'sqlcipher') {
      // SQLCipher - Show modal for password + settings
      console.log('🔒 SQLCipher detected - showing connection modal');
      pendingConnectionPath.value = fixedPath;
      showConnectionModal.value = true;
    } else {
      // SQLite - Connect directly (no password needed)
      console.log('📄 SQLite detected - connecting directly');
      await connectDatabase(fixedPath, '', null);
    }
  } catch (err: any) {
    error.value = `Failed to check database type: ${err}`;
  }
};

const connectDatabase = async (path: string, password: string, settings: any) => {
  isConnecting.value = true;
  
  if (connectionModalRef.value) {
    connectionModalRef.value.setConnecting(true);
  }

  try {
    const response = await invoke<DatabaseInfo>('connect_database', {
      path: path,
      password: password,
      settings: settings // Can be null for SQLite
    });

    const dbInfo: DatabaseInfo = {
      ...response,
      path: path,
      is_connected: true
    };
    
    dbInfo.password = password || '';
    if (dbAlias.value.trim()) {
      dbInfo.alias = dbAlias.value.trim();
    }
    
    connectedDatabases.value.push(dbInfo);
    await checkDatabaseType(path);
    
    const dbType = password ? 'SQLCipher (encrypted)' : 'SQLite (unencrypted)';
    successMessage.value = `Successfully connected to ${dbInfo.alias || dbInfo.name} as ${dbType} (${dbInfo.table_count} tables)`;
    emit('database-connected', dbInfo);
    
    if (canCompare.value) {
      emit('databases-ready', connectedDatabases.value);
    }
    
    // Clear form
    dbPath.value = '';
    //password.value = '';
    dbAlias.value = '';
    
    
    // Close modal if open
    showConnectionModal.value = false;
    
  } catch (err: any) {
    error.value = `Failed to connect: ${err}`;
    
    if (connectionModalRef.value) {
      connectionModalRef.value.setError(err.toString());
    }
  } finally {
    isConnecting.value = false;
    
    if (connectionModalRef.value) {
      connectionModalRef.value.setConnecting(false);
    }
  }
};

const handleConnectionModalConnect = async (password: string, settings: any) => {
  await connectDatabase(pendingConnectionPath.value, password, settings);
};


// const selectDatabase = (db: DatabaseInfo) => {
//   emit('select-database', db);
// };

const removeDatabase = (path: string) => {
  connectedDatabases.value = connectedDatabases.value.filter(db => db.path !== path);
  databaseTypes.value.delete(path);
  
  if (connectedDatabases.value.length === 0) {
    successMessage.value = '';
  }
};

// const startComparison = () => {
//   emit('start-comparison', connectedDatabases.value);
// };

// const viewTables = (db: DatabaseInfo) => {
//   emit('view-tables', db);
// };

// Migration handlers
const openMigrationModal = (db: DatabaseInfo) => {
  selectedDatabaseForMigration.value = db;
  showMigrationModal.value = true;
};

const openRekeyModal = (db: DatabaseInfo) => {
  selectedDatabaseForMigration.value = db;
  showRekeyModal.value = true;
};

const handleMigrationSuccess = async (outputPath: string) => {
  showMigrationModal.value = false;
  successMessage.value = `✅ Migration successful! Encrypted database created:\n${outputPath}`;
  error.value = '';
};

const handleRekeySuccess = async (dbPath: string) => {
  showRekeyModal.value = false;
  successMessage.value = `✅ Database password changed successfully!`;
  error.value = '';
  await checkDatabaseType(dbPath);
};
</script>

<style scoped>
.database-connection {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}

h3 {
  margin: 0 0 8px 0;
  color: var(--text-primary);
  font-size: 1.75em;
}

.subtitle {
  color: var(--text-secondary);
  margin: 0 0 24px 0;
}

/* Connection Form */
.connection-form {
  background: var(--bg-card);
  border: 2px solid var(--border-primary);
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
  max-width: 100%; /* Ensure it doesn't overflow */
  box-sizing: border-box; /* Include padding in width calculation */
}

.connection-form h4 {
  margin: 0 0 20px 0;
  color: var(--text-primary);
  font-size: 1.2em;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: var(--text-primary);
  font-size: 0.95em;
}

.file-input-group {
  display: flex;
  gap: 12px;
  align-items: stretch; /* Make both elements same height */
  width: 100%; /* Ensure it takes full width */
}

.file-path-input {
  flex: 1;
  padding: 12px 16px;
  border: 2px solid #d1d5db;
  border-radius: 8px;
  font-size: 1em;
  min-width: 0; /* Important: allows input to shrink properly */
}

.file-path-input:focus {
  outline: none;
  border-color: #3b82f6;
}

.select-file-btn {
  padding: 12px 24px;
  background: #f3f4f6;
  border: 2px solid #d1d5db;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap; /* Prevent text wrapping */
  flex-shrink: 0; /* Prevent button from shrinking */
}

.select-file-btn:hover {
  background: #e5e7eb;
}

.password-input,
.alias-input {
  width: 100%;
  padding: 12px 16px;
  border: 2px solid #d1d5db;
  border-radius: 8px;
  font-size: 1em;
}

.password-input:focus,
.alias-input:focus {
  outline: none;
  border-color: #3b82f6;
}

.help-text {
  display: block;
  margin-top: 6px;
  color: var(--text-secondary);
  font-size: 0.875em;
}

.form-actions {
  margin-top: 24px;
}

.connect-btn {
  padding: 14px 32px;
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  font-size: 1em;
  cursor: pointer;
  transition: all 0.2s;
}

.connect-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
}

.connect-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-message {
  margin-top: 16px;
  padding: 12px 16px;
  background: #fef2f2;
  border: 1px solid #fca5a5;
  border-radius: 8px;
  color: #991b1b;
}

.success-message {
  margin-top: 16px;
  padding: 12px 16px;
  background: #f0fdf4;
  border: 1px solid #86efac;
  border-radius: 8px;
  color: #166534;
  white-space: pre-line;
}

/* Migration Tools Section - Compact Version */
.migration-tools-section {
  background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
  border: 2px solid var(--border-primary);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 24px;
}

.migration-tools-section h4 {
  margin: 0 0 6px 0;
  color: var(--text-primary);
  font-size: 1.2em;
}

.section-subtitle {
  color: var(--text-secondary);
  margin: 0 0 16px 0;
  font-size: 0.9em;
}

.migration-list-compact {
  background: var(--bg-card);
  border-radius: 8px;
  overflow: hidden;
  box-shadow: var(--shadow-sm);
}

.migration-row-compact {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border-primary);
  gap: 12px;
  transition: background 0.2s;
}

.migration-row-compact:hover {
  background: var(--bg-hover);
}

.migration-row-compact:last-child {
  border-bottom: none;
}

.db-info-compact {
  flex: 1;
  min-width: 0; /* Allow text truncation */
}

.db-name-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.db-name-row strong {
  color: var(--text-primary);
  font-size: 0.95em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.type-badge-inline {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 12px;
  font-size: 0.75em;
  font-weight: 600;
  white-space: nowrap;
}

.badge-sqlite {
  background: #dbeafe;
  color: #1e40af;
  border: 1px solid #93c5fd;
}

.badge-sqlcipher {
  background: #dcfce7;
  color: #166534;
  border: 1px solid #86efac;
}

/* Dark mode badges */
[data-theme="dark"] .badge-sqlite {
  background: #1e3a8a;
  color: #93c5fd;
  border-color: #3730a3;
}

[data-theme="dark"] .badge-sqlcipher {
  background: #064e3b;
  color: #86efac;
  border-color: #065f46;
}

.action-compact {
  display: flex;
  align-items: center;
}

.compact-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  font-size: 0.85em;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 6px;
}

.migrate-btn-compact {
  background: linear-gradient(135deg, #10b981, #059669);
  color: white;
  box-shadow: 0 2px 4px rgba(5, 150, 105, 0.2);
}

.migrate-btn-compact:hover {
  background: linear-gradient(135deg, #059669, #047857);
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(5, 150, 105, 0.3);
}

.rekey-btn-compact {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  color: white;
  box-shadow: 0 2px 4px rgba(217, 119, 6, 0.2);
}

.rekey-btn-compact:hover {
  background: linear-gradient(135deg, #d97706, #b45309);
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(217, 119, 6, 0.3);
}

.no-action {
  color: var(--text-tertiary);
  font-size: 0.85em;
  font-style: italic;
}

/* Responsive: Stack on very small screens */
@media (max-width: 600px) {
  .migration-row-compact {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  
  .compact-btn {
    width: 100%;
    justify-content: center;
  }
}


/* Connected Databases */
.connected-databases {
  background: var(--bg-card);
  border: 2px solid var(--border-primary);
  border-radius: 12px;
  padding: 24px;
}

.connected-databases h4 {
  margin: 0 0 20px 0;
  color: var(--text-primary);
  font-size: 1.2em;
}

.comparison-ready {
  margin-bottom: 20px;
  padding: 16px;
  background: #dbeafe;
  border: 2px solid #93c5fd;
  border-radius: 8px;
  text-align: center;
}

.compare-btn {
  padding: 12px 24px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.compare-btn:hover {
  background: #2563eb;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
}

.database-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.db-card {
  border: 2px solid var(--border-primary);
  border-radius: 8px;
  padding: 16px;
  transition: all 0.2s;
}

.db-card:hover {
  border-color: #cbd5e1;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.db-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.db-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.db-number {
  background: #3b82f6;
  color: white;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 0.9em;
}

.db-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.db-info strong {
  color: var(--text-primary);
  font-size: 1.05em;
}

.table-count {
  color: var(--text-secondary);
  font-size: 0.875em;
}

.db-card {
  display: flex;
  flex-direction: column;
}

.db-actions {
  margin-top: auto; /* Pushes the button to the bottom */
  padding-top: 12px;
  text-align: right; /* Aligns the button to the right */
}


.select-btn,
.remove-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  font-size: 0.9em;
  cursor: pointer;
  transition: all 0.2s;
}

.select-btn {
  background: #3b82f6;
  color: white;
}

.select-btn:hover {
  background: #2563eb;
}

.remove-btn {
  background: #ef4444;
  color: white;
}

.remove-btn:hover {
  background: #dc2626;
}

.db-path {
  color: var(--text-secondary);
  font-size: 0.875em;
  font-family: 'Courier New', monospace;
  word-break: break-all;
}

.comparison-actions {
  margin-top: 24px;
  text-align: center;
}

.compare-databases-btn {
  padding: 14px 32px;
  background: linear-gradient(135deg, #10b981, #059669);
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  font-size: 1em;
  cursor: pointer;
  transition: all 0.2s;
}

.compare-databases-btn:hover {
  background: linear-gradient(135deg, #059669, #047857);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(5, 150, 105, 0.3);
}

.no-databases {
  background: var(--bg-card);
  border: 2px dashed #d1d5db;
  border-radius: 12px;
  padding: 48px 24px;
  text-align: center;
}

.empty-state h4 {
  margin: 0 0 8px 0;
  color: var(--text-secondary);
  font-size: 1.2em;
}

.empty-state p {
  margin: 0;
  color: #9ca3af;
}
</style>