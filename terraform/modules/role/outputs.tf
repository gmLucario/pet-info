output "role_arn" {
  description = "ARN of the IAM role"
  value       = aws_iam_role.ec2_app_role.arn
}

output "role_name" {
  description = "Name of the IAM role"
  value       = aws_iam_role.ec2_app_role.name
}

output "policy_arn" {
  description = "ARN of the IAM policy"
  value       = aws_iam_policy.ec2_app_policy.arn
}

output "policy_name" {
  description = "Name of the IAM policy"
  value       = aws_iam_policy.ec2_app_policy.name
}

output "instance_profile_name" {
  description = "Name of the IAM instance profile"
  value       = aws_iam_instance_profile.ec2_app_profile.name
}

output "instance_profile_arn" {
  description = "ARN of the IAM instance profile"
  value       = aws_iam_instance_profile.ec2_app_profile.arn
}