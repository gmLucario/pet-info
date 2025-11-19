output "info" {
  value = {
    arn  = aws_sfn_state_machine.workflow.arn
    name = var.step_function_name
  }
}
