# Phala Job Processor Contract

A confidential job execution contract for Phala Network, enabling encrypted job submission and TEE attestation verification.

## Overview

The `PhalaJobProcessor` contract manages the lifecycle of confidential jobs submitted for execution on Phala's Trusted Execution Environment (TEE) workers. Jobs are submitted with encrypted payloads and verified with cryptographic attestations from TEE workers.

## Features

- **Confidential Job Submission**: Submit encrypted job parameters to the contract
- **Attestation Tracking**: Record and verify cryptographic proofs from Phala TEE workers
- **Job Lifecycle Management**: Track jobs from submission through processing to completion
- **Owner Verification**: Jobs are tied to their submitter's account
- **Result Hash Verification**: Verify the integrity of job results via attestation

## Architecture

### Data Structures

#### ConfidentialJob
```rust
pub struct ConfidentialJob {
    pub job_id: u128,              // Unique job identifier
    pub owner: AccountId,          // Job submitter
    pub encrypted_payload: String, // Encrypted job parameters
    pub public_key: String,        // Public key for encryption
    pub created_at: u64,           // Creation timestamp
    pub processed: bool,           // Processing status
}
```

#### JobAttestation
```rust
pub struct JobAttestation {
    pub job_id: u128,              // Job ID
    pub result_hash: String,       // Hash of the result
    pub attestation_proof: String, // Cryptographic proof
    pub tee_worker_pubkey: String, // TEE worker's public key
    pub timestamp: u64,            // Attestation timestamp
}
```

### Contract Methods

#### Public Methods

##### `submit_confidential_job(encrypted_payload, public_key) -> u128`
Submits a confidential job for execution on Phala TEE.

**Parameters:**
- `encrypted_payload`: Encrypted job parameters (hex string)
- `public_key`: Public key for verification (hex string)

**Returns:** Job ID (u128)

**Events:** Emits `JobSubmitted` event

**Example:**
```rust
let job_id = contract.submit_confidential_job(
    "0xdeadbeef...".into(),
    "0xcafebabe...".into()
);
```

##### `record_attestation(job_id, result_hash, attestation_proof, tee_worker_pubkey) -> bool`
Records an attestation proof from a Phala TEE worker.

**Parameters:**
- `job_id`: ID of the job (u128)
- `result_hash`: Hash of the job result (String)
- `attestation_proof`: Cryptographic proof (String)
- `tee_worker_pubkey`: TEE worker's public key (String)

**Returns:** `true` if attestation was recorded, `false` if job doesn't exist

**Events:** Emits `AttestationRecorded` event

##### `verify_attestation(job_id) -> bool`
Verifies if a job has a recorded attestation.

**Parameters:**
- `job_id`: ID of the job (u128)

**Returns:** `true` if attestation exists, `false` otherwise

##### `mark_job_processed(job_id) -> bool`
Marks a job as processed after attestation verification.

**Parameters:**
- `job_id`: ID of the job (u128)

**Returns:** `true` if marked successfully, `false` if no attestation exists

**Events:** Emits `JobProcessed` event

##### `get_job(job_id) -> Option<ConfidentialJob>`
Retrieves a job by ID.

**Parameters:**
- `job_id`: ID of the job (u128)

**Returns:** Job details if exists, `None` otherwise

##### `get_attestation(job_id) -> Option<JobAttestation>`
Retrieves an attestation by job ID.

**Parameters:**
- `job_id`: ID of the job (u128)

**Returns:** Attestation details if exists, `None` otherwise

##### `get_job_counter() -> u128`
Gets the total number of jobs submitted.

**Returns:** Job counter value

## Events

### JobSubmitted
Emitted when a confidential job is successfully submitted.
```rust
#[ink(event)]
pub struct JobSubmitted {
    #[ink(topic)]
    pub job_id: u128,
}
```

### AttestationRecorded
Emitted when an attestation proof is recorded.
```rust
#[ink(event)]
pub struct AttestationRecorded {
    #[ink(topic)]
    pub job_id: u128,
}
```

### JobProcessed
Emitted when a job is marked as processed.
```rust
#[ink(event)]
pub struct JobProcessed {
    #[ink(topic)]
    pub job_id: u128,
}
```

## Job Lifecycle

```
1. SUBMISSION
   └─> User calls submit_confidential_job()
   └─> Contract stores encrypted payload
   └─> Emits JobSubmitted event
   └─> Returns job_id

2. EXECUTION (Off-chain)
   └─> Phala TEE workers pick up the job
   └─> Workers decrypt and execute in TEE
   └─> Generate attestation proof

3. ATTESTATION RECORDING
   └─> TEE worker calls record_attestation()
   └─> Contract verifies job exists
   └─> Stores attestation proof
   └─> Emits AttestationRecorded event

4. VERIFICATION & COMPLETION
   └─> Contract or user verifies attestation
   └─> User calls mark_job_processed()
   └─> Contract marks job as processed
   └─> Emits JobProcessed event
```

## Testing

The contract includes 10 comprehensive test cases:

### Test Coverage

1. **test_new** - Contract initialization
2. **test_submit_confidential_job** - Job submission with validation
3. **test_record_attestation** - Attestation recording
4. **test_record_attestation_nonexistent_job** - Error handling
5. **test_verify_attestation** - Attestation verification
6. **test_mark_job_processed** - Job completion
7. **test_multiple_jobs** - Multiple concurrent jobs
8. **test_complete_job_lifecycle** - Full submission→attestation→processing flow
9. **test_get_nonexistent_job** - Query non-existent job
10. **test_get_nonexistent_attestation** - Query non-existent attestation

### Running Tests

```bash
cd phala_job_processor
cargo test --lib
```

## Integration with SDK

The contract is wrapped by the TypeScript SDK for easy integration:

```typescript
import { PhalaJobProcessor } from '@polka-mesh/sdk';

// Submit a confidential job
const jobPayload = {
  taskType: 'compute',
  input: 'encrypted_data',
  parameters: { timeout: 3600 }
};

const result = await contract.submitConfidentialJob(jobPayload);
const jobId = result.jobId;

// Record attestation from TEE worker
const attestationRecorded = await contract.recordAttestation(
  jobId,
  resultHash,
  attestationProof,
  teeWorkerPubkey
);

// Mark job as processed
if (attestationRecorded) {
  await contract.markJobProcessed(jobId);
}
```

## Security Considerations

### Current Implementation
- ✅ Job ownership tracking via AccountId
- ✅ Attestation structure validation
- ✅ Event logging for auditing
- ✅ Timestamp recording

### Production Enhancements Needed
- 🔒 Signature verification on attestation proofs
- 🔒 Trusted TEE worker list
- 🔒 Result hash validation
- 🔒 Timestamp validation (prevent old attestations)
- 🔒 Rate limiting on job submissions
- 🔒 Job payload size limits

## Deployment

### Compile to WASM

```bash
# Build for wasm target
cargo build --release --target wasm32-unknown-unknown

# Optimize with wasm-opt
wasm-opt -Oz target/wasm32-unknown-unknown/release/phala_job_processor.wasm \
  -o phala_job_processor_optimized.wasm
```

### Deploy to Testnet

```bash
# Using Polkadot.js
polkadot-js-tools contract upload \
  --file phala_job_processor_optimized.wasm \
  --suri //Alice
```

## Configuration

### Cargo.toml

```toml
[package]
name = "phala_job_processor"
version = "0.1.0"
edition = "2021"

[dependencies]
ink = { version = "5.0", default-features = false, features = ["std"] }
scale = { package = "parity-scale-codec", version = "3", default-features = false, features = ["derive"] }
scale-info = { version = "2", default-features = false, features = ["derive"] }

[features]
default = ["std"]
std = ["ink/std", "scale/std", "scale-info/std"]

[lib]
path = "src/lib.rs"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
```

## Files

- `src/lib.rs` - Main contract implementation (292 lines)
- `Cargo.toml` - Project manifest
- `README.md` - This documentation

## Version History

### v0.1.0 (Week 1)
- Initial contract skeleton
- Basic job submission and attestation recording
- 10 comprehensive tests
- All tests passing ✅

## Future Enhancements

### Phase 2
- [ ] Phat Contract for off-chain job execution
- [ ] XCM message handling for cross-chain communication
- [ ] Advanced signature verification (ECDSA, EdDSA)
- [ ] Batch job processing

### Phase 3
- [ ] MEV protection with intent batching
- [ ] Advanced encryption schemes
- [ ] Zero-knowledge proofs for result verification
- [ ] Performance optimization for high-throughput jobs

## Support & Questions

For issues or questions:
1. Check the test cases in `src/lib.rs`
2. Review the lifecycle diagram above
3. Consult the SDK integration guide

## License

This contract is part of the PolkaDot AI Mesh project.
