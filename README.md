# Pet Info Web Application

A pet information management system built with Rust, featuring QR code generation, payment processing via MercadoPago, WhatsApp notifications, Apple passes wallet, and PDF reports. The application provides pet owners with a centralized platform to manage pet profiles, health records, and reminders.

**Demo**: https://pet-info.link

## 🏗️ Current Infrastructure

### Architecture Overview
```
┌─────────────────┐    ┌──────────────────┐     ┌─────────────────────────────────┐
│   pet-info.link │────│   Route 53 DNS   │──── │      Single EC2 Instance        │
│   (Domain)      │    │                  │     │                                 │
└─────────────────┘    └──────────────────┘     │  ┌──────────────────────────┐   │
                                                │  │   Nginx Reverse Proxy    │   │
                                 HTTPS :443 ────┼─▶│   SSL/TLS Termination    │   │
                                                │  └────────────┬─────────────┘   │
                                                │               │                 │
                                                │  ┌────────────▼─────────────┐   │
                                  HTTP :8080 ───┼──│   Rust Application       │   │
                                                │  │   (localhost:8080)       │   │
                                                │  └──────────────────────────┘   │
                                                └─────────────────────────────────┘
                                                          │
                       ┌──────────────────────────────────┼─────────────────────────────────┐
                       │                                  │                                 │
               ┌───────▼────────┐               ┌─────────▼────────┐              ┌─────────▼────────┐
               │   SQLite DB    │               │      S3 Bucket   │              │  SSM Parameters  │
               │  (File-based)  │               │  (File Storage)  │              │ (Configuration)  │
               └────────────────┘               └──────────────────┘              └──────────────────┘
```

### Current Deployment Stack
- **Single EC2 Instance** (t3.small)
- **Nginx Reverse Proxy** - SSL/TLS termination on port 443, forwards to application on port 8080
- **Rust Application** - Runs on localhost:8080 (HTTP only)
- **SQLite Database** with SQLCipher encryption
- **S3 Bucket** for file storage (pet photos, documents)
- **Route 53** for DNS management
- **Let's Encrypt SSL/TLS** certificates with certbot-dns-route53
- **AWS SSM** Parameter Store for sensitive env values
- **AWS Step Functions** & **Lambda Functions** for scheduled notifications via WhatsApp

## 🚀 Application Features

### Core Functionality
- **Pet Profile Management** - Create, edit, delete, and manage complete pet information including name, birthday, breed, gender, spaying/neutering status, and a pic photo
- **QR Code Generation** - Unique QR codes for each pet linking to public profiles with PNG output using tiny-skia
- **Payment Processing** - Single payment via MercadoPago
- **WhatsApp Integration** - Two-way webhook integration for automated reminders, and interactive messages
- **Apple Wallet Passes** - Generate digital pet ID cards (.pkpass files) for iOS devices
- **Google OAuth 2.0** - Secure user authentication and session management
- **PDF Reports** - Generate comprehensive pet reports using Typst template engine
- **File Upload & Storage** - Pet pic photo stored in AWS S3 integration
- **Health Records** - Track vaccinations, deworming, and weight
- **Pet Notes** - Rich text notes with Quill.js editor for detailed pet information
- **Reminders System** - Create and manage pet care reminders with WhatsApp messages
- **Owner Contacts** - Manage multiple owner contact information for lost pet scenarios
- **Public Pet Profiles** - Shareable public profiles accessible via QR codes or direct links

### Security Features
- **CSRF Protection** - Argon2-based token generation
- **Database Encryption** - SQLCipher for data at rest
- **SSL/TLS** - End-to-end encryption
- **OAuth 2.0** - Secure authentication
- **TOTP** - Two-factor authentication support
- **Secrets Management** - AWS SSM Parameter Store integration
- **MTLS** - For webhooks

## 🛠️ Technical Stack

### Backend (Rust 2024 Edition)
- **Web Framework**: Ntex 2.17.0 (high-performance async web framework with tokio runtime)
- **Database**: SQLite with SQLCipher (bundled-sqlcipher) via SQLx 0.8.6
- **Authentication**: OAuth2 5.0.0 (Google), ntex-identity for session management
- **Templating**: Tera 1.20.1 with date-locale support
- **QR Codes**: qrcode 0.12 + tiny-skia 0.11.4 for rendering
- **PDF Generation**: Typst 0.13.1 with typst-pdf and typst-assets
- **Apple Wallet**: passes 1.0.1 for .pkpass generation
- **Image Processing**: image 0.25.5 with cropper support
- **Security**:
  - csrf 0.5.0 with AES-GCM encryption
  - argon2 0.5.3 for key derivation
  - openssl 0.10 (vendored) for TLS
  - mtls for webhook endpoints
- **Observability**:
  - logfire 0.5.0 for metrics and tracing
  - tracing 0.1.41 + opentelemetry 0.29.1
- **AWS SDK**:
  - aws-sdk-s3 1.115.0 for file storage
  - aws-sdk-ssm 1.100.0 for configuration
  - aws-sdk-sfn 1.95.0 for Step Functions
- **Testing**: mockall 0.13.1

### Frontend
- **Templates**: Tera HTML templates with HTMX
- **JavaScript**:
  - HTMX for dynamic interactions
  - Quill.js for rich text editing
  - Cropper.js for image cropping
- **CSS**: Pico CSS with custom theme switcher

### Infrastructure (Terraform)
- **Cloud Provider**: AWS (us-east-2)
- **IaC**: Terraform with modular architecture
- **Modules**: EC2, S3, Lambda, Step Functions, SSM, IAM roles, Route 53
- **Configuration**: SSM Parameter Store with KMS encryption
- **Storage**: S3 for files, SQLite for structured data
- **Compute**: Single EC2 instance (t3.small)
- **Networking**: Route 53 DNS with SSL/TLS
- **Serverless**: Lambda (Rust) + Step Functions for scheduled notifications

### Additional Components
- **Build System**: Docker-based builds for EC2 and Lambda deployments

## ⚙️ Configuration Management

### Dual Configuration System
The application supports two configuration sources:

#### 1. Environment Variables (Development)
```bash
export DB_HOST="sqlite:data/app.db"
export DB_PASS_ENCRYPT="encryption-key"
export CSRF_PASS="uuid-here"
export MERCADO_PAGO_PUBLIC_KEY="public-key"
# ... other variables
```

#### 2. AWS SSM Parameter Store (Production)
```bash
# Compile with SSM feature
cargo build --release --features ssm

# Parameters stored as:
/pet-info/DB_HOST
/pet-info/DB_PASS_ENCRYPT (SecureString)
/pet-info/CSRF_PASS (SecureString)
/pet-info/MERCADO_TOKEN (SecureString)
/pet-info/WHATSAPP_BUSINESS_AUTH (SecureString)
/pet-info/WHATSAPP_BUSINESS_PHONE_NUMBER_ID
/pet-info/WHATSAPP_VERIFY_TOKEN (SecureString)
/pet-info/AWS_SFN_ARN_WB_NOTIFICATIONS
/pet-info/GOOGLE_OAUTH_CLIENT_ID
/pet-info/GOOGLE_OAUTH_CLIENT_SECRET (SecureString)
```

#### Critical Issues
1. **Single Point of Failure** - One EC2 instance
2. **SQLite Limitations** - Not suitable for concurrent workloads
3. **No Load Balancing** - Traffic bottlenecks
4. **Limited Monitoring** - Basic logging only
5. **No Auto-scaling** - Fixed capacity

#### Scalability Concerns
- **Database**: SQLite → Should migrate to RDS PostgreSQL
- **Compute**: Single instance → Need Auto Scaling Groups
- **Storage**: Local files → Should use S3 + CloudFront CDN
- **Monitoring**: Basic → alerting

### ✅ What's Ready for Production

## 🎯 Performance Characteristics

### Current Capacity (Estimated)
- **Concurrent Users**: 5-10 users
- **Daily Active Users**: 50-100 users
- **Request Throughput**: ~100 req/min
- **Database Size**: <1GB (SQLite limit)

## 🚀 Deployment

### Project Structure
```
pet-info/
├── web_app/          # Main Rust web application
├── terraform/        # Infrastructure as Code
│   ├── modules/      # Reusable Terraform modules
│   └── lambda_package/send-reminders/  # Lambda function source
├── scripts/          # Database migration CLI tool
├── migrations/       # SQL migration files
├── docker/           # Docker build configurations
│   ├── ec2.Dockerfile
│   └── lambda_build.Dockerfile
└── Makefile          # Build automation
```

### Development

#### Local Development
```bash
# Set up environment variables
export DB_HOST="sqlite:data/app.db"
export DB_PASS_ENCRYPT="your-encryption-key"
export CSRF_PASS="uuid-here"
export CSRF_SALT="uuid-here"
# ... other required env vars

# Run the application
cd web_app
cargo run

# Run database migrations
cd scripts
cargo run -- run-migrations -f "../migrations/create_tables.sql"
```

#### Testing
```bash
cd web_app
cargo test
```

### Production Deployment

#### 1. Build Application with Docker
```bash
# Build web application for EC2
make build_web_app_ec2
# Output: web_app/out/pet-info

# Build migration scripts
make build_scripts_app_ec2
# Output: scripts/out/scripts

# Build Lambda function
make build_send_reminders
# Output: terraform/lambda_package/send-reminders/out/bootstrap.zip
```

#### 2. Deploy Infrastructure
```bash
# Format Terraform files
make tf_format

# Deploy all infrastructure (requires prod.tfvars)
make deploy_prod_infra
```

The infrastructure deployment includes:
- EC2 instance with IAM role and instance profile
- S3 bucket for file storage
- Lambda function for WhatsApp reminders
- Step Functions state machine for scheduled notifications
- SSM parameters for configuration
- Route 53 DNS configuration
- Security groups and networking


---

**Status**: Production (Alpha)
**Version**: 0.1.0
**Last Updated**: November 2025
**Production Readiness**: 75% (infrastructure needs scaling improvements, code is solid)
