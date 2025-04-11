resource "aws_route53domains_domain" "domain" {
  domain_name = var.domain_name
  auto_renew  = true


  admin_contact {
    address_line_1    = var.domain_owner_contact.address_line_1
    city              = var.domain_owner_contact.city
    contact_type      = var.domain_owner_contact.contact_type
    country_code      = var.domain_owner_contact.country_code
    email             = var.domain_owner_contact.email
    first_name        = var.domain_owner_contact.first_name
    last_name         = var.domain_owner_contact.last_name
    organization_name = var.domain_owner_contact.organization_name
    phone_number      = var.domain_owner_contact.phone_number
    state             = var.domain_owner_contact.state
    zip_code          = var.domain_owner_contact.zip_code
  }

  registrant_contact {
    address_line_1    = var.domain_owner_contact.address_line_1
    city              = var.domain_owner_contact.city
    contact_type      = var.domain_owner_contact.contact_type
    country_code      = var.domain_owner_contact.country_code
    email             = var.domain_owner_contact.email
    first_name        = var.domain_owner_contact.first_name
    last_name         = var.domain_owner_contact.last_name
    organization_name = var.domain_owner_contact.organization_name
    phone_number      = var.domain_owner_contact.phone_number
    state             = var.domain_owner_contact.state
    zip_code          = var.domain_owner_contact.zip_code
  }

  tech_contact {
    address_line_1    = var.domain_owner_contact.address_line_1
    city              = var.domain_owner_contact.city
    contact_type      = var.domain_owner_contact.contact_type
    country_code      = var.domain_owner_contact.country_code
    email             = var.domain_owner_contact.email
    first_name        = var.domain_owner_contact.first_name
    last_name         = var.domain_owner_contact.last_name
    organization_name = var.domain_owner_contact.organization_name
    phone_number      = var.domain_owner_contact.phone_number
    state             = var.domain_owner_contact.state
    zip_code          = var.domain_owner_contact.zip_code
  }
}

resource "aws_route53_zone" "zone" {
  name = var.domain_name
}

