variable "domain_owner_contact" {
  type = object({
    address_line_1    = string
    city              = string
    contact_type      = string
    country_code      = string
    email             = string
    first_name        = string
    last_name         = string
    organization_name = string
    phone_number      = string
    state             = string
    zip_code          = string
  })
  sensitive = true
}

variable "stack_tags" {
  type        = map(string)
  description = "tags all resources will have"
  default = {
    "app"      = "pet-info"
    "language" = "rust"
  }
}

variable "web_app_executable_path" {
  type = string
}

variable "ec2_user_data_path" {
  type = string
}

variable "cert_server_path" {
  type    = string
  default = "../web_app/server.crt"
}

variable "ssh_key_path" {
  type = string
}

variable "cert_key_path" {
  type    = string
  default = "../web_app/server.key"
}

variable "pass_cert_path" {
  type = string
}

variable "pass_key_path" {
  type = string
}

variable "sensitive_instance_envs" {
  type = map(object({
    value       = string
    type        = string
    description = string
  }))
  sensitive = true
}

variable "instance_envs" {
  type      = map(string)
  sensitive = true
}

