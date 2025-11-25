# WhatsApp Webhook Security Setup

This document explains how to secure your WhatsApp webhook endpoints using **Nginx reverse proxy with mTLS (Mutual TLS)** certificate verification.

## Overview

This implementation uses **Nginx as a reverse proxy** to handle all TLS/mTLS operations, providing robust security by:

1. ✅ **Nginx handles HTTPS/TLS**: All external traffic encrypted with Let's Encrypt certificates
2. ✅ **mTLS (Mutual TLS)**: Verifying client certificates from Meta/Facebook at the TLS layer
3. ✅ **CN verification**: Ensuring certificate Common Name matches `client.webhooks.fbclientcerts.com`
4. ✅ **Application verification**: Rust app validates Nginx-provided headers for defense-in-depth
5. ✅ **Separation of concerns**: TLS handled by Nginx, business logic in Rust application

## Architecture

```
┌─────────────┐
│ WhatsApp/   │
│ Meta        │
└──────┬──────┘
       │
       │ HTTPS + mTLS client cert
       │ (client.webhooks.fbclientcerts.com)
       ▼
┌──────────────────────────────────────┐
│ Nginx Reverse Proxy (Port 443)      │
│                                      │
│  • Verifies HTTPS/TLS                │
│  • Verifies client certificate       │
│    - Signed by DigiCert Root CA      │
│    - CN = client.webhooks...         │
│  • Adds headers:                     │
│    - X-Client-Cert-Verified          │
│    - X-Client-Cert-DN                │
└──────┬───────────────────────────────┘
       │
       │ HTTP (localhost only)
       │ + Verification headers
       ▼
┌──────────────────────────────────────┐
│ Rust App (localhost:8080)            │
│                                      │
│  • Validates Nginx headers           │
│  • Processes webhook payload         │
│  • Business logic                    │
└──────────────────────────────────────┘
```

## Security Implementation

### mTLS Certificate Verification (Nginx Layer)

1. **Meta presents client certificate**: When Meta sends a webhook request, they present a client certificate during the TLS handshake
2. **Nginx verifies certificate**: Nginx automatically verifies:
   - Certificate is signed by DigiCert High Assurance EV Root CA
   - Certificate hasn't expired
   - Certificate chain is valid
   - Certificate Common Name (CN) matches `client.webhooks.fbclientcerts.com`
3. **Connection rejected if invalid**: If the certificate is invalid or CN doesn't match, Nginx returns 403 before reaching your application
4. **Headers passed to app**: On success, Nginx forwards the request with verification headers

### Application Layer Verification (Rust)

1. **Checks X-Client-Cert-Verified header**: Must be "SUCCESS"
2. **Validates X-Client-Cert-DN header**: Must contain "CN=client.webhooks.fbclientcerts.com"
3. **Defense-in-depth**: Provides additional validation beyond Nginx layer

### Code Location

- **Nginx configuration**: `terraform/modules/ec2/files/nginx-pet-info.conf`
- **Certificate provisioning**: `terraform/modules/ec2/user_data/pet_info_start.sh`
- **Webhook endpoint**: `web_app/src/webhook/whatsapp/routes.rs`
- **Configuration**: `web_app/src/config.rs`
- **CA Certificate**: `/etc/nginx/certs/DigiCertHighAssuranceEVRootCA.pem` (auto-downloaded)

## Configuration

### Automatic Setup (Production)

The entire infrastructure is **automatically configured** during EC2 instance provisioning via the Terraform user_data script (`terraform/modules/ec2/user_data/pet_info_start.sh`):

✅ **Nginx installation**
✅ **DigiCert root CA certificate download** (to `/etc/nginx/certs/`)
✅ **Nginx configuration deployment**
✅ **Certbot installation** (for Let's Encrypt SSL certificates)
✅ **Directory setup** for ACME challenges

**No manual intervention required** - everything is provisioned automatically!

### Manual Setup (Development/Testing)

If you need to set up Nginx with mTLS manually for testing:

#### 1. Install Nginx

```bash
sudo dnf install -y nginx
```

#### 2. Download DigiCert Root CA Certificate

```bash
# Create certificate directory
sudo mkdir -p /etc/nginx/certs

# Download the certificate
sudo wget -q "https://cacerts.digicert.com/DigiCertHighAssuranceEVRootCA.crt" \
  -O /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.crt

# Convert from DER to PEM format
sudo openssl x509 -inform DER -in /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.crt \
  -out /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.pem

# Set proper permissions
sudo chmod 755 /etc/nginx/certs
sudo chmod 644 /etc/nginx/certs/*
```

**Certificate Details:**
- **Subject**: CN=DigiCert High Assurance EV Root CA, OU=www.digicert.com, O=DigiCert Inc, C=US
- **Valid Until**: November 10, 2031
- **Purpose**: Verify Meta's client certificates for webhook requests
- **Location**: `/etc/nginx/certs/DigiCertHighAssuranceEVRootCA.pem`

#### 3. Install Certbot for Let's Encrypt

```bash
sudo dnf install -y certbot python3-certbot-nginx

# Create directory for ACME challenges
sudo mkdir -p /var/www/certbot
sudo chown -R nginx:nginx /var/www/certbot
```

#### 4. Deploy Nginx Configuration

```bash
# Copy configuration file
sudo cp terraform/modules/ec2/files/nginx-pet-info.conf /etc/nginx/conf.d/pet-info.conf

# Test configuration
sudo nginx -t

# Enable Nginx
sudo systemctl enable nginx
```

#### 5. Obtain SSL Certificate

After deploying your EC2 instance and configuring DNS to point to your server:

```bash
sudo certbot --nginx -d pet-info.link -d www.pet-info.link \
  --non-interactive --agree-tos -m your-email@example.com
```

This will automatically:
- Obtain SSL certificates from Let's Encrypt
- Configure Nginx to use them
- Set up automatic renewal

#### 6. Start Nginx

```bash
sudo systemctl start nginx
```

### Environment Variables

Only the following WhatsApp-related environment variables are required:

```bash
# WhatsApp webhook verification token (for GET verification endpoint)
WHATSAPP_VERIFY_TOKEN="your-verify-token"

# WhatsApp Business API credentials
WHATSAPP_BUSINESS_PHONE_NUMBER_ID="your-phone-number-id"
WHATSAPP_BUSINESS_AUTH="your-auth-token"
```

**Note**: No SSL/TLS certificate paths or app secrets are needed in the Rust application configuration - Nginx handles all TLS operations.

## Testing

### Testing Nginx Configuration

```bash
# Validate Nginx configuration syntax
sudo nginx -t

# Check Nginx is running
sudo systemctl status nginx

# View Nginx logs
sudo tail -f /var/log/nginx/pet-info-access.log
sudo tail -f /var/log/nginx/pet-info-error.log
```

### Testing mTLS Verification

Test that Nginx correctly rejects requests without valid client certificates:

```bash
# Should fail - no client certificate
curl -v https://pet-info.link/webhook/whatsapp

# Should succeed - regular GET request (webhook verification)
curl -v "https://pet-info.link/webhook/whatsapp?hub.mode=subscribe&hub.verify_token=YOUR_TOKEN&hub.challenge=test123"
```

### Testing Application Layer

```bash
cd web_app
cargo test
```

## Security Best Practices

### 1. Certificate Security

- ✅ DigiCert root CA certificate is public and safe to distribute
- ✅ Let's Encrypt certificates automatically renewed by certbot
- ✅ Private keys protected with file system permissions (600)
- ⚠️ **NEVER** commit private keys to version control

### 2. Monitor Failed Verification Attempts

Failed mTLS verifications are logged by Nginx and the Rust application. Monitor these logs for potential security issues:

**Nginx logs** (`/var/log/nginx/pet-info-error.log`):
```
SSL_do_handshake() failed (SSL: error:14094412:SSL routines:ssl3_read_bytes:sslv3 alert bad certificate)
```

**Application logs**:
```rust
logfire::warn!("Missing X-Client-Cert-Verified header - mTLS verification failed");
logfire::warn!("Client certificate CN verification failed");
```

### 3. HTTPS Only

Nginx enforces HTTPS for all external traffic. HTTP requests on port 80 are automatically redirected to HTTPS on port 443.

### 4. Rate Limiting

Consider adding Nginx rate limiting for webhook endpoints:

```nginx
# Add to nginx-pet-info.conf
limit_req_zone $binary_remote_addr zone=webhook_limit:10m rate=10r/s;

location /webhook/whatsapp {
    limit_req zone=webhook_limit burst=20 nodelay;
    # ... rest of configuration
}
```

### 5. Firewall Configuration

Ensure your EC2 security group allows:
- Port 443 (HTTPS) from WhatsApp/Meta IP ranges
- Port 80 (HTTP) for Let's Encrypt challenges
- Block direct access to port 8080 (Rust app) from external IPs

## Environment-Specific Behavior

### Production Environment (ENV=prod)

**mTLS Enabled via Nginx:**
- ✅ Nginx handles all TLS/mTLS verification
- ✅ Client certificates verified against DigiCert High Assurance EV Root CA
- ✅ CN verification enforced (`client.webhooks.fbclientcerts.com`)
- ✅ Rust application validates Nginx headers for defense-in-depth
- ✅ Certificate automatically downloaded during EC2 provisioning

**Infrastructure:**
```bash
ENV=prod

# Nginx automatically configured with:
# - HTTPS/TLS on port 443 (Let's Encrypt)
# - mTLS client verification
# - DigiCert root CA at /etc/nginx/certs/

# Rust app configuration:
WHATSAPP_VERIFY_TOKEN=your-verify-token
# (No SSL/TLS configuration needed)
```

**Server Logs:**
```
[Nginx] SSL_do_handshake() successful with client certificate
[App]   Webhook request authenticated via mTLS - CN: client.webhooks.fbclientcerts.com
```

### Development/Local Environment (ENV != prod)

**mTLS Disabled:**
- ℹ️ Application skips mTLS header validation in development
- ℹ️ Easier local testing without Nginx setup
- ⚠️ **Do not expose development endpoints to the internet**

**Configuration:**
```bash
ENV=local  # or dev, staging, etc.

# Run Rust app directly (no Nginx):
# Binds to localhost:8080 (HTTP only)
```

**Server Logs:**
```
[INFO] Starting server on http://127.0.0.1:8080 (behind Nginx reverse proxy)
[INFO] Development environment - skipping mTLS verification
```

### Summary

| Feature | Production | Development |
|---------|-----------|-------------|
| **Nginx Reverse Proxy** | ✅ Required | ❌ Optional |
| **HTTPS/TLS (Nginx)** | ✅ Enabled (Port 443) | ❌ Disabled |
| **mTLS Certificate Verification** | ✅ Enabled (Nginx) | ❌ Disabled |
| **CN Verification** | ✅ Required | ❌ Skipped |
| **Certificate Auto-Download** | ✅ Yes (Terraform) | ❌ No (Manual) |
| **Application Binding** | localhost:8080 | localhost:8080 |

## Verification Flow

```
┌─────────────────┐
│  Meta/Facebook  │
└────────┬────────┘
         │
         │ 1. Prepare webhook request
         │    - Prepare client certificate (client.webhooks.fbclientcerts.com)
         │    - Prepare webhook payload
         │
         ▼
┌────────────────────────────────────────────────┐
│  Nginx Reverse Proxy (Port 443)               │
│                                                │
│  Step 1: TLS Handshake with mTLS              │
│  ┌──────────────────────────────────────────┐ │
│  │ Meta presents client certificate         │ │
│  │ Signed by: DigiCert High Assurance       │ │
│  │                                           │ │
│  │ Nginx verifies:                           │ │
│  │  ✓ Certificate chain valid               │ │
│  │  ✓ Signed by trusted CA (DigiCert)       │ │
│  │  ✓ Not expired                           │ │
│  │  ✓ CN = client.webhooks.fbclientcerts... │ │
│  └──────────────────────────────────────────┘ │
│         │                                      │
│         ├─── ✅ Valid → Continue               │
│         └─── ❌ Invalid → 403 Forbidden        │
└────────┬───────────────────────────────────────┘
         │
         │ 2. Nginx adds verification headers:
         │    X-Client-Cert-Verified: SUCCESS
         │    X-Client-Cert-DN: CN=client.webhooks...
         │
         ▼
┌────────────────────────────────────────────┐
│  POST http://localhost:8080/webhook/...   │
│  Headers:                                  │
│    X-Client-Cert-Verified: SUCCESS        │
│    X-Client-Cert-DN: CN=client.webhooks...│
│  Body: {"object":"whatsapp_business_...}   │
└────────┬───────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│  Rust Application (localhost:8080)      │
│                                         │
│  Step 1: Check X-Client-Cert-Verified   │
│    ✓ Must be "SUCCESS"                  │
│                                         │
│  Step 2: Check X-Client-Cert-DN         │
│    ✓ Must contain "CN=client.webhooks...│
│                                         │
│  Step 3: Parse and process webhook      │
└────────┬────────────────────────────────┘
         │
         ├─── ✅ All checks pass → Process webhook
         │
         └─── ❌ Any check fails → Reject with 403
```

## API Behavior

### Successful Request (With mTLS)

```http
POST /webhook/whatsapp HTTPS/1.1
Host: pet-info.link
Content-Type: application/json
[Client Certificate: CN=client.webhooks.fbclientcerts.com]

{"object":"whatsapp_business_account",...}

→ [Nginx verifies mTLS] → [Adds headers] → [Forwards to app]
→ 200 OK
{"status":"received"}
```

### Failed mTLS Verification (No Certificate)

```http
POST /webhook/whatsapp HTTPS/1.1
Host: pet-info.link
Content-Type: application/json
[No Client Certificate]

{"object":"whatsapp_business_account",...}

→ [Nginx rejects at TLS layer]
→ 403 Forbidden
```

### Failed CN Verification (Wrong Certificate)

```http
POST /webhook/whatsapp HTTPS/1.1
Host: pet-info.link
Content-Type: application/json
[Client Certificate: CN=wrong.example.com]

{"object":"whatsapp_business_account",...}

→ [Nginx rejects - CN mismatch]
→ 403 Forbidden
```

## Troubleshooting

### mTLS Verification Failing

**Issue: Nginx returns 403 for webhook requests**

1. **Check Nginx logs**:
   ```bash
   sudo tail -f /var/log/nginx/pet-info-error.log
   ```
   Look for SSL/TLS handshake errors

2. **Verify DigiCert certificate is installed**:
   ```bash
   ls -la /etc/nginx/certs/
   # Should show DigiCertHighAssuranceEVRootCA.pem
   ```

3. **Verify Nginx configuration**:
   ```bash
   sudo nginx -t
   # Should show "syntax is ok" and "test is successful"
   ```

4. **Check certificate permissions**:
   ```bash
   sudo chmod 644 /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.pem
   ```

**Issue: Application returns 403 with missing header errors**

1. **Ensure production environment**:
   ```bash
   echo $ENV
   # Should be "prod" for mTLS verification
   ```

2. **Check Nginx is properly forwarding headers**:
   ```bash
   # View access logs to see forwarded headers
   sudo tail -f /var/log/nginx/pet-info-access.log
   ```

3. **Verify application logs**:
   ```bash
   # Look for specific error messages
   # "Missing X-Client-Cert-Verified header"
   # "Client certificate CN verification failed"
   ```

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| 403 Forbidden (Nginx) | No client certificate | Ensure Meta is sending client certificate |
| 403 Forbidden (Nginx) | Wrong CN | Verify certificate CN matches client.webhooks.fbclientcerts.com |
| 403 Forbidden (App) | Missing headers | Check Nginx is forwarding X-Client-Cert-* headers |
| SSL handshake failed | Wrong CA certificate | Re-download DigiCert root CA certificate |
| Connection refused | Nginx not running | `sudo systemctl start nginx` |
| Certificate expired | Let's Encrypt renewal failed | Run `sudo certbot renew` |

### Testing Locally

For local development without Nginx:

```bash
# Set environment to development
export ENV=local

# Run application
cd web_app
cargo run

# Application will bind to localhost:8080 without mTLS verification
```

## References

- [Meta/Facebook Webhook mTLS Documentation](https://developers.facebook.com/docs/graph-api/webhooks/getting-started#mtls-for-webhooks)
- [Nginx SSL/TLS Configuration](https://nginx.org/en/docs/http/ngx_http_ssl_module.html)
- [Nginx Client Certificate Verification](https://nginx.org/en/docs/http/ngx_http_ssl_module.html#ssl_verify_client)
- [Let's Encrypt with Nginx](https://certbot.eff.org/instructions?ws=nginx)
- [DigiCert Root Certificates](https://www.digicert.com/kb/digicert-root-certificates.htm)

## Support

If you encounter issues with webhook security:

1. Check Nginx logs (`/var/log/nginx/pet-info-error.log`)
2. Check application logs
3. Verify Meta app configuration in Facebook Dashboard
4. Test Nginx configuration with `sudo nginx -t`
5. Review this documentation for common issues

## Architecture Benefits

This Nginx reverse proxy implementation provides:

- ✅ **Separation of concerns**: TLS/mTLS handled by Nginx, business logic in Rust
- ✅ **Industry standard**: Nginx is battle-tested for TLS/mTLS
- ✅ **Zero cost**: No additional AWS resources required (ALB would cost $25-45/month)
- ✅ **Simplified application**: No SSL/TLS dependencies in Rust application
- ✅ **Easy SSL management**: Let's Encrypt with automatic renewal
- ✅ **Performance**: Nginx handles TLS termination efficiently
- ✅ **Security**: mTLS at network edge, defense-in-depth with application validation

---

**Implementation**: This setup uses **Nginx reverse proxy** to handle all TLS/mTLS operations, with the Rust application providing defense-in-depth validation of Nginx-provided headers.
