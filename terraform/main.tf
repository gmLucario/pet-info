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
  aws_lambda_arn     = module.lambda_send_reminders.info.arn
  account_id         = data.aws_caller_identity.current.account_id
}

resource "aws_iam_policy" "ec2_rust_app_policy" {
  name        = "pet-info-ec2-app-policy"
  description = "Policy for ec2 instance to host pet-info web app"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow",
        Action = [
          "states:StartExecution",
          "states:StopExecution",
          "states:DescribeExecution",
          "states:ListExecutions",
        ],
        Resource = [
          module.send_reminders_step_function.info.arn,
        ]
      },
      {
        Effect = "Allow",
        Action = [
          "s3:PutObject",
          "s3:GetObject",
          "s3:ListBucket",
        ],
        Resource = [
          module.pet_info_bucket.info.arn,
          "${module.pet_info_bucket.info.arn}/*",
        ]
      }
    ]
  })
}

module "pet_info_domain" {
  source = "./modules/domain"

  domain_name          = "pet-info.link"
  domain_owner_contact = var.domain_owner_contact
}

module "pet_info_ec2_instance" {
  source = "./modules/ec2"

  user_data_path    = var.ec2_user_data_path
  ec2_name          = "pet-info-ec2"
  ec2_policy_arn    = resource.aws_iam_policy.ec2_rust_app_policy.arn
  availability_zone = data.aws_availability_zones.available.names[0]
  ssh_key_path      = var.ssh_key_path

  domain_zone_id = module.pet_info_domain.info.domain_zone_id
  cert_details = {
    server_path = var.cert_server_path
    key_path    = var.cert_key_path
  }
  instance_envs = var.instance_envs
}
