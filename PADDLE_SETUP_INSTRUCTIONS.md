# Paddle Integration Setup Instructions

## ✅ What's Already Done

1. ✅ License server deployed on Vercel
2. ✅ License service created in app
3. ✅ Paywall/License modal created
4. ✅ App.vue integrated with license system
5. ✅ Patch preview modal with copy & download
6. ✅ Products created in Paddle

---

## 🔧 Step 1: Configure Paddle Webhook

### Find Webhook Settings:

1. Go to **Paddle Dashboard**: https://vendors.paddle.com/
2. Click **"Developer Tools"** (left sidebar)
3. Click **"Notifications"** or **"Webhooks"**
4. Click **"Add Notification Destination"** or **"Add Endpoint"**

### Enter Webhook URL:

```
https://plandbdiff-license-02-manikandans-projects-be37ef3a.vercel.app/api/paddle-webhook
```

### Select Events to Listen For:

Check these events:
- ☑️ `subscription.created`
- ☑️ `subscription.updated`
- ☑️ `subscription.canceled`
- ☑️ `subscription.payment_succeeded`
- ☑️ `subscription.payment_failed`

**Save the webhook!**

---

## 💰 Step 2: Your Product IDs (Already Configured)

Your products are already set up in the code:

| Plan | Price | Product ID | Usage |
|------|-------|------------|-------|
| **Monthly** | $9/month | `pri_01k807azrz8asavdg42z0fqv34` | Recurring monthly |
| **Yearly** | $99/year | `pri_01k807dj8bv6h3gp9c1tvp6dfc` | Best value (saves $9) |
| **Lifetime** | $199 | `pri_01k807ewanhdkvhb06wr24ffy9` | One-time payment |

The yearly plan is set as default in the "Buy License" button.

---

## 🧪 Step 3: Test the Flow

### Test Webhook:

1. In Paddle Dashboard → Webhooks
2. Find your webhook
3. Click **"Send Test Event"**
4. Select `subscription.created`
5. Check Vercel logs to confirm it received the webhook

### Test License Activation:

1. Run your app: `npm run tauri dev`
2. License modal should appear on startup
3. Click **"Buy License"** → Opens Paddle checkout
4. Use Paddle test cards: https://developer.paddle.com/concepts/payment-methods/credit-debit-card#test-card-numbers

**Test Card:**
```
Card Number: 4242 4242 4242 4242
Expiry: Any future date
CVC: Any 3 digits
```

5. Complete test purchase
6. Check your email for license key
7. Enter email + license key in app
8. Should activate successfully!

---

## 📋 Step 4: What Happens When User Buys

```
User clicks "Buy License"
    ↓
Opens Paddle checkout page
    ↓
User enters payment info
    ↓
Paddle processes payment
    ↓
Paddle sends webhook to your server
    ↓
Your server saves license to Supabase
    ↓
User gets email with license key
    ↓
User enters license key in app
    ↓
App calls /api/verify endpoint
    ↓
Returns: { is_valid: true, license_type: "yearly" }
    ↓
App unlocks all features ✅
```

---

## 🎨 New Feature: Patch Preview

When user clicks **"Generate Patch"**, a modal now shows:

✅ **SQL Preview** - Full patch content with syntax highlighting
✅ **Copy Button** - Copy patch to clipboard
✅ **Download Button** - Save as .sql file
✅ **File Info** - Shows filename and file size
✅ **Warning** - Reminds user to review before executing

---

## 🔐 License Features

### What's Included:

1. **Email-based activation** - Simple for users
2. **Auto-renewal** - Paddle handles automatically
3. **Offline grace period** - Works for 7 days without re-verification
4. **Secure verification** - API-based, not easily bypassed
5. **Trial mode** - Users can try before buying

### Current Configuration:

- License checks on app startup
- Re-verifies every 7 days
- Stores license locally (encrypted by OS)
- Works offline after initial activation

---

## 📊 Monitor Your Sales

### Paddle Dashboard Shows:

- 💰 **Revenue** - Total earnings
- 👥 **Customers** - Active subscriptions
- 📈 **Conversion Rate** - Checkout success rate
- 🔄 **Churn** - Cancellation rate
- 💳 **Failed Payments** - Needs attention

### Vercel Logs Show:

- Webhook events received
- License activations
- Verification requests

Access logs:
```bash
vercel logs --follow
```

---

## 🎯 Next Steps

1. **Set up webhook** (5 minutes)
2. **Test with test card** (10 minutes)
3. **Verify license activation works** (5 minutes)
4. **Test patch preview feature** (2 minutes)
5. **Go live!** 🚀

---

## 🐛 Troubleshooting

### Webhook not receiving events:

```bash
# Check Vercel logs
vercel logs --follow

# Test webhook manually in Paddle dashboard
# Developer Tools → Notifications → Send Test Event
```

### License activation fails:

1. Check Supabase database has the license
2. Check email matches exactly (case-sensitive)
3. Check network tab for API errors
4. Verify Vercel environment variables are set

### Patch preview not showing:

- Check browser console for errors
- Ensure comparison completed successfully
- Try with smaller dataset first

---

## 💡 Tips

1. **Start with test mode** - Don't go live until tested
2. **Monitor first sales carefully** - Check webhooks working
3. **Respond to failed payments** - Paddle sends notifications
4. **Update product descriptions** - Make them compelling
5. **Add screenshots to checkout** - Increases conversion

---

## 📞 Support

- **Paddle Support**: https://www.paddle.com/support
- **Paddle Docs**: https://developer.paddle.com/
- **Vercel Support**: https://vercel.com/support

---

## ✅ Checklist

- [ ] Configure Paddle webhook
- [ ] Test webhook with test event
- [ ] Test purchase with test card
- [ ] Verify license activation
- [ ] Test patch preview modal
- [ ] Check Supabase has license data
- [ ] Test copy & download buttons
- [ ] Switch to live mode in Paddle
- [ ] Update product descriptions
- [ ] Set up customer emails (optional)
- [ ] Launch! 🎉

---

**Your license system is ready! Just configure the webhook and you're good to go!** 🚀
