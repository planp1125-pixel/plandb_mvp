<template>
  <div v-if="show" class="license-modal-overlay" @click="onClose">
    <div class="license-modal" @click.stop>
      <div class="modal-header">
        <h2>🔐 License Activation</h2>
        <button v-if="allowClose" @click="onClose" class="close-btn">✕</button>
      </div>

      <div class="modal-content">
        <!-- Success Message -->
        <div v-if="successMessage" class="success-message">
          <div class="success-icon">✓</div>
          <p>{{ successMessage }}</p>
        </div>

        <!-- Error Message -->
        <div v-if="errorMessage" class="error-message">
          <p>{{ errorMessage }}</p>
        </div>

        <!-- License Form -->
        <div v-if="!successMessage" class="license-form">
          <div class="form-section">
            <h3>Option 1: Buy License</h3>
            <p class="description">Get full access to all features</p>
            <ul class="features-list">
              <li>✓ Compare unlimited databases</li>
              <li>✓ Export comparison results</li>
              <li>✓ Advanced filtering and search</li>
              <li>✓ Priority email support</li>
              <li>✓ All future updates included</li>
            </ul>
            <button @click="buyLicense" class="buy-btn">
              Buy License - $29/year
            </button>
          </div>

          <div class="divider">
            <span>OR</span>
          </div>

          <div class="form-section">
            <h3>Option 2: Activate Existing License</h3>
            <p class="description">Already purchased? Enter your details below:</p>

            <div class="input-group">
              <label for="email">Email Address</label>
              <input
                id="email"
                v-model="email"
                type="email"
                placeholder="your@email.com"
                :disabled="isVerifying"
                @keyup.enter="activateLicense"
              />
            </div>

            <div class="input-group">
              <label for="licenseKey">License Key</label>
              <input
                id="licenseKey"
                v-model="licenseKey"
                type="text"
                placeholder="XXXX-XXXX-XXXX-XXXX"
                :disabled="isVerifying"
                @keyup.enter="activateLicense"
              />
              <small>Check your email for your license key</small>
            </div>

            <button
              @click="activateLicense"
              :disabled="!email || !licenseKey || isVerifying"
              class="activate-btn"
            >
              {{ isVerifying ? 'Verifying...' : 'Activate License' }}
            </button>
          </div>
        </div>

        <!-- Trial Info (if allowed) -->
        <div v-if="allowTrial && !successMessage" class="trial-info">
          <p>
            <strong>Want to try first?</strong> You can use limited features without a license.
            <button @click="startTrial" class="trial-btn">Start Trial</button>
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { LicenseService } from '../services/licenseService';

const props = defineProps<{
  show: boolean;
  allowClose?: boolean;
  allowTrial?: boolean;
}>();

const emit = defineEmits<{
  'license-activated': [];
  'close': [];
  'start-trial': [];
}>();

const email = ref('');
const licenseKey = ref('');
const isVerifying = ref(false);
const errorMessage = ref('');
const successMessage = ref('');

const buyLicense = () => {
  LicenseService.openCheckout();
};

const activateLicense = async () => {
  if (!email.value || !licenseKey.value) {
    errorMessage.value = 'Please enter both email and license key';
    return;
  }

  isVerifying.value = true;
  errorMessage.value = '';
  successMessage.value = '';

  try {
    const result = await LicenseService.verifyLicense(
      email.value,
      licenseKey.value
    );

    if (result.isValid) {
      successMessage.value = `License activated successfully! (${result.licenseType})`;
      setTimeout(() => {
        emit('license-activated');
      }, 1500);
    } else {
      errorMessage.value = 'Invalid license key or email. Please check and try again.';
    }
  } catch (error) {
    errorMessage.value = 'Failed to verify license. Please check your internet connection.';
  } finally {
    isVerifying.value = false;
  }
};

const startTrial = () => {
  emit('start-trial');
};

const onClose = () => {
  if (props.allowClose) {
    emit('close');
  }
};
</script>

<style scoped>
.license-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(4px);
}

.license-modal {
  background: var(--bg-primary);
  border-radius: 12px;
  max-width: 600px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-header {
  padding: 24px 30px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-secondary);
  border-radius: 12px 12px 0 0;
}

.modal-header h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.5em;
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.5em;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 0;
  width: 32px;
  height: 32px;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.modal-content {
  padding: 30px;
}

.success-message {
  text-align: center;
  padding: 40px 20px;
  color: var(--success-color, #10b981);
}

.success-icon {
  font-size: 4em;
  margin-bottom: 16px;
  color: var(--success-color, #10b981);
}

.success-message p {
  font-size: 1.2em;
  margin: 0;
}

.error-message {
  background: #fee;
  border: 1px solid #fcc;
  color: #c33;
  padding: 12px 16px;
  border-radius: 6px;
  margin-bottom: 20px;
}

.error-message p {
  margin: 0;
}

.license-form {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.form-section h3 {
  margin: 0 0 8px 0;
  color: var(--text-primary);
  font-size: 1.1em;
}

.description {
  color: var(--text-secondary);
  margin: 0 0 16px 0;
  font-size: 0.9em;
}

.features-list {
  list-style: none;
  padding: 0;
  margin: 0 0 20px 0;
}

.features-list li {
  padding: 8px 0;
  color: var(--text-primary);
}

.input-group {
  margin-bottom: 16px;
}

.input-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 600;
  color: var(--text-primary);
  font-size: 0.9em;
}

.input-group input {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 1em;
  background: var(--bg-primary);
  color: var(--text-primary);
  box-sizing: border-box;
}

.input-group input:focus {
  outline: none;
  border-color: var(--primary-500, #6366f1);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.input-group input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.input-group small {
  display: block;
  margin-top: 6px;
  color: var(--text-secondary);
  font-size: 0.85em;
}

.buy-btn,
.activate-btn {
  width: 100%;
  padding: 14px 24px;
  border: none;
  border-radius: 8px;
  font-size: 1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.buy-btn {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: white;
}

.buy-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.3);
}

.activate-btn {
  background: var(--primary-500, #6366f1);
  color: white;
}

.activate-btn:hover:not(:disabled) {
  background: var(--primary-600, #4f46e5);
}

.activate-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.divider {
  text-align: center;
  position: relative;
  margin: 20px 0;
}

.divider::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 1px;
  background: var(--border-color);
}

.divider span {
  position: relative;
  background: var(--bg-primary);
  padding: 0 16px;
  color: var(--text-secondary);
  font-size: 0.9em;
}

.trial-info {
  margin-top: 30px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
  text-align: center;
  color: var(--text-secondary);
}

.trial-btn {
  background: none;
  border: 1px solid var(--border-color);
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-primary);
  margin-left: 8px;
  transition: all 0.2s;
}

.trial-btn:hover {
  background: var(--bg-secondary);
  border-color: var(--primary-500, #6366f1);
}
</style>
