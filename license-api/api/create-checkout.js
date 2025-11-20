const cors = require('cors');
const https = require('https');

// Initialize CORS middleware
const corsMiddleware = cors();

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

  const { priceId } = req.body;

  if (!priceId) {
    return res.status(400).json({ error: 'priceId is required' });
  }

  try {
    // Create a transaction using Paddle API
    const postData = JSON.stringify({
      items: [
        {
          price_id: priceId,
          quantity: 1
        }
      ]
    });

    const options = {
      hostname: 'api.paddle.com',
      path: '/transactions',
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${process.env.PADDLE_API_KEY}`,
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(postData)
      }
    };

    const paddleResponse = await new Promise((resolve, reject) => {
      const req = https.request(options, (res) => {
        let data = '';
        res.on('data', (chunk) => {
          data += chunk;
        });
        res.on('end', () => {
          resolve({ statusCode: res.statusCode, body: data });
        });
      });

      req.on('error', (error) => {
        reject(error);
      });

      req.write(postData);
      req.end();
    });

    if (paddleResponse.statusCode !== 200 && paddleResponse.statusCode !== 201) {
      console.error('Paddle API error:', paddleResponse.statusCode, paddleResponse.body);
      return res.status(paddleResponse.statusCode).json({
        error: 'Paddle API error',
        details: paddleResponse.body,
        statusCode: paddleResponse.statusCode
      });
    }

    const transaction = JSON.parse(paddleResponse.body);

    // Return the checkout URL
    if (transaction.data && transaction.data.checkout && transaction.data.checkout.url) {
      res.json({
        checkoutUrl: transaction.data.checkout.url
      });
    } else {
      console.error('No checkout URL in response:', transaction);
      res.status(500).json({ error: 'No checkout URL received', data: transaction });
    }
  } catch (error) {
    console.error('Checkout creation error:', error.message, error.stack);
    res.status(500).json({ error: 'Failed to create checkout session', details: error.message });
  }
};
