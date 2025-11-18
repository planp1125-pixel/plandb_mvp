# License API - Vercel Deployment

## Active Deployment

**Project Name:** plandbdiff-license-0.2  
**Project ID:** prj_6pc7OmhdkcUJoHMKZkWPchN64Hjk  
**Deployment URL:** https://plandbdiff-license-02-1xyxdl83k-manikandans-projects-be37ef3a.vercel.app

## Endpoints

### 1. License Verification
- **URL:** `https://plandbdiff-license-02-1xyxdl83k-manikandans-projects-be37ef3a.vercel.app/api/verify`
- **Method:** POST
- **File:** `api/verify.js`

### 2. Paddle Webhook
- **URL:** `https://plandbdiff-license-02-1xyxdl83k-manikandans-projects-be37ef3a.vercel.app/api/paddle-webhook`
- **Method:** POST
- **File:** `api/paddle-webhook.js`
- **Configure in Paddle:** Developer Tools → Notifications → Add Endpoint

## Paddle Products

- **Monthly:** $9/month - `pri_01k807azrz8asavdg42z0fqv34`
- **Yearly:** $99/year - `pri_01k807dj8bv6h3gp9c1tvp6dfc`
- **Lifetime:** $199 - `pri_01k807ewanhdkvhb06wr24ffy9`

## Deploy Updates

```bash
cd license-api
vercel --prod
```

## Database

Uses Supabase for license storage.
