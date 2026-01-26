//! Example demonstrating how to use a custom key provider for SecureContent decryption
//!
//! This example shows how to implement the KeyProvider trait to provide your own
//! decryption keys for encrypted 3MF files.

use lib3mf::{AccessRight, CEKParams, KeyProvider, KEKParams, ParserConfig, Result, SecureContentInfo};
use std::fs::File;
use std::sync::Arc;

/// Example custom key provider implementation
/// 
/// In a real application, this would:
/// - Load private keys from secure storage (HSM, key vault, etc.)
/// - Implement RSA-OAEP key unwrapping with your keys
/// - Implement AES-256-GCM decryption
/// - Handle key rotation and multiple consumers
struct CustomKeyProvider {
    // In a real implementation, you would store your private keys here
    // For this example, we'll just use the test keys
}

impl KeyProvider for CustomKeyProvider {
    fn decrypt(
        &self,
        cipher_file_data: &[u8],
        cek_params: &CEKParams,
        access_right: &AccessRight,
        secure_content: &SecureContentInfo,
    ) -> Result<Vec<u8>> {
        // In a real implementation, you would:
        // 1. Parse the cipher file format
        // 2. Find the appropriate consumer (based on your consumer ID)
        // 3. Load your private key from secure storage
        // 4. Unwrap the CEK using RSA-OAEP with your private key
        // 5. Decrypt the content using AES-256-GCM
        // 6. Decompress if needed
        
        println!("Custom decryption called for file");
        println!("  Encryption algorithm: {}", cek_params.encryption_algorithm);
        println!("  Compression: {}", cek_params.compression);
        println!("  Consumer index: {}", access_right.consumer_index);
        
        if access_right.consumer_index < secure_content.consumers.len() {
            let consumer = &secure_content.consumers[access_right.consumer_index];
            println!("  Consumer ID: {}", consumer.consumer_id);
        }
        
        // For this example, we'll just delegate to the test keys
        // In a real implementation, you would use your own keys and crypto library
        lib3mf::decryption::decrypt_with_test_key(
            cipher_file_data,
            cek_params,
            access_right,
            secure_content,
        )
    }

    fn encrypt(
        &self,
        plaintext: &[u8],
        consumer_id: &str,
        compression: bool,
    ) -> Result<(Vec<u8>, CEKParams, KEKParams, String)> {
        // In a real implementation, you would:
        // 1. Optionally compress the plaintext
        // 2. Generate a random Content Encryption Key (CEK)
        // 3. Encrypt the plaintext using AES-256-GCM with the CEK
        // 4. Load the consumer's public key
        // 5. Wrap the CEK using RSA-OAEP with the consumer's public key
        // 6. Return the encrypted data and all parameters
        
        println!("Custom encryption called");
        println!("  Plaintext size: {} bytes", plaintext.len());
        println!("  Consumer ID: {}", consumer_id);
        println!("  Compression: {}", compression);
        
        // For now, this is not implemented in the example
        // You would use a crypto library like `ring`, `RustCrypto`, or `openssl`
        Err(lib3mf::Error::InvalidSecureContent(
            "Encryption not implemented in this example. Use a crypto library like ring or RustCrypto.".to_string()
        ))
    }
}

fn main() -> Result<()> {
    // Example 1: Using a custom key provider for decryption
    println!("=== Example 1: Custom Key Provider for Decryption ===\n");
    
    let test_file = "test_suites/suite8_secure/positive_test_cases/P_EPX_2102_01_materialExt.3mf";
    
    if !std::path::Path::new(test_file).exists() {
        println!("Test file not found: {}", test_file);
        println!("This example requires Suite 8 test files to be present.");
        println!("\nTo use custom keys in your application:");
        println!("1. Implement the KeyProvider trait with your own crypto library");
        println!("2. Create a ParserConfig with your key provider");
        println!("3. Parse the 3MF file with the custom config");
        return Ok(());
    }
    
    // Create the custom key provider
    let provider: Arc<dyn KeyProvider> = Arc::new(CustomKeyProvider {});
    
    // Configure the parser with the custom provider
    let config = ParserConfig::with_all_extensions()
        .with_key_provider(provider.clone());
    
    println!("Opening encrypted 3MF file: {}", test_file);
    let file = File::open(test_file)?;
    
    println!("Parsing with custom key provider...\n");
    let model = lib3mf::parser::parse_3mf_with_config(file, config)?;
    
    println!("Successfully parsed encrypted file!");
    println!("  Objects: {}", model.resources.objects.len());
    
    if let Some(ref sc) = model.secure_content {
        println!("  Consumers: {}", sc.consumers.len());
        println!("  Encrypted files: {}", sc.encrypted_files.len());
        
        for (i, consumer) in sc.consumers.iter().enumerate() {
            println!("\n  Consumer {}: {}", i, consumer.consumer_id);
            if let Some(ref key_id) = consumer.key_id {
                println!("    Key ID: {}", key_id);
            }
        }
    }
    
    // Example 2: Metadata extraction without decryption
    println!("\n=== Example 2: Metadata Extraction (No Decryption) ===\n");
    
    let file2 = File::open(test_file)?;
    
    // Parse without custom provider - will use test keys as fallback
    let config2 = ParserConfig::with_all_extensions();
    let model2 = lib3mf::parser::parse_3mf_with_config(file2, config2)?;
    
    if let Some(ref sc) = model2.secure_content {
        println!("SecureContent metadata:");
        
        if let Some(ref uuid) = sc.keystore_uuid {
            println!("  Keystore UUID: {}", uuid);
        }
        
        println!("\n  Resource Data Groups:");
        for (i, group) in sc.resource_data_groups.iter().enumerate() {
            println!("    Group {}: Key UUID {}", i, group.key_uuid);
            println!("      Access rights: {}", group.access_rights.len());
            println!("      Resources: {}", group.resource_data.len());
            
            for resource in &group.resource_data {
                println!("        File: {}", resource.path);
                println!("          Algorithm: {}", resource.cek_params.encryption_algorithm);
                println!("          Compression: {}", resource.cek_params.compression);
            }
        }
    }
    
    println!("\n=== Next Steps ===");
    println!("\nTo implement production-grade encryption/decryption:");
    println!("1. Use a well-audited crypto library:");
    println!("   - ring: https://crates.io/crates/ring (recommended)");
    println!("   - RustCrypto: https://github.com/RustCrypto");
    println!("   - openssl: https://crates.io/crates/openssl");
    println!("\n2. Implement secure key management:");
    println!("   - Use HSMs or key vaults for private key storage");
    println!("   - Implement proper key rotation");
    println!("   - Follow the principle of least privilege");
    println!("\n3. Follow the SecureContent specification:");
    println!("   - RSA-2048 OAEP for key wrapping");
    println!("   - AES-256-GCM for content encryption");
    println!("   - Proper IV and tag generation");
    println!("\nSee SECURE_CONTENT_SUPPORT.md for more details.");
    
    Ok(())
}
