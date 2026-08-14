data "aws_region" "this" {}

resource "aws_instance" "app_instance" {
  ami                  = data.aws_ami.amazon_arm.id
  instance_type        = "t4g.micro"
  key_name             = aws_key_pair.web_app_key.key_name
  iam_instance_profile = var.instance_profile_name
  availability_zone    = aws_ebs_volume.db.availability_zone

  vpc_security_group_ids      = [aws_security_group.web_app_sg.id]
  associate_public_ip_address = false
  subnet_id                   = data.aws_subnet.selected.id

  metadata_options {
    http_endpoint               = "enabled"
    http_tokens                 = "required"
    http_put_response_hop_limit = 1
    instance_metadata_tags      = "disabled"
  }

  user_data = templatefile(
    var.user_data_path,
    {
      git_branch = var.git_branch
    }
  )

  tags = {
    Name = "pet-info-app"
  }
}

# Deploy the application binary
resource "null_resource" "deploy_app" {
  depends_on = [
    aws_instance.app_instance,
    aws_eip_association.ip_ec2,
    aws_volume_attachment.ebs_att
  ]

  # Deploy only on initial instance creation
  triggers = {
    instance_id = aws_instance.app_instance.id
  }

  # Wait for user-data script to complete
  provisioner "remote-exec" {
    inline = [
      "echo 'Waiting for user-data script to complete...'",
      "while [ ! -f /home/ec2-user/user-data-complete ]; do echo 'File not found yet, listing /home/ec2-user:'; ls -la /home/ec2-user/; sleep 10; done",
      "echo 'User-data script completed, ready for deployment'"
    ]

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "15m"
    }
  }

  # Copy application binary
  provisioner "file" {
    source      = var.web_app_executable_path
    destination = "/tmp/pet-info"

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }

  # Move binary to final location and start the application
  provisioner "remote-exec" {
    inline = concat([
      "mkdir -p /home/ec2-user/pet-info/web_app",
      "mv /tmp/pet-info /home/ec2-user/pet-info/web_app/pet-info",
      "chmod +x /home/ec2-user/pet-info/web_app/pet-info",
      ], [
      for key, value in var.instance_envs : "echo 'export ${key}=${value}' >> /home/ec2-user/.bashrc"
      ], [
      "cd /home/ec2-user/pet-info/web_app",
      "source ~/.bashrc && nohup ./pet-info > /dev/null 2>&1 &",
      "sleep 2",
      "echo 'Server started in background'"
    ])

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }
}

# Upload SSL certificates for Nginx HTTPS
resource "null_resource" "upload_ssl_certificates" {
  depends_on = [null_resource.deploy_app]

  # Force re-upload on every deployment
  triggers = {
    instance_id = aws_instance.app_instance.id
    always_run  = timestamp()
  }

  # Upload server certificate
  provisioner "file" {
    source      = var.cert_details.server_path
    destination = "/tmp/server.crt"

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }

  # Upload server private key
  provisioner "file" {
    source      = var.cert_details.key_path
    destination = "/tmp/server.key"

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }

  # Upload Meta Outbound API CA certificate
  provisioner "file" {
    source      = "${path.module}/files/MetaOutboundAPICA2025-12.pem"
    destination = "/tmp/MetaOutboundAPICA2025-12.pem"

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }

  # Move SSL certificates to their final locations. The dependent Nginx
  # configuration resource validates the complete setup before starting it.
  provisioner "remote-exec" {
    inline = [
      "sudo install -d -o root -g root -m 0700 /etc/nginx/private",
      "sudo install -o root -g root -m 0644 /tmp/server.crt /etc/nginx/server.crt",
      "sudo install -o root -g root -m 0600 /tmp/server.key /etc/nginx/private/server.key",
      "rm -f /tmp/server.crt /tmp/server.key",
      "sudo mkdir -p /etc/nginx/certs",
      "sudo mv /tmp/MetaOutboundAPICA2025-12.pem /etc/nginx/certs/MetaOutboundAPICA2025-12.pem",
      "sudo chmod 755 /etc/nginx/certs",
      "sudo chmod 644 /etc/nginx/certs/MetaOutboundAPICA2025-12.pem",
      "echo 'SSL certificates uploaded; Nginx will be started after its configuration is deployed'"
    ]

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }
}

# Deploy Nginx configuration whenever the local configuration changes.
# user_data only runs when the instance is created, so it cannot propagate
# subsequent webhook proxy changes to an existing instance.
resource "null_resource" "deploy_nginx_configuration" {
  depends_on = [null_resource.upload_ssl_certificates]

  triggers = {
    instance_id      = aws_instance.app_instance.id
    config_hash      = filesha256("${path.module}/files/nginx-pet-info.conf")
    certificate_hash = filesha256(var.cert_details.server_path)
    private_key_hash = filesha256(var.cert_details.key_path)
    meta_ca_hash     = filesha256("${path.module}/files/MetaOutboundAPICA2025-12.pem")
  }

  provisioner "file" {
    source      = "${path.module}/files/nginx-pet-info.conf"
    destination = "/tmp/nginx-pet-info.conf"

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }

  provisioner "remote-exec" {
    inline = [
      "sudo install -m 0644 /tmp/nginx-pet-info.conf /etc/nginx/conf.d/pet-info.conf",
      "sudo nginx -t",
      "sudo systemctl enable nginx",
      "sudo systemctl restart nginx",
      "sudo systemctl is-active --quiet nginx",
    ]

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }
}

data "aws_ami" "amazon_arm" {
  most_recent = true
  owners      = ["amazon"]
  filter {
    name   = "architecture"
    values = ["arm64"]
  }
  filter {
    name   = "name"
    values = ["al2023-ami-2023*"]
  }
}

resource "aws_eip" "this" {
  domain = "vpc"
}

resource "aws_eip_association" "ip_ec2" {
  instance_id   = aws_instance.app_instance.id
  allocation_id = aws_eip.this.id
}

resource "aws_security_group" "web_app_sg" {
  name        = "${var.ec2_name}-security-group"
  description = "Allow HTTPS traffic to the ec2 instance"
  vpc_id      = data.aws_vpc.default.id

  ingress {
    description = "ssh access"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "HTTP traffic for LetsEncrypt ACME challenges"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "HTTPS traffic"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "tls_private_key" "web_key" {
  algorithm = "RSA"
  rsa_bits  = 4096
}


resource "local_file" "private_key" {
  content         = tls_private_key.web_key.private_key_pem
  filename        = "./pet-info.pem"
  file_permission = "0400"
}

resource "aws_key_pair" "web_app_key" {
  key_name   = "${var.ec2_name}-ssh-key"
  public_key = tls_private_key.web_key.public_key_openssh
}

resource "aws_ebs_volume" "db" {
  availability_zone = var.availability_zone
  size              = 5 #Gib
  type              = "gp3"
  encrypted         = true

  lifecycle {
    prevent_destroy = true
  }

  tags = {
    Name = "pet-info-data"
  }
}

resource "aws_volume_attachment" "ebs_att" {
  device_name = "/dev/xvdf"
  volume_id   = aws_ebs_volume.db.id
  instance_id = aws_instance.app_instance.id

  force_detach = true
}

data "aws_vpc" "default" {
  default = true
}

data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_subnet" "selected" {
  availability_zone = data.aws_availability_zones.available.names[0]
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default.id]
  }
}

resource "aws_route53_record" "dns_record" {
  zone_id = var.domain_zone_id
  name    = ""
  type    = "A"
  ttl     = "300"
  records = [aws_eip.this.public_ip]
}
