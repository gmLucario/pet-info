# SSL/TLS Certificates Directory

This directory is **no longer used** in the current architecture.

## Architecture Change

The application has been refactored to use **Nginx reverse proxy** for all TLS/mTLS operations.

### Previous Architecture (Deprecated)
- Rust application handled SSL/TLS directly
- Certificates stored in this directory
- Application loaded certificates at runtime

### Current Architecture (Nginx Reverse Proxy)
- **Nginx** handles all TLS/mTLS operations
- **Certificates** stored in `/etc/nginx/certs/` (on production servers)
- **Application** runs on localhost:8080 (HTTP only, no SSL)
- **Security** implemented via Nginx mTLS + application header validation

## Certificate Location (Production)

Certificates are now managed by Nginx and stored at:

```
/etc/nginx/certs/
├── DigiCertHighAssuranceEVRootCA.crt  (DER format)
└── DigiCertHighAssuranceEVRootCA.pem  (PEM format, used by Nginx)
```

**Automatic Setup:**
- Certificates are **automatically downloaded** during EC2 instance provisioning
- See: `terraform/modules/ec2/user_data/pet_info_start.sh`
- No manual intervention required

## mTLS Configuration

**Nginx handles mTLS verification:**
- Verifies client certificates from Meta/Facebook webhooks
- Ensures certificates are signed by DigiCert High Assurance EV Root CA
- Validates Common Name (CN) matches `client.webhooks.fbclientcerts.com`
- Passes verification headers to application for defense-in-depth

See `WHATSAPP_WEBHOOK_SECURITY.md` for complete security documentation.

## Certificate Details

- **Subject**: CN=DigiCert High Assurance EV Root CA, OU=www.digicert.com, O=DigiCert Inc, C=US
- **Valid Until**: November 10, 2031
- **Purpose**: Verify client certificates from Meta/Facebook webhooks
- **Used By**: Nginx reverse proxy (not Rust application)
- **Location**: `/etc/nginx/certs/DigiCertHighAssuranceEVRootCA.pem`

## SSL/TLS Certificates (Server)

Server SSL certificates are managed by **Let's Encrypt** via certbot:

```
/etc/letsencrypt/live/pet-info.link/
├── fullchain.pem       (Server certificate chain)
├── privkey.pem         (Private key)
└── chain.pem           (Intermediate certificates)
```

**Automatic renewal** configured via certbot systemd timer.

## Local Development

For local development without Nginx:
- Set `ENV=local` or `ENV=dev`
- Application runs on `localhost:8080` (HTTP only)
- mTLS verification is disabled in development mode
- No certificate setup required

**Note:** Certificate files (*.crt, *.pem) are excluded from git via `.gitignore`.
