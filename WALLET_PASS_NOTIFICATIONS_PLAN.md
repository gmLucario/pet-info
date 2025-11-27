# Wallet Pass Notifications Implementation Plan

## Current State Summary

Your Pet Info app is a Rust-based web application that:
- **Generates Apple Wallet passes** for pets with QR codes, pet info, and photos
- **Uses SQLite database** (encrypted with SQLCipher) for data storage
- **Has AWS infrastructure** with S3, Lambda, Step Functions for WhatsApp reminders
- **Stack**: Ntex web framework, OAuth2 authentication, MercadoPago payments
- **Current pass limitation**: Passes are **static** - once downloaded, they never update

## Apple Wallet Pass Notifications: What's Required

Based on research, here's what Apple requires for pass updates:

### Technical Requirements:
1. **Web Service URL** in the pass that Apple Wallet can call
2. **5 specific REST endpoints** that follow Apple's PassKit Web Service protocol
3. **Database to track device registrations** and push tokens
4. **APNS connection** using your Pass Type ID Certificate to send push notifications
5. **Authentication tokens** unique to each pass instance

---

## Prerequisites: Apple Developer Account Setup

**⚠️ IMPORTANT:** If you don't already have an Apple Developer account with a Pass Type ID certificate, you MUST complete these steps BEFORE implementing wallet pass notifications.

### Step 1: Enroll in Apple Developer Program

**Requirements:**
- Cost: **$99 USD per year**
- Apple ID (create one at https://appleid.apple.com if you don't have one)
- Valid payment method (credit card)
- Business information (if enrolling as an organization)

**Enrollment Process:**

1. **Go to Apple Developer Program enrollment page:**
   - Visit: https://developer.apple.com/programs/enroll/

2. **Sign in with your Apple ID**
   - Use your personal Apple ID or create a new one
   - If enrolling as an organization, you'll need D-U-N-S number

3. **Choose enrollment type:**
   - **Individual**: Personal account, apps published under your name
   - **Organization**: Business account, requires company verification (2-3 days)

4. **Complete enrollment form:**
   - Provide contact information
   - Accept license agreement
   - Pay $99 annual fee

5. **Wait for approval:**
   - Individual: Usually instant
   - Organization: 1-3 business days for verification

6. **Confirmation:**
   - You'll receive email confirmation
   - Access to Apple Developer portal at https://developer.apple.com/account

---

### Step 2: Create Pass Type ID

**What is a Pass Type ID?**
- A unique reverse-domain identifier for your wallet passes (e.g., `pass.com.petinfo.link`)
- Required to sign wallet passes and send push notifications
- Cannot be changed once created

**Creation Steps:**

1. **Log in to Apple Developer Account:**
   - Visit: https://developer.apple.com/account
   - Go to: **Certificates, Identifiers & Profiles**

2. **Create new Pass Type ID:**
   - Click **Identifiers** in sidebar
   - Click the **+** button (top right)
   - Select **Pass Type IDs**
   - Click **Continue**

3. **Register Pass Type ID:**
   - **Description**: `Pet Info Pass` (or your app name)
   - **Identifier**: `pass.com.petinfo.link` (use your domain)
     - Format: `pass.com.yourdomain.yourapp`
     - Must start with `pass.`
     - Use reverse-domain notation
   - Click **Continue**
   - Review and click **Register**

4. **Verify creation:**
   - You should see your Pass Type ID in the list
   - **Status**: Should show "Enabled"
   - **Note the identifier** - you'll need it in your code

---

### Step 3: Generate Pass Type ID Certificate

**What is this certificate for?**
- Signs your .pkpass files (cryptographic signature)
- Authenticates with Apple Push Notification Service (APNS)
- **CRITICAL**: You need the SAME certificate for both pass signing AND push notifications

**Certificate Generation Process:**

#### 3.1: Create Certificate Signing Request (CSR) on Mac

**On macOS:**

1. **Open Keychain Access:**
   - Applications → Utilities → Keychain Access

2. **Request a Certificate:**
   - Menu: **Keychain Access** → **Certificate Assistant** → **Request a Certificate from a Certificate Authority**

3. **Fill in CSR information:**
   - **User Email Address**: Your email (e.g., `dev@petinfo.link`)
   - **Common Name**: `Pet Info Pass Certificate` (descriptive name)
   - **CA Email Address**: Leave blank
   - **Request is**: Select **"Saved to disk"**
   - Click **Continue**

4. **Save CSR file:**
   - Save as: `PassTypeID.certSigningRequest`
   - Location: Desktop or Documents
   - Click **Save**

**On Linux (using OpenSSL):**

```bash
# Generate private key
openssl genrsa -out pass_private_key.pem 2048

# Generate CSR
openssl req -new -key pass_private_key.pem -out PassTypeID.certSigningRequest \
  -subj "/emailAddress=dev@petinfo.link/CN=Pet Info Pass Certificate/C=US"

# IMPORTANT: Keep pass_private_key.pem secure - you'll need it later!
```

#### 3.2: Upload CSR to Apple Developer Portal

1. **Go to Certificates section:**
   - https://developer.apple.com/account/resources/certificates/list
   - Click the **+** button (top right)

2. **Select certificate type:**
   - Scroll to **Services** section
   - Select **Pass Type ID Certificate**
   - Click **Continue**

3. **Choose Pass Type ID:**
   - Select the Pass Type ID you created earlier (`pass.com.petinfo.link`)
   - Click **Continue**

4. **Upload CSR:**
   - Click **Choose File**
   - Select your `PassTypeID.certSigningRequest` file
   - Click **Continue**

5. **Download certificate:**
   - Click **Download**
   - Save as: `pass_certificate.cer`
   - **Status**: Certificate is now "Active"

#### 3.3: Install and Export Certificate

**On macOS:**

1. **Install certificate:**
   - Double-click `pass_certificate.cer`
   - It will open in Keychain Access
   - Certificate is added to **login** keychain

2. **Export certificate with private key:**
   - In Keychain Access, find "Pass Type ID: pass.com.petinfo.link"
   - Expand the arrow to see private key underneath
   - **Right-click** on the certificate (not the key)
   - Select **Export "Pass Type ID: pass.com.petinfo.link"**
   - **File Format**: Select **Personal Information Exchange (.p12)**
   - Save as: `pass_certificate.p12`
   - **Set a password** (remember this!)
   - Click **Save**
   - Enter your Mac password to allow export

3. **Convert P12 to PEM format (for Rust app):**

```bash
# Extract certificate
openssl pkcs12 -in pass_certificate.p12 -clcerts -nokeys -out pass_cert.pem

# Extract private key
openssl pkcs12 -in pass_certificate.p12 -nocerts -out pass_key_encrypted.pem

# Remove passphrase from private key (optional, for production use environment variables)
openssl rsa -in pass_key_encrypted.pem -out pass_key.pem

# Verify files
ls -la pass_cert.pem pass_key.pem
```

**On Linux:**

```bash
# You already have the private key from Step 3.1
# Just need to convert the .cer to PEM format

# Convert certificate to PEM
openssl x509 -inform DER -in pass_certificate.cer -out pass_cert.pem

# Your private key is already in PEM format
# Copy it to the final name
cp pass_private_key.pem pass_key.pem

# Verify files
ls -la pass_cert.pem pass_key.pem
```

---

### Step 4: Download Apple WWDR Certificate

**What is WWDR Certificate?**
- Apple Worldwide Developer Relations Certificate Authority certificate
- Required intermediate certificate for pass signing
- Different from your Pass Type ID certificate

**Download Process:**

1. **Download WWDR G4 Certificate:**
   - Visit: https://www.apple.com/certificateauthority/
   - Scroll to **Apple Intermediate Certificates**
   - Download: **Worldwide Developer Relations - G4** (for passes created after 2022)
   - File: `AppleWWDRCAG4.cer`

2. **Convert to PEM format:**

```bash
openssl x509 -inform DER -in AppleWWDRCAG4.cer -out wwdr.pem
```

3. **Verify you have all certificate files:**

```bash
ls -la pass_cert.pem pass_key.pem wwdr.pem
```

You should see three files:
- `pass_cert.pem` - Your Pass Type ID certificate
- `pass_key.pem` - Private key for your certificate
- `wwdr.pem` - Apple WWDR intermediate certificate

---

### Step 5: Update Your Application Configuration

**File locations for certificates:**

1. **For development (local):**
   ```bash
   mkdir -p ~/pet-info/certs
   cp pass_cert.pem ~/pet-info/certs/
   cp pass_key.pem ~/pet-info/certs/
   cp wwdr.pem ~/pet-info/certs/
   chmod 600 ~/pet-info/certs/*.pem  # Secure permissions
   ```

2. **Update environment variables:**
   ```bash
   # Add to .env or export
   export APNS_PASS_CERT_PATH="/home/user/pet-info/certs/pass_cert.pem"
   export APNS_PASS_KEY_PATH="/home/user/pet-info/certs/pass_key.pem"
   export APNS_ENVIRONMENT="sandbox"  # Use sandbox for testing
   export PASS_WWDR_CERT_PATH="/home/user/pet-info/certs/wwdr.pem"
   ```

3. **For production (AWS EC2):**
   ```bash
   # Upload certificates to EC2 instance
   scp pass_cert.pem user@your-ec2:/etc/pet-info/certs/
   scp pass_key.pem user@your-ec2:/etc/pet-info/certs/
   scp wwdr.pem user@your-ec2:/etc/pet-info/certs/

   # Set secure permissions
   ssh user@your-ec2 "chmod 600 /etc/pet-info/certs/*.pem"

   # Add to SSM Parameter Store (recommended)
   aws ssm put-parameter \
     --name /pet-info/APNS_PASS_CERT_PATH \
     --value "/etc/pet-info/certs/pass_cert.pem" \
     --type String

   aws ssm put-parameter \
     --name /pet-info/APNS_PASS_KEY_PATH \
     --value "/etc/pet-info/certs/pass_key.pem" \
     --type String
   ```

---

### Step 6: Verify Your Pass Type ID

**Check your existing passes.rs implementation:**

1. **Find your current Pass Type ID:**
   ```bash
   grep -n "passTypeIdentifier" web_app/src/api/passes.rs
   ```

2. **Verify it matches your Apple Developer account:**
   - Should be: `pass.com.petinfo.link`
   - If different, you'll need to either:
     - **Option A**: Update your code to use the new Pass Type ID
     - **Option B**: Create a new Pass Type ID in Apple Developer portal to match your code

3. **Check Team Identifier:**
   ```bash
   grep -n "teamIdentifier" web_app/src/api/passes.rs
   ```

   - Find your Team ID in Apple Developer portal:
     - Go to: https://developer.apple.com/account
     - Click **Membership** in sidebar
     - **Team ID**: 10-character alphanumeric (e.g., `S89P27T8CF`)
   - Verify it matches your code

---

### Step 7: Testing Strategy

**Important: Use Sandbox Environment First**

1. **Start with APNS Sandbox:**
   ```bash
   export APNS_ENVIRONMENT="sandbox"
   ```

2. **Development workflow:**
   - Generate passes with `webServiceURL` and `authenticationToken`
   - Test on physical iOS device (Simulator doesn't support push notifications)
   - Monitor logs for APNS responses
   - Check device registration in database

3. **Switch to Production:**
   - Only after successful sandbox testing
   - Update environment variable:
     ```bash
     export APNS_ENVIRONMENT="production"
     ```
   - Deploy to production server
   - Test with real users

---

### Certificate Expiration and Renewal

**Important Notes:**

1. **Certificate Validity:**
   - Pass Type ID certificates are valid for **3 years**
   - Check expiration date:
     ```bash
     openssl x509 -in pass_cert.pem -noout -enddate
     ```

2. **Before Expiration:**
   - Renew certificate 30 days before expiration
   - Generate new CSR
   - Download new certificate
   - Replace old certificate files
   - **No code changes needed** if Pass Type ID stays the same
   - Restart application to load new certificate

3. **Calendar Reminder:**
   - Set reminder for certificate expiration
   - Apple Developer portal will also send email reminders

---

### Troubleshooting Certificate Issues

#### Issue: "Certificate not found" error
**Solution:**
- Verify file paths in configuration
- Check file permissions (should be 600)
- Ensure certificates are in PEM format

#### Issue: "Invalid certificate" when signing passes
**Solution:**
- Verify certificate is for Pass Type ID (not iOS App Development)
- Check that Pass Type ID in code matches certificate
- Ensure WWDR certificate is included

#### Issue: APNS connection fails
**Solution:**
- Verify using correct certificate (Pass Type ID, not APNs)
- Check environment (sandbox vs production)
- Ensure certificate hasn't expired
- Test network connectivity to APNS servers

#### Issue: "Unable to load private key"
**Solution:**
- Verify private key is in PEM format
- Check if key is encrypted (should be decrypted)
- Ensure key matches certificate
- Test with: `openssl rsa -in pass_key.pem -check`

---

### Summary Checklist: Apple Developer Account Setup

- [ ] Enroll in Apple Developer Program ($99/year)
- [ ] Wait for account approval (instant to 3 days)
- [ ] Create Pass Type ID (`pass.com.petinfo.link`)
- [ ] Generate Certificate Signing Request (CSR)
- [ ] Create Pass Type ID Certificate in Apple Developer portal
- [ ] Download certificate (.cer file)
- [ ] Export certificate with private key (.p12 on Mac)
- [ ] Convert to PEM format (pass_cert.pem and pass_key.pem)
- [ ] Download Apple WWDR G4 Certificate
- [ ] Convert WWDR to PEM format (wwdr.pem)
- [ ] Upload certificates to server (development and production)
- [ ] Set secure file permissions (chmod 600)
- [ ] Update configuration with certificate paths
- [ ] Verify Pass Type ID matches your code
- [ ] Verify Team ID matches your code
- [ ] Set APNS environment to "sandbox" for testing
- [ ] Test pass generation and signing
- [ ] Test APNS connection
- [ ] Set calendar reminder for certificate expiration (3 years)

**Estimated Time:** 2-4 hours (including waiting for account approval)

**Cost:** $99 USD/year (Apple Developer Program membership)

---

## Implementation Plan

### Phase 1: Database Schema Changes

**New tables needed:**

```sql
-- Track which devices have registered for pass updates
pass_registration
├── id (PK)
├── device_library_identifier (Apple device ID)
├── pass_type_identifier (pass.com.petinfo.link)
├── serial_number (pet.external_id)
├── push_token (APNS token)
├── created_at
├── updated_at
└── UNIQUE(device_library_identifier, pass_type_identifier, serial_number)

-- Track authentication tokens for each pass instance
pass_authentication_token
├── id (PK)
├── serial_number (pet.external_id - FK to pet)
├── authentication_token (UUID, unique per pass)
├── created_at
└── UNIQUE(serial_number)

-- Track when passes were last updated (for change detection)
pass_update_tag
├── id (PK)
├── serial_number (pet.external_id)
├── update_tag (timestamp or version number)
├── updated_at
└── UNIQUE(serial_number)

-- Optional: Log errors from Apple Wallet for debugging
pass_error_log
├── id (PK)
├── serial_number
├── error_message (TEXT)
├── created_at
```

**SQL Migration File: `migrations/add_pass_registrations.sql`**

```sql
-- Track device registrations for pass updates
CREATE TABLE IF NOT EXISTS pass_registration (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_library_identifier TEXT NOT NULL,
    pass_type_identifier TEXT NOT NULL,
    serial_number TEXT NOT NULL,
    push_token TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(device_library_identifier, pass_type_identifier, serial_number)
);

CREATE INDEX idx_pass_registration_device ON pass_registration(device_library_identifier);
CREATE INDEX idx_pass_registration_serial ON pass_registration(serial_number);

-- Track authentication tokens for each pass
CREATE TABLE IF NOT EXISTS pass_authentication_token (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    serial_number TEXT NOT NULL UNIQUE,
    authentication_token TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (serial_number) REFERENCES pet(external_id) ON DELETE CASCADE
);

CREATE INDEX idx_pass_auth_token ON pass_authentication_token(authentication_token);

-- Track pass update tags (versioning)
CREATE TABLE IF NOT EXISTS pass_update_tag (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    serial_number TEXT NOT NULL UNIQUE,
    update_tag TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (serial_number) REFERENCES pet(external_id) ON DELETE CASCADE
);

-- Optional: Log errors from Apple Wallet
CREATE TABLE IF NOT EXISTS pass_error_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    serial_number TEXT,
    error_message TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_pass_error_serial ON pass_error_log(serial_number);
```

---

### Phase 2: Pass Generation Updates

**Modify `web_app/src/api/passes.rs`** to include:

```rust
// Add to pass.json structure:
{
  "webServiceURL": "https://pet-info.link/api/v1",
  "authenticationToken": "{unique_uuid_per_pass}",
  // ... existing fields
}
```

**Implementation steps:**

1. Generate unique `authenticationToken` (UUID) when creating pass
2. Store it in `pass_authentication_token` table
3. Initialize `pass_update_tag` with current timestamp
4. Include both `webServiceURL` and `authenticationToken` in the .pkpass file

**Code changes in `generate_pet_pass()` function:**

```rust
// Generate or retrieve authentication token
let auth_token = get_or_create_auth_token(&pet.external_id, &repo).await?;

// Add to pass JSON
pass_json["webServiceURL"] = json!("https://pet-info.link/api/v1");
pass_json["authenticationToken"] = json!(auth_token);

// Initialize update tag if not exists
initialize_pass_update_tag(&pet.external_id, &repo).await?;
```

---

### Phase 3: Apple PassKit Web Service Webhook Endpoints

**Create new webhook module following the same structure as `webhook/whatsapp/`:**

```
web_app/src/webhook/passkit/
├── mod.rs              # Module documentation and re-exports
├── routes.rs           # HTTP endpoint handlers (5 PassKit endpoints)
├── handler.rs          # Business logic (validate tokens, update registrations)
└── schemas.rs          # Request/response data structures
```

Required endpoints per Apple specification:

#### 1. Register Device for Pass Updates
```rust
POST /api/v1/devices/{deviceLibraryIdentifier}/registrations/{passTypeIdentifier}/{serialNumber}
Headers: Authorization: ApplePass {authenticationToken}
Body: { "pushToken": "..." }
Response: 201 Created (new) or 200 OK (existing)
```

#### 2. Get Serial Numbers for Passes
```rust
GET /api/v1/devices/{deviceLibraryIdentifier}/registrations/{passTypeIdentifier}
Query: ?passesUpdatedSince={timestamp}
Response: {
  "lastUpdated": "{timestamp}",
  "serialNumbers": ["pet-uuid-1", "pet-uuid-2"]
}
```

#### 3. Unregister Device
```rust
DELETE /api/v1/devices/{deviceLibraryIdentifier}/registrations/{passTypeIdentifier}/{serialNumber}
Headers: Authorization: ApplePass {authenticationToken}
Response: 200 OK
```

#### 4. Get Latest Version of Pass
```rust
GET /api/v1/passes/{passTypeIdentifier}/{serialNumber}
Headers: Authorization: ApplePass {authenticationToken}
If-Modified-Since: {last_known_update}
Response: 200 OK with .pkpass binary or 304 Not Modified
```

#### 5. Log Errors from Devices
```rust
POST /api/v1/log
Body: { "logs": ["error message 1", "error message 2"] }
Response: 200 OK
```

---

**Module structure:**

#### `web_app/src/webhook/passkit/mod.rs`

```rust
//! Apple PassKit Web Service webhook integration module
//!
//! This module provides webhook handling for Apple Wallet PassKit integration.
//! It implements the 5 required endpoints per Apple's PassKit Web Service specification.
//!
//! ## Submodules
//!
//! - [`handler`] - Business logic for processing PassKit webhook requests
//! - [`routes`] - HTTP endpoint handlers for PassKit webhooks
//! - [`schemas`] - Data structures for PassKit webhook payloads
//!
//! ## Apple PassKit Web Service Protocol
//!
//! When a user adds a pass to Apple Wallet, the device registers with these endpoints
//! to receive automatic updates. When pass data changes, we send APNS push notifications
//! and the device fetches the latest pass version.
//!
//! ## Security
//!
//! Authentication is handled via unique authentication tokens embedded in each pass.
//! Apple Wallet sends these tokens in the `Authorization: ApplePass {token}` header.

pub mod handler;
pub mod routes;
pub mod schemas;

// Re-export route handlers for convenience
pub use routes::{
    register_device,
    get_serial_numbers,
    unregister_device,
    get_latest_pass,
    log_errors
};
```

#### `web_app/src/webhook/passkit/schemas.rs`

```rust
//! Data structures for PassKit webhook requests and responses

use serde::{Deserialize, Serialize};

/// Request body for device registration
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// APNS push token for this device
    #[serde(rename = "pushToken")]
    pub push_token: String,
}

/// Response for get serial numbers endpoint
#[derive(Debug, Serialize)]
pub struct SerialNumbersResponse {
    /// Timestamp of the last update
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,

    /// List of serial numbers for passes updated since the given timestamp
    #[serde(rename = "serialNumbers")]
    pub serial_numbers: Vec<String>,
}

/// Request body for log errors endpoint
#[derive(Debug, Deserialize)]
pub struct LogRequest {
    /// Array of error messages from Apple Wallet devices
    pub logs: Vec<String>,
}

/// Query parameters for get serial numbers endpoint
#[derive(Debug, Deserialize)]
pub struct SerialNumbersQuery {
    /// Optional timestamp - only return passes updated after this time
    #[serde(rename = "passesUpdatedSince")]
    pub passes_updated_since: Option<String>,
}
```

#### `web_app/src/webhook/passkit/handler.rs`

```rust
//! Business logic for PassKit webhook processing

use crate::repo;
use anyhow::{Context, Result};

/// Validates authentication token for a pass
///
/// Checks if the provided token matches the stored authentication token
/// for the given serial number.
///
/// # Arguments
///
/// * `serial_number` - The pass serial number (pet external_id)
/// * `auth_token` - The authentication token from the request header
/// * `repo` - Database repository
///
/// # Returns
///
/// * `Ok(true)` if token is valid
/// * `Ok(false)` if token is invalid
/// * `Err` if database error occurs
pub async fn validate_auth_token(
    serial_number: &str,
    auth_token: &str,
    repo: &repo::ImplAppRepo,
) -> Result<bool> {
    repo.validate_auth_token(serial_number, auth_token)
        .await
        .context("Failed to validate auth token")
}

/// Registers a device for pass updates
///
/// Creates or updates a device registration in the database.
///
/// # Arguments
///
/// * `device_id` - Apple device library identifier
/// * `pass_type_id` - Pass type identifier (e.g., "pass.com.petinfo.link")
/// * `serial_number` - Pass serial number (pet external_id)
/// * `push_token` - APNS push token for notifications
/// * `repo` - Database repository
///
/// # Returns
///
/// * `Ok(true)` if new registration was created
/// * `Ok(false)` if existing registration was updated
pub async fn register_device_for_pass(
    device_id: &str,
    pass_type_id: &str,
    serial_number: &str,
    push_token: &str,
    repo: &repo::ImplAppRepo,
) -> Result<bool> {
    let is_new = repo
        .register_device(device_id, pass_type_id, serial_number, push_token)
        .await
        .context("Failed to register device")?;

    logfire::info!(
        "Device registered for pass updates: device={device_id}, pass={serial_number}, new={is_new}",
        device_id = device_id,
        serial_number = serial_number,
        is_new = is_new
    );

    Ok(is_new)
}

/// Unregisters a device from pass updates
///
/// Removes the device registration from the database.
///
/// # Arguments
///
/// * `device_id` - Apple device library identifier
/// * `pass_type_id` - Pass type identifier
/// * `serial_number` - Pass serial number
/// * `repo` - Database repository
pub async fn unregister_device_from_pass(
    device_id: &str,
    pass_type_id: &str,
    serial_number: &str,
    repo: &repo::ImplAppRepo,
) -> Result<()> {
    repo.unregister_device(device_id, pass_type_id, serial_number)
        .await
        .context("Failed to unregister device")?;

    logfire::info!(
        "Device unregistered from pass updates: device={device_id}, pass={serial_number}",
        device_id = device_id,
        serial_number = serial_number
    );

    Ok(())
}

/// Gets serial numbers for passes updated since a given timestamp
///
/// Returns all serial numbers for passes registered to a device
/// that have been updated after the specified timestamp.
///
/// # Arguments
///
/// * `device_id` - Apple device library identifier
/// * `pass_type_id` - Pass type identifier
/// * `updated_since` - Optional timestamp filter
/// * `repo` - Database repository
///
/// # Returns
///
/// * Tuple of (last_updated_timestamp, serial_numbers)
pub async fn get_updated_serial_numbers(
    device_id: &str,
    pass_type_id: &str,
    updated_since: Option<&str>,
    repo: &repo::ImplAppRepo,
) -> Result<(String, Vec<String>)> {
    repo.get_serial_numbers_updated_since(device_id, pass_type_id, updated_since)
        .await
        .context("Failed to get updated serial numbers")
}

/// Logs errors from Apple Wallet devices
///
/// Stores error messages in the database for debugging.
///
/// # Arguments
///
/// * `logs` - Array of error messages
/// * `repo` - Database repository
pub async fn log_device_errors(
    logs: Vec<String>,
    repo: &repo::ImplAppRepo,
) -> Result<()> {
    for log in logs {
        logfire::warn!("PassKit device error: {log}", log = log);

        // Store in database for debugging
        repo.insert_pass_error_log(None, &log)
            .await
            .context("Failed to insert pass error log")?;
    }

    Ok(())
}
```

#### `web_app/src/webhook/passkit/routes.rs`

```rust
//! PassKit webhook HTTP endpoint handlers
//!
//! Implements the 5 required endpoints per Apple's PassKit Web Service specification.
//! These endpoints allow Apple Wallet to register devices, check for updates,
//! and fetch the latest version of passes.

use super::{handler, schemas};
use crate::{
    api::passes,
    config,
    front::{AppState, errors},
};
use ntex::web;

/// Extract authentication token from Authorization header
///
/// Expected format: "Authorization: ApplePass {token}"
fn extract_auth_token(req: &web::HttpRequest) -> Result<String, web::Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| errors::UserError::Unauthorized)?
        .to_str()
        .map_err(|_| errors::UserError::Unauthorized)?;

    // Remove "ApplePass " prefix
    let token = auth_header
        .strip_prefix("ApplePass ")
        .ok_or_else(|| errors::UserError::Unauthorized)?;

    Ok(token.to_string())
}

/// Register device for pass updates (POST)
///
/// Called when a user adds a pass to Apple Wallet.
/// Stores the device's push token for sending update notifications.
#[web::post("/devices/{device_id}/registrations/{pass_type_id}/{serial_number}")]
pub async fn register_device(
    req: web::HttpRequest,
    path: web::types::Path<(String, String, String)>,
    body: web::types::Json<schemas::RegisterRequest>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let (device_id, pass_type_id, serial_number) = path.into_inner();

    // Validate authentication token
    let auth_token = extract_auth_token(&req)?;
    let is_valid = handler::validate_auth_token(&serial_number, &auth_token, &app_state.repo)
        .await
        .map_err(|_| errors::UserError::Unauthorized)?;

    if !is_valid {
        return Err(errors::UserError::Unauthorized.into());
    }

    // Register device
    let is_new = handler::register_device_for_pass(
        &device_id,
        &pass_type_id,
        &serial_number,
        &body.push_token,
        &app_state.repo,
    )
    .await
    .map_err(|e| {
        logfire::error!("Failed to register device: {error}", error = e.to_string());
        errors::UserError::InternalError
    })?;

    // Return 201 for new registration, 200 for update
    if is_new {
        Ok(web::HttpResponse::Created().finish())
    } else {
        Ok(web::HttpResponse::Ok().finish())
    }
}

/// Get serial numbers for updated passes (GET)
///
/// Returns list of pass serial numbers that have been updated
/// since the provided timestamp.
#[web::get("/devices/{device_id}/registrations/{pass_type_id}")]
pub async fn get_serial_numbers(
    path: web::types::Path<(String, String)>,
    query: web::types::Query<schemas::SerialNumbersQuery>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let (device_id, pass_type_id) = path.into_inner();

    // Get updated serial numbers
    let (last_updated, serial_numbers) = handler::get_updated_serial_numbers(
        &device_id,
        &pass_type_id,
        query.passes_updated_since.as_deref(),
        &app_state.repo,
    )
    .await
    .map_err(|e| {
        logfire::error!("Failed to get serial numbers: {error}", error = e.to_string());
        errors::UserError::InternalError
    })?;

    let response = schemas::SerialNumbersResponse {
        last_updated,
        serial_numbers,
    };

    Ok(web::HttpResponse::Ok().json(&response))
}

/// Unregister device from pass updates (DELETE)
///
/// Called when a user removes a pass from Apple Wallet.
#[web::delete("/devices/{device_id}/registrations/{pass_type_id}/{serial_number}")]
pub async fn unregister_device(
    req: web::HttpRequest,
    path: web::types::Path<(String, String, String)>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let (device_id, pass_type_id, serial_number) = path.into_inner();

    // Validate authentication token
    let auth_token = extract_auth_token(&req)?;
    let is_valid = handler::validate_auth_token(&serial_number, &auth_token, &app_state.repo)
        .await
        .map_err(|_| errors::UserError::Unauthorized)?;

    if !is_valid {
        return Err(errors::UserError::Unauthorized.into());
    }

    // Unregister device
    handler::unregister_device_from_pass(&device_id, &pass_type_id, &serial_number, &app_state.repo)
        .await
        .map_err(|e| {
            logfire::error!("Failed to unregister device: {error}", error = e.to_string());
            errors::UserError::InternalError
        })?;

    Ok(web::HttpResponse::Ok().finish())
}

/// Get latest version of pass (GET)
///
/// Returns the latest .pkpass file for a given serial number.
/// Supports If-Modified-Since header to return 304 if pass hasn't changed.
#[web::get("/passes/{pass_type_id}/{serial_number}")]
pub async fn get_latest_pass(
    req: web::HttpRequest,
    path: web::types::Path<(String, String)>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let (_pass_type_id, serial_number) = path.into_inner();

    // Validate authentication token
    let auth_token = extract_auth_token(&req)?;
    let is_valid = handler::validate_auth_token(&serial_number, &auth_token, &app_state.repo)
        .await
        .map_err(|_| errors::UserError::Unauthorized)?;

    if !is_valid {
        return Err(errors::UserError::Unauthorized.into());
    }

    // Check If-Modified-Since header
    if let Some(if_modified_since) = req.headers().get("If-Modified-Since") {
        // TODO: Compare with pass_update_tag timestamp
        // If not modified, return 304
        // For now, always return the pass
    }

    // Generate and return the latest pass
    // Reuse the existing pass generation logic from api/passes.rs
    let pass_bytes = passes::generate_pet_pass_by_serial(
        &serial_number,
        &app_state.repo,
        &app_state.storage_service,
    )
    .await
    .map_err(|e| {
        logfire::error!("Failed to generate pass: {error}", error = e.to_string());
        errors::UserError::NotFound
    })?;

    Ok(web::HttpResponse::Ok()
        .content_type("application/vnd.apple.pkpass")
        .body(pass_bytes))
}

/// Log errors from devices (POST)
///
/// Apple Wallet devices can report errors for debugging.
#[web::post("/log")]
pub async fn log_errors(
    body: web::types::Json<schemas::LogRequest>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    handler::log_device_errors(body.logs.clone(), &app_state.repo)
        .await
        .map_err(|e| {
            logfire::error!("Failed to log device errors: {error}", error = e.to_string());
            errors::UserError::InternalError
        })?;

    Ok(web::HttpResponse::Ok().finish())
}
```

---

### Phase 4: APNS Push Notification Service

**Create new module: `web_app/src/services/apns.rs`**

```rust
use a2::{Client, Endpoint, NotificationBuilder, NotificationOptions};
use std::fs::File;
use std::io::Read;

pub struct ApnsPassNotificationService {
    client: Client,
    pass_type_id: String, // "pass.com.petinfo.link"
}

impl ApnsPassNotificationService {
    pub async fn new(
        cert_path: &str,
        key_path: &str,
        environment: &str,
    ) -> Result<Self> {
        // Load certificate and key
        let mut cert_file = File::open(cert_path)?;
        let mut cert_data = Vec::new();
        cert_file.read_to_end(&mut cert_data)?;

        let mut key_file = File::open(key_path)?;
        let mut key_data = Vec::new();
        key_file.read_to_end(&mut key_data)?;

        // Create APNS client
        let endpoint = match environment {
            "production" => Endpoint::Production,
            _ => Endpoint::Sandbox,
        };

        let client = Client::certificate(
            &cert_data,
            &key_data,
            endpoint,
        )?;

        Ok(Self {
            client,
            pass_type_id: "pass.com.petinfo.link".to_string(),
        })
    }

    /// Send push notification to a single device
    pub async fn send_pass_update_notification(&self, push_token: &str) -> Result<()> {
        // Empty payload triggers pass update
        let payload = json!({});

        let options = NotificationOptions {
            apns_topic: Some(&self.pass_type_id),
            ..Default::default()
        };

        let builder = NotificationBuilder::new()
            .set_content_available()
            .set_payload(payload);

        let notification = builder.build(push_token, options);
        let response = self.client.send(notification).await?;

        if response.error.is_some() {
            return Err(anyhow!("APNS error: {:?}", response.error));
        }

        Ok(())
    }

    /// Send push to all devices registered for a pass
    pub async fn notify_all_devices_for_pass(
        &self,
        serial_number: &str,
        repo: &Repo,
    ) -> Result<u32> {
        // Get all registrations for this pass
        let registrations = repo.get_pass_registrations_by_serial(serial_number).await?;

        let mut success_count = 0;
        for registration in registrations {
            match self.send_pass_update_notification(&registration.push_token).await {
                Ok(_) => {
                    success_count += 1;
                    log::info!(
                        "Sent pass update notification for {} to device {}",
                        serial_number,
                        registration.device_library_identifier
                    );
                }
                Err(e) => {
                    log::error!(
                        "Failed to send pass update for {}: {}",
                        serial_number,
                        e
                    );
                }
            }
        }

        Ok(success_count)
    }
}
```

**Dependencies to add to `Cargo.toml`:**

```toml
[dependencies]
# Apple Push Notification Service
a2 = "0.10"

# Existing dependencies already support this:
# - uuid (for auth tokens) ✅
# - serde_json (for JSON) ✅
# - sqlx (for DB) ✅
```

---

### Phase 5: Update Triggers

**Modify existing pet update endpoints** to trigger pass updates.

**Create helper module: `web_app/src/api/pass_update_helper.rs`**

```rust
use crate::repo::Repo;
use crate::services::apns::ApnsPassNotificationService;
use chrono::Utc;

/// Update the pass update tag and notify all registered devices
pub async fn trigger_pass_update(
    serial_number: &str,
    repo: &Repo,
    apns_service: &ApnsPassNotificationService,
) -> Result<()> {
    // Update the pass update tag
    let new_tag = Utc::now().timestamp().to_string();
    repo.update_pass_tag(serial_number, &new_tag).await?;

    // Send push notifications to all registered devices
    let notification_count = apns_service
        .notify_all_devices_for_pass(serial_number, repo)
        .await?;

    log::info!(
        "Pass update triggered for {}: {} notifications sent",
        serial_number,
        notification_count
    );

    Ok(())
}
```

**Update these endpoints in `web_app/src/api/pet.rs`:**

```rust
// After successful pet update
pub async fn update_pet_info(
    pet_id: i64,
    data: UpdatePetRequest,
    repo: &Repo,
    apns_service: &ApnsPassNotificationService,
) -> Result<()> {
    // ... existing update logic
    let pet = repo.update_pet(pet_id, data).await?;

    // NEW: Trigger pass update
    trigger_pass_update(&pet.external_id.to_string(), repo, apns_service).await?;

    Ok(())
}
```

**Triggers needed for:**
- ✅ Pet info edit (`PUT /pet/edit/{pet_id}`)
- ✅ Pet photo update
- ✅ Health record added (`POST /pet/health`)
- ✅ Weight update (`POST /pet/weight`)
- ✅ Lost status change (`PUT /pet/lost`)
- ✅ Sterilization status change

---

### Phase 6: Configuration Updates

**Add to environment variables / SSM Parameters:**

```bash
# Apple Push Notification Service
APNS_PASS_CERT_PATH=/path/to/pass_cert.pem
APNS_PASS_KEY_PATH=/path/to/pass_key.pem
APNS_ENVIRONMENT=production  # or sandbox for testing

# PassKit Web Service
PASSKIT_WEB_SERVICE_URL=https://pet-info.link/api/v1
```

**Update `web_app/src/config.rs`:**

```rust
pub struct Config {
    // ... existing fields

    // APNS Configuration
    pub apns_pass_cert_path: String,
    pub apns_pass_key_path: String,
    pub apns_environment: String,

    // PassKit Web Service
    pub passkit_web_service_url: String,
}
```

---

### Phase 7: Route Registration

**Update `web_app/src/webhook/mod.rs` to include passkit module:**

```rust
pub mod passkit;
pub mod whatsapp;
```

**Update `web_app/src/front/routes.rs` to register PassKit webhook routes:**

```rust
use crate::webhook::passkit;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // ... existing routes

        // PassKit Web Service webhook endpoints
        .service(
            web::scope("/api/v1")
                .service(passkit::register_device)
                .service(passkit::get_serial_numbers)
                .service(passkit::unregister_device)
                .service(passkit::get_latest_pass)
                .service(passkit::log_errors)
        );
}
```

---

### Phase 8: Database Queries

**Add to `web_app/src/repo/sqlite_queries.rs`:**

```rust
// Get or create authentication token for a pass
pub async fn get_or_create_auth_token(&self, serial_number: &str) -> Result<String> {
    // Check if token exists
    let existing = sqlx::query!(
        "SELECT authentication_token FROM pass_authentication_token WHERE serial_number = ?",
        serial_number
    )
    .fetch_optional(&self.pool)
    .await?;

    if let Some(record) = existing {
        return Ok(record.authentication_token);
    }

    // Create new token
    let new_token = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO pass_authentication_token (serial_number, authentication_token) VALUES (?, ?)",
        serial_number,
        new_token
    )
    .execute(&self.pool)
    .await?;

    Ok(new_token)
}

// Register or update device for pass updates
pub async fn register_device(
    &self,
    device_id: &str,
    pass_type_id: &str,
    serial_number: &str,
    push_token: &str,
) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        INSERT INTO pass_registration
        (device_library_identifier, pass_type_identifier, serial_number, push_token, updated_at)
        VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(device_library_identifier, pass_type_identifier, serial_number)
        DO UPDATE SET push_token = ?, updated_at = CURRENT_TIMESTAMP
        "#,
        device_id,
        pass_type_id,
        serial_number,
        push_token,
        push_token
    )
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// Get all registrations for a serial number
pub async fn get_pass_registrations_by_serial(&self, serial_number: &str) -> Result<Vec<PassRegistration>> {
    let records = sqlx::query_as!(
        PassRegistration,
        "SELECT * FROM pass_registration WHERE serial_number = ?",
        serial_number
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(records)
}

// Update pass update tag
pub async fn update_pass_tag(&self, serial_number: &str, update_tag: &str) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO pass_update_tag (serial_number, update_tag, updated_at)
        VALUES (?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(serial_number)
        DO UPDATE SET update_tag = ?, updated_at = CURRENT_TIMESTAMP
        "#,
        serial_number,
        update_tag,
        update_tag
    )
    .execute(&self.pool)
    .await?;

    Ok(())
}

// Validate authentication token
pub async fn validate_auth_token(&self, serial_number: &str, token: &str) -> Result<bool> {
    let result = sqlx::query!(
        "SELECT 1 as valid FROM pass_authentication_token WHERE serial_number = ? AND authentication_token = ?",
        serial_number,
        token
    )
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.is_some())
}
```

---

### Phase 9: Testing Strategy

**Test scenarios:**

1. ✅ **Initial Pass Download**
   - Download pass to iPhone
   - Verify device registration in database
   - Check push_token is stored

2. ✅ **Pass Update Flow**
   - Update pet info via web app
   - Verify `pass_update_tag` is updated
   - Confirm APNS push notification sent
   - Check pass updates on device

3. ✅ **Change Messages**
   - Test visible updates (with changeMessage)
   - Test silent updates (without changeMessage)
   - Verify notification display behavior

4. ✅ **Edge Cases**
   - Invalid authentication token
   - Device unregistration
   - Multiple devices for same pass
   - Pass deleted while registered

5. ✅ **Error Handling**
   - APNS connection failures
   - Invalid push tokens
   - Network timeouts
   - Check error logs

**Testing commands:**

```bash
# Register device (simulated)
curl -X POST https://pet-info.link/api/v1/devices/DEVICE123/registrations/pass.com.petinfo.link/PET-UUID \
  -H "Authorization: ApplePass TOKEN123" \
  -H "Content-Type: application/json" \
  -d '{"pushToken":"APNS-TOKEN-XYZ"}'

# Get serial numbers
curl https://pet-info.link/api/v1/devices/DEVICE123/registrations/pass.com.petinfo.link

# Get latest pass
curl https://pet-info.link/api/v1/passes/pass.com.petinfo.link/PET-UUID \
  -H "Authorization: ApplePass TOKEN123"
```

---

## Architecture Diagram: Pass Update Flow

```
┌─────────────────────────────────────────────────────────┐
│ User's iPhone (Apple Wallet)                            │
│                                                         │
│ 1. User adds .pkpass to Wallet                         │
│    ├─ Reads: webServiceURL from pass                   │
│    └─ Reads: authenticationToken                       │
│         ┌───────────────────────────────┐              │
│         │ POST /api/v1/devices/.../registrations/... │ │
│         │ Auth: ApplePass {token}                    │ │
│         │ Body: { "pushToken": "apns-token-xyz" }    │ │
│         └───────────────┬───────────────────────────┘  │
└─────────────────────────┼──────────────────────────────┘
                          │
        ┌─────────────────▼──────────────────┐
        │ Pet Info Web Service (Rust)        │
        │                                    │
        │ 2. Store registration in DB        │
        │    ├─ device_id                    │
        │    ├─ serial_number (pet UUID)     │
        │    └─ push_token                   │
        │                                    │
        │ 3. User updates pet via web        │
        │    └─ PUT /pet/edit/123            │
        │         ┌────────────────┐         │
        │         │ Update DB      │         │
        │         │ Update pass_tag│         │
        │         │ Trigger notify │         │
        │         └────────┬───────┘         │
        └──────────────────┼─────────────────┘
                           │
        ┌──────────────────▼─────────────────┐
        │ APNS Service (new module)          │
        │                                    │
        │ 4. Send push notification          │
        │    ├─ Lookup: push_tokens for pet  │
        │    ├─ Connect: APNS production     │
        │    ├─ Cert: Pass Type ID cert      │
        │    ├─ Topic: pass.com.petinfo.link │
        │    └─ Payload: {} (empty)          │
        └──────────────────┬─────────────────┘
                           │
        ┌──────────────────▼─────────────────┐
        │ Apple Push Notification Service    │
        │                                    │
        │ 5. Delivers push to devices        │
        └──────────────────┬─────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│ User's iPhone (Apple Wallet)                            │
│                                                         │
│ 6. Receives silent push notification                   │
│    └─ GET /api/v1/passes/.../pet-uuid                  │
│       Header: If-Modified-Since: {last_update}         │
│       Header: Authorization: ApplePass {token}         │
│         ┌─────────────────┐                            │
│         │ Response: 200 OK│                            │
│         │ New .pkpass file│                            │
│         └────────┬────────┘                            │
│                  │                                      │
│ 7. Wallet updates pass in background                   │
│    ├─ If change message present → Show notification    │
│    └─ If no change message → Silent update             │
└─────────────────────────────────────────────────────────┘
```

---

## File Structure Changes

```
web_app/
├── src/
│   ├── api/
│   │   ├── passes.rs (UPDATE - add webServiceURL & authToken generation)
│   │   ├── pass_update_helper.rs (NEW - update trigger logic)
│   │   └── pet.rs (UPDATE - add update triggers)
│   │
│   ├── webhook/
│   │   ├── mod.rs (UPDATE - add passkit module export)
│   │   ├── passkit/  (NEW - PassKit Web Service webhook module)
│   │   │   ├── mod.rs (NEW - module documentation)
│   │   │   ├── routes.rs (NEW - 5 HTTP endpoint handlers)
│   │   │   ├── handler.rs (NEW - business logic)
│   │   │   └── schemas.rs (NEW - request/response structures)
│   │   └── whatsapp/ (EXISTING - keep as is)
│   │
│   ├── models/
│   │   ├── pass_registration.rs (NEW)
│   │   └── pass_update.rs (NEW)
│   │
│   ├── services/
│   │   ├── apns.rs (NEW - APNS push notification service)
│   │   └── notification.rs (EXISTING - keep for WhatsApp)
│   │
│   ├── repo/
│   │   └── sqlite_queries.rs (UPDATE - add pass-related queries)
│   │
│   ├── front/
│   │   └── routes.rs (UPDATE - register PassKit webhook routes)
│   │
│   └── config.rs (UPDATE - add APNS config)
│
├── Cargo.toml (UPDATE - add a2 dependency)
│
└── migrations/
    └── add_pass_registrations.sql (NEW)
```

---

## Change Message Strategy

### For Visible Notifications (user sees alert):

Add `changeMessage` to pass fields that should trigger visible notifications:

```json
{
  "key": "name",
  "label": "Nombre",
  "value": "Buddy",
  "changeMessage": "Your pet's name has been updated to %@"
}
```

**Use for:**
- Pet name change
- Lost status change (critical!)
- Important health record added

### For Silent Updates (no notification):

Omit `changeMessage` from fields:

```json
{
  "key": "weight",
  "label": "Peso",
  "value": "15.5 kg"
  // No changeMessage = silent update
}
```

**Use for:**
- Weight updates
- Photo changes
- Minor info edits
- About text changes

---

## Security Considerations

1. ✅ **Authentication token validation** - verify on every endpoint call
2. ✅ **Rate limiting** - prevent abuse of registration endpoints
3. ✅ **Token uniqueness** - each pass instance has unique auth token
4. ✅ **HTTPS only** - already enforced by Nginx reverse proxy
5. ✅ **APNS certificate security** - store in secure location (like current pass certs)
6. ✅ **Input validation** - validate device IDs, push tokens, serial numbers
7. ✅ **Error logging** - capture and log errors from Apple Wallet devices

---

## Estimated Implementation Complexity

| Phase | Complexity | Time Est. | Files to Modify/Create |
|-------|-----------|-----------|------------------------|
| Database Schema | Low | 1 hour | 1 migration file |
| Pass Generation Updates | Low | 2 hours | 1 file (passes.rs) |
| PassKit Endpoints | Medium | 6 hours | 3 new files |
| APNS Service | Medium-High | 8 hours | 1 new module + config |
| Update Triggers | Low | 3 hours | Modify existing endpoints |
| Database Queries | Low | 2 hours | Update sqlite_queries.rs |
| Route Registration | Low | 1 hour | Update routes.rs |
| Testing | Medium | 6 hours | End-to-end flow |
| **Total** | **Medium** | **~29 hours** | **~12 files** |

---

## Implementation Checklist

### Phase 1: Database
- [ ] Create `migrations/add_pass_registrations.sql`
- [ ] Run migration on local dev database
- [ ] Verify tables created correctly
- [ ] Test cascade delete behavior

### Phase 2: Models
- [ ] Create `models/pass_registration.rs`
- [ ] Create `models/pass_update.rs`
- [ ] Add serialization/deserialization derives
- [ ] Test model compilation

### Phase 3: Database Queries
- [ ] Add `get_or_create_auth_token()` to sqlite_queries.rs
- [ ] Add `register_device()` to sqlite_queries.rs
- [ ] Add `unregister_device()` to sqlite_queries.rs
- [ ] Add `get_pass_registrations_by_serial()` to sqlite_queries.rs
- [ ] Add `update_pass_tag()` to sqlite_queries.rs
- [ ] Add `validate_auth_token()` to sqlite_queries.rs
- [ ] Add `get_serial_numbers_updated_since()` to sqlite_queries.rs

### Phase 4: Pass Generation
- [ ] Update `api/passes.rs` to generate auth tokens
- [ ] Add `webServiceURL` to pass JSON
- [ ] Add `authenticationToken` to pass JSON
- [ ] Initialize pass_update_tag on first pass creation
- [ ] Test pass generation with new fields

### Phase 5: APNS Service
- [ ] Create `services/apns.rs`
- [ ] Add `a2` dependency to Cargo.toml
- [ ] Implement `ApnsPassNotificationService` struct
- [ ] Implement `send_pass_update_notification()` method
- [ ] Implement `notify_all_devices_for_pass()` method
- [ ] Add APNS config to `config.rs`
- [ ] Test APNS connection (sandbox first)

### Phase 6: PassKit Web Service Webhook Endpoints
- [ ] Create `webhook/passkit/` directory
- [ ] Create `webhook/passkit/mod.rs` (module documentation)
- [ ] Create `webhook/passkit/schemas.rs` (request/response structures)
- [ ] Create `webhook/passkit/handler.rs` (business logic)
  - [ ] Implement `validate_auth_token()` function
  - [ ] Implement `register_device_for_pass()` function
  - [ ] Implement `unregister_device_from_pass()` function
  - [ ] Implement `get_updated_serial_numbers()` function
  - [ ] Implement `log_device_errors()` function
- [ ] Create `webhook/passkit/routes.rs` (HTTP endpoint handlers)
  - [ ] Implement `register_device()` endpoint
  - [ ] Implement `get_serial_numbers()` endpoint
  - [ ] Implement `unregister_device()` endpoint
  - [ ] Implement `get_latest_pass()` endpoint
  - [ ] Implement `log_errors()` endpoint
  - [ ] Implement `extract_auth_token()` helper
- [ ] Update `webhook/mod.rs` to export passkit module
- [ ] Add routes to `front/routes.rs`

### Phase 7: Update Triggers
- [ ] Create `api/pass_update_helper.rs`
- [ ] Implement `trigger_pass_update()` function
- [ ] Update `PUT /pet/edit` to trigger updates
- [ ] Update pet photo upload to trigger updates
- [ ] Update health record creation to trigger updates
- [ ] Update weight creation to trigger updates
- [ ] Update lost status change to trigger updates

### Phase 8: Configuration
- [ ] Add APNS environment variables
- [ ] Add PassKit web service URL config
- [ ] Update SSM parameters in AWS (production)
- [ ] Update local .env file (development)
- [ ] Verify certificate paths are correct

### Phase 9: Testing
- [ ] Test pass download with new fields
- [ ] Test device registration endpoint
- [ ] Test get serial numbers endpoint
- [ ] Test unregister endpoint
- [ ] Test get latest pass endpoint
- [ ] Test pet update triggers APNS push
- [ ] Test pass updates on real iPhone
- [ ] Test change message displays correctly
- [ ] Test silent updates work correctly
- [ ] Test error logging endpoint

### Phase 10: Deployment
- [ ] Run migration on production database
- [ ] Deploy updated application
- [ ] Verify APNS certificate is in production
- [ ] Monitor logs for errors
- [ ] Test with real device in production

---

## Potential Issues & Solutions

### Issue 1: APNS Connection Failures
**Symptoms:** Push notifications not being delivered
**Solutions:**
- Verify certificate is Pass Type ID certificate (not APNs)
- Check certificate expiration date
- Ensure using production APNS endpoint
- Verify `passTypeIdentifier` matches certificate

### Issue 2: Devices Not Receiving Updates
**Symptoms:** Pass doesn't update on device
**Solutions:**
- Check push token is valid
- Verify device is registered in database
- Ensure `update_tag` is being updated
- Check APNS response for errors
- Test with sandbox environment first

### Issue 3: Authentication Failures
**Symptoms:** 401 Unauthorized errors
**Solutions:**
- Verify authentication token format
- Check token is stored in database
- Ensure `Authorization: ApplePass {token}` header format
- Validate token hasn't been rotated

### Issue 4: Pass Not Found Errors
**Symptoms:** 404 errors when fetching pass
**Solutions:**
- Verify serial number exists
- Check pet hasn't been deleted
- Ensure pass generation logic works
- Validate external_id mapping

---

## Future Enhancements

1. **Google Wallet Support**
   - Different protocol than Apple Wallet
   - Uses Google Wallet API
   - JWT-based updates

2. **Pass Analytics**
   - Track pass views
   - Monitor update frequency
   - Device distribution metrics

3. **Conditional Updates**
   - Only update if specific fields changed
   - Rate limit updates per pass
   - Batch updates for efficiency

4. **Advanced Change Messages**
   - Localization support
   - Dynamic message templates
   - Rich formatting

5. **Admin Dashboard**
   - View registered devices
   - Monitor push notification success rate
   - Debug failed updates

---

## Resources & Documentation

### Official Apple Documentation
- [PassKit Web Service Reference](https://developer.apple.com/library/archive/documentation/PassKit/Reference/PassKit_WebService/WebService.html)
- [Wallet Developer Guide](https://developer.apple.com/library/archive/documentation/UserExperience/Conceptual/PassKit_PG/)
- [APNs Provider API](https://developer.apple.com/documentation/usernotifications/setting_up_a_remote_notification_server)

### Third-Party Resources
- [PassKit Updates and Notifications](https://www.passcreator.com/en/blog/apple-wallet-pass-updates-and-push-notifications-how-they-work-and-how-to-use-them)
- [Understanding Push Notifications for Wallet Passes](https://help.passkit.com/en/articles/11905171-understanding-push-notifications-for-apple-and-google-wallet-passes)
- [Apple Wallet Pass Auto Update Tutorial](https://reinteractive.com/articles/tutorial-series-for-experienced-rails-developers/apple-wallet-pass-auto-update)

### Rust Crates
- [a2 - APNS Client](https://crates.io/crates/a2)
- [passes - PassKit Pass Generation](https://crates.io/crates/passes)

---

## Questions to Consider Before Implementation

1. **Certificate Management**
   - Do you have the Pass Type ID certificate for APNS?
   - Is it the same certificate used to sign passes?
   - When does it expire?

2. **Testing Environment**
   - Will you test in APNS sandbox first?
   - Do you have a test iOS device?
   - How will you monitor APNS responses?

3. **Update Frequency**
   - Should all pet updates trigger pass updates?
   - Any rate limiting needed?
   - How to handle bulk updates?

4. **User Experience**
   - Which updates should show notifications?
   - Which should be silent?
   - What change messages to use?

5. **Error Handling**
   - How to handle failed APNS pushes?
   - Retry logic needed?
   - User notifications for failures?

---

## Next Steps

1. Review this plan and confirm approach
2. Answer questions above
3. Begin Phase 1 implementation (Database Schema)
4. Test each phase incrementally
5. Deploy to production with monitoring

---

**Document Version:** 1.0
**Last Updated:** 2025-11-27
**Status:** Ready for Implementation
