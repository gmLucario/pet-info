resource "aws_s3_bucket" "bucket" {
  bucket = "${var.app_name}-${var.bucket_name}"
}


resource "aws_s3_bucket_public_access_block" "access_block" {
  bucket = aws_s3_bucket.bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_ownership_controls" "bucket_controls" {
  bucket = aws_s3_bucket.bucket.id
  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_acl" "bucket_acl" {
  depends_on = [aws_s3_bucket_ownership_controls.bucket_controls]

  bucket = aws_s3_bucket.bucket.id
  acl    = var.bucket_acl
}

resource "aws_s3_object" "object" {
  for_each = {
    for index, f in var.s3_folders :
    f.alias => { "folder" = f.folder, "acl" = f.acl }
  }

  bucket = aws_s3_bucket.bucket.id
  key    = each.value.folder
  acl    = each.value.acl
}
