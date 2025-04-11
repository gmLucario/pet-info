variable "step_function_name" {
  type        = string
  description = "step function name"
}

variable "aws_lambda_arn" {
  type        = string
  description = "lambda arn step function will trigger"
  sensitive   = true
}

variable "account_id" {
  type        = string
  sensitive   = true
  description = "AWS Account ID number"
}
