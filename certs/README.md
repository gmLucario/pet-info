# SSL/TLS Certificates Directory

This directory contains SSL/TLS certificates used by the application.

## mTLS Client CA Certificate

**Production:**
- The DigiCert High Assurance EV Root CA certificate is **automatically downloaded** during EC2 instance provisioning
- See: `terraform/modules/ec2/user_data/pet_info_start.sh`
- No manual intervention required

**Development/Local:**
- Download the certificate manually for local testing:

```bash
cd certs

# Download the certificate
wget -q "https://cacerts.digicert.com/DigiCertHighAssuranceEVRootCA.crt"

# Convert from DER to PEM format
openssl x509 -inform DER -in DigiCertHighAssuranceEVRootCA.crt \
  -out DigiCertHighAssuranceEVRootCA.pem
```

## Files in this directory

This directory should contain (after setup):
- `DigiCertHighAssuranceEVRootCA.crt` - Root CA certificate (DER format)
- `DigiCertHighAssuranceEVRootCA.pem` - Root CA certificate (PEM format, used by application)

**Note:** Certificate files (*.crt, *.pem) are excluded from git via `.gitignore` as they are automatically provisioned in production environments.

## Certificate Details

- **Subject**: CN=DigiCert High Assurance EV Root CA, OU=www.digicert.com, O=DigiCert Inc, C=US
- **Valid Until**: November 10, 2031
- **Purpose**: Verify client certificates from Meta/Facebook webhooks
- **Used For**: mTLS authentication on WhatsApp webhook endpoints
