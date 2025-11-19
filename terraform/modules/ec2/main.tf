data "aws_region" "this" {}

data "local_file" "cert" {
  filename = var.cert_details.server_path
}

data "local_file" "key" {
  filename = var.cert_details.key_path
}


resource "aws_instance" "app_instance" {
  ami                  = data.aws_ami.amazon_arm.id
  instance_type        = "t4g.small"
  key_name             = aws_key_pair.web_app_key.key_name
  iam_instance_profile = var.instance_profile_name
  availability_zone    = aws_ebs_volume.db.availability_zone

  vpc_security_group_ids      = [aws_security_group.web_app_sg.id]
  associate_public_ip_address = false
  subnet_id                   = data.aws_subnet.selected.id

  user_data = templatefile(
    var.user_data_path,
    {
      certificate        = data.local_file.cert.content
      private_key_pem    = data.local_file.key.content
      instance_envs      = var.instance_envs
      volume_device_name = "/dev/xvdf"
      git_branch         = var.git_branch
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
      "while [ ! -f /tmp/user-data-complete ]; do sleep 10; done",
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
    source      = "${path.module}/../../web_app/out/pet-info"
    destination = "/tmp/pet-info"

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }

  # Move binary to final location and set up service
  provisioner "remote-exec" {
    inline = [
      "mkdir -p /home/ec2-user/pet-info/web_app",
      "mv /tmp/pet-info /home/ec2-user/pet-info/web_app/pet-info",
      "chmod +x /home/ec2-user/pet-info/web_app/pet-info",
      "sudo setcap CAP_NET_BIND_SERVICE=+ep /home/ec2-user/pet-info/web_app/pet-info",
      "sudo systemctl daemon-reload",
      "sudo systemctl enable pet-info.service",
      "sudo systemctl restart pet-info.service"
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

# Upload Apple Wallet Pass certificate files if provided
resource "null_resource" "upload_pass_certificates" {
  count = var.pass_cert_path != "" && var.pass_key_path != "" ? 1 : 0

  depends_on = [null_resource.wait_for_instance]

  # Upload pass certificate
  provisioner "file" {
    source      = var.pass_cert_path
    destination = "/tmp/pass_certificate.pem"

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }

  # Upload pass private key
  provisioner "file" {
    source      = var.pass_key_path
    destination = "/tmp/pass_private_key.pem"

    connection {
      type        = "ssh"
      user        = "ec2-user"
      private_key = tls_private_key.web_key.private_key_pem
      host        = aws_eip.this.public_ip
      timeout     = "5m"
    }
  }

  # Move pass files to final location
  provisioner "remote-exec" {
    inline = [
      "sudo mv /tmp/pass_certificate.pem /opt/pet-info/pass_certificate.pem",
      "sudo mv /tmp/pass_private_key.pem /opt/pet-info/pass_private_key.pem",
      "sudo chown ec2-user:ec2-user /opt/pet-info/pass_certificate.pem /opt/pet-info/pass_private_key.pem",
      "sudo chmod 644 /opt/pet-info/pass_certificate.pem",
      "sudo chmod 600 /opt/pet-info/pass_private_key.pem",
      "sudo systemctl restart pet-info.service"
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
    description = "HTTPS traffic from api gateway"
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

