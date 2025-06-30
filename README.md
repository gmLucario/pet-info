# Pet Info Web Application

A pet information management system built with Rust, featuring QR code generation, payment processing, WhatsApp notifications, and Apple Wallet integration.

**Demo**: https://pet-info.link

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
- **Pet Profile Management** - Create, edit, and manage pet information
- **QR Code Generation** - Unique QR codes for each pet linking to public profiles
- **Payment Processing** - MercadoPago integration for premium features
- **WhatsApp Notifications** - Automated reminders and notifications
- **Apple Wallet Passes** - Generate digital pet ID cards
- **Google OAuth** - Secure user authentication
- **PDF Reports** - Generate comprehensive pet health reports
- **File Upload** - Pet photos and document management

### Security Features
- **CSRF Protection** - Argon2-based token generation
- **Database Encryption** - SQLCipher for data at rest
- **SSL/TLS** - End-to-end encryption
- **OAuth 2.0** - Secure authentication
- **TOTP** - Two-factor authentication support
- **Secrets Management** - AWS SSM Parameter Store integration

## 🛠️ Technical Stack

### Backend (Rust)
- **Framework**: Ntex (high-performance async web framework)
- **Database**: SQLite with SQLCipher encryption
- **Authentication**: OAuth2 (Google), session-based
- **Templating**: Tera template engine
- **Configuration**: Environment variables + AWS SSM
- **Logging**: Logfire integration
- **Testing**: Built-in Rust testing framework

### Infrastructure (Terraform)
- **Cloud Provider**: AWS
- **IaC**: Terraform modules
- **Configuration**: SSM Parameter Store
- **Storage**: S3 for files, SQLite for structured data
- **Compute**: Single EC2 instance
- **Networking**: Route 53 DNS

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
```


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

### Development
```bash
cd web_app
cargo run
```

### Production (Current)
```bash
# Deploy infrastructure
make deploy_prod_infra

# Build and deploy application
make build_scripts_app_ec2

# Manual deployment (current process)
scp target/release/pet-info ec2-user@instance:/opt/app/
```

---

**Status**: Development/Alpha
**Version**: 0.1.0  
**Last Updated**: June 2025
**Production Readiness**: 60% (infrastructure needs work, code is solid)
