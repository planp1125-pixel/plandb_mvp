// License Service - Handles license verification with backend API

const LICENSE_API_URL = 'https://plandbdiff-license-02-1xyxdl83k-manikandans-projects-be37ef3a.vercel.app/api';

// Paddle Product IDs
const PADDLE_PRODUCTS = {
  monthly: 'pri_01k807azrz8asavdg42z0fqv34',   // $9/month
  yearly: 'pri_01k807dj8bv6h3gp9c1tvp6dfc',    // $99/year
  lifetime: 'pri_01k807ewanhdkvhb06wr24ffy9'   // $199/lifetime
};

// Default to yearly plan
const PADDLE_CHECKOUT_URL = `https://buy.paddle.com/product/${PADDLE_PRODUCTS.yearly}`;

export interface LicenseStatus {
  isValid: boolean;
  licenseType?: 'monthly' | 'yearly' | 'lifetime';
  expiryDate?: string;
  email?: string;
  licenseKey?: string;
}

export class LicenseService {
  private static readonly STORAGE_KEY_EMAIL = 'license_email';
  private static readonly STORAGE_KEY_LICENSE = 'license_key';
  private static readonly STORAGE_KEY_VALID = 'license_valid';
  private static readonly STORAGE_KEY_TYPE = 'license_type';
  private static readonly STORAGE_KEY_EXPIRY = 'license_expiry';

  /**
   * Verify license with backend API
   */
  static async verifyLicense(email: string, licenseKey: string): Promise<LicenseStatus> {
    try {
      const response = await fetch(`${LICENSE_API_URL}/verify`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email: email.trim(),
          license_key: licenseKey.trim(),
          machine_id: await this.getMachineId()
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        console.error('License verification failed:', error);
        return { isValid: false };
      }

      const data = await response.json();

      // Store license info in localStorage
      this.saveLicense(email, licenseKey, data.license_type, data.expiry_date);

      return {
        isValid: data.is_valid,
        licenseType: data.license_type,
        expiryDate: data.expiry_date,
        email,
        licenseKey
      };
    } catch (error) {
      console.error('License verification error:', error);
      return { isValid: false };
    }
  }

  /**
   * Check if there's a saved valid license
   */
  static async checkSavedLicense(): Promise<LicenseStatus> {
    const email = localStorage.getItem(this.STORAGE_KEY_EMAIL);
    const licenseKey = localStorage.getItem(this.STORAGE_KEY_LICENSE);
    const isValid = localStorage.getItem(this.STORAGE_KEY_VALID) === 'true';
    const licenseType = localStorage.getItem(this.STORAGE_KEY_TYPE) as 'monthly' | 'yearly' | 'lifetime' | null;
    const expiryDate = localStorage.getItem(this.STORAGE_KEY_EXPIRY);

    if (!email || !licenseKey) {
      return { isValid: false };
    }

    // Check expiry
    if (expiryDate && new Date(expiryDate) < new Date()) {
      console.log('License expired, re-verifying...');
      // Re-verify with server
      return await this.verifyLicense(email, licenseKey);
    }

    // Periodically re-verify with server (every 7 days)
    const lastCheck = localStorage.getItem('license_last_check');
    const daysSinceCheck = lastCheck
      ? (Date.now() - parseInt(lastCheck)) / (1000 * 60 * 60 * 24)
      : 999;

    if (daysSinceCheck > 7) {
      console.log('Re-verifying license with server...');
      return await this.verifyLicense(email, licenseKey);
    }

    return {
      isValid,
      licenseType: licenseType || undefined,
      expiryDate: expiryDate || undefined,
      email,
      licenseKey
    };
  }

  /**
   * Save license to localStorage
   */
  private static saveLicense(email: string, licenseKey: string, licenseType: string, expiryDate: string) {
    localStorage.setItem(this.STORAGE_KEY_EMAIL, email);
    localStorage.setItem(this.STORAGE_KEY_LICENSE, licenseKey);
    localStorage.setItem(this.STORAGE_KEY_VALID, 'true');
    localStorage.setItem(this.STORAGE_KEY_TYPE, licenseType);
    localStorage.setItem(this.STORAGE_KEY_EXPIRY, expiryDate);
    localStorage.setItem('license_last_check', Date.now().toString());
  }

  /**
   * Clear saved license
   */
  static clearLicense() {
    localStorage.removeItem(this.STORAGE_KEY_EMAIL);
    localStorage.removeItem(this.STORAGE_KEY_LICENSE);
    localStorage.removeItem(this.STORAGE_KEY_VALID);
    localStorage.removeItem(this.STORAGE_KEY_TYPE);
    localStorage.removeItem(this.STORAGE_KEY_EXPIRY);
    localStorage.removeItem('license_last_check');
  }

  /**
   * Open Paddle checkout page
   */
  static openCheckout() {
    // Open in default browser
    window.open(PADDLE_CHECKOUT_URL, '_blank');
  }

  /**
   * Get machine ID (for future device limiting)
   */
  private static async getMachineId(): Promise<string> {
    // For now, generate a simple machine ID based on user agent + screen
    // In production, you might use Tauri's machine ID API
    const machineData = `${navigator.userAgent}-${screen.width}x${screen.height}`;
    const encoder = new TextEncoder();
    const data = encoder.encode(machineData);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('').slice(0, 32);
  }

  /**
   * Check if license allows feature
   */
  // static canUseFeature(licenseStatus: LicenseStatus, feature: 'comparison' | 'export' | 'unlimited'): boolean {
  //   if (!licenseStatus.isValid) {
  //     return false;
  //   }

  //   // All valid licenses get all features for now
  //   // You can add tiered licensing here later
  //   return true;
  // }
}
