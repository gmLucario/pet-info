variable "app_name" {
  type        = string
  description = "app name owned this bucket"
}

variable "bucket_name" {
  type = string
}

variable "bucket_acl" {
  type    = string
  default = "private"
}

variable "s3_folders" {
  type = list(object({
    alias  = string
    folder = string
    acl    = string
  }))
  default     = []
  description = "empty folders to add to the new bucket"
}

