variable "ec2_name" {
  type        = string
  description = "name ec2 instance"
}

variable "ec2_policy_arn" {
  type        = string
  description = "policy ec2 to allow instance to perform action"
  sensitive   = true
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

variable "user_data_path" {
  type        = string
  description = "script to run when instance is initiated"
}

variable "cert_details" {
  type = object({
    server_path = string
    key_path    = string
  })
}

variable "instance_envs" {
  type = map(string)
  sensitive = true
}
