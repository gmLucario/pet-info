variable "whatsapp_business_phone_number_id" {
  type        = string
  description = "whatsapp business phone number id"
  sensitive   = true
}

variable "whatsapp_business_auth" {
  type        = string
  description = "whatsapp business auth"
  sensitive   = true
}

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

variable "ec2_user_data_path" {
  type = string
}

variable "cert_server_path" {
  type = string
}

variable "ssh_key_path" {
  type = string
}

variable "cert_key_path" {
  type = string
}

variable "instance_envs" {
  type      = map(string)
  sensitive = true
}
