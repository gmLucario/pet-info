# Certificate


```bash
uv venv && source .venv/bin/activate && uv pip install certbot-dns-route53
```

## Create or renew certificate
```bash
sudo -E certbot certonly --dns-route53 -d pet-info.link
```

```bash
Certificate is saved at: /etc/letsencrypt/live/pet-info.link/fullchain.pem
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
