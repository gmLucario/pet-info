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

# Copy Meta Outbound API CA certificate
log "Configuring Meta Outbound API CA for Nginx mTLS..."
sudo mkdir -p /etc/nginx/certs
sudo cp /home/ec2-user/pet-info/terraform/modules/ec2/files/MetaOutboundAPICA2025-12.pem \
    /etc/nginx/certs/MetaOutboundAPICA2025-12.pem

# Set proper permissions for certificate directory
sudo chmod 755 /etc/nginx/certs
sudo chmod 644 /etc/nginx/certs/MetaOutboundAPICA2025-12.pem
log "✓ Meta mTLS certificate authority configured for Nginx"

# Configure Nginx (but don't start it yet - SSL certs will be uploaded by Terraform provisioner)
log "Configuring Nginx..."
sudo cp /home/ec2-user/pet-info/terraform/modules/ec2/files/nginx-pet-info.conf \
    /etc/nginx/conf.d/pet-info.conf

# Enable Nginx to start on boot (but don't start it yet)
log "Enabling Nginx for startup..."
sudo systemctl enable nginx
log "✓ Nginx configured and enabled (will be started after SSL certificates are uploaded)"

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
touch /home/ec2-user/user-data-complete
chown ec2-user:ec2-user /home/ec2-user/user-data-complete
