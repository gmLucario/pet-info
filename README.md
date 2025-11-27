# Pet Info Web Application

A comprehensive pet information management system built with Rust, featuring QR code generation, payment processing via MercadoPago, WhatsApp notifications, Apple Wallet integration, and PDF health reports. The application provides pet owners with a centralized platform to manage pet profiles, health records, reminders, and owner contact information.

**Demo**: https://pet-info.link
**Edition**: Rust 2024

## 🏗️ Current Infrastructure

### Architecture Overview
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   pet-info.link │────│   Route 53 DNS   │────│  Single EC2     │
│   (Domain)      │    │                  │    │  Instance       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
                       ┌──────────────────────────────────┼──────────────────────────────────┐
                       │                                  │                                  │
               ┌───────▼────────┐              ┌─────────▼────────┐              ┌─────────▼────────┐
               │   SQLite DB    │              │      S3 Bucket   │              │  SSM Parameters  │
               │  (File-based)  │              │  (File Storage)  │              │ (Configuration)  │
               └────────────────┘              └──────────────────┘              └──────────────────┘
```

### Current Deployment Stack
- **Single EC2 Instance** (t3.small)
- **SQLite Database** with SQLCipher encryption
- **S3 Bucket** for file storage (pet photos, documents)
- **Route 53** for DNS management
- **SSL/TLS** termination on EC2
- **AWS SSM** Parameter Store for sensitive env values
- **AWS Step Functions** for scheduled notifications
- **Lambda Functions** for WhatsApp message processing

## 🚀 Application Features

### Core Functionality
- **Pet Profile Management** - Create, edit, delete, and manage complete pet information including name, birthday, breed, gender, spaying/neutering status, and photos
- **QR Code Generation** - Unique QR codes for each pet linking to public profiles with PNG output using tiny-skia
- **Payment Processing** - MercadoPago integration with subscription management and payment balance tracking
- **WhatsApp Integration** - Two-way webhook integration for automated reminders, notifications, and interactive messages
- **Apple Wallet Passes** - Generate digital pet ID cards (.pkpass files) for iOS devices
- **Google OAuth 2.0** - Secure user authentication and session management
- **PDF Reports** - Generate comprehensive pet health reports using Typst template engine
- **File Upload & Storage** - Pet photos and document management with AWS S3 integration
- **Health Records** - Track vaccinations, deworming, vet visits, and custom health events
- **Pet Notes** - Rich text notes with Quill.js editor for detailed pet information
- **Reminders System** - Create and manage pet care reminders with WhatsApp notifications via AWS Step Functions
- **Owner Contacts** - Manage multiple owner contact information for lost pet scenarios
- **Public Pet Profiles** - Shareable public profiles accessible via QR codes or direct links

### Security Features
- **CSRF Protection** - Argon2-based token generation
- **Database Encryption** - SQLCipher for data at rest
- **SSL/TLS** - End-to-end encryption
- **OAuth 2.0** - Secure authentication
- **TOTP** - Two-factor authentication support
- **Secrets Management** - AWS SSM Parameter Store integration

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
- **Async Runtime**: tokio 1.48 with sync features
- **Serialization**: serde 1.0.228 + serde_json 1.0.145
- **HTTP Client**: reqwest 0.12.24 with JSON and multipart support
- **Observability**:
  - logfire 0.5.0 for metrics and tracing
  - tracing 0.1.41 + opentelemetry 0.29.1
- **AWS SDK**:
  - aws-sdk-s3 1.115.0 for file storage
  - aws-sdk-ssm 1.100.0 for configuration (optional feature)
  - aws-sdk-sfn 1.95.0 for Step Functions
- **Testing**: mockall 0.13.1 for mocking

### Frontend
- **Templates**: Tera HTML templates with HTMX
- **JavaScript**:
  - HTMX for dynamic interactions
  - Quill.js for rich text editing
  - Cropper.js for image cropping
- **CSS**: Pico CSS with custom theme switcher
- **Markdown**: pulldown-cmark 0.12.2 + ammonia 4.1.2 for sanitization

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
- **Lambda Functions**: Rust-based send-reminders function with WhatsApp integration
- **Database Migrations**: SQL-based migrations in migrations/ directory
- **Scripts**: CLI tool for running migrations (Clap 4.5.32)
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

## 🗄️ Database Schema

The application uses SQLite with SQLCipher encryption. The database includes the following tables:

### Core Tables
- **user_app** - User accounts with email, phone, subscription status, and account roles
- **pet** - Pet profiles with name, birthday, breed, photos, and characteristics
- **pet_external_id** - External IDs for public pet profiles (used in QR codes)
- **pet_linked** - Links pets to their external IDs

### Health & Care
- **pet_health** - Health records (vaccinations, deworming, vet visits)
- **pet_weight** - Weight tracking over time
- **pet_note** - Rich text notes about pets

### User Management
- **owner_contact** - Emergency contact information for pet owners
- **reminder** - Scheduled reminders with WhatsApp notification integration

### Payments
- **user_sub_payment** - MercadoPago payment records with idempotency
- **add_pet_balance** - User balance for adding additional pets

### Key Features
- **Encryption**: All data encrypted at rest with SQLCipher
- **Cascade Deletes**: Automatic cleanup of related records
- **Indexes**: Optimized queries on email, external_id, health_record, and execution_id
- **Timestamps**: UTC timestamps for all records
- **Constraints**: Unique constraints on emails, payments, and external IDs


## 🔔 Reminder System Architecture

The application uses AWS Step Functions and Lambda for scheduled WhatsApp reminders:

### Workflow
1. **User Creates Reminder** → `POST /reminder/create`
2. **Step Function Execution Starts** → State machine is triggered with:
   ```json
   {
     "when": "2025-12-01T14:00:00Z",
     "reminder": {
       "phone": "whatsapp-phone-number",
       "body": "Reminder message"
     }
   }
   ```
3. **Wait State** → Step Function waits until scheduled time
4. **Lambda Invocation** → `send-reminders` Lambda function is invoked
5. **WhatsApp Message** → Lambda sends message via WhatsApp Business API

### Lambda Function (send-reminders)
- **Runtime**: Rust (ARM64)
- **Trigger**: AWS Step Functions
- **Build**: `cargo lambda build --release --arm64 --output-format zip`
- **Payload**:
  ```json
  {
    "phone": "whatsapp-phone-number",
    "body": "Desparasitar galleta"
  }
  ```

### Step Function Definition
- **State Machine**: `reminder_workflow`
- **States**: WaitState → InvokeLambda
- **IAM Permissions**: Lambda invoke, CloudWatch logs
- **Execution Tracking**: Each reminder has unique `execution_id` stored in database

### Error Handling
- **TODO**: Implement SQS dead letter queue for failed reminders
- **Retry Logic**: Configured at Lambda level
- **Monitoring**: CloudWatch logs and metrics

## 🌐 API Routes & Endpoints

The application provides a comprehensive REST-like API organized into logical route groups:

### Public Routes
- `GET /info/{pet_external_id}` - View public pet profile (QR code destination)
- `GET /blog/{entry_id}` - Blog entries (privacy, terms, about, questions)
- `GET /` - Landing page
- `GET /static/*` - Static assets (CSS, JS, images)

### Pet Management (`/pet`)
- `GET /pet` - Pet dashboard
- `GET /pet/list` - List user's pets
- `GET /pet/details/{pet_id}` - Pet details form
- `POST /pet/create` - Create new pet
- `PUT /pet/edit/{pet_id}` - Update pet
- `DELETE /pet/delete/{pet_id}` - Delete pet
- `GET /pet/qr_code/{pet_external_id}` - Generate QR code (PNG)
- `GET /pet/pdf_report/{pet_id}` - Generate PDF health report
- `GET /pet/public_pic/{pet_external_id}` - Get pet photo
- `GET /pet/pass/{pet_external_id}` - Download Apple Wallet pass

### Health Records (`/pet/health`)
- `GET /pet/health/{pet_external_id}/{health_type}` - View health records
- `POST /pet/health/add` - Add health record
- `DELETE /pet/health/delete` - Delete health record

### Pet Notes (`/pet/note`)
- `GET /pet/note/{pet_id}` - Notes view
- `POST /pet/note/new` - Create note
- `GET /pet/note/list/{pet_id}` - List notes
- `DELETE /pet/note/delete` - Delete note

### Reminders (`/reminder`)
- `GET /reminder` - Reminders dashboard
- `GET /reminder/list` - List reminders
- `POST /reminder/create` - Create reminder (triggers Step Function)
- `DELETE /reminder/delete/{reminder_id}` - Delete reminder
- `POST /reminder/phone/start-verification` - Start phone verification
- `POST /reminder/phone/send-code` - Send verification code
- `POST /reminder/phone/verify` - Verify phone number
- `DELETE /reminder/phone/remove` - Remove verified phone

### User Profile (`/profile`)
- `GET /profile` - Profile management
- `POST /profile/contact/add` - Add owner contact
- `GET /profile/contact/list` - List contacts
- `DELETE /profile/contact/delete/{contact_id}` - Delete contact
- `DELETE /profile/delete-data` - Delete all user data
- `POST /profile/logout` - Logout

### Checkout (`/checkout`)
- `GET /checkout` - Checkout page
- `POST /checkout/process` - Process MercadoPago payment

### Webhooks (`/webhook`)
- `GET /webhook/whatsapp` - WhatsApp webhook verification
- `POST /webhook/whatsapp` - WhatsApp webhook receiver (messages & statuses)

### Authentication
- `GET /oauth/google/callback` - Google OAuth callback
- `GET /reactivate-account` - Account reactivation page
- `POST /reactivate-account` - Reactivate deleted account

## 📊 Current Limitations & Production Readiness

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
- **Monitoring**: Basic → Need CloudWatch + alerting

### ✅ What's Ready for Production

#### Code Quality
- **Memory Safety** - Rust prevents common vulnerabilities
- **Type Safety** - Compile-time error prevention
- **Error Handling** - Comprehensive error management
- **Security** - Industry-standard cryptographic implementations

#### Infrastructure Foundation
- **IaC with Terraform** - Reproducible infrastructure
- **SSL/TLS** - Secure communications
- **Domain Management** - Professional DNS setup
- **Configuration Management** - Secure parameter storage

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

#### 3. EC2 Instance Setup

```bash
# SSH into EC2 instance
ssh -i terraform/pet-info.pem ec2-user@<instance-ip>

# Upload compiled binary
scp -i terraform/pet-info.pem web_app/out/pet-info ec2-user@<instance-ip>:/home/ec2-user/pet-info/web_app/

# Set up permissions
cd /home/ec2-user/pet-info
sudo chown -R ec2-user:ec2-user .
chmod +x web_app/pet-info

# Apply capability to bind to port 443 (if needed)
sudo setcap CAP_NET_BIND_SERVICE=+ep /home/ec2-user/pet-info/web_app/pet-info

# Start application (with SSM configuration)
cd web_app
nohup ./pet-info > app.log 2>&1 &

# Verify it's running
ps aux | grep pet-info

# Stop application
pkill -f pet-info
```

#### 4. SSL/TLS Certificate Setup
```bash
# Install certbot with Route53 plugin
uv venv && source .venv/bin/activate && uv pip install certbot-dns-route53

# Create or renew certificate
sudo -E certbot certonly --dns-route53 -d pet-info.link

# Certificate locations:
# /etc/letsencrypt/live/pet-info.link/fullchain.pem
# /etc/letsencrypt/live/pet-info.link/privkey.pem

# Check certificate status
sudo certbot certificates
```

### Build Features

#### Compilation Modes
```bash
# Development build (with env vars)
cargo build

# Production build (with SSM Parameter Store)
cargo build --release --features ssm
```

#### Docker Builds
The project uses multi-stage Docker builds for cross-compilation:
- **ec2.Dockerfile**: Builds for x86_64 EC2 instances
- **lambda_build.Dockerfile**: Builds Lambda functions with ARM64 target

### Server Architecture
- **Application Port**: 8080 (HTTP, localhost only)
- **Public Port**: 443 (HTTPS, handled by nginx reverse proxy)
- **Protocol**: HTTP/1.1
- **Compression**: Gzip compression enabled
- **Sessions**: Secure cookies with configurable expiration
- **CORS**: Configured for Google OAuth, MercadoPago, and WhatsApp APIs

---

## 📝 Additional Resources

- **Migrations**: SQL schema files in `migrations/create_tables.sql`
- **Blog Content**: Markdown files in `web_app/web/blog/` (privacy, terms, about, questions)
- **PDF Templates**: Typst templates in `web_app/web/reports/`
- **Frontend Assets**: Static files in `web_app/web/static/`
- **Templates**: Tera HTML templates in `web_app/web/templates/`

## 🛠️ Development Tools

- **Language**: Rust 2024 Edition
- **Build System**: Cargo + Make + Docker
- **Database CLI**: Custom migration tool in `scripts/`
- **Testing**: mockall for unit testing
- **Linting**: clippy (recommended)
- **Formatting**: rustfmt (recommended)

## 📊 Observability

- **Logging**: Logfire with OpenTelemetry integration
- **Metrics**: Prometheus-compatible metrics via Logfire
- **Tracing**: Distributed tracing with tracing crate
- **Monitoring**: CloudWatch (Lambda, Step Functions)

---

**Status**: Production (Alpha)
**Version**: 0.1.0
**Last Updated**: November 2025
**Production Readiness**: 60% (infrastructure needs scaling improvements, code is solid)
**Demo**: https://pet-info.link
