variable "role_name" {
  type        = string
  description = "Name of the IAM role"
}

variable "role_description" {
  type        = string
  description = "Description of the IAM role"
  default     = "IAM role for EC2 instance hosting pet-info web app"
}

variable "policy_name" {
  type        = string
  description = "Name of the IAM policy"
}

variable "policy_description" {
  type        = string
  description = "Description of the IAM policy"
  default     = "Policy for EC2 instance to access S3 and Step Functions"
}

variable "instance_profile_name" {
  type        = string
  description = "Name of the IAM instance profile"
}

variable "policy_document" {
  type        = any
  description = "IAM policy document object (will be JSON encoded)"
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to all resources"
  default     = {}
}

variable "step_function_arns" {
  type        = list(string)
  description = "List of Step Function ARNs that the role can access"
  default     = []
}

variable "s3_bucket_arns" {
  type        = list(string)
  description = "List of S3 bucket ARNs that the role can access"
  default     = []
}

variable "s3_object_arns" {
  type        = list(string)
  description = "List of S3 object ARNs that the role can access (typically bucket_arn/*)"
  default     = []
}