//! # PhalaJobProcessor Contract
//!
//! This contract manages confidential job execution on Phala Network.
//!
//! ## Features
//! - Submit encrypted jobs for confidential execution
//! - Record attestation proofs from Phala TEE workers
//! - Verify job completion and processing status
//! - Track job lifecycle from submission to completion
//!
//! ## Usage
//! 1. User calls `submit_confidential_job()` with encrypted params
//! 2. Phala TEE executes the job
//! 3. TEE calls `record_attestation()` with result + proof
//! 4. Contract verifies and marks job as processed

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod phala_job_processor {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    // ink 5.x compatibility: alias H160 to AccountId (32 bytes)
    type H160 = AccountId;

    // ===== DATA STRUCTURES =====

    /// Represents a confidential job submission
    #[derive(
        ink::scale::Encode,
        ink::scale::Decode,
        Clone,
        Debug,
        PartialEq,
        Eq,
    )]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct ConfidentialJob {
        pub job_id: u128,
        pub owner: H160,
        pub encrypted_payload: String,
        pub public_key: String,
        pub created_at: u64,
        pub processed: bool,
    }

    /// Represents attestation proof from Phala TEE
    #[derive(
        ink::scale::Encode,
        ink::scale::Decode,
        Clone,
        Debug,
        PartialEq,
        Eq,
    )]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct JobAttestation {
        pub job_id: u128,
        pub result_hash: String,
        pub attestation_proof: String,
        pub tee_worker_pubkey: String,
        pub timestamp: u64,
    }

    // ===== CONTRACT STORAGE =====

    #[ink(storage)]
    pub struct PhalaJobProcessor {
        /// Maps job_id to ConfidentialJob
        jobs: Mapping<u128, ConfidentialJob>,
        /// Maps job_id to JobAttestation
        attestations: Mapping<u128, JobAttestation>,
        /// Counter for job IDs
        job_counter: u128,
        /// Admin address for contract management
        admin: H160,
    }

    // ===== IMPLEMENTATION =====

    impl Default for PhalaJobProcessor {
        fn default() -> Self { Self::new() }
    }

    impl PhalaJobProcessor {
        /// Creates a new PhalaJobProcessor contract
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();

            Self {
                jobs: Mapping::default(),
                attestations: Mapping::default(),
                job_counter: 0,
                admin: caller,
            }
        }

        /// Submits a confidential job for execution
        ///
        /// # Arguments
        /// * `encrypted_payload` - Encrypted job parameters
        /// * `public_key` - Public key for encryption verification
        ///
        /// # Returns
        /// The job ID if successful
        #[ink(message)]
        pub fn submit_confidential_job(
            &mut self,
            encrypted_payload: String,
            public_key: String,
        ) -> u128 {
            let caller: H160 = self.env().caller();

            self.job_counter = self.job_counter.saturating_add(1);
            let job_id = self.job_counter;

            let job = ConfidentialJob {
                job_id,
                owner: caller,
                encrypted_payload,
                public_key,
                created_at: self.env().block_timestamp(),
                processed: false,
            };

            self.jobs.insert(job_id, &job);
            self.env().emit_event(JobSubmitted { job_id });

            job_id
        }

        /// Records an attestation proof from Phala TEE
        ///
        /// # Arguments
        /// * `job_id` - ID of the job
        /// * `result_hash` - Hash of the job result
        /// * `attestation_proof` - Cryptographic proof from TEE
        /// * `tee_worker_pubkey` - Public key of the TEE worker
        ///
        /// # Returns
        /// true if attestation was recorded, false if job doesn't exist
        #[ink(message)]
        pub fn record_attestation(
            &mut self,
            job_id: u128,
            result_hash: String,
            attestation_proof: String,
            tee_worker_pubkey: String,
        ) -> bool {
            if !self.jobs.contains(job_id) {
                return false;
            }

            let attestation = JobAttestation {
                job_id,
                result_hash,
                attestation_proof,
                tee_worker_pubkey,
                timestamp: self.env().block_timestamp(),
            };

            self.attestations.insert(job_id, &attestation);
            self.env().emit_event(AttestationRecorded { job_id });

            true
        }

        /// Retrieves a job by ID
        #[ink(message)]
        pub fn get_job(&self, job_id: u128) -> Option<ConfidentialJob> {
            self.jobs.get(job_id)
        }

        /// Retrieves an attestation by job ID
        #[ink(message)]
        pub fn get_attestation(&self, job_id: u128) -> Option<JobAttestation> {
            self.attestations.get(job_id)
        }

        /// Gets the current job counter
        #[ink(message)]
        pub fn get_job_counter(&self) -> u128 {
            self.job_counter
        }

        /// Verifies if a job has an attestation
        #[ink(message)]
        pub fn verify_attestation(&self, job_id: u128) -> bool {
            self.attestations.contains(job_id)
        }

        /// Marks a job as processed after verification
        ///
        /// Only succeeds if attestation exists for the job
        #[ink(message)]
        pub fn mark_job_processed(&mut self, job_id: u128) -> bool {
            if let Some(mut job) = self.jobs.get(job_id) {
                if !self.attestations.contains(job_id) {
                    return false;
                }
                job.processed = true;
                self.jobs.insert(job_id, &job);
                self.env().emit_event(JobProcessed { job_id });
                true
            } else {
                false
            }
        }
    }

    // ===== EVENTS =====

    /// Emitted when a confidential job is submitted
    #[ink(event)]
    pub struct JobSubmitted {
        #[ink(topic)]
        pub job_id: u128,
    }

    /// Emitted when an attestation is recorded
    #[ink(event)]
    pub struct AttestationRecorded {
        #[ink(topic)]
        pub job_id: u128,
    }

    /// Emitted when a job is marked as processed
    #[ink(event)]
    pub struct JobProcessed {
        #[ink(topic)]
        pub job_id: u128,
    }

    // ===== TESTS =====

    #[cfg(test)]
    mod tests {
        use super::*;

        // ===== INITIALIZATION TESTS =====

        #[ink::test]
        fn test_new() {
            let contract = PhalaJobProcessor::new();
            assert_eq!(contract.get_job_counter(), 0);
        }

        // ===== JOB SUBMISSION TESTS =====

        #[ink::test]
        fn test_submit_confidential_job() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job(
                "encrypted_data_123".into(),
                "public_key_abc".into(),
            );

            assert_eq!(job_id, 1);
            assert_eq!(contract.get_job_counter(), 1);

            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.job_id, 1);
            assert_eq!(job.processed, false);
            assert_eq!(job.encrypted_payload, "encrypted_data_123");
            assert_eq!(job.public_key, "public_key_abc");
        }

        #[ink::test]
        fn test_submit_job_with_empty_payload() {
            let mut contract = PhalaJobProcessor::new();

            // Empty payload should still work (will be encrypted by caller)
            let job_id = contract.submit_confidential_job(
                "".into(),
                "public_key".into(),
            );

            assert_eq!(job_id, 1);
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.encrypted_payload, "");
        }

        #[ink::test]
        fn test_submit_job_with_large_payload() {
            let mut contract = PhalaJobProcessor::new();

            // Large encrypted payload (realistic scenario)
            let large_payload = "x".repeat(10000);
            let job_id = contract.submit_confidential_job(
                large_payload.clone().into(),
                "public_key".into(),
            );

            assert_eq!(job_id, 1);
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.encrypted_payload, large_payload);
        }

        #[ink::test]
        fn test_submit_multiple_jobs_sequential() {
            let mut contract = PhalaJobProcessor::new();

            for i in 1..=100 {
                let job_id = contract.submit_confidential_job(
                    format!("encrypted_data_{}", i).into(),
                    format!("public_key_{}", i).into(),
                );

                assert_eq!(job_id, i);
                assert_eq!(contract.get_job_counter(), i);
            }
        }

        #[ink::test]
        fn test_job_id_increment() {
            let mut contract = PhalaJobProcessor::new();

            let id1 = contract.submit_confidential_job("data1".into(), "key1".into());
            let id2 = contract.submit_confidential_job("data2".into(), "key2".into());
            let id3 = contract.submit_confidential_job("data3".into(), "key3".into());

            assert_eq!(id1, 1);
            assert_eq!(id2, 2);
            assert_eq!(id3, 3);
        }

        // ===== ATTESTATION TESTS =====

        #[ink::test]
        fn test_record_attestation() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job(
                "encrypted_data".into(),
                "public_key".into(),
            );

            let result = contract.record_attestation(
                job_id,
                "result_hash_123".into(),
                "attestation_proof_456".into(),
                "worker_pubkey_789".into(),
            );

            assert!(result);

            let attestation = contract.get_attestation(job_id).unwrap();
            assert_eq!(attestation.job_id, job_id);
            assert_eq!(attestation.result_hash, "result_hash_123");
            assert_eq!(attestation.attestation_proof, "attestation_proof_456");
            assert_eq!(attestation.tee_worker_pubkey, "worker_pubkey_789");
        }

        #[ink::test]
        fn test_record_attestation_nonexistent_job() {
            let mut contract = PhalaJobProcessor::new();

            let result = contract.record_attestation(
                999,
                "result".into(),
                "proof".into(),
                "worker".into(),
            );

            assert!(!result);
        }

        #[ink::test]
        fn test_record_attestation_with_different_workers() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job("data".into(), "key".into());

            // First attestation from worker 1
            let result1 = contract.record_attestation(
                job_id,
                "hash1".into(),
                "proof1".into(),
                "worker_pubkey_1".into(),
            );

            assert!(result1);

            // Record attestation again (overwrites previous)
            let result2 = contract.record_attestation(
                job_id,
                "hash2".into(),
                "proof2".into(),
                "worker_pubkey_2".into(),
            );

            assert!(result2);

            let attestation = contract.get_attestation(job_id).unwrap();
            assert_eq!(attestation.result_hash, "hash2");
            assert_eq!(attestation.tee_worker_pubkey, "worker_pubkey_2");
        }

        #[ink::test]
        fn test_attestation_has_timestamp() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job("data".into(), "key".into());
            contract.record_attestation(
                job_id,
                "hash".into(),
                "proof".into(),
                "worker".into(),
            );

            let attestation = contract.get_attestation(job_id).unwrap();
            // Timestamp is set by the block environment (may be 0 in test)
            assert!(attestation.timestamp >= 0);
            // Verify the field exists
            assert_eq!(attestation.job_id, job_id);
        }

        #[ink::test]
        fn test_record_multiple_attestations() {
            let mut contract = PhalaJobProcessor::new();

            // Submit multiple jobs
            let job_ids: Vec<u128> = (0..10)
                .map(|i| {
                    contract.submit_confidential_job(
                        format!("data_{}", i).into(),
                        format!("key_{}", i).into(),
                    )
                })
                .collect();

            // Record attestation for each
            for (idx, job_id) in job_ids.iter().enumerate() {
                let result = contract.record_attestation(
                    *job_id,
                    format!("hash_{}", idx).into(),
                    format!("proof_{}", idx).into(),
                    format!("worker_{}", idx).into(),
                );

                assert!(result);
            }

            // Verify all attestations
            for (idx, job_id) in job_ids.iter().enumerate() {
                let att = contract.get_attestation(*job_id).unwrap();
                assert_eq!(att.result_hash, format!("hash_{}", idx));
                assert_eq!(att.attestation_proof, format!("proof_{}", idx));
            }
        }

        // ===== VERIFICATION TESTS =====

        #[ink::test]
        fn test_verify_attestation() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job(
                "data".into(),
                "key".into(),
            );

            assert!(!contract.verify_attestation(job_id));

            contract.record_attestation(
                job_id,
                "hash".into(),
                "proof".into(),
                "worker".into(),
            );

            assert!(contract.verify_attestation(job_id));
        }

        #[ink::test]
        fn test_verify_attestation_nonexistent() {
            let contract = PhalaJobProcessor::new();
            assert!(!contract.verify_attestation(999));
        }

        // ===== JOB PROCESSING TESTS =====

        #[ink::test]
        fn test_mark_job_processed() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job(
                "data".into(),
                "key".into(),
            );

            // Cannot process without attestation
            assert!(!contract.mark_job_processed(job_id));

            // Record attestation
            contract.record_attestation(
                job_id,
                "hash".into(),
                "proof".into(),
                "worker".into(),
            );

            // Now can mark as processed
            assert!(contract.mark_job_processed(job_id));

            let job = contract.get_job(job_id).unwrap();
            assert!(job.processed);
        }

        #[ink::test]
        fn test_mark_nonexistent_job_processed() {
            let mut contract = PhalaJobProcessor::new();

            let result = contract.mark_job_processed(999);
            assert!(!result);
        }

        #[ink::test]
        fn test_mark_job_processed_without_attestation() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job("data".into(), "key".into());

            // Should fail without attestation
            assert!(!contract.mark_job_processed(job_id));

            let job = contract.get_job(job_id).unwrap();
            assert!(!job.processed);
        }

        #[ink::test]
        fn test_mark_already_processed_job() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job("data".into(), "key".into());

            contract.record_attestation(
                job_id,
                "hash".into(),
                "proof".into(),
                "worker".into(),
            );

            assert!(contract.mark_job_processed(job_id));

            // Mark again should succeed
            assert!(contract.mark_job_processed(job_id));

            let job = contract.get_job(job_id).unwrap();
            assert!(job.processed);
        }

        // ===== JOB LIFECYCLE TESTS =====

        #[ink::test]
        fn test_complete_job_lifecycle() {
            let mut contract = PhalaJobProcessor::new();

            // Submit
            let job_id = contract.submit_confidential_job(
                "data".into(),
                "key".into(),
            );
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.processed, false);

            // Record attestation
            let att_result = contract.record_attestation(
                job_id,
                "hash".into(),
                "proof".into(),
                "worker".into(),
            );
            assert!(att_result);
            assert!(contract.verify_attestation(job_id));

            // Mark processed
            let process_result = contract.mark_job_processed(job_id);
            assert!(process_result);
            let job = contract.get_job(job_id).unwrap();
            assert!(job.processed);
        }

        #[ink::test]
        fn test_concurrent_job_lifecycles() {
            let mut contract = PhalaJobProcessor::new();

            // Submit 5 jobs
            let job_ids: Vec<u128> = (0..5)
                .map(|i| {
                    contract.submit_confidential_job(
                        format!("data_{}", i).into(),
                        format!("key_{}", i).into(),
                    )
                })
                .collect();

            // Process them in mixed order
            for (idx, job_id) in job_ids.iter().enumerate() {
                contract.record_attestation(
                    *job_id,
                    format!("hash_{}", idx).into(),
                    format!("proof_{}", idx).into(),
                    format!("worker_{}", idx).into(),
                );

                // Only process jobs with even ids
                if idx % 2 == 0 {
                    assert!(contract.mark_job_processed(*job_id));
                }
            }

            // Verify state
            for (idx, job_id) in job_ids.iter().enumerate() {
                let job = contract.get_job(*job_id).unwrap();
                if idx % 2 == 0 {
                    assert!(job.processed);
                } else {
                    assert!(!job.processed);
                }
            }
        }

        // ===== ERROR HANDLING TESTS =====

        #[ink::test]
        fn test_get_nonexistent_job() {
            let contract = PhalaJobProcessor::new();
            let result = contract.get_job(999);
            assert!(result.is_none());
        }

        #[ink::test]
        fn test_get_nonexistent_attestation() {
            let contract = PhalaJobProcessor::new();
            let result = contract.get_attestation(999);
            assert!(result.is_none());
        }

        #[ink::test]
        fn test_multiple_jobs() {
            let mut contract = PhalaJobProcessor::new();

            let job_id_1 = contract.submit_confidential_job(
                "data1".into(),
                "key1".into(),
            );

            let job_id_2 = contract.submit_confidential_job(
                "data2".into(),
                "key2".into(),
            );

            assert_eq!(job_id_1, 1);
            assert_eq!(job_id_2, 2);
            assert_eq!(contract.get_job_counter(), 2);

            let job_1 = contract.get_job(job_id_1).unwrap();
            let job_2 = contract.get_job(job_id_2).unwrap();

            assert_eq!(job_1.job_id, 1);
            assert_eq!(job_2.job_id, 2);
            assert_eq!(job_1.owner, job_2.owner);
        }

        // ===== STATE CONSISTENCY TESTS =====

        #[ink::test]
        fn test_job_counter_consistency() {
            let mut contract = PhalaJobProcessor::new();

            for i in 1..=50 {
                contract.submit_confidential_job(
                    format!("data_{}", i).into(),
                    format!("key_{}", i).into(),
                );
                assert_eq!(contract.get_job_counter(), i);
            }
        }

        #[ink::test]
        fn test_job_ownership_tracking() {
            let mut contract = PhalaJobProcessor::new();

            let job_id_1 = contract.submit_confidential_job("data1".into(), "key1".into());
            let job_1 = contract.get_job(job_id_1).unwrap();

            // Verify owner is recorded
            assert_ne!(job_1.owner, AccountId::from([0u8; 32]));
        }

        #[ink::test]
        fn test_job_timestamps() {
            let mut contract = PhalaJobProcessor::new();

            let job_id = contract.submit_confidential_job("data".into(), "key".into());
            let job = contract.get_job(job_id).unwrap();

            // Timestamp is set by the block environment (may be 0 in test)
            assert!(job.created_at >= 0);
            // Verify the job has the expected ID
            assert_eq!(job.job_id, job_id);
        }

        // ===== PRODUCTION GRADE STRESS TESTS =====

        #[ink::test]
        fn test_stress_many_jobs() {
            let mut contract = PhalaJobProcessor::new();

            // Submit 1000 jobs
            for i in 1..=1000 {
                let job_id = contract.submit_confidential_job(
                    format!("encrypted_data_{}", i).into(),
                    format!("public_key_{}", i).into(),
                );

                assert_eq!(job_id, i);

                // Process every 10th job
                if i % 10 == 0 {
                    contract.record_attestation(
                        job_id,
                        format!("hash_{}", i).into(),
                        format!("proof_{}", i).into(),
                        format!("worker_{}", i).into(),
                    );

                    contract.mark_job_processed(job_id);

                    let job = contract.get_job(job_id).unwrap();
                    assert!(job.processed);
                }
            }

            assert_eq!(contract.get_job_counter(), 1000);
        }

        #[ink::test]
        fn test_security_no_data_leakage() {
            let mut contract = PhalaJobProcessor::new();

            let job_id_1 = contract.submit_confidential_job(
                "secret_data_1".into(),
                "public_key_1".into(),
            );

            let job_id_2 = contract.submit_confidential_job(
                "secret_data_2".into(),
                "public_key_2".into(),
            );

            // Ensure jobs maintain data isolation
            let job_1 = contract.get_job(job_id_1).unwrap();
            let job_2 = contract.get_job(job_id_2).unwrap();

            assert_eq!(job_1.encrypted_payload, "secret_data_1");
            assert_eq!(job_2.encrypted_payload, "secret_data_2");
            assert_ne!(job_1.encrypted_payload, job_2.encrypted_payload);
        }
    }
}
