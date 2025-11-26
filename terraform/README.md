# Certificate


```bash
uv venv && source .venv/bin/activate && uv pip install certbot-dns-route53
```

## Create or renew certificate
```bash
sudo -E certbot certonly --dns-route53 -d pet-info.link
```

```bash
Certificate is saved at: /etc/letsencrypt/live/pet-info.link/fullchain.pem (1st section)
Key is saved at:         /etc/letsencrypt/live/pet-info.link/privkey.pem
```

## Certificate info
```bash
sudo certbot certificates
```

```py3
from datetime import datetime, timezone
from zoneinfo import ZoneInfo

expiry_date = datetime.fromisoformat("2026-02-17 18:48:29+00:00")
expiry_date.astimezone(ZoneInfo("America/Mexico_City")).strftime("%A, %B %d, %Y")
'Tuesday, February 17, 2026'
```

# Run the app
inside the instance: 
```bash
git reset --hard origin/main

cd /home/ec2-user/ && sudo chown -R ec2-user:ec2-user pet-info/
```

```bash
sudo -E ENV=prod nohup ./pet-info > app.log 2>&1 &
sudo -E nohup ./pet-info > app.log 2>&1 &

# Stop the app
pkill -f pet-info

# Copy new executable (after uploading it)
# chmod +x pet-info (if needed)

# Apply capability to bind to port 443
sudo setcap CAP_NET_BIND_SERVICE=+ep /home/ec2-user/pet-info/web_app/pet-info

# Start the app
cd /home/ec2-user/pet-info/web_app && source ~/.bashrc && nohup ./pet-info > app.log 2>&1 &

# Verify it's running
sleep 2 && ps aux | grep pet-info
```

[1] 27340

local
```bash
scp -i /Users/gmlucario/Documents/learning/rust/pet-info/terraform/pet-info.pem /Users/gmlucario/Documents/learning/rust/pet-info/web_app/out/pet-info  ec2-user@3.23.154.245:/home/ec2-user/pet-info/web_app
```