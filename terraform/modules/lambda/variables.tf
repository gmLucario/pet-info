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
