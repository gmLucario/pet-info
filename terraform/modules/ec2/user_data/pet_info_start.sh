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
sudo dnf install -y git python3 python3-pip

# Install uv for Python package management
log "Installing uv..."
curl -LsSf https://astral.sh/uv/install.sh | sh
export PATH="/root/.local/bin:$PATH"
echo 'export PATH="/root/.local/bin:$PATH"' >> /home/ec2-user/.bashrc

# Set up initial SSL certificates (provided by Terraform)
log "Setting up initial SSL certificates..."
sudo mkdir -p /opt/pet-info /etc/ssl/certs
echo "${certificate}" | sudo tee /opt/pet-info/server.crt > /dev/null
echo "${private_key_pem}" | sudo tee /opt/pet-info/server.key > /dev/null
sudo chown ec2-user:ec2-user /opt/pet-info/server.crt /opt/pet-info/server.key
sudo chmod 644 /opt/pet-info/server.crt
sudo chmod 600 /opt/pet-info/server.key

# Legacy paths for backward compatibility (symlinks)
sudo ln -sf /opt/pet-info/server.crt /etc/ssl/certs/server.crt
sudo ln -sf /opt/pet-info/server.key /etc/ssl/certs/server.key

# Set up environment variables
log "Configuring environment variables..."
cat <<EOF >> /home/ec2-user/.bashrc
%{ for key, value in instance_envs ~}
export ${key}="${value}"
%{ endfor ~}
EOF
echo "export PRIVATE_KEY_PATH=/opt/pet-info/server.key" >> /home/ec2-user/.bashrc
echo "export CERTIFICATE_PATH=/opt/pet-info/server.crt" >> /home/ec2-user/.bashrc
echo "export APP_CERT_DIR=/opt/pet-info" >> /home/ec2-user/.bashrc
source /home/ec2-user/.bashrc

# Clone repository
log "Cloning pet-info repository..."
VOLUME_UUID=$(lsblk -o UUID -n "/dev/xvdf")
cd /home/ec2-user
git clone https://github.com/gmLucario/pet-info.git --depth 1
chown -R ec2-user:ec2-user pet-info/
cd pet-info

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

# Set up automatic certificate renewal
log "Setting up automatic SSL certificate renewal..."

# Install certbot dependencies
log "Installing certbot and Route53 plugin..."
/root/.local/bin/uv venv /home/ec2-user/pet-info/.venv
source /home/ec2-user/pet-info/.venv/bin/activate
/root/.local/bin/uv pip install certbot certbot-dns-route53

# Copy systemd service and timer files
log "Installing systemd units..."
sudo cp /home/ec2-user/pet-info/systemd/certbot-renewal.service /etc/systemd/system/
sudo cp /home/ec2-user/pet-info/systemd/certbot-renewal.timer /etc/systemd/system/
sudo cp /home/ec2-user/pet-info/systemd/pet-info.service /etc/systemd/system/

# Make renewal script executable
sudo chmod +x /home/ec2-user/pet-info/scripts/renew-certs.sh

# Update renewal script path in systemd service
sudo sed -i "s|/opt/pet-info/scripts/renew-certs.sh|/home/ec2-user/pet-info/scripts/renew-certs.sh|g" /etc/systemd/system/certbot-renewal.service

# Create systemd environment file for renewal script
log "Creating environment configuration..."
sudo tee /etc/sysconfig/certbot-renewal > /dev/null <<EOF
APP_CERT_DIR=/opt/pet-info
DOMAIN=pet-info.link
EOF

# Update systemd service to use environment file
sudo sed -i '/\[Service\]/a EnvironmentFile=/etc/sysconfig/certbot-renewal' /etc/systemd/system/certbot-renewal.service

# Enable and start the renewal timer
log "Enabling automatic certificate renewal..."
sudo systemctl daemon-reload
sudo systemctl enable certbot-renewal.timer
sudo systemctl start certbot-renewal.timer

# Verify timer is running
if sudo systemctl is-active --quiet certbot-renewal.timer; then
    log "✓ Certificate renewal timer activated successfully"
    sudo systemctl status certbot-renewal.timer --no-pager
else
    log "⚠ Warning: Certificate renewal timer failed to start"
fi

# Create initial Let's Encrypt certificate (if terraform certs are temporary)
# Uncomment the following lines if you want to request Let's Encrypt certs immediately
# log "Requesting initial Let's Encrypt certificate..."
# source /home/ec2-user/pet-info/.venv/bin/activate
# sudo -E certbot certonly --dns-route53 -d pet-info.link --non-interactive --agree-tos --email your-email@example.com || log "⚠ Certbot certificate request failed (will retry via timer)"
# if [ -f "/etc/letsencrypt/live/pet-info.link/fullchain.pem" ]; then
#     sudo cp /etc/letsencrypt/live/pet-info.link/fullchain.pem /opt/pet-info/server.crt
#     sudo cp /etc/letsencrypt/live/pet-info.link/privkey.pem /opt/pet-info/server.key
#     sudo chmod 644 /opt/pet-info/server.crt
#     sudo chmod 600 /opt/pet-info/server.key
#     log "✓ Let's Encrypt certificates installed"
# fi

log "=== Certificate renewal automation setup complete ==="
log "Next steps:"
log "  1. Build and deploy the Rust application"
log "  2. Certificates will auto-renew when <30 days to expiry"
log "  3. Check renewal status: sudo systemctl status certbot-renewal.timer"
log "  4. View logs: sudo journalctl -u certbot-renewal.service"
log ""
log "=== pet-info EC2 instance setup complete ==="

# Create completion marker file for terraform provisioner
touch /tmp/user-data-complete