variable "parameters" {
  description = "Map of SSM parameters to create"
  type = map(object({
    value       = string
    type        = string
    description = string
  }))

  validation {
    condition = alltrue([
      for param in var.parameters : contains(["String", "StringList", "SecureString"], param.type)
    ])
    error_message = "Parameter type must be one of: String, StringList, SecureString."
  }
}

variable "kms_key_id" {
  description = "KMS key ID for encrypting SecureString parameters"
  type        = string
  default     = null
}

variable "common_tags" {
  description = "Common tags to apply to all parameters"
  type        = map(string)
  default     = {}
}
