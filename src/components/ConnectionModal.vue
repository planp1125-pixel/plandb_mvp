<template>
  <Teleport to="body">
    <div v-if="isOpen" class="modal-overlay" @click="closeModal">
      <div class="modal-container" @click.stop>
        <div class="modal-header">
          <h2>🔒 Connect to SQLCipher Database</h2>
          <button class="close-btn" @click="closeModal">×</button>
        </div>

        <div class="modal-body">
          <div class="info-banner">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M13,9H11V7H13M13,17H11V11H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z" />
            </svg>
            <p>This database is encrypted with SQLCipher. Enter the password and settings to connect.</p>
          </div>

          <div class="info-section">
            <label class="info-label">Database Path:</label>
            <div class="info-value">{{ databasePath }}</div>
          </div>

          <!-- Password Input -->
          <div class="form-group">
            <label for="password">Password *</label>
            <div class="password-input-wrapper">
              <input
                id="password"
                :type="showPassword ? 'text' : 'password'"
                v-model="password"
                placeholder="Enter database password"
                class="form-input"
                @keyup.enter="handleConnect"
              />
              <button
                type="button"
                class="password-toggle"
                @click="showPassword = !showPassword"
              >
                {{ showPassword ? '👁️' : '👁️‍🗨️' }}
              </button>
            </div>
          </div>

          <!-- SQLCipher Settings -->
          <div class="advanced-settings">
            <button
              class="advanced-toggle"
              @click="showAdvanced = !showAdvanced"
              type="button"
            >
              ⚙️ SQLCipher Settings
              <span class="toggle-icon">{{ showAdvanced ? '▼' : '▶' }}</span>
            </button>

            <div v-if="showAdvanced" class="advanced-content">
              <p class="settings-help">Use the same settings that were used when this database was created.</p>
              
              <div class="form-group">
                <label for="pageSize">Page Size</label>
                <select id="pageSize" v-model="settings.page_size" class="form-select">
                  <option value="1024">1024 bytes</option>
                  <option value="2048">2048 bytes</option>
                  <option value="4096">4096 bytes (Default)</option>
                  <option value="8192">8192 bytes</option>
                  <option value="16384">16384 bytes</option>
                </select>
              </div>

              <div class="form-group">
                <label for="kdfIterations">KDF Iterations</label>
                <select id="kdfIterations" v-model="settings.kdf_iterations" class="form-select">
                  <option value="4000">4,000 (Fast)</option>
                  <option value="64000">64,000 (Default)</option>
                  <option value="256000">256,000 (Recommended)</option>
                  <option value="500000">500,000 (Very Secure)</option>
                </select>
              </div>

              <div class="form-group">
                <label for="hmacAlgorithm">HMAC Algorithm</label>
                <select id="hmacAlgorithm" v-model="settings.hmac_algorithm" class="form-select">
                  <option value="HMAC_SHA1">HMAC SHA1</option>
                  <option value="HMAC_SHA256">HMAC SHA256 (Default)</option>
                  <option value="HMAC_SHA512">HMAC SHA512</option>
                </select>
              </div>

              <div class="form-group">
                <label for="kdfAlgorithm">KDF Algorithm</label>
                <select id="kdfAlgorithm" v-model="settings.kdf_algorithm" class="form-select">
                  <option value="PBKDF2_HMAC_SHA1">PBKDF2 HMAC SHA1</option>
                  <option value="PBKDF2_HMAC_SHA256">PBKDF2 HMAC SHA256 (Default)</option>
                  <option value="PBKDF2_HMAC_SHA512">PBKDF2 HMAC SHA512</option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <div v-if="isConnecting" class="progress-section">
          <div class="progress-bar">
            <div class="progress-fill animate-pulse"></div>
          </div>
          <p class="progress-text">Connecting to database...</p>
        </div>

        <div v-if="errorMessage" class="error-banner">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M13,13H11V7H13M13,17H11V15H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z" />
          </svg>
          <div>
            <strong>Connection Failed</strong>
            <p>{{ errorMessage }}</p>
            <p class="error-hint">💡 Tip: Make sure password and settings match the database encryption.</p>
          </div>
        </div>

        <div class="modal-footer">
          <button
            class="btn btn-secondary"
            @click="closeModal"
            :disabled="isConnecting"
          >
            Cancel
          </button>
          <button
            class="btn btn-primary"
            @click="handleConnect"
            :disabled="!password || isConnecting"
          >
            {{ isConnecting ? 'Connecting...' : '🔒 Connect Database' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  isOpen: boolean;
  databasePath: string;
}

//const props = defineProps<Props>();

defineProps<Props>();
const emit = defineEmits<{
  close: [];
  connect: [password: string, settings: any];
}>();

const password = ref('');
const showPassword = ref(false);
const showAdvanced = ref(false);
const isConnecting = ref(false);
const errorMessage = ref('');

const settings = ref({
  page_size: '4096',
  kdf_iterations: '256000',
  hmac_algorithm: 'HMAC_SHA256',
  kdf_algorithm: 'PBKDF2_HMAC_SHA256'
});

const handleConnect = () => {
  if (!password.value) return;
  
  errorMessage.value = '';
  emit('connect', password.value, settings.value);
};

const closeModal = () => {
  if (!isConnecting.value) {
    password.value = '';
    showPassword.value = false;
    showAdvanced.value = false;
    errorMessage.value = '';
    emit('close');
  }
};

// Expose methods for parent to control
defineExpose({
  setConnecting: (val: boolean) => { isConnecting.value = val; },
  setError: (msg: string) => { errorMessage.value = msg; }
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999999;
  backdrop-filter: blur(4px);
  padding: 20px;
}

.modal-container {
  background: var(--bg-card);
  border-radius: 16px;
  max-width: 600px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 28px;
  border-bottom: 2px solid var(--border-primary);
}

.modal-header h2 {
  margin: 0;
  font-size: 1.5em;
  color: var(--text-primary);
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 32px;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.modal-body {
  padding: 28px;
}

.info-banner {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: #dbeafe;
  border: 1px solid #93c5fd;
  border-radius: 8px;
  margin-bottom: 24px;
  color: #1e40af;
}

[data-theme="dark"] .info-banner {
  background: #1e3a8a;
  border-color: #3730a3;
  color: #93c5fd;
}

.info-banner svg {
  flex-shrink: 0;
  margin-top: 2px;
}

.info-banner p {
  margin: 0;
  line-height: 1.5;
}

.info-section {
  margin-bottom: 20px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.info-label {
  font-size: 0.875em;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 4px;
  display: block;
}

.info-value {
  font-size: 0.9em;
  color: var(--text-primary);
  word-break: break-all;
  font-family: 'Courier New', monospace;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
  font-size: 0.95em;
}

.form-input,
.form-select {
  width: 100%;
  padding: 12px 14px;
  border: 2px solid var(--border-primary);
  border-radius: 8px;
  font-size: 1em;
  transition: all 0.2s;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.password-input-wrapper {
  position: relative;
}

.password-toggle {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.2em;
  padding: 4px 8px;
}

.advanced-settings {
  margin-top: 24px;
  border-top: 2px solid var(--border-primary);
  padding-top: 20px;
}

.advanced-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  color: var(--text-primary);
  font-size: 0.95em;
  transition: all 0.2s;
}

.advanced-toggle:hover {
  background: var(--bg-hover);
}

.toggle-icon {
  font-size: 0.8em;
}

.advanced-content {
  margin-top: 20px;
  padding: 20px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.settings-help {
  margin: 0 0 16px 0;
  padding: 12px;
  background: var(--bg-primary);
  border-left: 4px solid #6366f1;
  border-radius: 4px;
  font-size: 0.9em;
  color: var(--text-secondary);
}

.progress-section {
  padding: 20px 28px;
  border-top: 2px solid var(--border-primary);
}

.progress-bar {
  width: 100%;
  height: 12px;
  background: var(--bg-tertiary);
  border-radius: 6px;
  overflow: hidden;
  margin-bottom: 12px;
}

.progress-fill {
  height: 100%;
  width: 100%;
  background: linear-gradient(90deg, #3b82f6, #2563eb);
}

.animate-pulse {
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.progress-text {
  text-align: center;
  color: var(--text-secondary);
  font-size: 0.95em;
  margin: 0;
}

.error-banner {
  display: flex;
  gap: 12px;
  padding: 16px;
  border-radius: 8px;
  margin: 20px 28px;
  background: var(--error-bg);
  border: 1px solid var(--error-border);
  color: var(--error-text);
}

.error-banner svg {
  flex-shrink: 0;
  margin-top: 2px;
}

.error-banner p {
  margin: 4px 0;
}

.error-hint {
  font-size: 0.9em;
  opacity: 0.9;
  margin-top: 8px !important;
}

.modal-footer {
  display: flex;
  gap: 12px;
  padding: 20px 28px;
  border-top: 2px solid var(--border-primary);
  justify-content: flex-end;
}

.btn {
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  font-size: 0.95em;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--bg-hover);
}

.btn-primary {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
}
</style>