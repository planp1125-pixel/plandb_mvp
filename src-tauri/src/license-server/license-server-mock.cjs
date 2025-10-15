const express = require('express');
const cors = require('cors');

const app = express();
app.use(cors());
app.use(express.json());

// Mock license database
const validLicenses = {
  'TEST-1234-5678-ABCD': {
    email: 'test@example.com',
    type: 'yearly'
  },
  'LIFE-TIME-TEST-KEY1': {
    email: 'test@example.com',
    type: 'lifetime'
  },
  'MONTH-1234-5678-ABCD': {
    email: 'test@example.com',
    type: 'monthly'
  }
};

app.post('/api/verify', (req, res) => {
  const { email, license_key, machine_id } = req.body;
  
  console.log('📨 Verification request:', { email, license_key, machine_id });
  
  const license = validLicenses[license_key];
  
  if (!license) {
    console.log('❌ License not found');
    return res.status(404).json({ error: 'License not found' });
  }
  
  if (license.email.toLowerCase() !== email.toLowerCase()) {
    console.log('❌ Email mismatch');
    return res.status(403).json({ error: 'Email mismatch' });
  }
  
  // Calculate expiry
  let expiry_date = null;
  if (license.type === 'monthly') {
    const date = new Date();
    date.setMonth(date.getMonth() + 1);
    expiry_date = date.toISOString();
  } else if (license.type === 'yearly') {
    const date = new Date();
    date.setFullYear(date.getFullYear() + 1);
    expiry_date = date.toISOString();
  }
  
  console.log('✅ License valid:', license.type);
  
  res.json({
    license_type: license.type,
    expiry_date: expiry_date,
    is_valid: true
  });
});

const PORT = 3000;
app.listen(PORT, () => {
  console.log('\n🚀 Mock License Server Running');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log(`📡 URL: http://localhost:${PORT}`);
  console.log('\n🔑 Test Credentials:');
  console.log('  Email: test@example.com');
  console.log('  Keys:');
  console.log('    • TEST-1234-5678-ABCD (Yearly)');
  console.log('    • LIFE-TIME-TEST-KEY1 (Lifetime)');
  console.log('    • MONTH-1234-5678-ABCD (Monthly)');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
});