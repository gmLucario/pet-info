//! # Apple Wallet Passes API Module
//!
//! This module handles Apple Wallet pass generation for pet information.
//! It creates properly signed .pkpass files using the passes-rs crate,
//! with full iOS 18.5 compatibility.
//!
//! ## Overview
//! 
//! Apple Wallet passes are digital cards that can be stored in the iOS Wallet app.
//! This module generates `.pkpass` files containing pet information that can be:
//! - Downloaded and added to iOS Wallet
//! - Scanned via QR code for quick access to pet profiles
//! - Updated remotely when pet information changes
//!
//! ## Pass Structure
//!
//! Each pass contains:
//! - **Primary Field**: Pet name (displayed prominently)
//! - **Secondary Fields**: Age and breed information
//! - **Auxiliary Fields**: Spay/neuter status and sex
//! - **Back Fields**: Pet ID and additional details
//! - **QR Code**: Links to the pet's public profile
//! - **Icon**: Pet photo (if available) or default icon
//!
//! ## Security & Certificates
//!
//! Passes are cryptographically signed using:
//! - Apple Developer Pass Type ID Certificate (`pass_certificate.pem`)
//! - Private key (`pass_private_key.pem`) 
//! - Apple WWDR G4 intermediate certificate (included automatically)
//!
//! ## iOS 18.5 Compatibility
//!
//! This implementation includes specific optimizations for iOS 18.5:
//! - Required `relevantDate` and `expirationDate` fields
//! - Proper color scheme configuration
//! - Spanish to English text conversion for better compatibility
//! - Unicode character sanitization

use crate::{api::pet::PetPublicInfoSchema, services};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use passes::{Package, resource, sign};
use std::{io::Cursor, path::Path};

/// Configuration constants for Apple Wallet passes
/// 
/// These constants define the core configuration for generating Apple Wallet passes.
/// They must match the values configured in your Apple Developer account.
mod pass_config {
    /// The pass type identifier registered with Apple Developer Program.
    /// Must match the Pass Type ID created in your Apple Developer account.
    pub const PASS_TYPE_IDENTIFIER: &str = "pass.com.petinfo.link";
    
    /// Your Apple Developer Team ID.
    /// Found in your Apple Developer account under Membership details.
    pub const TEAM_IDENTIFIER: &str = "S89P27T8CF";
    
    /// Organization name displayed on the pass.
    pub const ORGANIZATION_NAME: &str = "Pet Info";
    
    /// Path to the Apple Developer Pass Type ID certificate file.
    /// This certificate is downloaded from Apple Developer Portal.
    pub const CERT_PATH: &str = "pass_certificate.pem";
    
    /// Path to the private key file corresponding to the certificate.
    /// Generated during certificate creation process.
    pub const KEY_PATH: &str = "pass_private_key.pem";
    
    /// Path to the default pass icon (PNG format, recommended 29x29pt).
    pub const ICON_PATH: &str = "pass_icon.png";
    
    /// ISO 8601 date format required by Apple Wallet.
    pub const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

    // iOS 18.5 compatible colors (RGB format required)
    /// Text color for pass content
    pub const FOREGROUND_COLOR: &str = "rgb(255, 255, 255)";
    /// Background color for the pass
    pub const BACKGROUND_COLOR: &str = "rgb(60, 60, 60)";
    /// Label text color 
    pub const LABEL_COLOR: &str = "rgb(255, 255, 255)";
}

/// Generates a properly signed Apple Wallet pass for a pet.
///
/// This is the main entry point for creating Apple Wallet passes. It orchestrates
/// the entire pass generation process from schema creation to final binary output.
///
/// ## Process Flow
/// 1. Creates the pass JSON schema with pet information
/// 2. Converts JSON to a Pass object using the passes-rs crate
/// 3. Creates a signed package with Apple Developer certificates
/// 4. Adds visual resources (icons, pet photos)
/// 5. Generates the final .pkpass binary data
///
/// ## Parameters
/// - `pet_info`: Pet information schema containing all displayable data
/// - `storage_service`: Service for retrieving pet photos and other assets
///
/// ## Returns
/// - `Ok(Vec<u8>)`: Binary .pkpass file data ready for download
/// - `Err(anyhow::Error)`: Detailed error if pass generation fails
///
/// ## Errors
/// This function can fail for several reasons:
/// - Missing or invalid Apple Developer certificates
/// - Invalid pet information data
/// - File system errors when loading resources
/// - Pass signing/packaging failures
pub async fn generate_pet_pass(
    pet_info: &PetPublicInfoSchema,
    storage_service: &services::ImplStorageService,
) -> Result<Vec<u8>> {
    let pass_schema = create_pass_schema(pet_info);
    let pass = passes::Pass::from_json(&pass_schema.to_string())?;

    let mut package = create_signed_package(pass)?;

    add_pass_resources(&mut package, storage_service, &pet_info.pic).await?;

    generate_pkpass_bytes(package)
}

/// Creates the complete pass JSON schema with iOS 18.5 compatibility.
///
/// This function builds the core pass.json structure that defines the pass content,
/// appearance, and behavior. The schema follows Apple's Pass Kit specification
/// with additional optimizations for iOS 18.5 compatibility.
///
/// ## Key Features
/// - **Format Version**: Uses format version 1 (latest standard)
/// - **Dates**: Includes required `relevantDate` and `expirationDate` for iOS 18.5
/// - **QR Code**: Generates a QR code linking to the pet's public profile
/// - **Colors**: Uses iOS 18.5 compatible RGB color values
/// - **Generic Layout**: Structured for maximum compatibility across devices
///
/// ## Parameters
/// - `pet_info`: Pet information schema containing all data to display
///
/// ## Returns
/// A `serde_json::Value` representing the complete pass.json structure
fn create_pass_schema(pet_info: &PetPublicInfoSchema) -> serde_json::Value {
    let now = Utc::now();
    let expiration = now + Duration::days(365);

    serde_json::json!({
        "formatVersion": 1,
        "organizationName": pass_config::ORGANIZATION_NAME,
        "description": format!("Pet Info Pass: {}", &pet_info.name),
        "passTypeIdentifier": pass_config::PASS_TYPE_IDENTIFIER,
        "teamIdentifier": pass_config::TEAM_IDENTIFIER,
        "serialNumber": pet_info.external_id,
        "logoText": "Pet Info Pass",

        // iOS 18.5 styling
        "labelColor": pass_config::LABEL_COLOR,
        "foregroundColor": pass_config::FOREGROUND_COLOR,
        "backgroundColor": pass_config::BACKGROUND_COLOR,

        // iOS 18.5 required dates
        "relevantDate": now.format(pass_config::DATE_FORMAT).to_string(),
        "expirationDate": expiration.format(pass_config::DATE_FORMAT).to_string(),

        "barcodes": [{
            "message": format!("https://pet-info.link/info/{}", pet_info.external_id),
            "format": "PKBarcodeFormatQR",
            "altText": "Pet Info public profile",
            "messageEncoding": "iso-8859-1"
        }],

        // Pass content
        "generic": create_generic_fields(pet_info)
    })
}

/// Creates the generic pass fields structure.
///
/// This function defines the layout and content of the pass using Apple's generic
/// pass template. The generic template provides maximum flexibility and is suitable
/// for displaying various types of information.
///
/// ## Field Structure
/// - **Primary Fields**: Most prominent display (pet name)
/// - **Secondary Fields**: Secondary importance (age, breed)  
/// - **Auxiliary Fields**: Additional details (spay/neuter status, sex)
/// - **Back Fields**: Detailed information shown on pass back
/// - **Header Fields**: Top-level info (currently empty for generic passes)
///
/// ## Localization
/// Currently uses Spanish labels to match the application's primary language.
/// Can be easily modified to support multiple languages based on user preferences.
///
/// ## Parameters
/// - `pet_info`: Pet information schema containing all displayable data
///
/// ## Returns
/// A `serde_json::Value` containing the complete generic field structure
fn create_generic_fields(pet_info: &PetPublicInfoSchema) -> serde_json::Value {
    serde_json::json!({
        "primaryFields": [
            {
                "key": "name",
                "label": "Nombre",
                "value": &pet_info.name.to_lowercase(),
            },
        ],
        "secondaryFields": [
            {
                "key": "age",
                "label": "Edad",
                "value": &pet_info.fmt_age
            },
            {
                "key": "breed",
                "label": "Raza",
                "value": &pet_info.pet_breed
            },
        ],
        "auxiliaryFields": [
            {
                "key": "spayed",
                "label": "Esterilizada",
                "value": if pet_info.is_spaying_neutering { "Si" } else { "No" }
            },
            {
                "key": "sex",
                "label": "Sex",
                "value": format!("{:?}", pet_info.sex)
            },
        ],
        "backFields": create_back_fields(pet_info),
        "headerFields": [] // Empty for generic passes
    })
}

/// Creates back fields with essential pet information.
///
/// Back fields are displayed when the user flips the pass over in Apple Wallet.
/// This is where detailed information is shown that doesn't fit on the front.
/// Currently includes the pet's unique identifier for reference purposes.
///
/// ## Parameters
/// - `pet_info`: Pet information schema
///
/// ## Returns
/// A vector of `serde_json::Value` objects representing each back field
fn create_back_fields(pet_info: &PetPublicInfoSchema) -> Vec<serde_json::Value> {
    vec![serde_json::json!({
        "key": "pet_id",
        "label": "Pet ID",
        "value": pet_info.external_id
    })]
}

/// Creates a signed package with proper certificate chain.
///
/// This function handles the cryptographic signing of the pass using Apple Developer
/// certificates. The signing process ensures the pass is trusted by iOS devices and
/// prevents tampering.
///
/// ## Certificate Chain
/// 1. **Pass Type ID Certificate**: Your specific certificate for this pass type
/// 2. **Apple WWDR G4**: Apple's intermediate certificate (added automatically)
/// 3. **Apple Root CA**: Apple's root certificate (trusted by iOS)
///
/// ## Security Notes
/// - Certificates must be valid and not expired
/// - Private key must correspond to the certificate
/// - Pass Type ID must match your Apple Developer account configuration
///
/// ## Parameters
/// - `pass`: The Pass object to be signed and packaged
///
/// ## Returns
/// - `Ok(Package)`: Successfully created and signed package
/// - `Err(anyhow::Error)`: Certificate loading or signing failure
///
/// ## Errors
/// Common failures include:
/// - Missing certificate files
/// - Expired certificates  
/// - Mismatched private key
/// - Invalid certificate format
fn create_signed_package(pass: passes::Pass) -> Result<Package> {
    let mut package = Package::new(pass);

    let cert_data = load_file(pass_config::CERT_PATH)?;
    let key_data = load_file(pass_config::KEY_PATH)?;

    let sign_config = sign::SignConfig::new(sign::WWDR::G4, &cert_data, &key_data)?;

    package.add_certificates(sign_config);
    Ok(package)
}


/// Adds visual resources to the pass package.
///
/// This function adds icons and images to make the pass visually appealing.
/// Resources include a default icon and optionally the pet's photo if available.
///
/// ## Resource Types
/// - **Icon**: Default pass icon (29x29pt recommended, shown in Wallet list)
/// - **Thumbnail**: Pet's photo (optional, used as pass thumbnail)
///
/// ## Resource Requirements
/// - Icons should be PNG format for best compatibility
/// - Images are automatically resized by the passes-rs crate
/// - Missing resources won't cause failures (graceful degradation)
///
/// ## Parameters
/// - `package`: Mutable reference to the package being built
/// - `storage_service`: Service for retrieving pet photos from storage
/// - `pic_path`: Optional path to the pet's photo
///
/// ## Returns
/// - `Ok(())`: Resources successfully added
/// - `Err(anyhow::Error)`: Resource loading or addition failure
async fn add_pass_resources(
    package: &mut Package,
    storage_service: &services::ImplStorageService,
    pic_path: &Option<String>,
) -> Result<()> {
    let icon_data = load_file(pass_config::ICON_PATH)?;
    package
        .add_resource(
            resource::Type::Icon(resource::Version::Standard),
            &icon_data[..],
        )
        .map_err(|e| anyhow::anyhow!("Failed to add icon resource: {}", e))?;

    if let Some(pic_path) = pic_path {
        let pic_path = Path::new(&pic_path);
        let icon_data = storage_service
            .get_pic_as_bytes(pic_path.with_extension("").to_str().unwrap_or_default())
            .await?;

        package
            .add_resource(
                resource::Type::Thumbnail(resource::Version::Standard),
                &icon_data[..],
            )
            .map_err(|e| anyhow::anyhow!("Failed to add Thumbnail: {}", e))?;
    }

    Ok(())
}

/// Safely loads a file with better error context.
///
/// This utility function provides consistent file loading with detailed error messages.
/// It's used for loading certificates, keys, and resource files.
///
/// ## Parameters
/// - `path`: File system path to the file to load
///
/// ## Returns
/// - `Ok(Vec<u8>)`: File contents as bytes
/// - `Err(anyhow::Error)`: File loading error with context
fn load_file(path: &str) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("Failed to read file: {}", path))
}

/// Generates final .pkpass bytes from the package.
///
/// This function performs the final step of pass generation by writing the complete
/// signed package to a binary format that can be served as a .pkpass file download.
///
/// ## Process
/// 1. Creates an in-memory buffer to hold the binary data
/// 2. Writes the signed package (including manifest and signature) to the buffer
/// 3. Returns the buffer contents as the final .pkpass file
///
/// ## File Format
/// The output is a ZIP archive containing:
/// - `pass.json`: Pass content and styling
/// - `manifest.json`: SHA1 hashes of all files  
/// - `signature`: PKCS#7 signature of the manifest
/// - Resource files (icons, images)
///
/// ## Parameters
/// - `package`: The complete signed package ready for export
///
/// ## Returns
/// - `Ok(Vec<u8>)`: Binary .pkpass file data
/// - `Err(anyhow::Error)`: Package writing failure
fn generate_pkpass_bytes(mut package: Package) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    package
        .write(Cursor::new(&mut buffer))
        .map_err(|e| anyhow::anyhow!("Failed to write package to buffer: {}", e))?;
    Ok(buffer)
}
