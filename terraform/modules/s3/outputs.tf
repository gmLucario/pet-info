output "info" {
  value = {
    name = aws_s3_bucket.bucket.id
    arn  = aws_s3_bucket.bucket.arn
  }
}
