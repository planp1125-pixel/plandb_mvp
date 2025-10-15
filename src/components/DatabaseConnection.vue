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
          <!-- <button @click="selectFile" class="select-file-btn" disabled>Browse</button> -->
           <button @click="selectFile" class="select-file-btn">Browse</button>
        </div>
        
      </div>

      <div class="form-group">
        <label for="password">Password (Optional for regular SQLite):</label>
        <input 
          id="password"
          v-model="password" 
          type="password" 
          placeholder="Leave empty for unencrypted SQLite, or enter SQLCipher password"
          class="password-input"
        />
        
      </div>

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

        <!-- <button 
          @click="testBackend" 
          class="test-btn"
        >
          Test Backend
        </button> -->
      </div>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <div v-if="successMessage" class="success-message">
        {{ successMessage }}
      </div>
    </div>

    <!-- Connected Databases List -->
    <div v-if="connectedDatabases.length > 0" class="connected-databases">
      <h4>Connected Databases ({{ connectedDatabases.length }})</h4>
      
      <!-- Comparison Status -->
      <div class="comparison-status">
        <div class="status-item">
          <span class="status-label">Ready for comparison:</span>
          <span :class="['status-value', canCompare ? 'ready' : 'waiting']">
            {{ canCompare ? 'YES' : `Need ${2 - connectedDatabases.length} more database(s)` }}
          </span>
        </div>
      </div>

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
            <div class="db-actions">
              <button @click="selectDatabase(db)" class="select-btn">Select</button>
              <button @click="removeDatabase(db.path)" class="remove-btn">Remove</button>
            </div>
          </div>
          <div class="db-path">{{ db.path }}</div>
        </div>
      </div>

      <!-- Quick Actions -->
      <div v-if="canCompare" class="quick-actions">
        <h5>Quick Actions</h5>
        <div class="action-buttons">
          <button @click="startComparison" class="action-btn comparison-btn">
            🔄 Compare All Databases
          </button>
          <button @click="viewTables" class="action-btn view-btn">
            👁️ View Database Tables
          </button>
        </div>
      </div>
    </div>

    <div v-else class="no-databases">
      <div class="empty-state">
        <h4>No Databases Connected</h4>
        <p>Connect at least 2 SQLCipher databases to start comparing schemas.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
// import { open } from '@tauri-apps/plugin-dialog';
import * as dialog from '@tauri-apps/plugin-dialog';

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
const dbPath = ref('');
const password = ref('');
const dbAlias = ref('');
const isConnecting = ref(false);
const error = ref('');
const successMessage = ref('');
const connectedDatabases = ref<DatabaseInfo[]>([]);

// Computed
const canCompare = computed(() => connectedDatabases.value.length >= 2);

// Methods
// const selectFile = async () => {
//   // Placeholder for future file dialog implementation
//   alert('File browser not available yet. Please enter the full path manually.');
// };
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

const handleConnect = async () => {
  if (!dbPath.value) {
    error.value = 'Please enter database path';
    return;
  }

  // Auto-fix common path issues
  let fixedPath = dbPath.value.trim();
  
  // Add leading slash if missing (Linux/Mac paths)
  if (!fixedPath.startsWith('/') && !fixedPath.startsWith('~') && !fixedPath.match(/^[A-Za-z]:\\/)) {
    fixedPath = '/' + fixedPath;
    dbPath.value = fixedPath; // Update the input field
  }

  // Check if already connected
  const existing = connectedDatabases.value.find(db => db.path === fixedPath);
  if (existing) {
    error.value = 'This database is already connected';
    return;
  }

  isConnecting.value = true;
  error.value = '';
  successMessage.value = '';

  try {
    const dbInfo = await invoke('connect_database', {
      path: fixedPath,
      password: password.value || '' // Send empty string if no password
    }) as DatabaseInfo;
    
    dbInfo.password = password.value || '';
    // Add alias if provided
    if (dbAlias.value.trim()) {
      dbInfo.alias = dbAlias.value.trim();
    }
    
    connectedDatabases.value.push(dbInfo);
    
    const dbType = password.value ? 'SQLCipher (encrypted)' : 'SQLite (unencrypted)';
    successMessage.value = `Successfully connected to ${dbInfo.alias || dbInfo.name} as ${dbType} (${dbInfo.table_count} tables)`;
    emit('database-connected', dbInfo);
    
    // Emit ready signal if we have enough databases
    if (canCompare.value) {
      emit('databases-ready', connectedDatabases.value);
    }
    
    // Clear form
    dbPath.value = '';
    password.value = '';
    dbAlias.value = '';
    
  } catch (err) {
    const errorMsg = String(err);
    
    if (errorMsg.includes('appears to be encrypted')) {
      error.value = '🔐 This database is encrypted (SQLCipher). Please enter the password.';
    } else if (errorMsg.includes('wrong password')) {
      error.value = '❌ Wrong password for SQLCipher database. Please check your password.';
    } else if (errorMsg.includes('unable to open database file')) {
      error.value = `❌ Cannot open database file: ${errorMsg}\n\n💡 Tips:\n• Check if the file path is correct\n• Ensure the file exists\n• Verify you have read permissions`;
    } else {
      error.value = `Connection failed: ${errorMsg}`;
    }
  } finally {
    isConnecting.value = false;
  }
};

// const testBackend = async () => {
//   try {
//     const result = await invoke('test_connection') as string;
//     successMessage.value = `Backend test: ${result}`;
//     error.value = '';
//   } catch (err) {
//     error.value = `Backend test failed: ${err}`;
//     successMessage.value = '';
//   }
// };

const selectDatabase = (database: DatabaseInfo) => {
  emit('select-database', database);
};

const removeDatabase = (path: string) => {
  const index = connectedDatabases.value.findIndex(db => db.path === path);
  if (index > -1) {
    const removed = connectedDatabases.value.splice(index, 1)[0];
    successMessage.value = `Removed ${removed.alias || removed.name}`;
    error.value = '';
  }
};

const startComparison = () => {
  if (canCompare.value) {
    emit('start-comparison', connectedDatabases.value);
  }
};

const viewTables = () => {
  if (connectedDatabases.value.length > 0) {
    emit('view-tables', connectedDatabases.value[0]);
  }
};
</script>

<style scoped>
.database-connection {
  padding: 20px;
}

.subtitle {
  color: #6c757d;
  margin-bottom: 25px;
  font-size: 0.95em;
}

.connection-form {
  background: #f8f9fa;
  padding: 25px;
  border-radius: 8px;
  margin-bottom: 30px;
  border: 1px solid #dee2e6;
}

.connection-form h4 {
  margin-top: 0;
  margin-bottom: 20px;
  color: #343a40;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 600;
  color: #495057;
}

.help-text {
  display: block;
  margin-top: 8px;
  color: #6c757d;
  font-size: 0.85em;
  line-height: 1.4;
}

.help-text code {
  background: #f1f3f4;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 0.9em;
  color: #495057;
}

.file-input-group {
  display: flex;
  gap: 10px;
}

.file-path-input,
.password-input,
.alias-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid #ced4da;
  border-radius: 6px;
  font-size: 0.95em;
  transition: border-color 0.2s;
}

.file-path-input {
  flex: 1;
}

.file-path-input:focus,
.password-input:focus,
.alias-input:focus {
  outline: none;
  border-color: #007bff;
  box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.25);
}

.select-file-btn {
  padding: 10px 16px;
  background: #007bff;  /* Change from #6c757d */
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;      /* Change from cursor: not-allowed */
  font-size: 0.9em;
  transition: background-color 0.2s;
}

.select-file-btn:hover {
  background: #0056b3;  /* Add hover effect */
}


.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 25px;
}

.connect-btn,
.test-btn {
  padding: 12px 24px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  font-size: 0.95em;
  transition: background-color 0.2s;
}

.connect-btn {
  background: #28a745;
  color: white;
}

.connect-btn:hover:not(:disabled) {
  background: #218838;
}

.connect-btn:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.test-btn {
  background: #17a2b8;
  color: white;
}

.test-btn:hover {
  background: #138496;
}

.error-message {
  margin-top: 15px;
  padding: 12px;
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
  border-radius: 6px;
}

.success-message {
  margin-top: 15px;
  padding: 12px;
  background: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
  border-radius: 6px;
}

.connected-databases {
  background: white;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  padding: 20px;
}

.connected-databases h4 {
  margin-top: 0;
  margin-bottom: 20px;
  color: #343a40;
}

.comparison-status {
  background: #e3f2fd;
  padding: 15px;
  border-radius: 6px;
  margin-bottom: 20px;
  border-left: 4px solid #2196f3;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-label {
  font-weight: 500;
  color: #1565c0;
}

.status-value {
  font-weight: 600;
}

.status-value.ready {
  color: #2e7d32;
}

.status-value.waiting {
  color: #f57c00;
}

.database-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.db-card {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  padding: 15px;
}

.db-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.db-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.db-number {
  background: #007bff;
  color: white;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 0.9em;
}

.db-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.db-info strong {
  color: #495057;
}

.table-count {
  font-size: 0.85em;
  color: #6c757d;
}

.db-actions {
  display: flex;
  gap: 8px;
}

.select-btn,
.remove-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
  font-weight: 500;
}

.select-btn {
  background: #007bff;
  color: white;
}

.select-btn:hover {
  background: #0056b3;
}

.remove-btn {
  background: #dc3545;
  color: white;
}

.remove-btn:hover {
  background: #c82333;
}

.db-path {
  font-family: monospace;
  font-size: 0.8em;
  color: #6c757d;
  background: #e9ecef;
  padding: 6px 10px;
  border-radius: 4px;
  word-break: break-all;
}

.quick-actions {
  margin-top: 25px;
  padding: 20px;
  background: #f1f8ff;
  border-radius: 8px;
  border: 1px solid #bee5eb;
}

.quick-actions h5 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #0c5460;
}

.action-buttons {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.action-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  font-size: 0.9em;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
}

.comparison-btn {
  background: #28a745;
  color: white;
}

.comparison-btn:hover {
  background: #218838;
  transform: translateY(-1px);
}

.view-btn {
  background: #17a2b8;
  color: white;
}

.view-btn:hover {
  background: #138496;
  transform: translateY(-1px);
}

.no-databases {
  text-align: center;
  padding: 60px 20px;
}

.empty-state {
  background: #f8f9fa;
  padding: 40px;
  border-radius: 8px;
  border: 2px dashed #dee2e6;
}

.empty-state h4 {
  color: #6c757d;
  margin-bottom: 10px;
}

.empty-state p {
  color: #6c757d;
  margin: 0;
}

/* Responsive Design */
@media (max-width: 768px) {
  .db-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .db-actions {
    align-self: stretch;
    justify-content: space-between;
  }

  .action-buttons {
    flex-direction: column;
  }

  .action-btn {
    justify-content: center;
  }
}
</style>