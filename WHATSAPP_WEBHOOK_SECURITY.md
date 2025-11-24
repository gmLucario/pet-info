# WhatsApp Webhook Security Setup

This document explains how to secure your WhatsApp webhook endpoints using **mTLS (Mutual TLS)** and **X-Hub-Signature-256** verification.

## Overview

WhatsApp/Meta webhooks use a **dual-layer security approach** to ensure webhook requests are authentic and haven't been tampered with. This implementation provides robust security by:

1. ✅ **mTLS (Mutual TLS)**: Verifying client certificates from Meta/Facebook at the TLS layer
2. ✅ **X-Hub-Signature-256**: Verifying HMAC signatures to ensure payload integrity
3. ✅ Using constant-time comparison to prevent timing attacks
4. ✅ Rejecting requests with invalid or missing authentication

## Security Implementation

### Dual-Layer Security Architecture

This implementation uses **defense in depth** with two independent security layers:

#### Layer 1: mTLS (Mutual TLS) Certificate Verification

1. **Meta presents client certificate**: When Meta sends a webhook request, they present a client certificate during the TLS handshake
2. **Server verifies certificate**: The OpenSSL layer automatically verifies:
   - Certificate is signed by DigiCert High Assurance EV Root CA
   - Certificate hasn't expired
   - Certificate chain is valid
   - Certificate matches `client.webhooks.fbclientcerts.com`
3. **TLS handshake fails if invalid**: If the certificate is invalid, the connection is rejected at the TLS layer before reaching your application

#### Layer 2: X-Hub-Signature-256 HMAC Verification

1. **Meta signs the payload**: When Meta sends a webhook request, they compute an HMAC-SHA256 signature using your app secret
2. **Signature is sent in header**: The signature is included in the `X-Hub-Signature-256` header with format: `sha256=<hex_signature>`
3. **Your server verifies**: Before processing any webhook, the server:
   - Extracts the signature from the header
   - Computes HMAC-SHA256 of the raw request body using your app secret
   - Compares the computed signature with the received signature using constant-time comparison
   - Only processes the request if signatures match

### Why Two Layers?

- **mTLS**: Verifies the identity of the sender (Meta/Facebook)
- **X-Hub-Signature-256**: Verifies the integrity of the message content
- **Defense in depth**: Even if one layer is compromised, the other provides protection

### Code Location

- **mTLS configuration**: `web_app/src/main.rs` (setup_ssl_acceptor function, production only)
- **Certificate provisioning**: `terraform/modules/ec2/user_data/pet_info_start.sh`
- **Signature verification**: `web_app/src/webhook/whatsapp/security.rs`
- **Webhook endpoint**: `web_app/src/webhook/whatsapp/routes.rs`
- **Configuration**: `web_app/src/config.rs`
- **CA Certificate**: `certs/DigiCertHighAssuranceEVRootCA.pem`

## Configuration

### Step 1: Download DigiCert Root CA Certificate

**For Production Deployments:**

The certificate is automatically downloaded during EC2 instance provisioning via the Terraform user_data script (`terraform/modules/ec2/user_data/pet_info_start.sh`). No manual intervention required.

**For Development/Testing:**

The DigiCert High Assurance EV Root CA certificate is already included in this repository at:
```
certs/DigiCertHighAssuranceEVRootCA.pem
```

If you need to download it manually:

```bash
# Download the certificate
mkdir -p certs
cd certs
wget -q "https://cacerts.digicert.com/DigiCertHighAssuranceEVRootCA.crt"

# Convert from DER to PEM format
openssl x509 -inform DER -in DigiCertHighAssuranceEVRootCA.crt \
  -out DigiCertHighAssuranceEVRootCA.pem
```

**Certificate Details:**
- **Subject**: CN=DigiCert High Assurance EV Root CA, OU=www.digicert.com, O=DigiCert Inc, C=US
- **Valid Until**: November 10, 2031
- **Purpose**: Verify Meta's client certificates for webhook requests

### Step 2: Configure Environment Variables

Add the following environment variables to your configuration:

```bash
# WhatsApp App Secret for HMAC signature verification
WHATSAPP_APP_SECRET="your-app-secret-from-meta"

# Path to DigiCert root CA certificate for mTLS (optional, uses default if not set)
CLIENT_CA_CERT_PATH="certs/DigiCertHighAssuranceEVRootCA.pem"
```

### Step 3: Where to Find Your App Secret

1. Go to the [Meta for Developers](https://developers.facebook.com/) dashboard
2. Navigate to your app
3. Go to **Settings** → **Basic**
4. Find your **App Secret** (you may need to click "Show" to reveal it)
5. Copy the secret and add it to your environment configuration

### For AWS SSM Parameter Store

If using the `ssm` feature, add these parameters:

```bash
# App Secret
Parameter Name: /pet-info/WHATSAPP_APP_SECRET
Type: SecureString
Value: your-app-secret-from-meta

# Client CA certificate path (optional)
Parameter Name: /pet-info/CLIENT_CA_CERT_PATH
Type: String
Value: certs/DigiCertHighAssuranceEVRootCA.pem
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

## Environment-Specific Behavior

### Production Environment (ENV=prod)

**mTLS Enabled:**
- ✅ Client certificate verification is **active**
- ✅ Certificates must be signed by DigiCert High Assurance EV Root CA
- ✅ Certificate downloaded automatically during EC2 provisioning
- ✅ Both mTLS and X-Hub-Signature-256 verification required

**Configuration:**
```bash
ENV=prod
CLIENT_CA_CERT_PATH=certs/DigiCertHighAssuranceEVRootCA.pem  # Auto-downloaded
WHATSAPP_APP_SECRET=your-app-secret-from-meta
```

**Server Logs:**
```
[INFO] Production environment detected - configuring mTLS
[INFO] ✓ mTLS configured: Client certificates will be verified using CA from certs/DigiCertHighAssuranceEVRootCA.pem
[INFO] Webhook request with verified mTLS client certificate
```

### Development/Local Environment (ENV != prod)

**mTLS Disabled:**
- ℹ️ Client certificate verification is **disabled**
- ℹ️ Easier local testing without certificate setup
- ✅ X-Hub-Signature-256 verification still **active** and required

**Configuration:**
```bash
ENV=local  # or dev, staging, etc.
WHATSAPP_APP_SECRET=your-app-secret-from-meta
# CLIENT_CA_CERT_PATH not required
```

**Server Logs:**
```
[INFO] Development environment - mTLS disabled for easier local testing
```

### Summary

| Feature | Production | Development |
|---------|-----------|-------------|
| **mTLS Certificate Verification** | ✅ Enabled | ❌ Disabled |
| **X-Hub-Signature-256 Verification** | ✅ Enabled | ✅ Enabled |
| **Certificate Auto-Download** | ✅ Yes (Terraform) | ❌ No (Manual) |
| **Client Certificate Required** | ✅ Yes (webhooks) | ❌ No |

## Verification Flow

```
┌─────────────────┐
│  Meta/Facebook  │
└────────┬────────┘
         │
         │ 1. Prepare webhook request
         │    - Sign payload: HMAC-SHA256(payload, app_secret)
         │    - Prepare client certificate (client.webhooks.fbclientcerts.com)
         │
         ▼
┌────────────────────────────────────────────────┐
│  TLS Handshake with mTLS                       │
│  ┌──────────────────────────────────────────┐  │
│  │ Meta presents client certificate         │  │
│  │ Signed by: DigiCert High Assurance       │  │
│  │                                           │  │
│  │ Server verifies:                          │  │
│  │  ✓ Certificate chain valid               │  │
│  │  ✓ Signed by trusted CA (DigiCert)       │  │
│  │  ✓ Not expired                           │  │
│  │  ✓ Hostname matches                      │  │
│  └──────────────────────────────────────────┘  │
│         │                                       │
│         ├─── ✅ Valid → TLS connection         │
│         └─── ❌ Invalid → Connection rejected  │
└────────┬───────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────┐
│  POST /webhook/whatsapp                    │
│  Headers:                                  │
│    X-Hub-Signature-256: sha256=abc123...  │
│  Body: {"object":"whatsapp_business_...}   │
└────────┬───────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│  Application Layer (Your Server)        │
│                                         │
│  Step 1: mTLS verified (TLS layer)      │
│  ✓ Client certificate validated         │
│                                         │
│  Step 2: Extract X-Hub-Signature-256    │
│  Step 3: Compute HMAC-SHA256(body)      │
│  Step 4: Constant-time compare          │
└────────┬────────────────────────────────┘
         │
         ├─── ✅ Both layers valid → Process webhook
         │
         └─── ❌ Any layer fails → Reject with 403
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

**Note**: This implementation uses **dual-layer security** with both:
1. **mTLS (Mutual TLS)** - Client certificate verification at the TLS layer
2. **X-Hub-Signature-256** (HMAC-SHA256) - Payload signature verification at the application layer

This provides defense-in-depth security for Meta/Facebook/WhatsApp webhooks.
