variable "step_function_name" {
  type        = string
  description = "step function name"
}

variable "policy_document" {
  type        = any
  description = "IAM policy document object (will be JSON encoded)"
}

variable "step_function_definition" {
  type        = any
  description = "document defining the step function steps (will be JSON encoded)"
}
