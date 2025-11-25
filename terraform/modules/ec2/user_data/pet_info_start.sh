#!/bin/bash
set -euo pipefail

# Log function for tracking setup progress
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" | tee -a /var/log/user-data.log
}

log "=== Starting pet-info EC2 instance setup ==="

# Update system and install dependencies
log "Installing system dependencies..."
sudo dnf update -y
sudo dnf install -y git openssl

# Install Nginx
log "Installing Nginx..."
sudo dnf install -y nginx
log "✓ Nginx installed"

# Clone repository
log "Cloning pet-info repository..."
cd /home/ec2-user
git clone --depth 1 --branch ${git_branch} https://github.com/gmLucario/pet-info.git
chown -R ec2-user:ec2-user pet-info/
cd pet-info

# Download DigiCert root CA certificate for mTLS webhook verification
log "Downloading DigiCert High Assurance EV Root CA certificate for Nginx mTLS..."
sudo mkdir -p /etc/nginx/certs

# Download the certificate from DigiCert
log "  Fetching certificate from DigiCert..."
sudo wget -q "https://cacerts.digicert.com/DigiCertHighAssuranceEVRootCA.crt" \
    -O /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.crt

if [ ! -f /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.crt ]; then
    log "ERROR: Failed to download DigiCert certificate"
    exit 1
fi

# Convert from DER to PEM format
log "  Converting certificate to PEM format..."
sudo openssl x509 -inform DER -in /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.crt \
    -out /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.pem

if [ ! -f /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.pem ]; then
    log "ERROR: Failed to convert certificate to PEM format"
    exit 1
fi

# Verify the certificate
log "  Verifying certificate..."
sudo openssl x509 -in /etc/nginx/certs/DigiCertHighAssuranceEVRootCA.pem -text -noout | grep -q "DigiCert High Assurance EV Root CA"
if [ $? -eq 0 ]; then
    log "✓ DigiCert certificate downloaded and verified successfully"
else
    log "ERROR: Certificate verification failed"
    exit 1
fi

# Set proper permissions for certificate directory
sudo chmod 755 /etc/nginx/certs
sudo chmod 644 /etc/nginx/certs/*
log "✓ mTLS certificates configured for Nginx"

# Install certbot for Let's Encrypt
log "Installing certbot for Let's Encrypt..."
sudo dnf install -y certbot python3-certbot-nginx
log "✓ Certbot installed"

# Create directory for Let's Encrypt ACME challenges
log "Setting up Let's Encrypt directories..."
sudo mkdir -p /var/www/certbot
sudo chown -R nginx:nginx /var/www/certbot
log "✓ Let's Encrypt directories created"

# Copy Nginx configuration
log "Configuring Nginx..."
sudo cp /home/ec2-user/pet-info/terraform/modules/ec2/files/nginx-pet-info.conf \
    /etc/nginx/conf.d/pet-info.conf

# Test Nginx configuration (will fail initially due to missing SSL certs, but validates syntax)
log "Testing Nginx configuration syntax..."
sudo nginx -t || log "WARNING: Nginx config test failed (expected if SSL certs not yet provisioned)"

# Note: SSL certificates will be provisioned separately via certbot
# For initial setup, you'll need to run:
# sudo certbot --nginx -d pet-info.link -d www.pet-info.link --non-interactive --agree-tos -m your-email@example.com

log "✓ Nginx configured (SSL certificates need to be provisioned separately)"

# Enable and start Nginx (will be started after SSL cert provisioning)
sudo systemctl enable nginx
log "✓ Nginx enabled to start on boot"

# Wait for EBS volume to be attached
log "Waiting for EBS volume to be attached..."
while [ ! -b /dev/xvdf ]; do
    log "  EBS volume /dev/xvdf not yet available, waiting..."
    sleep 5
done
log "✓ EBS volume detected"

# Get volume UUID
VOLUME_UUID=$(lsblk -o UUID -n "/dev/xvdf")
log "Volume UUID: $VOLUME_UUID"

# Mount data volume
log "Setting up data volume..."
if ! lsblk -o MOUNTPOINT | grep -q "pet-info"; then
    mkdir -p data
    sudo mount -U $VOLUME_UUID /home/ec2-user/pet-info/data
    sudo chown -R ec2-user:ec2-user /home/ec2-user/pet-info/data
    touch /home/ec2-user/pet-info/data/pet_info.sqlite
    echo "UUID=$VOLUME_UUID /home/ec2-user/pet-info/data ext4 defaults,nofail 0 2" >> /etc/fstab
fi

# Wait for volume to be mounted
until lsblk -o MOUNTPOINT | grep -q "pet-info"; do
    log "Waiting for data volume to mount..."
    sleep 5
done
log "Data volume mounted successfully"

log "=== pet-info EC2 instance setup complete ==="

# Create completion marker file for terraform provisioner
touch /tmp/user-data-complete