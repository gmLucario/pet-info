output "cloudfront_domain_name" {
  value = aws_cloudfront_distribution.s3_distribution.domain_name
}

output "cloudfront_arn" {
  value = aws_cloudfront_distribution.s3_distribution.arn
}
