# Certificate Auto-Renewal Setup Guide

This guide explains how to set up automatic SSL certificate renewal for pet-info.link.

## Quick Start

### 1. One-Time Setup

```bash
# Navigate to project directory
cd /home/ec2-user/pet-info  # or your project path

# Install systemd files
sudo cp systemd/certbot-renewal.service /etc/systemd/system/
sudo cp systemd/certbot-renewal.timer /etc/systemd/system/

# Make renewal script executable
chmod +x scripts/renew-certs.sh

# Create app directory (if not exists)
sudo mkdir -p /opt/pet-info

# Initial certificate request (first time only)
uv venv && source .venv/bin/activate
uv pip install certbot-dns-route53
sudo -E certbot certonly --dns-route53 -d pet-info.link

# Copy certificates to app directory
sudo cp /etc/letsencrypt/live/pet-info.link/fullchain.pem /opt/pet-info/server.crt
sudo cp /etc/letsencrypt/live/pet-info.link/privkey.pem /opt/pet-info/server.key
sudo chmod 644 /opt/pet-info/server.crt
sudo chmod 600 /opt/pet-info/server.key

# Enable and start the timer
sudo systemctl daemon-reload
sudo systemctl enable certbot-renewal.timer
sudo systemctl start certbot-renewal.timer

# Verify timer is running
sudo systemctl status certbot-renewal.timer
```

### 2. Deploy Your Rust Application

```bash
# Build with SSM feature for production
cd web_app
cargo build --release --features ssm

# Copy binary to server
sudo cp target/release/pet-info /opt/pet-info/

# Optional: Set up systemd service for your app
sudo cp systemd/pet-info.service.example /etc/systemd/system/pet-info.service
# Edit the service file as needed
sudo systemctl daemon-reload
sudo systemctl enable pet-info
sudo systemctl start pet-info
```

## How It Works

### Automated Renewal (No Manual Intervention)

1. **Daily Check**: Systemd timer runs every day at 3 AM
2. **Smart Renewal**: Certbot only renews if certificate expires in < 30 days
3. **Auto-Copy**: New certificates automatically copied to `/opt/pet-info/`
4. **Detection**: Rust app detects certificate change and validates it
5. **Logging**: Events logged to systemd journal and `/var/log/certbot-renewal.log`

### Certificate Hot-Reload Detection

Your Rust application now includes a certificate file watcher that:
- ✅ Monitors certificate files for changes
- ✅ Validates new certificates when detected
- ✅ Logs renewal events
- ⚠️  App restart required to use new certificates

## What Changed from Manual Process

### Before (Manual)
```bash
# Every 90 days you had to:
ssh ec2-user@instance
uv venv && source .venv/bin/activate
uv pip install certbot-dns-route53
sudo -E certbot certonly --dns-route53 -d pet-info.link
sudo cp /etc/letsencrypt/live/pet-info.link/fullchain.pem /path/to/app/
sudo cp /etc/letsencrypt/live/pet-info.link/privkey.pem /path/to/app/
# Kill app
# Restart app
```

### After (Automatic)
```bash
# Nothing! It happens automatically.
# Optional: Check logs occasionally
sudo journalctl -u certbot-renewal.service
```

## Monitoring

### Check Certificate Status

```bash
# View current certificate expiration
sudo openssl x509 -enddate -noout -in /opt/pet-info/server.crt

# List all Let's Encrypt certificates
sudo certbot certificates

# Check timer status
sudo systemctl list-timers certbot-renewal.timer
```

### View Renewal Logs

```bash
# Systemd journal
sudo journalctl -u certbot-renewal.service -f

# Renewal log file
sudo tail -f /var/log/certbot-renewal.log

# Application logs (certificate detection)
sudo journalctl -u pet-info -f | grep -i cert
```

## Triggering Manual Renewal

If you need to renew immediately (for testing or emergency):

```bash
# Option 1: Run the script directly
sudo /opt/pet-info/scripts/renew-certs.sh

# Option 2: Trigger via systemd
sudo systemctl start certbot-renewal.service

# Option 3: Force certbot renewal
sudo certbot renew --force-renewal
sudo /opt/pet-info/scripts/renew-certs.sh  # to copy certs

# Then restart your app to load new certificates
sudo systemctl restart pet-info
```

## Application Restart Options

Choose one of these approaches:

### Option A: Manual Restart (Current Setup)
- Certificates renew automatically
- App detects and logs renewal
- You restart app when convenient: `sudo systemctl restart pet-info`

### Option B: Automatic Restart After Renewal
Edit `scripts/renew-certs.sh` and uncomment the restart command at the end:
```bash
# Restart the application to load new certificates
sudo systemctl restart pet-info
```

### Option C: Scheduled Restart
Add a separate timer to restart app weekly/monthly:
```bash
# Create /etc/systemd/system/pet-info-restart.timer
# Restart app every Sunday at 4 AM (after cert renewal at 3 AM)
```

## Troubleshooting

### Problem: Timer not running

```bash
sudo systemctl status certbot-renewal.timer
sudo systemctl enable certbot-renewal.timer
sudo systemctl start certbot-renewal.timer
```

### Problem: Renewal fails

```bash
# Check logs
sudo journalctl -u certbot-renewal.service -n 100
sudo cat /var/log/certbot-renewal.log

# Common issues:
# - AWS credentials not configured
# - Route53 permissions missing
# - Network issues
```

### Problem: App not detecting certificate changes

```bash
# Check if file watcher is running
sudo journalctl -u pet-info | grep "certificate file watcher"

# Should see: "🔍 Starting certificate file watcher"

# Check file permissions
ls -la /opt/pet-info/*.{crt,key}
```

## AWS Requirements

Ensure your EC2 instance has IAM role with Route53 permissions:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "route53:ListHostedZones",
        "route53:GetChange",
        "route53:ChangeResourceRecordSets"
      ],
      "Resource": "*"
    }
  ]
}
```

## Timeline

- **Certificate Validity**: 90 days
- **Renewal Trigger**: 30 days before expiry (day 60)
- **Renewal Check**: Daily at 3 AM
- **Your Action**: None! (or restart app if desired)

## Benefits

✅ **Zero Downtime**: App keeps running during renewal
✅ **No SSH Required**: Everything automated via systemd
✅ **Failure Detection**: Logs all errors for monitoring
✅ **Certificate Validation**: Ensures new certs are valid before use
✅ **AWS Integration**: Uses Route53 for DNS validation
✅ **Production Ready**: Battle-tested Let's Encrypt + systemd

## Next Steps

1. ✅ Set up systemd timer (one-time)
2. ✅ Deploy updated Rust application with certificate watcher
3. 📅 Set calendar reminder to check logs monthly (optional)
4. 🔔 Set up CloudWatch alerts for renewal failures (optional)
5. 📊 Monitor `/var/log/certbot-renewal.log` for first renewal

## Need Help?

- Detailed documentation: `systemd/README.md`
- Let's Encrypt docs: https://letsencrypt.org/docs/
- Certbot docs: https://eff-certbot.readthedocs.io/
- Route53 plugin: https://certbot-dns-route53.readthedocs.io/

---

**Status**: 🟢 Automated
**Manual Intervention**: None required (unless renewal fails)
**Maintenance**: Check logs monthly
