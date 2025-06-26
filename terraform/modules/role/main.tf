resource "aws_iam_role" "app_role" {
  name        = var.role_name
  description = var.role_description

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })

  tags = merge(var.tags, {
    Name    = var.role_name
    Purpose = "EC2 web app hosting"
  })
}

resource "aws_iam_policy" "app_policy" {
  name        = var.policy_name
  description = var.policy_description

  policy = jsonencode(var.policy_document)

  tags = merge(var.tags, {
    Name    = var.policy_name
    Purpose = "aws resources access for an app"
  })
}

resource "aws_iam_role_policy_attachment" "app_policy_attachment" {
  role       = aws_iam_role.app_role.name
  policy_arn = aws_iam_policy.app_policy.arn
}

resource "aws_iam_instance_profile" "app_profile" {
  name = var.instance_profile_name
  role = aws_iam_role.app_role.name

  tags = merge(var.tags, {
    Name    = var.instance_profile_name
    Purpose = "EC2 instance profile for an app"
  })
}
