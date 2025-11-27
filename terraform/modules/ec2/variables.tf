variable "ec2_name" {
  type        = string
  description = "name ec2 instance"
}

variable "instance_profile_name" {
  type        = string
  description = "IAM instance profile name for EC2 instance"
}

variable "availability_zone" {
  type        = string
  description = "availability zone name"
  sensitive   = true
}

variable "ssh_key_path" {
  type        = string
  description = "ssh key to access the instance"
  sensitive   = true
}

variable "domain_zone_id" {
  type        = string
  description = "zone id of the domain"
}

variable "web_app_executable_path" {
  type = string
}

variable "user_data_path" {
  type        = string
  description = "script to run when instance is initiated"
}

variable "cert_details" {
  type = object({
    server_path = string
    key_path    = string
  })
  description = "SSL certificate paths for Nginx HTTPS"
}

variable "instance_envs" {
  type      = map(string)
  sensitive = true
}

variable "git_branch" {
  type        = string
  description = "Git branch to clone from repository"
}

variable "sensitive_instance_envs" {
  type = map(object({
    value       = string
    type        = string
    description = string
  }))
  sensitive = true
}
