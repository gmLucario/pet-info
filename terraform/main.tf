data "aws_caller_identity" "current" {}
data "aws_region" "current" {}

data "aws_availability_zones" "available" {
  state = "available"
}

locals {
  # Default non-sensitive SSM parameters that are always needed
  default_ssm_parameters = {
    ENV = {
      value       = "prod"
      type        = "String"
      description = "Environment name (prod, dev, staging)"
    }
    CLOUDFRONT_URL = {
      value       = "https://${module.cloudfront.cloudfront_domain_name}"
      type        = "String"
      description = "CloudFront URL for serving images"
    }
  }

  # Merge default parameters with user-provided sensitive parameters
  all_ssm_parameters = merge(local.default_ssm_parameters, var.sensitive_instance_envs)
}

module "ssm_items" {
  source = "./modules/ssm"

  parameters = local.all_ssm_parameters
  kms_key_id = "alias/aws/ssm"

  common_tags = {
    Project   = "pet-info"
    ManagedBy = "terraform"
  }
}

module "pet_info_bucket" {
  source = "./modules/s3"

  app_name    = "pet-info"
  bucket_name = "app-storage"
  s3_folders = [
    {
      alias  = "pics"
      folder = "pics/"
      acl    = "public-read"
    },
  ]
}

module "cloudfront" {
  source = "./modules/cloudfront"

  project_name                = "pet-info"
  bucket_regional_domain_name = module.pet_info_bucket.info.bucket_regional_domain_name
  origin_id                   = module.pet_info_bucket.info.name

  tags = {
    Project = "pet-info"
  }
}

resource "aws_s3_bucket_policy" "allow_access_from_cloudfront" {
  bucket = module.pet_info_bucket.info.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowCloudFrontServicePrincipal"
        Effect = "Allow"
        Principal = {
          Service = "cloudfront.amazonaws.com"
        }
        Action   = "s3:GetObject"
        Resource = "${module.pet_info_bucket.info.arn}/pics/*"
        Condition = {
          StringEquals = {
            "AWS:SourceArn" = module.cloudfront.cloudfront_arn
          }
        }
      }
    ]
  })
}

locals {
  # Construct Step Function ARN for Lambda environment (avoids circular dependency)
  step_function_arn = "arn:aws:states:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:stateMachine:reminder_workflow"
}

module "lambda_send_reminders" {
  source = "./modules/lambda"

  env = {
    WHATSAPP_BUSINESS_PHONE_NUMBER_ID = var.sensitive_instance_envs["WHATSAPP_BUSINESS_PHONE_NUMBER_ID"].value
    WHATSAPP_BUSINESS_AUTH            = var.sensitive_instance_envs["WHATSAPP_BUSINESS_AUTH"].value
    STEP_FUNCTION_ARN                 = local.step_function_arn
    WEB_APP_API_URL                   = var.sensitive_instance_envs["WEB_APP_API_URL"].value
    INTERNAL_API_SECRET               = var.sensitive_instance_envs["INTERNAL_API_SECRET"].value
  }
  lambda_details = {
    name         = "send_reminders"
    desc         = "send a whats reminder to a user"
    code_package = "lambda_package/send-reminders/out/bootstrap.zip"
  }

  # Permission for Lambda to start new Step Function executions (for recurring reminders)
  additional_policy = {
    actions   = ["states:StartExecution"]
    resources = [local.step_function_arn]
  }
}

module "send_reminders_step_function" {
  source = "./modules/step_function"

  step_function_name = "reminder_workflow"

  policy_document = {
    Version = "2012-10-17"
    "Statement" : [
      {
        "Action" : "lambda:InvokeFunction",
        "Effect" : "Allow",
        "Resource" : module.lambda_send_reminders.info.arn
      },
      {
        "Effect" : "Allow",
        "Action" : [
          "logs:CreateLogDelivery",
          "logs:GetLogDelivery",
          "logs:UpdateLogDelivery",
          "logs:DeleteLogDelivery",
          "logs:ListLogDelivery",
          "logs:PutResourcePolicy",
          "logs:DescribeResourcePolicies",
          "logs:DescribeLogGroups",
        ],
        "Resource" : "arn:aws:logs:*:${data.aws_caller_identity.current.account_id}:*"
      }
    ]
  }

  step_function_definition = {
    StartAt = "WaitState"
    States = {
      "WaitState" = {
        Type          = "Wait"
        TimestampPath = "$.when"
        "Next" : "InvokeLambda"
      },
      "InvokeLambda" = {
        Type     = "Task"
        End      = true
        Resource = "arn:aws:states:::lambda:invoke"
        Parameters = {
          FunctionName = module.lambda_send_reminders.info.arn,
          # Pass full payload so Lambda can access repeat_config and reminder_id
          "Payload.$" = "$"
        }
      }
    }
  }
}

module "pet_info_role" {
  source = "./modules/role"

  role_name             = "pet-info-ec2-app-role"
  policy_name           = "pet-info-ec2-app-policy"
  instance_profile_name = "pet-info-ec2-app-profile"

  policy_document = {
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "StepFunctionsStateMachineAccess"
        Effect = "Allow"
        Action = [
          "states:StartExecution",
          "states:ListExecutions",
          "states:DescribeStateMachine"
        ]
        Resource = [
          module.send_reminders_step_function.info.arn
        ]
      },
      {
        Sid    = "StepFunctionsExecutionAccess"
        Effect = "Allow"
        Action = [
          "states:StopExecution",
          "states:DescribeExecution",
        ]
        Resource = [
          "arn:aws:states:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:execution:${module.send_reminders_step_function.info.name}:*",
        ]
      },
      {
        Sid    = "S3BucketAccess"
        Effect = "Allow"
        Action = [
          "s3:ListBucket",
          "s3:GetBucketLocation"
        ]
        Resource = [
          module.pet_info_bucket.info.arn
        ]
      },
      {
        Sid    = "S3ObjectAccess"
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:PutObjectAcl"
        ]
        Resource = [
          "${module.pet_info_bucket.info.arn}/*"
        ]
      },
      {
        Sid    = "SSMParameterAccess"
        Effect = "Allow"
        Action = [
          "ssm:GetParameter",
          "ssm:GetParameters",
          "ssm:GetParametersByPath"
        ]
        Resource = [
          "arn:aws:ssm:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:parameter/pet-info/*"
        ]
      },
      {
        Sid    = "KMSDecryptAccess"
        Effect = "Allow"
        Action = [
          "kms:Decrypt"
        ]
        Resource = "*"
        Condition = {
          StringEquals = {
            "kms:ViaService" = "ssm.${data.aws_region.current.name}.amazonaws.com"
          }
        }
      }
    ]
  }

  tags = {
    Project = "pet-info"
  }
}

module "pet_info_domain" {
  source = "./modules/domain"

  domain_name          = "pet-info.link"
  domain_owner_contact = var.domain_owner_contact
}

module "pet_info_ec2_instance" {
  source = "./modules/ec2"

  user_data_path        = var.ec2_user_data_path
  ec2_name              = "pet-info-ec2"
  instance_profile_name = module.pet_info_role.instance_profile_name
  availability_zone     = data.aws_availability_zones.available.names[0]
  ssh_key_path          = var.ssh_key_path

  domain_zone_id = module.pet_info_domain.info.domain_zone_id
  cert_details = {
    server_path = var.cert_server_path
    key_path    = var.cert_key_path
  }

  instance_envs           = var.instance_envs
  sensitive_instance_envs = var.sensitive_instance_envs

  web_app_executable_path = var.web_app_executable_path

  git_branch = var.git_branch
}
