build_web_app_ec2:
	@echo "Building pet-info web application for EC2..."
	@docker build --build-arg SOURCE_NAME=pet-info -t build_ec2:latest -f docker/ec2.Dockerfile web_app --output web_app/out
	@echo "✓ Build complete: web_app/out/pet-info"

build_scripts_app_ec2:
	@docker build --build-arg SOURCE_NAME=scripts -t build_ec2:latest -f docker/ec2.Dockerfile scripts --output scripts/out

build_send_reminders:
	@docker build -t build_lambda:latest -f docker/lambda_build.Dockerfile terraform/lambda_package/send-reminders --output terraform/lambda_package/send-reminders/out

deploy_prod_infra: build_web_app_ec2
	@echo "Deploying infrastructure and application..."
	@terraform -chdir=terraform apply -var-file=prod.tfvars
	@echo "✓ Deployment complete!"
