variable "lambda_details" {
  type = object({
    name         = string
    code_package = string
    desc         = string
  })
}

variable "env" {
  type      = map(string)
  default   = {}
  sensitive = true
}

variable "additional_policy" {
  description = "Optional additional IAM policy for the Lambda function"
  type = object({
    actions   = list(string)
    resources = list(string)
  })
  default = null
}
