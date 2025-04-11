variable "domain_name" {
  type        = string
  description = "domain to register"
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

