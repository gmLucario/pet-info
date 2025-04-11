output "info" {
  value = {
    arn = aws_sfn_state_machine.workflow.arn
  }
}
