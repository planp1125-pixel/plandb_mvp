<template>
  <div class="content-panel">
    <div class="content-header">
      <h2>License & Activation</h2>
      <p>Manage your SQLCipher Tool license</p>
    </div>
    
    <div class="license-content">
      <!-- License Status Card -->
      <div class="license-status-card" :class="licenseStatus?.is_valid ? 'valid' : 'expired'">
        <div class="status-icon">
          {{ licenseStatus?.is_valid ? '✅' : '⚠️' }}
        </div>
        <div class="status-info">
          <h3>{{ licenseStatus?.message || 'Loading...' }}</h3>
          <p v-if="licenseStatus?.license_type === 'Trial' && licenseStatus?.days_remaining !== undefined">
            {{ licenseStatus.days_remaining }} days remaining in trial
          </p>
          <p v-else-if="licenseStatus?.license_type === 'Lifetime'">
            Thank you for your purchase! 🎉
          </p>
          <p v-else-if="licenseStatus?.is_valid && licenseStatus?.days_remaining !== undefined">
            Renews in {{ licenseStatus.days_remaining }} days
          </p>
        </div>
        <div class="status-actions" v-if="licenseStatus?.is_valid && licenseStatus?.license_type !== 'Trial'">
          <button @click="refreshLicense" class="btn btn-outline" :disabled="refreshing">
            {{ refreshing ? 'Refreshing...' : '🔄 Refresh' }}
          </button>
          <button @click="showDeactivate = true" class="btn btn-outline-danger">
            Deactivate
          </button>
        </div>
      </div>

      <!-- Activation Form -->
      <div v-if="!licenseStatus?.is_valid || showActivation" class="activation-section"></div>
      <!-- disabled for testing  -->
      <!-- <div v-if="(!licenseStatus?.is_valid || showActivation) && licenseStatus?.license_type !== 'Trial'" class="activation-section"> -->
        <h3>Activate License</h3>
        <div v-if="activationError" class="error-message">
          {{ activationError }}
        </div>
        <div v-if="activationSuccess" class="success-message">
          {{ activationSuccess }}
        </div>
        <div class="activation-form">
          <div class="form-group">
            <label>Email Address</label>
            <input 
              v-model="email" 
              type="email" 
              placeholder="your@email.com"
              class="input-field"
              :disabled="activating"
            />
          </div>
          <div class="form-group">
            <label>License Key</label>
            <input 
              v-model="licenseKey" 
              type="text" 
              placeholder="XXXX-XXXX-XXXX-XXXX"
              class="input-field"
              :disabled="activating"
              @input="formatLicenseKey"
            />
            <small class="help-text">Format: XXXX-XXXX-XXXX-XXXX</small>
          </div>
          <button 
            @click="activateLicense" 
            class="btn btn-primary" 
            :disabled="!email || !licenseKey || activating"
          >
            {{ activating ? 'Activating...' : 'Activate License' }}
          </button>
          <p class="help-text">
            Don't have a license key? <a @click="scrollToPricing">Purchase below</a>
          </p>
          <p class="help-text">
            Lost your license? <a @click="openSupport">Contact Support</a>
          </p>
        </div>
      </div>

      <!-- Pricing Options -->
      <div class="pricing-section" ref="pricingSection">
        <div class="pricing-header">
          <h3>Choose Your Plan</h3>
          <p>Start with 14-day free trial • No credit card required</p>
        </div>
        
        <div class="pricing-grid">
          <!-- Free Trial (Always shown first) -->
          <div class="pricing-card" :class="{ 'current-plan': licenseStatus?.license_type === 'Trial' }">
            <div v-if="licenseStatus?.license_type === 'Trial'" class="badge current">CURRENT PLAN</div>
            <h4>Free Trial</h4>
            <div class="price">
              <span class="amount">$0</span>
              <span class="period">/14 days</span>
            </div>
            <ul class="features">
              <li>✅ All features included</li>
              <li>✅ Unlimited comparisons</li>
              <li>✅ Schema & data patches</li>
              <li>✅ Export to SQL/JSON/Excel</li>
              <li>✅ No credit card needed</li>
            </ul>
            <button 
              v-if="licenseStatus?.license_type !== 'Trial'" 
              @click="startTrial" 
              class="btn btn-outline"
              disabled
            >
              Auto-Started
            </button>
            <button 
              v-else 
              class="btn btn-outline" 
              disabled
            >
              Active ({{ licenseStatus.days_remaining }} days left)
            </button>
            <p class="pricing-note">Perfect for trying out</p>
          </div>

          <!-- Monthly Plan -->
          <div class="pricing-card" :class="{ 'current-plan': licenseStatus?.license_type === 'Monthly' }">
            <div v-if="licenseStatus?.license_type === 'Monthly'" class="badge current">CURRENT PLAN</div>
            <h4>Monthly</h4>
            <div class="price">
              <span class="amount">$5</span>
              <span class="period">/month</span>
            </div>
            <ul class="features">
              <li>✅ Everything in Free Trial</li>
              <li>✅ Continued access</li>
              <li>✅ Email support</li>
              <li>✅ Cancel anytime</li>
            </ul>
            <button 
              @click="openPurchase('monthly')" 
              class="btn btn-outline"
              :disabled="licenseStatus?.license_type === 'Monthly'"
            >
              {{ licenseStatus?.license_type === 'Monthly' ? 'Current Plan' : 'Subscribe Monthly' }}
            </button>
            <p class="pricing-note">Cancel anytime</p>
          </div>

          <!-- Yearly Plan (Most Popular) -->
          <div class="pricing-card featured" :class="{ 'current-plan': licenseStatus?.license_type === 'Yearly' }">
            <div v-if="licenseStatus?.license_type === 'Yearly'" class="badge current">CURRENT PLAN</div>
            <div v-else class="badge">MOST POPULAR</div>
            <h4>Yearly</h4>
            <div class="price">
              <span class="amount">$39</span>
              <span class="period">/year</span>
            </div>
            <div class="savings">Save $21 (35% off)</div>
            <ul class="features">
              <li>✅ Everything in Monthly</li>
              <li>✅ Priority support</li>
              <li>✅ Early access to features</li>
              <li>✅ Just $3.25 per month</li>
            </ul>
            <button 
              @click="openPurchase('yearly')" 
              class="btn btn-primary"
              :disabled="licenseStatus?.license_type === 'Yearly'"
            >
              {{ licenseStatus?.license_type === 'Yearly' ? 'Current Plan' : 'Subscribe Yearly' }}
            </button>
            <p class="pricing-note">Best value for regular use</p>
          </div>

          <!-- Lifetime Plan -->
          <div class="pricing-card" :class="{ 'current-plan': licenseStatus?.license_type === 'Lifetime' }">
            <div v-if="licenseStatus?.license_type === 'Lifetime'" class="badge current">CURRENT PLAN</div>
            <div v-else class="badge special">BEST DEAL</div>
            <h4>Lifetime</h4>
            <div class="price">
              <span class="amount">$69</span>
              <span class="period">one-time</span>
            </div>
            <div class="savings">Early Adopter Price 🎉</div>
            <ul class="features">
              <li>✅ Everything in Yearly</li>
              <li>✅ Pay once, use forever</li>
              <li>✅ All future updates</li>
              <li>✅ No recurring fees ever</li>
            </ul>
            <button 
              @click="openPurchase('lifetime')" 
              class="btn btn-primary"
              :disabled="licenseStatus?.license_type === 'Lifetime'"
            >
              {{ licenseStatus?.license_type === 'Lifetime' ? 'Current Plan' : 'Buy Lifetime Access' }}
            </button>
            <p class="pricing-note">Limited time offer</p>
          </div>
        </div>

        <div class="guarantee-badge">
          <span class="guarantee-icon">🛡️</span>
          <strong>30-Day Money-Back Guarantee</strong>
          <span>Try risk-free. Full refund if not satisfied.</span>
        </div>
      </div>

      <!-- FAQ Section -->
      <div class="faq-section">
        <h3>Frequently Asked Questions</h3>
        <div class="faq-list">
          <div class="faq-item" v-for="faq in faqs" :key="faq.id" @click="toggleFaq(faq.id)">
            <div class="faq-question">
              <span>{{ faq.question }}</span>
              <span class="faq-icon">{{ expandedFaqs.has(faq.id) ? '−' : '+' }}</span>
            </div>
            <div v-if="expandedFaqs.has(faq.id)" class="faq-answer">
              {{ faq.answer }}
            </div>
          </div>
        </div>
      </div>

      <div class="support-section">
        <p>Need help? <a @click="openSupport">Contact Support</a></p>
      </div>
    </div>

    <!-- Deactivation Modal -->
    <div v-if="showDeactivate" class="modal-overlay" @click="showDeactivate = false">
      <div class="modal-content" @click.stop>
        <h3>Deactivate License?</h3>
        <p>This will remove the license from this computer. You can reactivate it later on this or another device.</p>
        <div class="modal-actions">
          <button @click="showDeactivate = false" class="btn btn-outline">Cancel</button>
          <button @click="confirmDeactivate" class="btn btn-danger" :disabled="deactivating">
            {{ deactivating ? 'Deactivating...' : 'Deactivate' }}
          </button>
        </div>
      </div>
    </div>
  <!-- </div> -->
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
    question: 'How does the 14-day trial work?',
    answer: 'The free trial starts automatically when you first launch the app. You get full access to all features for 14 days. No credit card required, no automatic charges. After 14 days, choose a plan to continue using the app.'
  },
  {
    id: 2,
    question: 'Can I cancel my subscription?',
    answer: 'Yes, cancel anytime. You keep access until the end of your billing period. No partial refunds, but you continue using the app until your paid period expires.'
  },
  {
    id: 3,
    question: 'What happens to my data?',
    answer: 'Your databases never leave your computer. All data stays local and private. Even if you cancel, you can still access your saved comparisons and exports.'
  },
  {
    id: 4,
    question: 'Do you offer refunds?',
    answer: 'Yes, 30-day money-back guarantee for all purchases. Contact support with your license key for a full refund, no questions asked.'
  },
  {
    id: 5,
    question: 'Does this work offline?',
    answer: 'Yes! The app works fully offline. License validates once during activation, then automatically every 7 days. If offline, you get a 30-day grace period.'
  },
  {
    id: 6,
    question: 'Can I use on multiple computers?',
    answer: 'Yes, use your license on up to 2 computers you personally own (e.g., work laptop + home desktop). Not for team sharing. Need more? Contact us about team licenses.'
  },
  {
    id: 7,
    question: 'Can I upgrade my plan?',
    answer: 'Yes! You can upgrade from Trial to any paid plan, or from Monthly to Yearly/Lifetime anytime. Contact support and we will apply credit for any overlapping time.'
  },
  {
    id: 8,
    question: 'What if I lose my license key?',
    answer: 'Your license key is emailed after purchase. If lost, contact support with your purchase email and we will resend it immediately.'
  }
];

onMounted(async () => {
  await checkLicense();
});

const checkLicense = async () => {
  try {
    const status = await invoke<LicenseStatus>('get_license_status');
    licenseStatus.value = status;
    
    // Only show activation form if expired (not for trial)
    if (!status.is_valid && status.license_type !== 'Trial') {
      showActivation.value = true;
    }
  } catch (err) {
    console.error('License check failed:', err);
    activationError.value = 'Failed to check license. Please restart.';
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
      activationSuccess.value = '✅ License activated successfully!';
      setTimeout(() => {
        email.value = '';
        licenseKey.value = '';
        activationSuccess.value = '';
      }, 3000);
    } else {
      activationError.value = '❌ Invalid license key or email.';
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

const startTrial = () => {
  // Trial starts automatically, this is just UI feedback
  alert('Trial is already active! Enjoy all features for 14 days.');
};

const openPurchase = (plan: string) => {
  const urls: Record<string, string> = {
    monthly: 'https://planplabs.lemonsqueezy.com/buy/91cb4129-6a90-4ef7-9d2d-4dbc7499d239',//'https://yourstore.lemonsqueezy.com/checkout/buy/MONTHLY_ID',
    yearly: 'https://planplabs.lemonsqueezy.com/buy/39100320-dce1-4487-af6f-bf8ff51b6d67', //'https://yourstore.lemonsqueezy.com/checkout/buy/YEARLY_ID',
    lifetime: 'https://planplabs.lemonsqueezy.com/buy/401e3fbf-b3c5-4151-8e3c-f629a0f9a753' //'https://yourstore.lemonsqueezy.com/checkout/buy/LIFETIME_ID'
  };
  open(urls[plan]);
};

const openSupport = () => {
  open('mailto:support@yourdomain.com?subject=SQLCipher Tool - License Support');
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
.license-content {
  max-width: 1400px;
  margin: 0 auto;
  padding: 20px;
}

.content-header {
  margin-bottom: 30px;
}

.content-header h2 {
  font-size: 2rem;
  margin-bottom: 8px;
}

.content-header p {
  color: #666;
}

.license-status-card {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 30px;
  border-radius: 12px;
  margin-bottom: 30px;
  border: 2px solid;
}

.license-status-card.valid {
  background: linear-gradient(135deg, #d4edda 0%, #c3e6cb 100%);
  border-color: #28a745;
}

.license-status-card.expired {
  background: linear-gradient(135deg, #fff3cd 0%, #ffeaa7 100%);
  border-color: #ffc107;
}

.status-icon {
  font-size: 3em;
}

.status-info {
  flex: 1;
}

.status-info h3 {
  margin: 0 0 8px 0;
  font-size: 1.5em;
}

.status-info p {
  margin: 4px 0;
  color: #555;
}

.status-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.activation-section {
  background: white;
  padding: 30px;
  border-radius: 12px;
  margin-bottom: 40px;
  border: 1px solid #dee2e6;
  box-shadow: 0 2px 8px rgba(0,0,0,0.05);
}

.activation-section h3 {
  margin-top: 0;
  margin-bottom: 20px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: #333;
}

.input-field {
  width: 100%;
  padding: 12px 16px;
  border: 2px solid #ddd;
  border-radius: 8px;
  font-size: 1rem;
  transition: border-color 0.2s;
}

.input-field:focus {
  outline: none;
  border-color: #007bff;
}

.input-field:disabled {
  background: #f5f5f5;
  cursor: not-allowed;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 1rem;
  border: none;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #0056b3;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0,123,255,0.3);
}

.btn-outline {
  background: white;
  color: #007bff;
  border: 2px solid #007bff;
}

.btn-outline:hover:not(:disabled) {
  background: #007bff;
  color: white;
}

.btn-outline-danger {
  background: white;
  color: #dc3545;
  border: 2px solid #dc3545;
}

.btn-outline-danger:hover:not(:disabled) {
  background: #dc3545;
  color: white;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #c82333;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-message {
  background: #fee;
  border: 1px solid #fcc;
  color: #c33;
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 16px;
}

.success-message {
  background: #efe;
  border: 1px solid #cfc;
  color: #3c3;
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 16px;
}

.help-text {
  text-align: center;
  margin-top: 12px;
  font-size: 0.9em;
  color: #666;
}

.help-text a {
  color: #007bff;
  cursor: pointer;
  text-decoration: underline;
}

.help-text a:hover {
  color: #0056b3;
}

small.help-text {
  display: block;
  text-align: left;
  margin-top: 4px;
  font-size: 0.85em;
}

.pricing-section {
  margin-top: 60px;
}

.pricing-header {
  text-align: center;
  margin-bottom: 40px;
}

.pricing-header h3 {
  font-size: 2em;
  margin-bottom: 8px;
}

.pricing-header p {
  color: #666;
  font-size: 1.1em;
}

.pricing-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
  margin-bottom: 40px;
}

/* Responsive: 3 columns on tablets */
@media (max-width: 1200px) {
  .pricing-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* Responsive: 1 column on mobile */
@media (max-width: 768px) {
  .pricing-grid {
    grid-template-columns: 1fr;
  }
  
  .pricing-card.featured {
    transform: none !important;
  }
}

.pricing-card {
  background: white;
  padding: 32px 24px;
  border-radius: 12px;
  border: 2px solid #dee2e6;
  position: relative;
  transition: all 0.3s;
  display: flex;
  flex-direction: column;
}

.pricing-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,0.12);
  border-color: #007bff;
}

.pricing-card.featured {
  border-color: #007bff;
  box-shadow: 0 4px 20px rgba(0,123,255,0.15);
}

.pricing-card.current-plan {
  border-color: #28a745;
  background: linear-gradient(135deg, #f8fff9 0%, #f0fff4 100%);
}

.pricing-card h4 {
  font-size: 1.5em;
  margin: 0 0 16px 0;
  color: #333;
}

.badge {
  position: absolute;
  top: -12px;
  right: 20px;
  background: #007bff;
  color: white;
  padding: 6px 16px;
  border-radius: 20px;
  font-size: 0.75em;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.badge.special {
  background: linear-gradient(135deg, #ff6b00 0%, #ff8c00 100%);
}

.badge.current {
  background: #28a745;
}

.price {
  margin: 16px 0;
}

.amount {
  font-size: 3em;
  font-weight: 700;
  color: #007bff;
  line-height: 1;
}

.period {
  font-size: 1em;
  color: #6c757d;
  margin-left: 4px;
}

.savings {
  color: #28a745;
  font-weight: 600;
  margin-bottom: 16px;
  font-size: 0.95em;
}

.features {
  list-style: none;
  padding: 0;
  margin: 24px 0;
  flex: 1;
}

.features li {
  padding: 10px 0;
  border-bottom: 1px solid #f0f0f0;
  font-size: 0.95em;
  line-height: 1.4;
}

.features li:last-child {
  border-bottom: none;
}

.pricing-card .btn {
  width: 100%;
  margin-top: auto;
}

.pricing-note {
  text-align: center;
  font-size: 0.85em;
  color: #6c757d;
  margin-top: 12px;
}

.guarantee-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 24px;
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  border-radius: 12px;
  text-align: center;
  flex-wrap: wrap;
  margin-top: 40px;
}

.guarantee-icon {
  font-size: 2em;
}

.guarantee-badge strong {
  color: #28a745;
  font-size: 1.1em;
}

.guarantee-badge span:last-child {
  color: #666;
  flex-basis: 100%;
  margin-top: 8px;
}

.faq-section {
  margin-top: 60px;
}

.faq-section h3 {
  font-size: 1.8em;
  margin-bottom: 24px;
  text-align: center;
}

.faq-list {
  max-width: 900px;
  margin: 0 auto;
}

.faq-item {
  background: white;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  margin-bottom: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.faq-item:hover {
  border-color: #007bff;
  box-shadow: 0 2px 8px rgba(0,123,255,0.1);
}

.faq-question {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  font-weight: 600;
  color: #333;
}

.faq-icon {
  font-size: 1.5em;
  color: #007bff;
  font-weight: 300;
}

.faq-answer {
  padding: 0 24px 20px 24px;
  color: #666;
  line-height: 1.6;
  animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.support-section {
  text-align: center;
  margin-top: 40px;
  padding: 20px;
}

.support-section a {
  color: #007bff;
  cursor: pointer;
  text-decoration: underline;
  font-weight: 600;
}

.support-section a:hover {
  color: #0056b3;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  animation: fadeIn 0.2s;
}

.modal-content {
  background: white;
  padding: 32px;
  border-radius: 12px;
  max-width: 500px;
  width: 90%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.modal-content h3 {
  margin: 0 0 16px 0;
  font-size: 1.5rem;
  color: #333;
}

.modal-content p {
  margin-bottom: 12px;
  color: #666;
  line-height: 1.5;
}

.modal-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
  justify-content: flex-end;
}
</style>