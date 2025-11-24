# WhatsApp Webhook Security Setup

This document explains how to secure your WhatsApp webhook endpoints using X-Hub-Signature-256 verification.

## Overview

WhatsApp/Meta webhooks use **X-Hub-Signature-256** HMAC verification to ensure webhook requests are authentic and haven't been tampered with. This implementation provides robust security by:

1. ✅ Verifying that requests originate from Meta/Facebook
2. ✅ Ensuring payload integrity (no tampering)
3. ✅ Using constant-time comparison to prevent timing attacks
4. ✅ Rejecting requests with invalid or missing signatures

## Security Implementation

### How It Works

1. **Meta signs the payload**: When Meta sends a webhook request, they compute an HMAC-SHA256 signature using your app secret
2. **Signature is sent in header**: The signature is included in the `X-Hub-Signature-256` header with format: `sha256=<hex_signature>`
3. **Your server verifies**: Before processing any webhook, the server:
   - Extracts the signature from the header
   - Computes HMAC-SHA256 of the raw request body using your app secret
   - Compares the computed signature with the received signature using constant-time comparison
   - Only processes the request if signatures match

### Code Location

- **Signature verification**: `web_app/src/webhook/whatsapp/security.rs`
- **Webhook endpoint**: `web_app/src/webhook/whatsapp/routes.rs`
- **Configuration**: `web_app/src/config.rs`

## Configuration

### Required Environment Variable

Add the following environment variable to your configuration:

```bash
WHATSAPP_APP_SECRET="your-app-secret-from-meta"
```

### Where to Find Your App Secret

1. Go to the [Meta for Developers](https://developers.facebook.com/) dashboard
2. Navigate to your app
3. Go to **Settings** → **Basic**
4. Find your **App Secret** (you may need to click "Show" to reveal it)
5. Copy the secret and add it to your environment configuration

### For AWS SSM Parameter Store

If using the `ssm` feature, add the parameter:

```bash
Parameter Name: /pet-info/WHATSAPP_APP_SECRET
Type: SecureString
Value: your-app-secret-from-meta
```

## Testing

The implementation includes comprehensive tests:

```bash
cd web_app
cargo test security
```

### Test Coverage

- ✅ Valid signature verification
- ✅ Invalid signature rejection
- ✅ Wrong secret detection
- ✅ Tampered payload detection
- ✅ Invalid header format handling
- ✅ Invalid hex encoding handling

## Security Best Practices

### 1. Keep Your App Secret Secure

- ⚠️ **NEVER** commit your app secret to version control
- ⚠️ **NEVER** expose it in logs or error messages
- ✅ Use environment variables or secret management systems
- ✅ Use AWS SSM Parameter Store SecureString in production
- ✅ Rotate your app secret regularly

### 2. Monitor Failed Verification Attempts

Failed webhook verifications are logged with `logfire::warn!`. Monitor these logs for potential security issues:

```rust
logfire::warn!("Webhook signature verification failed - rejecting request");
```

### 3. HTTPS Only

Always use HTTPS for webhook endpoints. The signature verification protects against tampering, but HTTPS protects against eavesdropping.

### 4. Rate Limiting

Consider implementing rate limiting on webhook endpoints to prevent abuse.

## Verification Flow

```
┌─────────────────┐
│  Meta/Facebook  │
└────────┬────────┘
         │
         │ 1. Sign payload with app secret
         │    HMAC-SHA256(payload, app_secret)
         │
         ▼
┌────────────────────────────────────────────┐
│  POST /webhook/whatsapp                    │
│  Headers:                                  │
│    X-Hub-Signature-256: sha256=abc123...  │
│  Body: {"object":"whatsapp_business_...}   │
└────────┬───────────────────────────────────┘
         │
         │ 2. Extract signature from header
         │
         ▼
┌─────────────────────────────────────────┐
│  Your Server                            │
│  1. Get raw request body                │
│  2. Compute HMAC-SHA256(body, secret)   │
│  3. Constant-time compare signatures    │
└────────┬────────────────────────────────┘
         │
         ├─── ✅ Signatures match → Process webhook
         │
         └─── ❌ Signatures don't match → Reject with 403
```

## API Behavior

### Successful Request

```http
POST /webhook/whatsapp HTTP/1.1
Content-Type: application/json
X-Hub-Signature-256: sha256=abc123...

{"object":"whatsapp_business_account",...}

→ 200 OK
{"status":"received"}
```

### Failed Verification

```http
POST /webhook/whatsapp HTTP/1.1
Content-Type: application/json
X-Hub-Signature-256: sha256=invalid...

{"object":"whatsapp_business_account",...}

→ 403 Forbidden
```

### Missing Signature

```http
POST /webhook/whatsapp HTTP/1.1
Content-Type: application/json

{"object":"whatsapp_business_account",...}

→ 403 Forbidden
```

## Troubleshooting

### Signature Verification Failing

1. **Check your app secret**: Ensure the `WHATSAPP_APP_SECRET` environment variable matches your Meta app's secret
2. **Check the signature header**: The header must be exactly `X-Hub-Signature-256` (case-sensitive)
3. **Verify you're using the raw body**: Signature must be computed on raw bytes, not parsed JSON
4. **Check for middleware interference**: Ensure no middleware is modifying the request body before verification

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| 403 Forbidden | Wrong app secret | Verify your app secret matches Meta dashboard |
| 403 Forbidden | Missing header | Ensure Meta is configured to send webhooks |
| 403 Forbidden | Invalid hex | Check if there's middleware modifying headers |
| Parse error | Body modified | Ensure signature verification happens before parsing |

## References

- [Facebook Webhook Security Best Practices](https://stackoverflow.com/questions/36620841/what-is-the-best-practice-to-secure-your-facebook-chatbot-webhook)
- [WhatsApp Cloud API Webhook Signature Verification](https://stackoverflow.com/questions/73820222/whatsapp-cloud-api-unable-to-verify-webhook-signature)
- [HMAC-SHA256 Signature Verification Guide](https://hookdeck.com/webhooks/guides/how-to-implement-sha256-webhook-signature-verification)

## Support

If you encounter issues with webhook security:

1. Check the logs for specific error messages
2. Verify your Meta app configuration
3. Test with the included test suite
4. Review this documentation for common issues

---

**Note**: This implementation uses **X-Hub-Signature-256** (HMAC-SHA256), not mTLS. This is the standard authentication mechanism for Meta/Facebook/WhatsApp webhooks as of 2025.
