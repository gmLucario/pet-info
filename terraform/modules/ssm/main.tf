resource "aws_ssm_parameter" "parameters" {
  for_each = nonsensitive(var.parameters)

  name        = "/pet-info/${each.key}"
  type        = each.value.type
  value_wo         = each.value.value
  value_wo_version = 1
  description = each.value.description

  # Use KMS key for SecureString parameters if provided
  key_id = each.value.type == "SecureString" && var.kms_key_id != null ? var.kms_key_id : null

  tags = merge(
    var.common_tags,
    {
      Name    = "/pet-info/${each.key}"
      Project = "pet-info"
    }
  )
}
