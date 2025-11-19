# SSL Certificate Auto-Renewal Setup

This directory contains the systemd configuration for automatic SSL certificate renewal for pet-info.link.

## 🎯 Quick Start

**Using Terraform?** Good news! Just run `terraform apply` and everything is configured automatically. The EC2 user_data script handles the entire setup.

**Manual setup?** Follow the [Installation](#installation) section below.

## Overview

The certificate renewal system consists of two components:

1. **Automated Renewal** - Systemd timer that runs certbot daily
2. **Auto-Reload Detection** - Rust application monitors certificate files and logs when renewal occurs

## Terraform Integration

These systemd files are **automatically deployed** when you provision your EC2 instance via Terraform. The `terraform/modules/ec2/user_data/pet_info_start.sh` script:

- ✅ Installs all dependencies (Python, uv, certbot)
- ✅ Copies systemd files to `/etc/systemd/system/`
- ✅ Configures environment variables
- ✅ Enables and starts the renewal timer
- ✅ Verifies the setup

No manual intervention required!

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Systemd Timer                            │
│              (Runs daily at 3 AM)                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Certbot Renewal Script                          │
│  1. Checks if renewal needed (<30 days to expiry)           │
│  2. Runs certbot with Route53 DNS validation                │
│  3. Copies new certificates to /opt/pet-info/               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│           Certificate File Watcher (Rust)                    │
│  1. Detects certificate file changes                         │
│  2. Validates new certificates                               │
│  3. Logs renewal event                                       │
│  4. App continues serving with old certs until restart       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Systemd Auto-Restart                            │
│  Optional: Configure restart policy to reload certs         │
└─────────────────────────────────────────────────────────────┘
```

## Files

- `certbot-renewal.service` - Systemd service for running renewal script
- `certbot-renewal.timer` - Systemd timer (daily at 3 AM)
- `../scripts/renew-certs.sh` - Bash script that handles renewal

## Installation

### 1. Copy systemd files

```bash
sudo cp systemd/certbot-renewal.service /etc/systemd/system/
sudo cp systemd/certbot-renewal.timer /etc/systemd/system/
```

### 2. Make the renewal script executable

```bash
chmod +x scripts/renew-certs.sh
```

### 3. Create the installation directory (if not exists)

```bash
sudo mkdir -p /opt/pet-info
```

### 4. Configure environment variables

Edit the systemd service or create an environment file:

```bash
# Set the app installation directory
export APP_CERT_DIR=/opt/pet-info
```

### 5. Enable and start the timer

```bash
# Reload systemd to recognize new files
sudo systemctl daemon-reload

# Enable the timer to start on boot
sudo systemctl enable certbot-renewal.timer

# Start the timer immediately
sudo systemctl start certbot-renewal.timer

# Check timer status
sudo systemctl status certbot-renewal.timer
```

## Verification

### Check timer status

```bash
# List all timers
sudo systemctl list-timers

# View timer details
sudo systemctl status certbot-renewal.timer
```

### Test manual renewal

```bash
# Run the renewal script manually
sudo /opt/pet-info/scripts/renew-certs.sh

# Or trigger via systemd
sudo systemctl start certbot-renewal.service

# Check logs
sudo journalctl -u certbot-renewal.service -n 50
```

### Monitor renewal logs

```bash
# Real-time logs
sudo journalctl -u certbot-renewal.service -f

# View renewal log file
sudo tail -f /var/log/certbot-renewal.log
```

## Certificate Lifecycle

1. **Initial Setup** (Manual)
   ```bash
   # First time: Request certificate
   uv venv && source .venv/bin/activate
   uv pip install certbot-dns-route53
   sudo -E certbot certonly --dns-route53 -d pet-info.link

   # Copy to app directory
   sudo cp /etc/letsencrypt/live/pet-info.link/fullchain.pem /opt/pet-info/server.crt
   sudo cp /etc/letsencrypt/live/pet-info.link/privkey.pem /opt/pet-info/server.key
   ```

2. **Automatic Renewal** (Systemd Timer)
   - Timer triggers daily at 3 AM
   - Certbot checks if renewal needed (< 30 days to expiry)
   - If needed, requests new certificate from Let's Encrypt
   - Script copies new certificates to `/opt/pet-info/`
   - Rust app detects change and logs event

3. **Certificate Reload** (Automatic Detection)
   - Rust app's file watcher detects certificate changes
   - Validates new certificates (ensures they're not corrupted)
   - Logs renewal event to application logs
   - App continues running with old certificates until restarted

## Let's Encrypt Certificate Details

- **Validity Period**: 90 days
- **Renewal Threshold**: 30 days before expiry
- **Renewal Method**: DNS-01 challenge via AWS Route53
- **Storage Location**: `/etc/letsencrypt/live/pet-info.link/`
- **Application Location**: `/opt/pet-info/`

## AWS Requirements

The renewal process requires:

1. **IAM Permissions** for Route53:
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Effect": "Allow",
         "Action": [
           "route53:ListHostedZones",
           "route53:GetChange"
         ],
         "Resource": "*"
       },
       {
         "Effect": "Allow",
         "Action": "route53:ChangeResourceRecordSets",
         "Resource": "arn:aws:route53:::hostedzone/YOUR_ZONE_ID"
       }
     ]
   }
   ```

2. **AWS Credentials** configured on the EC2 instance:
   - Instance IAM role (recommended)
   - Or AWS credentials in `~/.aws/credentials`

## Monitoring & Alerts

### Check certificate expiration

```bash
# Check current certificate expiration
sudo openssl x509 -enddate -noout -in /opt/pet-info/server.crt

# Check Let's Encrypt certificate
sudo certbot certificates
```

### Application logs

The Rust application logs certificate events:

```bash
# View app logs (adjust path to your systemd service)
sudo journalctl -u pet-info -f | grep -i cert

# Look for these messages:
# "🔍 Starting certificate file watcher"
# "🔄 Detected certificate file change"
# "🔐 New SSL certificates detected and validated!"
# "⚠️  Server restart required to use new certificates"
```

### Set up monitoring alerts

Consider setting up CloudWatch alarms or log monitoring for:
- Certificate renewal failures
- Certificate validation errors
- Certificates expiring in < 14 days

## Troubleshooting

### Timer not running

```bash
# Check if timer is enabled
sudo systemctl is-enabled certbot-renewal.timer

# Enable it
sudo systemctl enable certbot-renewal.timer

# Start it
sudo systemctl start certbot-renewal.timer
```

### Renewal script fails

```bash
# Check service logs
sudo journalctl -u certbot-renewal.service -n 100

# Check renewal log
sudo cat /var/log/certbot-renewal.log

# Common issues:
# 1. AWS credentials not configured
# 2. Route53 permissions missing
# 3. Python venv issues
# 4. Network connectivity
```

### Certificates not being detected

```bash
# Check file permissions
ls -la /opt/pet-info/*.{crt,key}

# Should be:
# -rw-r--r-- root root server.crt
# -rw------- root root server.key

# Check if watcher is running (in app logs)
sudo journalctl -u pet-info | grep "certificate file watcher"
```

### Manual certificate renewal

If you need to renew immediately:

```bash
# Force renewal
sudo certbot renew --force-renewal

# Or use the script
sudo /opt/pet-info/scripts/renew-certs.sh

# Then restart the app
sudo systemctl restart pet-info
```

## Optional: Auto-restart on Certificate Renewal

If you want the application to automatically restart when certificates are renewed, you can configure the renewal script to trigger a service restart:

### Option A: Add restart to renewal script

Edit `scripts/renew-certs.sh` and add at the end:

```bash
# Restart the application to load new certificates
log "Restarting pet-info service..."
sudo systemctl restart pet-info
log "Service restarted successfully"
```

### Option B: Create a separate watcher service

Create a systemd path unit that watches certificate files and triggers restart:

```ini
# /etc/systemd/system/cert-reload.path
[Unit]
Description=Watch for SSL certificate changes

[Path]
PathModified=/opt/pet-info/server.crt
PathModified=/opt/pet-info/server.key

[Install]
WantedBy=multi-user.target
```

```ini
# /etc/systemd/system/cert-reload.service
[Unit]
Description=Reload pet-info on certificate change

[Service]
Type=oneshot
ExecStart=/bin/systemctl restart pet-info
```

Enable:
```bash
sudo systemctl enable cert-reload.path
sudo systemctl start cert-reload.path
```

## Security Considerations

1. **File Permissions**: Private keys have 600 permissions (root only)
2. **AWS Credentials**: Use IAM instance roles instead of access keys
3. **Logging**: Renewal logs are written to `/var/log/certbot-renewal.log`
4. **Validation**: New certificates are validated before logging success
5. **No Downtime**: App continues serving with old certs until restart

## Testing

### Test the complete workflow

```bash
# 1. Check current cert expiration
sudo openssl x509 -enddate -noout -in /opt/pet-info/server.crt

# 2. Manually trigger renewal (only works if < 30 days to expiry)
sudo systemctl start certbot-renewal.service

# 3. Or force renewal for testing
cd /home/ec2-user/pet-info  # or your project directory
source .venv/bin/activate
sudo -E certbot renew --force-renewal --cert-name pet-info.link

# 4. Run the renewal script
sudo ./scripts/renew-certs.sh

# 5. Check app logs for detection
sudo journalctl -u pet-info -n 20 | grep cert

# 6. Verify new cert was copied
sudo openssl x509 -enddate -noout -in /opt/pet-info/server.crt
```

## Maintenance

### Regular checks (monthly)

1. Verify timer is running: `sudo systemctl status certbot-renewal.timer`
2. Check last renewal date: `sudo certbot certificates`
3. Review logs for errors: `sudo journalctl -u certbot-renewal.service --since "1 month ago"`
4. Test manual renewal: `sudo /opt/pet-info/scripts/renew-certs.sh`

### Before expiry (< 7 days)

If automated renewal fails and certificate is about to expire:

1. Check logs: `sudo journalctl -u certbot-renewal.service -n 200`
2. Verify AWS credentials: `aws sts get-caller-identity`
3. Verify Route53 access: `aws route53 list-hosted-zones`
4. Force manual renewal: `sudo certbot renew --force-renewal`
5. Restart application: `sudo systemctl restart pet-info`

## Resources

- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
- [Certbot Documentation](https://eff-certbot.readthedocs.io/)
- [Certbot Route53 Plugin](https://certbot-dns-route53.readthedocs.io/)
- [Systemd Timer Documentation](https://www.freedesktop.org/software/systemd/man/systemd.timer.html)
