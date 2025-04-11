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

  policy = jsonencode({
    Version = "2012-10-17"
    "Statement" : [
      {
        "Action" : "lambda:InvokeFunction",
        "Effect" : "Allow",
        "Resource" : var.aws_lambda_arn
      },
      {
        "Effect" : "Allow",
        "Action" : [
          "logs:CreateLogDelivery",
          "logs:GetLogDelivery",
          "logs:UpdateLogDelivery",
          "logs:DeleteLogDelivery",
          "logs:ListLogDelivery",
          "logs:PutResourcePolicy",
          "logs:DescribeResourcePolicies",
          "logs:DescribeLogGroups",
        ],
        "Resource" : "arn:aws:logs:*:${var.account_id}:*"
      }
    ]
  })
}

resource "aws_sfn_state_machine" "workflow" {
  name     = var.step_function_name
  role_arn = aws_iam_role.step_function_role.arn

  definition = jsonencode({
    StartAt = "WaitState"
    States = {
      "WaitState" = {
        Type          = "Wait"
        TimestampPath = "$.when"
        "Next" : "InvokeLambda"
      },
      "InvokeLambda" = {
        Type     = "Task"
        End      = true
        Resource = "arn:aws:states:::lambda:invoke"
        Parameters = {
          FunctionName = var.aws_lambda_arn,
          "Payload.$"  = "$.reminder"
        }
      }
    }
  })
}
