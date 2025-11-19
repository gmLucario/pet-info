#!/bin/bash
#
# Verbose TLS connection test

echo "=== Testing TLS handshake with maximum verbosity ==="
echo ""

echo "[Test 1] OpenSSL s_client with -debug and -state flags"
echo "This will show exactly where the handshake fails..."
timeout 10 openssl s_client -connect localhost:443 -servername pet-info.link -debug -state -msg 2>&1 | head -100

echo ""
echo "[Test 2] Check if there are any connection attempts in logs"
echo "Watching journalctl while making a connection..."
(
    sleep 2
    timeout 5 curl -v -k https://localhost/ --max-time 3 2>&1 &
) &
sudo journalctl -u pet-info.service -f -n 0 --since "1 second ago" &
JOURNAL_PID=$!
sleep 8
kill $JOURNAL_PID 2>/dev/null || true

echo ""
echo "[Test 3] Check system limits"
echo "File descriptors:"
cat /proc/$(pgrep pet-info)/limits | grep "open files"
echo ""
echo "Current open files by pet-info:"
sudo lsof -p $(pgrep pet-info) | wc -l
