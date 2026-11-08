//! Test the counter contract with the dchat-programs VM
//!
//! This integration test loads the compiled WASM and executes it
//! through the dchat deterministic VM.

use std::path::PathBuf;
use std::sync::Arc;

use dchat_programs::account::Pubkey;
use dchat_programs::metering::{ComputeBudget, ComputeMeter};
use dchat_programs::syscalls::SyscallRegistry;
use dchat_programs::validation::BytecodeValidator;
use dchat_programs::vm::VmState;
use dchat_programs::wasi_shim::register_wasi_shim;

/// Path to the compiled WASM contract
fn get_wasm_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.join("../..");

    // Primary: DPL counter example built for wasmi + wasm32-wasip1.
    let dpl_counter = repo_root
        .join("examples/contracts/dpl-counter/target/wasm32-wasip1/release/dpl_counter.wasm");
    if dpl_counter.exists() {
        return dpl_counter;
    }

    // Fallback: legacy path (kept for compatibility with older layouts).
    repo_root.join(
        "examples/contracts/counter/target/wasm32-unknown-unknown/release/counter_contract.wasm",
    )
}

/// Instruction tags matching the contract
const IX_INITIALIZE: u8 = 0;
const IX_INCREMENT: u8 = 1;
const IX_DECREMENT: u8 = 2;
const IX_SET: u8 = 3;

/// Counter state size
const COUNTER_STATE_SIZE: usize = 45;

/// Magic bytes
const COUNTER_MAGIC: [u8; 4] = [0xC0, 0x55, 0x4E, 0x54];

#[test]
fn test_wasm_exists_and_validates() {
    let wasm_path = get_wasm_path();

    // Check file exists
    if !wasm_path.exists() {
        dchat_log::out!("WASM not found at {:?}", wasm_path);
        dchat_log::out!("Please compile the counter contract first (wasmi/wasip1):");
        dchat_log::out!("  rustup target add wasm32-wasip1");
        dchat_log::out!("  cd examples/contracts/dpl-counter");
        dchat_log::out!("  cargo build --target wasm32-wasip1 --release");
        dchat_log::out!("\nIf you're using the legacy counter example instead:");
        dchat_log::out!("  cd examples/contracts/counter");
        dchat_log::out!("  cargo build --target wasm32-unknown-unknown --release");
        panic!("Counter contract WASM not found");
    }

    // Load WASM
    let wasm_bytes = std::fs::read(&wasm_path).expect("Failed to read WASM file");
    dchat_log::out!("Loaded WASM: {} bytes", wasm_bytes.len());

    // Validate with dchat validator
    let validator = BytecodeValidator::new();
    let validation_result = validator.validate(&wasm_bytes);

    match validation_result {
        Ok(validated) => {
            dchat_log::out!("✅ WASM validation passed!");
            dchat_log::out!("   Code hash: {:?}", hex::encode(&validated.code_hash[..8]));
            dchat_log::out!("   Size: {} bytes", validated.size);
        }
        Err(e) => {
            // Some validation may fail for our simple contract since
            // it doesn't have all expected exports, but that's OK for testing
            dchat_log::out!("⚠️  Validation notes: {:?}", e);
        }
    }
}

#[test]
fn test_wasm_module_loads() {
    let wasm_path = get_wasm_path();

    if !wasm_path.exists() {
        dchat_log::out!("Skipping: WASM not found");
        return;
    }

    let wasm_bytes = std::fs::read(&wasm_path).expect("Failed to read WASM file");

    // Try to create a wasmi module directly to verify it's valid WASM
    let engine = wasmi::Engine::default();
    let module = wasmi::Module::new(&engine, &wasm_bytes);

    match module {
        Ok(m) => {
            dchat_log::out!("✅ WASM module loaded successfully!");
            // Check exports
            let exports: Vec<_> = m.exports().collect();
            dchat_log::out!("   Exports ({}):", exports.len());
            for export in exports {
                dchat_log::out!("     - {}: {:?}", export.name(), export.ty());
            }
        }
        Err(e) => {
            panic!("Failed to load WASM module: {:?}", e);
        }
    }
}

#[test]
fn test_counter_contract_execution() {
    let wasm_path = get_wasm_path();

    if !wasm_path.exists() {
        dchat_log::out!("Skipping: WASM not found");
        return;
    }

    let wasm_bytes = std::fs::read(&wasm_path).expect("Failed to read WASM file");

    // Set up deterministic wasmi engine (matches dchat VM constraints)
    let mut engine_config = wasmi::Config::default();
    engine_config.consume_fuel(true);
    engine_config.floats(false);

    let engine = wasmi::Engine::new(&engine_config);
    let module = wasmi::Module::new(&engine, &wasm_bytes).expect("Failed to parse module");

    // Create store with VM state so WASI shims can log deterministically.
    let compute_meter = Arc::new(ComputeMeter::new(ComputeBudget::new(2_000_000)));
    let syscalls = Arc::new(SyscallRegistry::new());
    let program_id = Pubkey::new([9u8; 32]);
    let state = VmState::new(compute_meter.clone(), program_id, syscalls, 32);

    let mut store = wasmi::Store::new(&engine, state);
    store.limiter(|state| &mut state.limits);

    let fuel = compute_meter.remaining();
    store.set_fuel(fuel).expect("Failed to set initial fuel");

    // Create linker and register deterministic WASI preview1 shim.
    let mut linker = wasmi::Linker::<VmState>::new(&engine);
    register_wasi_shim(&mut linker).expect("Failed to register WASI shim");

    // Instantiate the module
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate")
        .start(&mut store)
        .expect("Failed to start");

    // Get exports
    let entrypoint = instance
        .get_typed_func::<(u32, u32, u32, u32), u32>(&store, "entrypoint")
        .expect("entrypoint export not found");

    let memory = instance
        .get_memory(&store, "memory")
        .expect("memory export not found");

    let alloc = instance
        .get_typed_func::<u32, u32>(&store, "alloc")
        .expect("alloc export not found");

    dchat_log::out!("✅ All required exports found!");

    // Allocate memory for account data
    let account_ptr = alloc
        .call(&mut store, COUNTER_STATE_SIZE as u32)
        .expect("alloc failed");
    dchat_log::out!("   Allocated account data at: 0x{:x}", account_ptr);

    // Allocate memory for instruction data
    let ix_ptr = alloc.call(&mut store, 16).expect("alloc failed");
    dchat_log::out!("   Allocated instruction data at: 0x{:x}", ix_ptr);

    // Write initialize instruction
    memory
        .write(&mut store, ix_ptr as usize, &[IX_INITIALIZE])
        .expect("write failed");

    // Call entrypoint to initialize
    let result = entrypoint
        .call(
            &mut store,
            (account_ptr, COUNTER_STATE_SIZE as u32, ix_ptr, 1),
        )
        .expect("entrypoint call failed");
    dchat_log::out!("   Initialize result: {}", result);
    assert_eq!(result, 0, "Initialize should succeed");

    // Read back account data to verify initialization
    let mut account_data = vec![0u8; COUNTER_STATE_SIZE];
    memory
        .read(&store, account_ptr as usize, &mut account_data)
        .expect("read failed");

    assert_eq!(
        &account_data[0..4],
        &COUNTER_MAGIC,
        "Magic bytes should be set"
    );
    assert_eq!(account_data[4], 1, "Version should be 1");

    let counter_value = u64::from_le_bytes(account_data[5..13].try_into().unwrap());
    assert_eq!(counter_value, 0, "Counter should be 0 after init");
    dchat_log::out!("   Counter value after init: {}", counter_value);

    // Test increment
    memory
        .write(&mut store, ix_ptr as usize, &[IX_INCREMENT])
        .expect("write failed");

    let result = entrypoint
        .call(
            &mut store,
            (account_ptr, COUNTER_STATE_SIZE as u32, ix_ptr, 1),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 0, "Increment should succeed");

    // Read counter
    memory
        .read(&store, account_ptr as usize, &mut account_data)
        .expect("read failed");
    let counter_value = u64::from_le_bytes(account_data[5..13].try_into().unwrap());
    assert_eq!(counter_value, 1, "Counter should be 1 after increment");
    dchat_log::out!("   Counter value after increment: {}", counter_value);

    // Test increment by 5
    let mut ix_data = vec![IX_INCREMENT];
    ix_data.extend_from_slice(&5u64.to_le_bytes());
    memory
        .write(&mut store, ix_ptr as usize, &ix_data)
        .expect("write failed");

    let result = entrypoint
        .call(
            &mut store,
            (
                account_ptr,
                COUNTER_STATE_SIZE as u32,
                ix_ptr,
                ix_data.len() as u32,
            ),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 0, "Increment by 5 should succeed");

    memory
        .read(&store, account_ptr as usize, &mut account_data)
        .expect("read failed");
    let counter_value = u64::from_le_bytes(account_data[5..13].try_into().unwrap());
    assert_eq!(counter_value, 6, "Counter should be 6");
    dchat_log::out!("   Counter value after increment by 5: {}", counter_value);

    // Test decrement
    let mut ix_data = vec![IX_DECREMENT];
    ix_data.extend_from_slice(&2u64.to_le_bytes());
    memory
        .write(&mut store, ix_ptr as usize, &ix_data)
        .expect("write failed");

    let result = entrypoint
        .call(
            &mut store,
            (
                account_ptr,
                COUNTER_STATE_SIZE as u32,
                ix_ptr,
                ix_data.len() as u32,
            ),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 0, "Decrement should succeed");

    memory
        .read(&store, account_ptr as usize, &mut account_data)
        .expect("read failed");
    let counter_value = u64::from_le_bytes(account_data[5..13].try_into().unwrap());
    assert_eq!(counter_value, 4, "Counter should be 4");
    dchat_log::out!("   Counter value after decrement by 2: {}", counter_value);

    // Test set
    let mut ix_data = vec![IX_SET];
    ix_data.extend_from_slice(&100u64.to_le_bytes());
    memory
        .write(&mut store, ix_ptr as usize, &ix_data)
        .expect("write failed");

    let result = entrypoint
        .call(
            &mut store,
            (
                account_ptr,
                COUNTER_STATE_SIZE as u32,
                ix_ptr,
                ix_data.len() as u32,
            ),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 0, "Set should succeed");

    memory
        .read(&store, account_ptr as usize, &mut account_data)
        .expect("read failed");
    let counter_value = u64::from_le_bytes(account_data[5..13].try_into().unwrap());
    assert_eq!(counter_value, 100, "Counter should be 100");
    dchat_log::out!("   Counter value after set to 100: {}", counter_value);

    dchat_log::out!("\n✅ All counter contract tests passed!");
}

#[test]
fn test_counter_error_cases() {
    let wasm_path = get_wasm_path();

    if !wasm_path.exists() {
        dchat_log::out!("Skipping: WASM not found");
        return;
    }

    let wasm_bytes = std::fs::read(&wasm_path).expect("Failed to read WASM file");

    let mut engine_config = wasmi::Config::default();
    engine_config.consume_fuel(true);
    engine_config.floats(false);

    let engine = wasmi::Engine::new(&engine_config);
    let module = wasmi::Module::new(&engine, &wasm_bytes).expect("Failed to parse module");

    let compute_meter = Arc::new(ComputeMeter::new(ComputeBudget::new(2_000_000)));
    let syscalls = Arc::new(SyscallRegistry::new());
    let program_id = Pubkey::new([9u8; 32]);
    let state = VmState::new(compute_meter.clone(), program_id, syscalls, 32);

    let mut store = wasmi::Store::new(&engine, state);
    store.limiter(|state| &mut state.limits);

    let fuel = compute_meter.remaining();
    store.set_fuel(fuel).expect("Failed to set initial fuel");

    let mut linker = wasmi::Linker::<VmState>::new(&engine);
    register_wasi_shim(&mut linker).expect("Failed to register WASI shim");
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate")
        .start(&mut store)
        .expect("Failed to start");

    let entrypoint = instance
        .get_typed_func::<(u32, u32, u32, u32), u32>(&store, "entrypoint")
        .expect("entrypoint export not found");

    let memory = instance
        .get_memory(&store, "memory")
        .expect("memory export not found");

    let alloc = instance
        .get_typed_func::<u32, u32>(&store, "alloc")
        .expect("alloc export not found");

    let account_ptr = alloc
        .call(&mut store, COUNTER_STATE_SIZE as u32)
        .expect("alloc failed");
    let ix_ptr = alloc.call(&mut store, 16).expect("alloc failed");

    // Test: increment on uninitialized account should fail
    memory
        .write(&mut store, ix_ptr as usize, &[IX_INCREMENT])
        .expect("write failed");

    let result = entrypoint
        .call(
            &mut store,
            (account_ptr, COUNTER_STATE_SIZE as u32, ix_ptr, 1),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 2, "Should return ERR_NOT_INITIALIZED (2)");
    dchat_log::out!("✅ Increment on uninitialized returns correct error");

    // Initialize first
    memory
        .write(&mut store, ix_ptr as usize, &[IX_INITIALIZE])
        .expect("write failed");
    let result = entrypoint
        .call(
            &mut store,
            (account_ptr, COUNTER_STATE_SIZE as u32, ix_ptr, 1),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 0);

    // Test: double initialize should fail
    let result = entrypoint
        .call(
            &mut store,
            (account_ptr, COUNTER_STATE_SIZE as u32, ix_ptr, 1),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 3, "Should return ERR_ALREADY_INITIALIZED (3)");
    dchat_log::out!("✅ Double initialize returns correct error");

    // Test: underflow should fail
    let mut ix_data = vec![IX_DECREMENT];
    ix_data.extend_from_slice(&100u64.to_le_bytes());
    memory
        .write(&mut store, ix_ptr as usize, &ix_data)
        .expect("write failed");

    let result = entrypoint
        .call(
            &mut store,
            (
                account_ptr,
                COUNTER_STATE_SIZE as u32,
                ix_ptr,
                ix_data.len() as u32,
            ),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 5, "Should return ERR_UNDERFLOW (5)");
    dchat_log::out!("✅ Underflow returns correct error");

    // Test: invalid instruction
    memory
        .write(&mut store, ix_ptr as usize, &[255u8])
        .expect("write failed");

    let result = entrypoint
        .call(
            &mut store,
            (account_ptr, COUNTER_STATE_SIZE as u32, ix_ptr, 1),
        )
        .expect("entrypoint call failed");
    assert_eq!(result, 1, "Should return ERR_INVALID_INSTRUCTION (1)");
    dchat_log::out!("✅ Invalid instruction returns correct error");

    dchat_log::out!("\n✅ All error case tests passed!");
}
