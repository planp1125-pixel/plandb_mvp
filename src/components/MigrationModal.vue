<template>
  <div v-if="isOpen" class="modal-overlay" @click="closeModal">
    <div class="modal-container" @click.stop>
      <div class="modal-header">
        <h2>{{ isRekey ? '🔐 Rekey SQLCipher Database' : '🔒 Migrate to SQLCipher' }}</h2>
        <button class="close-btn" @click="closeModal">×</button>
      </div>

      <div class="modal-body">
        <div v-if="!isRekey" class="info-banner">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M13,9H11V7H13M13,17H11V11H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z" />
          </svg>
          <p>This will create an encrypted copy of your SQLite database using SQLCipher. The original file will remain unchanged.</p>
        </div>

        <div v-else class="info-banner warning">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M13,14H11V10H13M13,18H11V16H13M1,21H23L12,2L1,21Z" />
          </svg>
          <p><strong>Warning:</strong> This will change the encryption key of the existing database file. Make sure you have a backup!</p>
        </div>

        <!-- Database Info -->
        <div class="info-section">
          <label class="info-label">Source Database:</label>
          <div class="info-value">{{ databaseName }}</div>
        </div>

        <!-- Password Input -->
        <div class="form-group">
          <label for="password">
            {{ isRekey ? 'New Encryption Password *' : 'Encryption Password *' }}
          </label>
          <div class="password-input-wrapper">
            <input
              id="password"
              :type="showPassword ? 'text' : 'password'"
              v-model="password"
              placeholder="Enter a strong password"
              class="form-input"
              @input="validatePassword"
            />
            <button
              type="button"
              class="password-toggle"
              @click="showPassword = !showPassword"
              :title="showPassword ? 'Hide password' : 'Show password'"
            >
              {{ showPassword ? '👁️' : '👁️‍🗨️' }}
            </button>
          </div>
          <div v-if="passwordError" class="error-message">{{ passwordError }}</div>
          <div v-if="password && !passwordError" class="password-strength">
            <div class="strength-bar" :class="passwordStrength.class" :style="{ width: passwordStrength.width }"></div>
            <span class="strength-text">{{ passwordStrength.text }}</span>
          </div>
        </div>

        <!-- Confirm Password -->
        <div class="form-group">
          <label for="confirmPassword">Confirm Password *</label>
          <input
            id="confirmPassword"
            :type="showPassword ? 'text' : 'password'"
            v-model="confirmPassword"
            placeholder="Re-enter password"
            class="form-input"
          />
          <div v-if="confirmPassword && password !== confirmPassword" class="error-message">
            Passwords do not match
          </div>
        </div>

        <!-- Old Password (for rekey only) -->
        <div v-if="isRekey" class="form-group">
          <label for="oldPassword">Current Password *</label>
          <input
            id="oldPassword"
            :type="showPassword ? 'text' : 'password'"
            v-model="oldPassword"
            placeholder="Enter current password"
            class="form-input"
          />
        </div>

        <!-- Advanced Settings -->
        <div class="advanced-settings">
          <button
            class="advanced-toggle"
            @click="showAdvanced = !showAdvanced"
            type="button"
          >
            ⚙️ Advanced Settings
            <span class="toggle-icon">{{ showAdvanced ? '▼' : '▶' }}</span>
          </button>

          <div v-if="showAdvanced" class="advanced-content">
            <!-- Page Size -->
            <div class="form-group">
              <label for="pageSize">Page Size</label>
              <select id="pageSize" v-model="settings.page_size" class="form-select">
                <option value="1024">1024 bytes</option>
                <option value="2048">2048 bytes</option>
                <option value="4096">4096 bytes (Default)</option>
                <option value="8192">8192 bytes</option>
                <option value="16384">16384 bytes</option>
                <option value="32768">32768 bytes</option>
              </select>
              <small class="form-hint">Larger page sizes can improve performance but increase file size</small>
            </div>

            <!-- KDF Iterations -->
            <div class="form-group">
              <label for="kdfIterations">KDF Iterations</label>
              <select id="kdfIterations" v-model="settings.kdf_iterations" class="form-select">
                <option value="4000">4,000 (Fast, Less Secure)</option>
                <option value="64000">64,000 (Default)</option>
                <option value="256000">256,000 (Recommended)</option>
                <option value="500000">500,000 (Very Secure)</option>
              </select>
              <small class="form-hint">Higher iterations = more secure but slower to open</small>
            </div>

            <!-- HMAC Algorithm -->
            <div class="form-group">
              <label for="hmacAlgorithm">HMAC Algorithm</label>
              <select id="hmacAlgorithm" v-model="settings.hmac_algorithm" class="form-select">
                <option value="HMAC_SHA1">HMAC SHA1 (SQLCipher 1-3)</option>
                <option value="HMAC_SHA256">HMAC SHA256 (Default)</option>
                <option value="HMAC_SHA512">HMAC SHA512 (Most Secure)</option>
              </select>
            </div>

            <!-- KDF Algorithm -->
            <div class="form-group">
              <label for="kdfAlgorithm">KDF Algorithm</label>
              <select id="kdfAlgorithm" v-model="settings.kdf_algorithm" class="form-select">
                <option value="PBKDF2_HMAC_SHA1">PBKDF2 HMAC SHA1</option>
                <option value="PBKDF2_HMAC_SHA256">PBKDF2 HMAC SHA256 (Default)</option>
                <option value="PBKDF2_HMAC_SHA512">PBKDF2 HMAC SHA512</option>
              </select>
            </div>

            <!-- Cipher -->
            <div class="form-group">
              <label for="cipher">Cipher</label>
              <select id="cipher" v-model="settings.cipher" class="form-select">
                <option value="aes-256-cbc">AES-256-CBC (Default)</option>
                <option value="aes-128-cbc">AES-128-CBC</option>
              </select>
            </div>
          </div>
        </div>
      </div>

      <!-- Progress Section -->
      <div v-if="isProcessing" class="progress-section">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
        </div>
        <p class="progress-text">{{ progressMessage }}</p>
      </div>

      <!-- Success Message -->
      <div v-if="successMessage" class="success-banner">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12,2A10,10 0 0,1 22,12A10,10 0 0,1 12,22A10,10 0 0,1 2,12A10,10 0 0,1 12,2M11,16.5L18,9.5L16.59,8.09L11,13.67L7.91,10.59L6.5,12L11,16.5Z" />
        </svg>
        <div>
          <strong>Success!</strong>
          <p>{{ successMessage }}</p>
          <p class="file-path">{{ outputPath }}</p>
        </div>
      </div>

      <!-- Error Message -->
      <div v-if="errorMessage" class="error-banner">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M13,13H11V7H13M13,17H11V15H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z" />
        </svg>
        <div>
          <strong>Error</strong>
          <p>{{ errorMessage }}</p>
        </div>
      </div>

      <div class="modal-footer">
        <button
          class="btn btn-secondary"
          @click="closeModal"
          :disabled="isProcessing"
        >
          Cancel
        </button>
        <button
          class="btn btn-primary"
          @click="startMigration"
          :disabled="!canMigrate || isProcessing"
        >
          {{ isProcessing ? 'Processing...' : (isRekey ? '🔐 Rekey Database' : '🔒 Migrate to SQLCipher') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed,watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';


interface Props {
  isOpen: boolean;
  databasePath: string;
  databaseName: string;
  isRekey?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isRekey: false
});

const emit = defineEmits<{
  close: [];
  success: [path: string];
}>();

// Form state
const password = ref('');
const confirmPassword = ref('');
const oldPassword = ref('');
const showPassword = ref(false);
const showAdvanced = ref(false);
const passwordError = ref('');

// Processing state
const isProcessing = ref(false);
const progress = ref(0);
const progressMessage = ref('');
const successMessage = ref('');
const errorMessage = ref('');
const outputPath = ref('');

// Settings
const settings = ref({
  page_size: '4096',           // Changed
  kdf_iterations: '256000',    // Changed
  hmac_algorithm: 'HMAC_SHA256', // Changed
  kdf_algorithm: 'PBKDF2_HMAC_SHA256', // Changed
  cipher: 'aes-256-cbc'
});

// Password validation
const validatePassword = () => {
  passwordError.value = '';
  
  if (password.value.length < 8) {
    passwordError.value = 'Password must be at least 8 characters long';
  } else if (!/[A-Z]/.test(password.value)) {
    passwordError.value = 'Password must contain at least one uppercase letter';
  } else if (!/[a-z]/.test(password.value)) {
    passwordError.value = 'Password must contain at least one lowercase letter';
  } else if (!/[0-9]/.test(password.value)) {
    passwordError.value = 'Password must contain at least one number';
  }
};

// Password strength indicator
const passwordStrength = computed(() => {
  const pwd = password.value;
  let strength = 0;
  
  if (pwd.length >= 8) strength++;
  if (pwd.length >= 12) strength++;
  if (/[a-z]/.test(pwd) && /[A-Z]/.test(pwd)) strength++;
  if (/[0-9]/.test(pwd)) strength++;
  if (/[^a-zA-Z0-9]/.test(pwd)) strength++;
  
  const levels = [
    { class: 'weak', width: '20%', text: 'Weak' },
    { class: 'weak', width: '40%', text: 'Weak' },
    { class: 'medium', width: '60%', text: 'Medium' },
    { class: 'strong', width: '80%', text: 'Strong' },
    { class: 'very-strong', width: '100%', text: 'Very Strong' }
  ];
  
  return levels[strength] || levels[0];
});

// Check if can migrate
const canMigrate = computed(() => {
  if (!password.value || !confirmPassword.value) return false;
  if (password.value !== confirmPassword.value) return false;
  if (passwordError.value) return false;
  if (props.isRekey && !oldPassword.value) return false;
  return true;
});

// Start migration
const startMigration = async () => {
  errorMessage.value = '';
  successMessage.value = '';
  isProcessing.value = true;
  progress.value = 0;

  try {
    if (props.isRekey) {
      // Rekey existing database
      progressMessage.value = 'Rekeying database...';
      progress.value = 30;

      // const _result = await invoke<{ success: boolean; message: string }>('rekey_sqlcipher_database', {

        await invoke<{ success: boolean; message: string }>('rekey_sqlcipher_database', {
        dbPath: props.databasePath,
        oldPassword: oldPassword.value,
        newPassword: password.value,
        settings: settings.value
      });

      progress.value = 100;
      successMessage.value = 'Database successfully rekeyed!';
      outputPath.value = props.databasePath;

      setTimeout(() => {
        emit('success', props.databasePath);
        closeModal();
      }, 2000);

    } else {
      // Migrate to new encrypted file
      progressMessage.value = 'Creating encrypted database...';
      progress.value = 20;

      const result = await invoke<{ outputPath: string; message: string }>('migrate_to_sqlcipher', {
        sourcePath: props.databasePath,
        password: password.value,
        settings: settings.value
      });

      progress.value = 60;
      progressMessage.value = 'Verifying encrypted database...';

      // Simulate verification delay
      await new Promise(resolve => setTimeout(resolve, 500));

      progress.value = 100;
      successMessage.value = 'Migration completed successfully!';
      outputPath.value = result.outputPath;

      setTimeout(() => {
        emit('success', result.outputPath);
        closeModal();
      }, 2000);
    }

  } catch (error: any) {
    errorMessage.value = error.toString();
    isProcessing.value = false;
  }
};

// Close modal
const closeModal = () => {
  if (!isProcessing.value) {
    // Reset form
    password.value = '';
    confirmPassword.value = '';
    oldPassword.value = '';
    showPassword.value = false;
    showAdvanced.value = false;
    passwordError.value = '';
    successMessage.value = '';
    errorMessage.value = '';
    outputPath.value = '';
    isProcessing.value = false;  
    progress.value = 0;           
    progressMessage.value = '';  
    emit('close');
  }
};

watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    // Reset state when modal opens
    password.value = '';
    confirmPassword.value = '';
    oldPassword.value = '';
    isProcessing.value = false;
    progress.value = 0;
    successMessage.value = '';
    errorMessage.value = '';
  }
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
  background: white;
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
  border-bottom: 2px solid #e5e7eb;
}

.modal-header h2 {
  margin: 0;
  font-size: 1.5em;
  color: #1f2937;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 32px;
  color: #6b7280;
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
  background: #f3f4f6;
  color: #1f2937;
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

.info-banner.warning {
  background: #fef3c7;
  border-color: #fcd34d;
  color: #92400e;
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
  background: #f9fafb;
  border-radius: 8px;
}

.info-label {
  font-size: 0.875em;
  font-weight: 600;
  color: #6b7280;
  margin-bottom: 4px;
  display: block;
}

.info-value {
  font-size: 0.95em;
  color: #1f2937;
  word-break: break-all;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  font-weight: 600;
  color: #374151;
  margin-bottom: 8px;
  font-size: 0.95em;
}

.form-input,
.form-select {
  width: 100%;
  padding: 12px 14px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 1em;
  transition: all 0.2s;
  background: white;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
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
  border-radius: 4px;
  transition: background 0.2s;
}

.password-toggle:hover {
  background: #f3f4f6;
}

.password-strength {
  margin-top: 8px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.strength-bar {
  height: 6px;
  border-radius: 3px;
  transition: all 0.3s;
  flex: 1;
}

.strength-bar.weak {
  background: #ef4444;
}

.strength-bar.medium {
  background: #f59e0b;
}

.strength-bar.strong {
  background: #10b981;
}

.strength-bar.very-strong {
  background: #059669;
}

.strength-text {
  font-size: 0.875em;
  font-weight: 600;
  min-width: 80px;
}

.error-message {
  color: #dc2626;
  font-size: 0.875em;
  margin-top: 6px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.error-message::before {
  content: '⚠️';
}

.form-hint {
  display: block;
  color: #6b7280;
  font-size: 0.825em;
  margin-top: 6px;
}

.advanced-settings {
  margin-top: 24px;
  border-top: 2px solid #e5e7eb;
  padding-top: 20px;
}

.advanced-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  color: #374151;
  font-size: 0.95em;
  transition: all 0.2s;
}

.advanced-toggle:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
}

.toggle-icon {
  font-size: 0.8em;
}

.advanced-content {
  margin-top: 20px;
  padding: 20px;
  background: #f9fafb;
  border-radius: 8px;
  border: 1px solid #e5e7eb;
}

.progress-section {
  padding: 20px 28px;
  border-top: 2px solid #e5e7eb;
  border-bottom: 2px solid #e5e7eb;
}

.progress-bar {
  width: 100%;
  height: 12px;
  background: #e5e7eb;
  border-radius: 6px;
  overflow: hidden;
  margin-bottom: 12px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #2563eb);
  transition: width 0.3s ease;
  border-radius: 6px;
}

.progress-text {
  text-align: center;
  color: #6b7280;
  font-size: 0.95em;
  margin: 0;
}

.success-banner,
.error-banner {
  display: flex;
  gap: 12px;
  padding: 16px;
  border-radius: 8px;
  margin: 20px 28px;
}

.success-banner {
  background: #d1fae5;
  border: 1px solid #6ee7b7;
  color: #065f46;
}

.error-banner {
  background: #fee2e2;
  border: 1px solid #fca5a5;
  color: #991b1b;
}

.success-banner svg,
.error-banner svg {
  flex-shrink: 0;
  margin-top: 2px;
}

.success-banner strong,
.error-banner strong {
  display: block;
  margin-bottom: 4px;
  font-size: 1.05em;
}

.success-banner p,
.error-banner p {
  margin: 4px 0;
  line-height: 1.5;
}

.file-path {
  font-family: monospace;
  font-size: 0.875em;
  background: rgba(0, 0, 0, 0.1);
  padding: 6px 10px;
  border-radius: 4px;
  margin-top: 8px !important;
  word-break: break-all;
}

.modal-footer {
  display: flex;
  gap: 12px;
  padding: 20px 28px;
  border-top: 2px solid #e5e7eb;
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
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: #f3f4f6;
  color: #374151;
}

.btn-secondary:hover:not(:disabled) {
  background: #e5e7eb;
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

/* Scrollbar Styling */
.modal-container::-webkit-scrollbar {
  width: 10px;
}

.modal-container::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.modal-container::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 5px;
}

.modal-container::-webkit-scrollbar-thumb:hover {
  background: #94a3b8;
}
</style>