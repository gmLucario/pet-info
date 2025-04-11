output "info" {
  value = {
    domain_zone_id = aws_route53_zone.zone.zone_id
  }
}
