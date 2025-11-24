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
sudo dnf install -y git

# Clone repository
log "Cloning pet-info repository..."
cd /home/ec2-user
git clone --depth 1 --branch ${git_branch} https://github.com/gmLucario/pet-info.git
chown -R ec2-user:ec2-user pet-info/
cd pet-info

# Download DigiCert root CA certificate for mTLS webhook verification
log "Downloading DigiCert High Assurance EV Root CA certificate..."
mkdir -p certs
cd certs

# Download the certificate from DigiCert
log "  Fetching certificate from DigiCert..."
wget -q "https://cacerts.digicert.com/DigiCertHighAssuranceEVRootCA.crt" -O DigiCertHighAssuranceEVRootCA.crt

if [ ! -f DigiCertHighAssuranceEVRootCA.crt ]; then
    log "ERROR: Failed to download DigiCert certificate"
    exit 1
fi

# Convert from DER to PEM format
log "  Converting certificate to PEM format..."
openssl x509 -inform DER -in DigiCertHighAssuranceEVRootCA.crt \
    -out DigiCertHighAssuranceEVRootCA.pem

if [ ! -f DigiCertHighAssuranceEVRootCA.pem ]; then
    log "ERROR: Failed to convert certificate to PEM format"
    exit 1
fi

# Verify the certificate
log "  Verifying certificate..."
openssl x509 -in DigiCertHighAssuranceEVRootCA.pem -text -noout | grep -q "DigiCert High Assurance EV Root CA"
if [ $? -eq 0 ]; then
    log "✓ DigiCert certificate downloaded and verified successfully"
else
    log "ERROR: Certificate verification failed"
    exit 1
fi

# Set proper permissions
chown -R ec2-user:ec2-user /home/ec2-user/pet-info/certs
cd /home/ec2-user/pet-info
log "✓ mTLS certificates configured"

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