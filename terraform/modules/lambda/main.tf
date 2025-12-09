resource "aws_iam_role" "lambda_role" {
  name = "${var.lambda_details.name}-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    "Statement" : [{
      "Action" : "sts:AssumeRole",
      "Principal" : {
        "Service" : "lambda.amazonaws.com"
      },
      "Effect" : "Allow",
      "Sid" : ""
    }]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_basic_execution" {
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
  role       = aws_iam_role.lambda_role.name
}

resource "aws_iam_role_policy" "lambda_additional_policy" {
  count = var.additional_policy != null ? 1 : 0

  name = "${var.lambda_details.name}-additional-policy"
  role = aws_iam_role.lambda_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = var.additional_policy.actions
      Resource = var.additional_policy.resources
    }]
  })
}

resource "aws_lambda_function" "lambda" {

  function_name    = var.lambda_details.name
  role             = aws_iam_role.lambda_role.arn
  filename         = var.lambda_details.code_package
  source_code_hash = filebase64sha256(var.lambda_details.code_package)
  description      = var.lambda_details.desc
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  memory_size      = 128
  timeout          = 30
  architectures    = ["arm64"]

  environment {
    variables = var.env
  }
}
