const cors = require('cors');
const { createClient } = require('@supabase/supabase-js');

// Initialize CORS middleware
const corsMiddleware = cors();

// Initialize Supabase
const supabase = createClient(
  process.env.SUPABASE_URL,
  process.env.SUPABASE_ANON_KEY
);

module.exports = async (req, res) => {
  // Run CORS
  await new Promise((resolve, reject) => {
    corsMiddleware(req, res, (result) => {
      if (result instanceof Error) {
        return reject(result);
      }
      return resolve(result);
    });
  });

  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { email, license_key, machine_id } = req.body;
  
  console.log('📨 Verification request:', { email, license_key });
  
  try {
    // Get license from Supabase
    const { data: license, error } = await supabase
      .from('licenses')
      .select('*')
      .eq('license_key', license_key)
      .single();

    if (error || !license) {
      return res.status(404).json({ error: 'License not found' });
    }
    
    if (license.email.toLowerCase() !== email.toLowerCase()) {
      return res.status(403).json({ error: 'Email mismatch' });
    }
    
    if (license.status !== 'active') {
      return res.status(403).json({ error: 'License not active' });
    }

    // Map product_id to license_type (you'll need to customize this)
    const licenseType = mapProductToLicenseType(license.product_id);

    res.json({
      license_type: licenseType,
      expiry_date: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString(),
      is_valid: true
    });
  } catch (error) {
    console.error('Verification error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
};

// Map Paddle product IDs to your license types
function mapProductToLicenseType(productId) {
  const mapping = {
    'pri_01k807azrz8asavdg42z0fqv34': 'monthly',  // Monthly product
    'pri_01k807dj8bv6h3gp9c1tvp6dfc': 'yearly',   // Yearly product  
    'pri_01k807ewanhdkvhb06wr24ffy9': 'lifetime'  // Lifetime product
  };
  return mapping[productId] || 'yearly';
}
