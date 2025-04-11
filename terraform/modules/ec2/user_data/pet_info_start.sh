#!/bin/bash
sudo dnf update -y
sudo dnf install -y git
echo "${certificate}" | sudo tee /etc/ssl/certs/server.crt > /dev/null
echo "${private_key_pem}" | sudo tee /etc/ssl/certs/server.key > /dev/null
sudo chmod o+r /etc/ssl/certs/server.crt /etc/ssl/certs/server.key
cat <<EOF >> /home/ec2-user/.bashrc
%{ for key, value in instance_envs ~}
export ${key}="${value}"
%{ endfor ~}
EOF
echo "export PRIVATE_KEY_PATH=/etc/ssl/certs/server.key" >> /home/ec2-user/.bashrc
echo "export CERTIFICATE_PATH=/etc/ssl/certs/server.crt" >> /home/ec2-user/.bashrc
source /home/ec2-user/.bashrc
VOLUME_UUID=$(lsblk -o UUID -n "/dev/xvdf")
cd home/ec2-user && git clone https://github.com/gmLucario/pet-info.git --depth 1 && chown -R ec2-user:ec2-user pet-info/ && cd pet-info
if ! lsblk -o MOUNTPOINT | grep -q "pet-info"; then
    mkdir -p data
    sudo mount -U $VOLUME_UUID /home/ec2-user/pet-info/data
    sudo chown -R ec2-user:ec2-user /home/ec2-user/pet-info/data
    touch /home/ec2-user/pet-info/data/pet_info.sqlite
    echo "UUID=$VOLUME_UUID /home/ec2-user/pet-info/data ext4 defaults,nofail 0 2" >> /etc/fstab
fi
until lsblk -o MOUNTPOINT | grep -q "pet-info"; do
    sleep 5
done