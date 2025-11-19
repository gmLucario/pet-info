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