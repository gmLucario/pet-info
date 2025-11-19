#!/bin/bash
#
# TLS Connection Diagnostic Script
# Helps identify why HTTPS connections are timing out

set -euo pipefail

echo "================================"
echo "TLS Connection Diagnostics"
echo "================================"
echo ""

# 1. Check if app is running
echo "[1] Checking if pet-info service is running..."
sudo systemctl status pet-info.service --no-pager || true
echo ""

# 2. Check what ports the app is listening on
echo "[2] Checking listening ports..."
sudo ss -tlnp | grep pet-info || echo "No pet-info process found listening"
echo ""

# 3. Check certificate files
echo "[3] Checking SSL certificate files..."
echo "Certificate file:"
ls -lh /opt/pet-info/server.crt || echo "Certificate file not found"
echo "Private key file:"
ls -lh /opt/pet-info/server.key || echo "Private key file not found"
echo ""

# 4. Validate certificate
echo "[4] Validating SSL certificate..."
openssl x509 -in /opt/pet-info/server.crt -noout -text | grep -E "(Subject:|Issuer:|Not Before|Not After|DNS:)" || true
echo ""

# 5. Check if certificate and key match
echo "[5] Checking if certificate and key match..."
CERT_MODULUS=$(openssl x509 -noout -modulus -in /opt/pet-info/server.crt | openssl md5)
KEY_MODULUS=$(openssl rsa -noout -modulus -in /opt/pet-info/server.key | openssl md5)
echo "Certificate modulus MD5: $CERT_MODULUS"
echo "Private key modulus MD5: $KEY_MODULUS"
if [ "$CERT_MODULUS" = "$KEY_MODULUS" ]; then
    echo "✓ Certificate and private key match"
else
    echo "✗ ERROR: Certificate and private key do NOT match!"
fi
echo ""

# 6. Test localhost connection
echo "[6] Testing localhost HTTPS connection..."
timeout 5 openssl s_client -connect localhost:443 -servername pet-info.link </dev/null 2>&1 | head -30 || echo "Connection failed or timed out"
echo ""

# 7. Check application logs
echo "[7] Recent application logs..."
sudo journalctl -u pet-info.service --no-pager -n 50 || true
echo ""

# 8. Check for iptables rules
echo "[8] Checking iptables rules..."
sudo iptables -L -n -v | head -20 || echo "Unable to check iptables"
echo ""

# 9. Check SSM parameters
echo "[9] Checking SSM parameters..."
aws ssm get-parameters-by-path --path "/pet-info/" --recursive --region us-east-2 2>&1 | jq -r '.Parameters[] | "\(.Name): \(.Type)"' || echo "Unable to fetch SSM parameters"
echo ""

# 10. Test TCP connection
echo "[10] Testing raw TCP connection to port 443..."
timeout 5 bash -c 'cat < /dev/null > /dev/tcp/localhost/443' && echo "✓ TCP connection successful" || echo "✗ TCP connection failed"
echo ""

echo "================================"
echo "Diagnostics complete"
echo "================================"
