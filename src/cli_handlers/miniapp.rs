// Mini-App CLI Command Handlers
//
// Dedicated handler functions for miniapp subcommands.
// Provides mini-app lifecycle management: launch, register, validate.
// Extracted from main.rs for better maintainability.

use dchat_core::error::{Error, Result};
use dchat_miniapps::{
    manifest::AppManifest,
    registry::{AppId, Developer, DeveloperId},
    sandbox::{SandboxConfig, SandboxId, SandboxMessage},
};
use std::path::PathBuf;

/// Handle `dchat miniapp launch` command
pub async fn handle_launch(
    manifest: Option<PathBuf>,
    app_id: Option<String>,
    user_id: Option<String>,
    developer_key: Option<String>,
    theme: String,
    width: u32,
    height: u32,
    debug: bool,
) -> Result<()> {
    dchat_log::out!("\n🚀 DCHAT MINI-APP LAUNCHER");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");

    // Load manifest from file or fetch from registry
    let app_manifest = if let Some(manifest_path) = manifest {
        let manifest_file = if manifest_path.is_dir() {
            manifest_path.join("manifest.json")
        } else {
            manifest_path.clone()
        };

        if !manifest_file.exists() {
            return Err(Error::validation(format!(
                "Manifest file not found: {:?}",
                manifest_file
            )));
        }

        dchat_log::out!("📦 Loading manifest from: {:?}", manifest_file);
        let manifest_json = std::fs::read_to_string(&manifest_file)
            .map_err(|e| Error::storage(format!("Failed to read manifest: {}", e)))?;

        serde_json::from_str::<AppManifest>(&manifest_json)
            .map_err(|e| Error::validation(format!("Invalid manifest JSON: {}", e)))?
    } else if let Some(app_id_hex) = app_id {
        dchat_log::out!("📦 Fetching app {} from registry...", app_id_hex);
        return Err(Error::validation(
            "Registry lookup not yet implemented. Use --manifest to launch local apps.".to_string(),
        ));
    } else {
        return Err(Error::validation(
            "Either --manifest or --app-id must be provided".to_string(),
        ));
    };

    // Validate manifest
    dchat_log::out!("🔍 Validating manifest...");
    app_manifest
        .validate()
        .map_err(|e| Error::validation(format!("Manifest validation failed: {:?}", e)))?;
    dchat_log::out!("   ✅ Manifest valid");

    // Display app info
    dchat_log::out!();
    dchat_log::out!("📱 App Information:");
    dchat_log::out!("   Name:        {}", app_manifest.metadata.name);
    dchat_log::out!("   Version:     {}", app_manifest.version);
    dchat_log::out!(
        "   Description: {}",
        app_manifest.metadata.short_description
    );
    dchat_log::out!("   Category:    {:?}", app_manifest.metadata.category);
    dchat_log::out!("   Entry Point: {}", app_manifest.resources.entry_point);

    // Display requested permissions
    if !app_manifest.permissions.is_empty() {
        dchat_log::out!();
        dchat_log::out!("🔐 Requested Permissions:");
        for perm in app_manifest.permissions.iter() {
            dchat_log::out!("   • {:?}", perm);
        }
    }

    // Create sandbox configuration
    let mut sandbox_config = SandboxConfig::from_manifest(&app_manifest);
    sandbox_config.debug_mode = debug;

    dchat_log::out!();
    dchat_log::out!("🔧 Sandbox Configuration:");
    dchat_log::out!(
        "   Max Memory:  {} MB",
        sandbox_config.max_memory / 1024 / 1024
    );
    dchat_log::out!("   CPU Timeout: {} ms", sandbox_config.max_cpu_time_ms);
    dchat_log::out!("   Network:     {}", sandbox_config.allow_network);
    dchat_log::out!("   Debug Mode:  {}", sandbox_config.debug_mode);

    // Generate session context
    let sandbox_id = SandboxId::new();
    let session_user_id = if let Some(uid) = user_id {
        let bytes =
            hex::decode(&uid).map_err(|_| Error::validation("Invalid user_id hex".to_string()))?;
        if bytes.len() != 32 {
            return Err(Error::validation("user_id must be 32 bytes".to_string()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        arr
    } else {
        // Generate temporary user ID
        let mut arr = [0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut arr);
        arr
    };

    // Derive developer ID from public key
    let developer_id = if let Some(ref key_hex) = developer_key {
        let key_bytes = hex::decode(key_hex)
            .map_err(|e| Error::validation(format!("Invalid developer key hex: {}", e)))?;
        if key_bytes.len() != 32 {
            return Err(Error::validation(
                "Developer key must be 32 bytes".to_string(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&key_bytes);
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&arr)
            .map_err(|e| Error::validation(format!("Invalid developer public key: {}", e)))?;
        DeveloperId::from_public_key(&verifying_key)
    } else {
        // For local development without a key, derive deterministic ID from app name
        // WARNING: This should only be used for local testing
        dchat_log::out!(
            "⚠️  No --developer-key provided. Using deterministic dev ID for local testing only."
        );
        let dev_hash = blake3::hash(format!("dev::{}", app_manifest.metadata.name).as_bytes());
        DeveloperId::from_bytes(*dev_hash.as_bytes())
    };
    let derived_app_id = AppId::derive(&developer_id, &app_manifest.metadata.name);

    dchat_log::out!();
    dchat_log::out!("🆔 Session Details:");
    dchat_log::out!("   Sandbox ID:  {}", sandbox_id);
    dchat_log::out!("   App ID:      {}", derived_app_id);
    dchat_log::out!(
        "   User ID:     0x{}...",
        hex::encode(&session_user_id[..8])
    );
    dchat_log::out!("   Viewport:    {}x{}", width, height);
    dchat_log::out!("   Theme:       {}", theme);

    // Initialize sandbox messages
    let init_message = SandboxMessage::Init {
        app_id: derived_app_id.to_string(),
        config: serde_json::json!({
            "theme": theme,
            "viewport": { "width": width, "height": height },
            "debug": debug,
        }),
    };

    dchat_log::out!();
    dchat_log::out!("══════════════════════════════════════════════════════════════════");
    dchat_log::out!("✅ MINI-APP READY TO LAUNCH");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");
    dchat_log::out!();
    dchat_log::out!("Entry point: {}", app_manifest.resources.entry_point);
    dchat_log::out!(
        "Init message: {}",
        init_message.to_json().unwrap_or_default()
    );
    dchat_log::out!();
    dchat_log::out!("💡 In a full client, this would open a WebView/iframe sandbox.");
    dchat_log::out!("   Use the dchat-miniapps crate to integrate into your application.");

    Ok(())
}

/// Handle `dchat miniapp register-developer` command
pub async fn handle_register_developer(
    name: String,
    keypair: PathBuf,
    website: Option<String>,
    email: Option<String>,
) -> Result<()> {
    dchat_log::out!("\n👤 REGISTER MINI-APP DEVELOPER");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");

    if !keypair.exists() {
        return Err(Error::validation(format!(
            "Keypair file not found: {:?}",
            keypair
        )));
    }

    // Load keypair
    let keypair_json = std::fs::read_to_string(&keypair)
        .map_err(|e| Error::storage(format!("Failed to read keypair: {}", e)))?;
    let keypair_data: serde_json::Value = serde_json::from_str(&keypair_json)
        .map_err(|e| Error::validation(format!("Invalid keypair JSON: {}", e)))?;

    let pubkey_hex = keypair_data
        .get("public_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::validation("Missing public_key in keypair file"))?;

    let pubkey_bytes = hex::decode(pubkey_hex)
        .map_err(|_| Error::validation("Invalid public key hex".to_string()))?;

    if pubkey_bytes.len() != 32 {
        return Err(Error::validation("Public key must be 32 bytes".to_string()));
    }

    let mut pubkey_arr = [0u8; 32];
    pubkey_arr.copy_from_slice(&pubkey_bytes);

    // Create developer registration
    let mut developer = Developer::new(name.clone(), pubkey_arr);
    developer.website = website;
    developer.email = email;

    dchat_log::out!("📝 Developer Registration:");
    dchat_log::out!("   Name:      {}", developer.name);
    dchat_log::out!("   ID:        {}", developer.id);
    dchat_log::out!("   Public Key: 0x{}...", &pubkey_hex[..16]);
    if let Some(ref w) = developer.website {
        dchat_log::out!("   Website:   {}", w);
    }
    if let Some(ref e) = developer.email {
        dchat_log::out!("   Email:     {}", e);
    }
    dchat_log::out!("   Status:    {:?}", developer.status);
    dchat_log::out!();
    dchat_log::out!("💡 In production, this would submit a registration transaction");
    dchat_log::out!("   to the chat chain for verification.");

    Ok(())
}

/// Handle `dchat miniapp register` command
pub async fn handle_register(
    manifest: PathBuf,
    keypair: PathBuf,
    bundle: Option<PathBuf>,
) -> Result<()> {
    dchat_log::out!("\n📦 REGISTER MINI-APP");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");

    if !manifest.exists() {
        return Err(Error::validation(format!(
            "Manifest file not found: {:?}",
            manifest
        )));
    }

    if !keypair.exists() {
        return Err(Error::validation(format!(
            "Keypair file not found: {:?}",
            keypair
        )));
    }

    // Load and validate manifest
    let manifest_json = std::fs::read_to_string(&manifest)
        .map_err(|e| Error::storage(format!("Failed to read manifest: {}", e)))?;
    let app_manifest: AppManifest = serde_json::from_str(&manifest_json)
        .map_err(|e| Error::validation(format!("Invalid manifest: {}", e)))?;

    app_manifest
        .validate()
        .map_err(|e| Error::validation(format!("Manifest validation failed: {:?}", e)))?;

    dchat_log::out!("📱 App: {}", app_manifest.metadata.name);
    dchat_log::out!("   Version: {}", app_manifest.version);

    if let Some(bundle_path) = bundle {
        if !bundle_path.exists() {
            return Err(Error::validation(format!(
                "Bundle not found: {:?}",
                bundle_path
            )));
        }
        let bundle_size = std::fs::metadata(&bundle_path)
            .map(|m| m.len())
            .unwrap_or(0);
        dchat_log::out!("   Bundle: {:?} ({} KB)", bundle_path, bundle_size / 1024);
    }

    dchat_log::out!();
    dchat_log::out!("💡 In production, this would:");
    dchat_log::out!("   1. Upload bundle to IPFS/decentralized storage");
    dchat_log::out!("   2. Submit registration transaction to chat chain");
    dchat_log::out!("   3. Await verification from network");

    Ok(())
}

/// Handle `dchat miniapp info` command
pub async fn handle_info(app_id: String) -> Result<()> {
    dchat_log::out!("\n📱 MINI-APP INFORMATION");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");
    dchat_log::out!("App ID: {}", app_id);
    dchat_log::out!();
    dchat_log::out!("💡 In production, this would query the on-chain registry");
    dchat_log::out!("   for app metadata, developer info, and download stats.");

    Ok(())
}

/// Handle `dchat miniapp list` command
pub async fn handle_list(category: Option<String>, installed: bool) -> Result<()> {
    dchat_log::out!("\n📋 MINI-APP LIST");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");

    if let Some(cat) = category {
        dchat_log::out!("Filter: category = {}", cat);
    }
    if installed {
        dchat_log::out!("Filter: installed only");
    }

    dchat_log::out!();
    dchat_log::out!("💡 In production, this would query the on-chain registry");
    dchat_log::out!("   and list available/installed mini-apps.");

    Ok(())
}

/// Handle `dchat miniapp init` command
pub async fn handle_init(name: String, path: PathBuf, category: String) -> Result<()> {
    dchat_log::out!("\n🆕 CREATE NEW MINI-APP PROJECT");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");

    let project_dir = path.join(&name);
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| Error::storage(format!("Failed to create directory: {}", e)))?;

    // Create manifest.json
    let manifest = serde_json::json!({
        "manifest_version": { "major": 1, "minor": 0 },
        "version": "1.0.0",
        "metadata": {
            "name": name,
            "short_description": format!("A {} mini-app", category),
            "description": format!("A {} mini-app built for dchat", category),
            "icon": "icon.png",
            "category": category,
            "tags": [category],
            "languages": ["en"],
            "age_rating": "everyone"
        },
        "permissions": [],
        "resources": {
            "entry_point": "index.html",
            "allowed_domains": [],
            "preload": ["app.js", "style.css"]
        },
        "runtime": {
            "max_memory_mb": 128,
            "max_cpu_time_ms": 5000
        }
    });

    let manifest_path = project_dir.join("manifest.json");
    let manifest_str = serde_json::to_string_pretty(&manifest)
        .map_err(|e| Error::internal(format!("Failed to serialize manifest: {}", e)))?;
    std::fs::write(&manifest_path, manifest_str)
        .map_err(|e| Error::storage(format!("Failed to write manifest: {}", e)))?;

    // Create index.html
    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div id="app">
        <h1>Welcome to {}</h1>
        <p>Your dchat mini-app is ready!</p>
        <button id="main-btn">Click Me</button>
    </div>
    <script src="app.js"></script>
</body>
</html>
"#,
        name, name
    );
    std::fs::write(project_dir.join("index.html"), index_html)
        .map_err(|e| Error::storage(format!("Failed to write index.html: {}", e)))?;

    // Create app.js
    let app_js = r#"// dchat Mini-App JavaScript
console.log('Mini-app loaded!');

// Listen for messages from the dchat sandbox
window.addEventListener('message', (event) => {
    const message = event.data;
    console.log('Received message:', message);
    
    switch (message.type) {
        case 'init':
            console.log('App initialized with config:', message.data.config);
            break;
        case 'theme_change':
            document.body.className = message.data.theme;
            break;
    }
});

// Send ready message to sandbox
window.parent.postMessage({ type: 'ready' }, '*');

// Main button click handler
document.getElementById('main-btn')?.addEventListener('click', () => {
    window.parent.postMessage({
        type: 'permission_request',
        data: { permissions: ['view_balance'] }
    }, '*');
});
"#;
    std::fs::write(project_dir.join("app.js"), app_js)
        .map_err(|e| Error::storage(format!("Failed to write app.js: {}", e)))?;

    // Create style.css
    let style_css = r#"/* dchat Mini-App Styles */
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
    color: white;
}

body.dark {
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
}

#app {
    text-align: center;
    padding: 2rem;
}

h1 {
    font-size: 2rem;
    margin-bottom: 1rem;
}

p {
    font-size: 1.1rem;
    opacity: 0.9;
    margin-bottom: 2rem;
}

button {
    background: white;
    color: #667eea;
    border: none;
    padding: 1rem 2rem;
    font-size: 1rem;
    border-radius: 8px;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
}

button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
}
"#;
    std::fs::write(project_dir.join("style.css"), style_css)
        .map_err(|e| Error::storage(format!("Failed to write style.css: {}", e)))?;

    dchat_log::out!("✅ Created mini-app project: {:?}", project_dir);
    dchat_log::out!();
    dchat_log::out!("📁 Project structure:");
    dchat_log::out!("   {:?}/", project_dir);
    dchat_log::out!("   ├── manifest.json");
    dchat_log::out!("   ├── index.html");
    dchat_log::out!("   ├── app.js");
    dchat_log::out!("   └── style.css");
    dchat_log::out!();
    dchat_log::out!("💡 Next steps:");
    dchat_log::out!("   1. Edit the files to build your app");
    dchat_log::out!(
        "   2. Validate: dchat miniapp validate --manifest {:?}",
        manifest_path
    );
    dchat_log::out!(
        "   3. Launch:   dchat miniapp launch --manifest {:?}",
        project_dir
    );

    Ok(())
}

/// Handle `dchat miniapp validate` command
pub async fn handle_validate(manifest: PathBuf, verbose: bool) -> Result<()> {
    dchat_log::out!("\n🔍 VALIDATE MINI-APP MANIFEST");
    dchat_log::out!("══════════════════════════════════════════════════════════════════");

    if !manifest.exists() {
        return Err(Error::validation(format!(
            "Manifest file not found: {:?}",
            manifest
        )));
    }

    let manifest_json = std::fs::read_to_string(&manifest)
        .map_err(|e| Error::storage(format!("Failed to read manifest: {}", e)))?;

    dchat_log::out!("📄 File: {:?}", manifest);
    dchat_log::out!("   Size: {} bytes", manifest_json.len());
    dchat_log::out!();

    // Parse JSON
    let app_manifest: AppManifest = match serde_json::from_str(&manifest_json) {
        Ok(m) => m,
        Err(e) => {
            dchat_log::out!("❌ JSON PARSE ERROR");
            dchat_log::out!("   {}", e);
            return Err(Error::validation(format!("JSON parse error: {}", e)));
        }
    };

    // Validate manifest
    match app_manifest.validate() {
        Ok(()) => {
            dchat_log::out!("✅ MANIFEST VALID");
            dchat_log::out!();
            dchat_log::out!("📱 App: {}", app_manifest.metadata.name);
            dchat_log::out!("   Version:     {}", app_manifest.version);
            dchat_log::out!("   Category:    {:?}", app_manifest.metadata.category);
            dchat_log::out!("   Entry Point: {}", app_manifest.resources.entry_point);
            dchat_log::out!(
                "   Permissions: {} requested",
                app_manifest.permissions.len()
            );

            if verbose {
                dchat_log::out!();
                dchat_log::out!("📋 Full Manifest:");
                dchat_log::out!(
                    "{}",
                    serde_json::to_string_pretty(&app_manifest).unwrap_or_default()
                );
            }
        }
        Err(e) => {
            dchat_log::out!("❌ VALIDATION FAILED");
            dchat_log::out!("   {:?}", e);
            return Err(Error::validation(format!("Validation failed: {:?}", e)));
        }
    }

    Ok(())
}
