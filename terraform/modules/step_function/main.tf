resource "aws_iam_role" "step_function_role" {
  name = "${var.step_function_name}-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    "Statement" : [{
      "Action" : "sts:AssumeRole",
      "Principal" : {
        "Service" : "states.amazonaws.com"
      },
      "Effect" : "Allow",
      "Sid" : ""
    }]
  })
}

resource "aws_iam_role_policy" "step_function_policy" {
  name = "${var.step_function_name}-policy"
  role = aws_iam_role.step_function_role.id

  policy = jsonencode(var.policy_document)
}

resource "aws_sfn_state_machine" "workflow" {
  name     = var.step_function_name
  role_arn = aws_iam_role.step_function_role.arn

  definition = jsonencode(var.step_function_definition)
}
