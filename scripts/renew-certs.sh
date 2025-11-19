#!/bin/bash
#
# Certificate Renewal Script for pet-info.link
#
# This script:
# 1. Activates Python virtual environment
# 2. Installs/updates certbot-dns-route53
# 3. Runs certbot renewal (only renews if <30 days remaining)
# 4. Copies renewed certificates to the application directory
# 5. The Rust app will automatically detect and reload the certificates
#
# Usage: Run manually or via systemd timer
# Requirements: AWS credentials configured, Route53 DNS access

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VENV_DIR="$PROJECT_ROOT/.venv"
DOMAIN="pet-info.link"
CERT_BASE_PATH="/etc/letsencrypt/live/$DOMAIN"
APP_CERT_DIR="${APP_CERT_DIR:-/opt/pet-info}"
LOG_FILE="/var/log/certbot-renewal.log"

# Logging function
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

log "=== Starting certificate renewal process ==="

# Step 1: Create/activate virtual environment
log "Setting up Python virtual environment..."
if [ ! -d "$VENV_DIR" ]; then
    uv venv "$VENV_DIR"
fi
source "$VENV_DIR/bin/activate"

# Step 2: Install/update certbot
log "Installing certbot-dns-route53..."
uv pip install certbot-dns-route53 --quiet

# Step 3: Run certbot renewal
log "Running certbot renewal for $DOMAIN..."
if sudo -E certbot renew --dns-route53 --quiet --no-self-upgrade; then
    log "Certbot renewal completed successfully"
else
    EXIT_CODE=$?
    log "ERROR: Certbot renewal failed with exit code $EXIT_CODE"
    exit $EXIT_CODE
fi

# Step 4: Check if certificates exist
if [ ! -f "$CERT_BASE_PATH/fullchain.pem" ] || [ ! -f "$CERT_BASE_PATH/privkey.pem" ]; then
    log "ERROR: Certificate files not found at $CERT_BASE_PATH"
    exit 1
fi

# Step 5: Copy certificates to application directory
log "Copying certificates to $APP_CERT_DIR..."
sudo cp "$CERT_BASE_PATH/fullchain.pem" "$APP_CERT_DIR/server.crt"
sudo cp "$CERT_BASE_PATH/privkey.pem" "$APP_CERT_DIR/server.key"

# Set proper permissions and ownership for ec2-user
sudo chown ec2-user:ec2-user "$APP_CERT_DIR/server.crt" "$APP_CERT_DIR/server.key"
sudo chmod 644 "$APP_CERT_DIR/server.crt"
sudo chmod 600 "$APP_CERT_DIR/server.key"

log "Certificates copied successfully"

# Step 6: Verify certificate expiration
EXPIRY=$(sudo openssl x509 -enddate -noout -in "$APP_CERT_DIR/server.crt" | cut -d= -f2)
log "Certificate expires: $EXPIRY"

log "=== Certificate renewal process completed successfully ==="
log "Note: The Rust application will automatically detect and reload the new certificates"

exit 0
