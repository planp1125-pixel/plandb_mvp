const cors = require('cors');
const { createClient } = require('@supabase/supabase-js');

const corsMiddleware = cors();
const supabase = createClient(
  process.env.SUPABASE_URL,
  process.env.SUPABASE_ANON_KEY
);

// Mock email function 
const sendLicenseEmail = async (email, licenseKey) => {
  console.log('📧 SEND THIS LICENSE TO USER:');
  console.log(`Email: ${email}`);
  console.log(`License Key: ${licenseKey}`);
  console.log('--- MANUALLY EMAIL THIS TO USER ---');
};

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

  const { alert_name, email, license_key, product_id, status } = req.body;
  
  console.log('📨 Paddle Webhook:', { alert_name, email, license_key, product_id });

  try {
    if (alert_name === 'subscription_created') {
      // Store in Supabase
      const { data, error } = await supabase
        .from('licenses')
        .insert([
          {
            license_key: license_key,
            email: email,
            status: 'active',
            product_id: product_id,
            created_at: new Date().toISOString()
          }
        ]);

      if (error) {
        console.error('Supabase error:', error);
        return res.status(500).json({ error: 'Database error' });
      }

      // Send email to user
      await sendLicenseEmail(email, license_key);
      
      console.log(`✅ License stored in Supabase for: ${email}`);
      
    } else if (alert_name === 'subscription_cancelled') {
      // Update license status in Supabase
      const { error } = await supabase
        .from('licenses')
        .update({ status: 'cancelled' })
        .eq('license_key', license_key);

      if (error) {
        console.error('Supabase update error:', error);
      } else {
        console.log(`❌ License cancelled for: ${email}`);
      }
    }

    res.status(200).send('OK');
  } catch (error) {
    console.error('❌ Webhook error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
};