<template>
  <div class="subscription-container">
    <!-- License Status Section -->
    <div class="settings-section">
      <div class="section-header">
        <h2>License & Subscription</h2>
        <p class="section-description">Manage your SQLCipher Tool license</p>
      </div>

      <div class="license-status-row">
        <div class="status-label">
          <span class="label-text">Status</span>
        </div>
        <div class="status-value">
          <span class="status-badge" :class="licenseStatus?.is_valid ? 'active' : 'inactive'">
            {{ licenseStatus?.is_valid ? '● Active' : '● Inactive' }}
          </span>
          <span class="status-text">{{ licenseStatus?.message || 'Checking...' }}</span>
        </div>
      </div>

      <div class="license-status-row" v-if="licenseStatus?.license_type">
        <div class="status-label">
          <span class="label-text">Plan</span>
        </div>
        <div class="status-value">
          <span class="plan-name">{{ licenseStatus.license_type }}</span>
          <span v-if="licenseStatus.days_remaining !== undefined" class="expires-text">
            {{ licenseStatus.license_type === 'Trial' ? `${licenseStatus.days_remaining} days remaining` :
               licenseStatus.license_type === 'Lifetime' ? 'Never expires' :
               `Renews in ${licenseStatus.days_remaining} days` }}
          </span>
        </div>
      </div>

      <div class="license-actions" v-if="licenseStatus?.is_valid">
        <button @click="refreshLicense" class="action-button secondary" :disabled="refreshing">
          {{ refreshing ? 'Checking...' : 'Refresh License' }}
        </button>
        <button @click="showDeactivate = true" class="action-button danger">
          Deactivate License
        </button>
      </div>
    </div>

    <!-- Activation Section -->
    <div class="settings-section" v-if="!licenseStatus?.is_valid || showActivation">
      <div class="section-header">
        <h3>Activate License</h3>
        <p class="section-description">Enter your license details to activate</p>
      </div>

      <div v-if="activationError" class="message-box error">
        {{ activationError }}
      </div>
      <div v-if="activationSuccess" class="message-box success">
        {{ activationSuccess }}
      </div>

      <div class="form-row">
        <label class="form-label">Email Address</label>
        <input
          v-model="email"
          type="email"
          placeholder="your@email.com"
          class="form-input"
          :disabled="activating"
        />
      </div>

      <div class="form-row">
        <label class="form-label">License Key</label>
        <input
          v-model="licenseKey"
          type="text"
          placeholder="XXXX-XXXX-XXXX-XXXX"
          class="form-input"
          :disabled="activating"
          @input="formatLicenseKey"
        />
        <span class="form-help">Format: XXXX-XXXX-XXXX-XXXX</span>
      </div>

      <div class="form-actions">
        <button
          @click="activateLicense"
          class="action-button primary"
          :disabled="!email || !licenseKey || activating"
        >
          {{ activating ? 'Activating...' : 'Activate License' }}
        </button>
      </div>

      <div class="help-links">
        <a @click="scrollToPricing" class="text-link">Don't have a license? Purchase below</a>
        <span class="separator">•</span>
        <a @click="openSupport" class="text-link">Lost your key? Contact support</a>
      </div>
    </div>

    <!-- Plans Section -->
    <div class="settings-section" ref="pricingSection">
      <div class="section-header">
        <h3>Available Plans</h3>
        <p class="section-description">Choose the plan that works best for you</p>
      </div>

      <div class="plans-list">
        <!-- Free Trial -->
        <div class="plan-item" :class="{ 'current': licenseStatus?.license_type === 'Trial' }">
          <div class="plan-header">
            <div class="plan-title">
              <h4>Free Trial</h4>
              <span v-if="licenseStatus?.license_type === 'Trial'" class="current-badge">CURRENT</span>
            </div>
            <div class="plan-price">
              <span class="price-amount">$0</span>
              <span class="price-period">/14 days</span>
            </div>
          </div>
          <p class="plan-description">Try all features for free, no credit card required</p>
          <button class="plan-button" disabled>
            {{ licenseStatus?.license_type === 'Trial' ? `Active (${licenseStatus.days_remaining} days left)` : 'Auto-started' }}
          </button>
        </div>

        <!-- Monthly -->
        <div class="plan-item" :class="{ 'current': licenseStatus?.license_type === 'Monthly' }">
          <div class="plan-header">
            <div class="plan-title">
              <h4>Monthly</h4>
              <span v-if="licenseStatus?.license_type === 'Monthly'" class="current-badge">CURRENT</span>
            </div>
            <div class="plan-price">
              <span class="price-amount">$9</span>
              <span class="price-period">/month</span>
            </div>
          </div>
          <p class="plan-description">Perfect for occasional use • Cancel anytime</p>
          <button
            @click="openPurchase('monthly')"
            class="plan-button"
            :disabled="licenseStatus?.license_type === 'Monthly'"
          >
            {{ licenseStatus?.license_type === 'Monthly' ? 'Current Plan' : 'Subscribe' }}
          </button>
        </div>

        <!-- Yearly (Recommended) -->
        <div class="plan-item recommended" :class="{ 'current': licenseStatus?.license_type === 'Yearly' }">
          <div class="plan-header">
            <div class="plan-title">
              <h4>Yearly</h4>
              <span v-if="licenseStatus?.license_type === 'Yearly'" class="current-badge">CURRENT</span>
              <span v-else class="recommended-badge">RECOMMENDED</span>
            </div>
            <div class="plan-price">
              <span class="price-amount">$99</span>
              <span class="price-period">/year</span>
            </div>
          </div>
          <p class="plan-description">Save 35% • Priority support • Just $3.25/month</p>
          <button
            @click="openPurchase('yearly')"
            class="plan-button primary"
            :disabled="licenseStatus?.license_type === 'Yearly'"
          >
            {{ licenseStatus?.license_type === 'Yearly' ? 'Current Plan' : 'Subscribe' }}
          </button>
        </div>

        <!-- Lifetime -->
        <div class="plan-item" :class="{ 'current': licenseStatus?.license_type === 'Lifetime' }">
          <div class="plan-header">
            <div class="plan-title">
              <h4>Lifetime</h4>
              <span v-if="licenseStatus?.license_type === 'Lifetime'" class="current-badge">CURRENT</span>
            </div>
            <div class="plan-price">
              <span class="price-amount">$199</span>
              <span class="price-period">one-time</span>
            </div>
          </div>
          <p class="plan-description">Pay once, use forever • All future updates included</p>
          <button
            @click="openPurchase('lifetime')"
            class="plan-button"
            :disabled="licenseStatus?.license_type === 'Lifetime'"
          >
            {{ licenseStatus?.license_type === 'Lifetime' ? 'Current Plan' : 'Purchase' }}
          </button>
        </div>
      </div>

      <div class="guarantee-notice">
        <span class="guarantee-icon">🛡️</span>
        <span>30-day money-back guarantee • All plans include email support</span>
      </div>
    </div>

    <!-- FAQ Section -->
    <div class="settings-section">
      <div class="section-header">
        <h3>Frequently Asked Questions</h3>
      </div>

      <div class="faq-list">
        <div
          class="faq-item"
          v-for="faq in faqs"
          :key="faq.id"
          @click="toggleFaq(faq.id)"
        >
          <div class="faq-question">
            <span>{{ faq.question }}</span>
            <span class="faq-chevron">{{ expandedFaqs.has(faq.id) ? '▼' : '▶' }}</span>
          </div>
          <div v-if="expandedFaqs.has(faq.id)" class="faq-answer">
            {{ faq.answer }}
          </div>
        </div>
      </div>
    </div>

    <!-- Support Section -->
    <div class="settings-section">
      <div class="support-row">
        <span>Need help?</span>
        <a @click="openSupport" class="text-link">Contact Support</a>
      </div>
    </div>

    <!-- Deactivation Modal -->
    <div v-if="showDeactivate" class="modal-overlay" @click="showDeactivate = false">
      <div class="modal-dialog" @click.stop>
        <h3>Deactivate License?</h3>
        <p>This will remove the license from this computer. You can reactivate it later on this or another device.</p>
        <div class="modal-actions">
          <button @click="showDeactivate = false" class="action-button secondary">Cancel</button>
          <button @click="confirmDeactivate" class="action-button danger" :disabled="deactivating">
            {{ deactivating ? 'Deactivating...' : 'Deactivate' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';

interface LicenseStatus {
  is_valid: boolean;
  license_type: 'Trial' | 'Monthly' | 'Yearly' | 'Lifetime';
  days_remaining?: number;
  message: string;
}

declare global {
  interface Window {
    Paddle: any;
  }
}

const licenseStatus = ref<LicenseStatus | null>(null);
const email = ref('');
const licenseKey = ref('');
const showActivation = ref(false);
const showDeactivate = ref(false);
const activating = ref(false);
const deactivating = ref(false);
const refreshing = ref(false);
const activationError = ref('');
const activationSuccess = ref('');
const expandedFaqs = ref<Set<number>>(new Set());
const pricingSection = ref<HTMLElement | null>(null);

const faqs = [
  {
    id: 1,
    question: 'How does the trial work?',
    answer: 'The free trial starts automatically when you first launch the app. You get full access to all features for 14 days with no credit card required.'
  },
  {
    id: 2,
    question: 'Can I cancel my subscription?',
    answer: 'Yes, cancel anytime. You keep access until the end of your billing period with no partial refunds.'
  },
  {
    id: 3,
    question: 'Where is my data stored?',
    answer: 'All your databases stay on your computer. Nothing is uploaded to any server. Your data remains completely private and local.'
  },
  {
    id: 4,
    question: 'Do you offer refunds?',
    answer: 'Yes, 30-day money-back guarantee for all purchases. Contact support for a full refund, no questions asked.'
  },
  {
    id: 5,
    question: 'Does this work offline?',
    answer: 'Yes! The app works fully offline. License validates once during activation, then checks every 7 days with a 30-day grace period.'
  },
  {
    id: 6,
    question: 'How many computers can I use?',
    answer: 'Use your license on up to 2 computers you personally own (e.g., work + home). Contact us about team licenses for more.'
  }
];

onMounted(async () => {
  await checkLicense();
});

const checkLicense = async () => {
  try {
    const status = await invoke<LicenseStatus>('get_license_status');
    licenseStatus.value = status;

    if (!status.is_valid && status.license_type !== 'Trial') {
      showActivation.value = true;
    }
  } catch (err) {
    console.error('License check failed:', err);
    activationError.value = 'Failed to check license status';
  }
};

const formatLicenseKey = (event: Event) => {
  const input = event.target as HTMLInputElement;
  let value = input.value.replace(/[^A-Za-z0-9]/g, '').toUpperCase();
  const formatted = value.match(/.{1,4}/g)?.join('-') || value;
  licenseKey.value = formatted.substring(0, 19);
};

const activateLicense = async () => {
  activating.value = true;
  activationError.value = '';
  activationSuccess.value = '';

  try {
    const status = await invoke<LicenseStatus>('activate_license', {
      email: email.value,
      licenseKey: licenseKey.value
    });

    licenseStatus.value = status;

    if (status.is_valid) {
      showActivation.value = false;
      activationSuccess.value = 'License activated successfully!';
      setTimeout(() => {
        email.value = '';
        licenseKey.value = '';
        activationSuccess.value = '';
      }, 3000);
    } else {
      activationError.value = 'Invalid license key or email';
    }
  } catch (err) {
    activationError.value = `Activation failed: ${err}`;
  } finally {
    activating.value = false;
  }
};

const confirmDeactivate = async () => {
  deactivating.value = true;
  try {
    await invoke('deactivate_license');
    licenseStatus.value = null;
    showDeactivate.value = false;
    showActivation.value = false;
    await checkLicense();
  } catch (err) {
    activationError.value = `Deactivation failed: ${err}`;
  } finally {
    deactivating.value = false;
  }
};

const refreshLicense = async () => {
  refreshing.value = true;
  await checkLicense();
  setTimeout(() => refreshing.value = false, 1000);
};

const openPurchase = async (plan: string) => {
  const paddlePriceIds: Record<string, string> = {
    monthly: 'pri_01k807azrz8asavdg42z0fqv34',
    yearly: 'pri_01k807dj8bv6h3gp9c1tvp6dfc',
    lifetime: 'pri_01k807ewanhdkvhb06wr24ffy9'
  };

  try {
    // Call our backend API to create a checkout session
    const response = await fetch('https://plandbdiff-license-02-1xyxdl83k-manikandans-projects-be37ef3a.vercel.app/api/create-checkout', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        priceId: paddlePriceIds[plan]
      })
    });

    const data = await response.json();

    if (data.checkoutUrl) {
      // Open the checkout URL in the user's default browser
      await open(data.checkoutUrl);
    } else {
      throw new Error('No checkout URL received');
    }
  } catch (error) {
    console.error('Failed to create checkout:', error);
    // Fallback: Open support email
    await open('mailto:support@planp.com?subject=Purchase SQLCipher Tool License&body=I would like to purchase a ' + plan + ' license. Price ID: ' + paddlePriceIds[plan]);
  }
};

const openSupport = () => {
  open('mailto:support@planp.com?subject=SQLCipher Tool - License Support');
};

const toggleFaq = (id: number) => {
  const newSet = new Set(expandedFaqs.value);
  newSet.has(id) ? newSet.delete(id) : newSet.add(id);
  expandedFaqs.value = newSet;
};

const scrollToPricing = () => {
  pricingSection.value?.scrollIntoView({ behavior: 'smooth' });
};
</script>

<style scoped>
.subscription-container {
  max-width: 900px;
  margin: 0 auto;
  padding: 32px 24px;
  color: var(--text-primary);
}

.settings-section {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 24px;
  margin-bottom: 16px;
}

.section-header {
  margin-bottom: 24px;
}

.section-header h2 {
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 4px 0;
  color: var(--text-primary);
}

.section-header h3 {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 4px 0;
  color: var(--text-primary);
}

.section-description {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}

/* License Status Rows */
.license-status-row {
  display: flex;
  align-items: start;
  padding: 12px 0;
  border-bottom: 1px solid var(--border-color);
}

.license-status-row:last-child {
  border-bottom: none;
}

.status-label {
  width: 120px;
  flex-shrink: 0;
}

.label-text {
  font-size: 13px;
  color: var(--text-secondary);
}

.status-value {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  font-size: 13px;
  font-weight: 500;
}

.status-badge.active {
  color: #10b981;
}

.status-badge.inactive {
  color: #f59e0b;
}

.status-text {
  font-size: 13px;
  color: var(--text-primary);
}

.plan-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.expires-text {
  font-size: 12px;
  color: var(--text-secondary);
}

.license-actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
}

/* Form Styles */
.form-row {
  margin-bottom: 20px;
}

.form-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 8px;
  color: var(--text-primary);
}

.form-input {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.form-input:focus {
  outline: none;
  border-color: #3b82f6;
}

.form-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-help {
  display: block;
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.form-actions {
  margin-top: 20px;
}

/* Buttons */
.action-button {
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.action-button:hover:not(:disabled) {
  background: var(--bg-hover);
}

.action-button.primary {
  background: #3b82f6;
  border-color: #3b82f6;
  color: white;
}

.action-button.primary:hover:not(:disabled) {
  background: #2563eb;
  border-color: #2563eb;
}

.action-button.secondary {
  background: var(--bg-tertiary);
}

.action-button.danger {
  color: #ef4444;
  border-color: #ef4444;
}

.action-button.danger:hover:not(:disabled) {
  background: #ef4444;
  color: white;
}

.action-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Messages */
.message-box {
  padding: 12px;
  border-radius: 4px;
  font-size: 13px;
  margin-bottom: 16px;
}

.message-box.error {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid #ef4444;
  color: #ef4444;
}

.message-box.success {
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid #10b981;
  color: #10b981;
}

/* Help Links */
.help-links {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
  font-size: 12px;
  color: var(--text-secondary);
}

.text-link {
  color: #3b82f6;
  cursor: pointer;
  text-decoration: none;
}

.text-link:hover {
  text-decoration: underline;
}

.separator {
  color: var(--border-color);
}

/* Plans List */
.plans-list {
  display: grid;
  gap: 12px;
}

.plan-item {
  padding: 16px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
  transition: all 0.2s;
}

.plan-item:hover {
  border-color: #3b82f6;
}

.plan-item.recommended {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.05);
}

.plan-item.current {
  border-color: #10b981;
  background: rgba(16, 185, 129, 0.05);
}

.plan-header {
  display: flex;
  justify-content: space-between;
  align-items: start;
  margin-bottom: 8px;
}

.plan-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plan-title h4 {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.current-badge,
.recommended-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.current-badge {
  background: #10b981;
  color: white;
}

.recommended-badge {
  background: #3b82f6;
  color: white;
}

.plan-price {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.price-amount {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
}

.price-period {
  font-size: 12px;
  color: var(--text-secondary);
}

.plan-description {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0 0 12px 0;
  line-height: 1.4;
}

.plan-button {
  width: 100%;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background: var(--bg-tertiary);
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.plan-button:hover:not(:disabled) {
  background: var(--bg-hover);
}

.plan-button.primary {
  background: #3b82f6;
  border-color: #3b82f6;
  color: white;
}

.plan-button.primary:hover:not(:disabled) {
  background: #2563eb;
}

.plan-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.guarantee-notice {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px;
  margin-top: 16px;
  border-radius: 6px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  font-size: 12px;
  color: var(--text-secondary);
}

.guarantee-icon {
  font-size: 16px;
}

/* FAQ */
.faq-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.faq-item {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.faq-item:hover {
  border-color: #3b82f6;
}

.faq-question {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.faq-chevron {
  font-size: 10px;
  color: var(--text-secondary);
}

.faq-answer {
  padding: 0 16px 14px 16px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

/* Support */
.support-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}

/* Modal */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.modal-dialog {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 24px;
  max-width: 450px;
  width: 90%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-dialog h3 {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: var(--text-primary);
}

.modal-dialog p {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin: 0 0 20px 0;
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}
</style>
