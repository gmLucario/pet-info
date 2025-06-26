data "aws_caller_identity" "current" {}
data "aws_region" "current" {}

data "aws_availability_zones" "available" {
  state = "available"
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

module "lambda_send_reminders" {
  source = "./modules/lambda"


  env = {
    WHATSAPP_BUSINESS_PHONE_NUMBER_ID = var.whatsapp_business_phone_number_id
    WHATSAPP_BUSINESS_AUTH            = var.whatsapp_business_auth
  }
  lambda_details = {
    name         = "send_reminders"
    desc         = "send a whats reminder to a user"
    code_package = "lambda_package/send-reminders/out/bootstrap.zip"
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
          "Payload.$"  = "$.reminder"
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
  instance_envs = var.instance_envs
}
