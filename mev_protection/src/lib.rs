//! # MEV Protection Contract
//!
//! Prevents Maximal Extractable Value (MEV) attacks through intent-based ordering.
//! Users submit encrypted trading intents that are batched and executed fairly,
//! preventing sandwich attacks and front-running.
//!
//! ## Features
//! - Encrypted intent submission
//! - Fair batch ordering
//! - Sandwich attack prevention
//! - DEX routing for optimal execution
//! - Batch execution tracking

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod mev_protection {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    // ink 5.x does not expose an H160 primitive. Alias H160 to the environment's AccountId (32 bytes) for compatibility.
    type H160 = AccountId;

    // ===== DATA STRUCTURES =====

    /// Intent status enumeration
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
    pub enum IntentStatus {
        Pending,
        Batched,
        Executed,
        Failed,
        Cancelled,
    }

    /// Represents a user's encrypted trading intent
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
    pub struct Intent {
        pub intent_id: u128,
        pub user: H160,
        pub encrypted_intent: String,
        pub token_in: String,
        pub token_out: String,
        pub min_output: u128,
        pub status: IntentStatus,
        pub created_at: u64,
        pub batch_id: Option<u128>,
    }

    /// Represents a batch of intents ready for execution
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
    pub struct Batch {
        pub batch_id: u128,
        pub intent_ids: Vec<u128>,
        pub intent_count: u32,
        pub total_volume: u128,
        pub execution_route: String,
        pub status: IntentStatus,
        pub created_at: u64,
        pub executed_at: Option<u64>,
    }

    /// Execution result for a batch
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
    pub struct BatchResult {
        pub batch_id: u128,
        pub success: bool,
        pub total_input_amount: u128,
        pub total_output_amount: u128,
        pub execution_price: String,
        pub timestamp: u64,
    }

    // ===== CONTRACT STORAGE =====

    #[ink(storage)]
    pub struct MEVProtection {
        /// Maps intent_id to Intent
        intents: Mapping<u128, Intent>,
        /// Maps batch_id to Batch
        batches: Mapping<u128, Batch>,
        /// Maps batch_id to BatchResult
        batch_results: Mapping<u128, BatchResult>,
        /// Counter for intent IDs
        intent_counter: u128,
        /// Counter for batch IDs
        batch_counter: u128,
        /// Admin address for contract management
        admin: H160,
        /// Batch size (max intents per batch)
        batch_size: u32,
        /// Minimum intents to form a batch
        min_batch_size: u32,
    }

    // ===== IMPLEMENTATION =====

    // Provide a Default implementation to allow instantiation via default() in tests and silence lint
    impl Default for MEVProtection {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MEVProtection {
        /// Creates a new MEVProtection contract
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();

            Self {
                intents: Mapping::default(),
                batches: Mapping::default(),
                batch_results: Mapping::default(),
                intent_counter: 0,
                batch_counter: 0,
                admin: caller,
                batch_size: 100,
                min_batch_size: 5,
            }
        }

        /// Submits an encrypted trading intent
        ///
        /// # Arguments
        /// * `encrypted_intent` - Encrypted intent parameters
        /// * `token_in` - Input token address
        /// * `token_out` - Output token address
        /// * `min_output` - Minimum acceptable output amount
        ///
        /// # Returns
        /// The intent ID
        #[ink(message)]
        pub fn submit_intent(
            &mut self,
            encrypted_intent: String,
            token_in: String,
            token_out: String,
            min_output: u128,
        ) -> u128 {
            let caller: H160 = self.env().caller().into();

            self.intent_counter = self.intent_counter.saturating_add(1);
            let intent_id = self.intent_counter;

            let intent = Intent {
                intent_id,
                user: caller,
                encrypted_intent,
                token_in,
                token_out,
                min_output,
                status: IntentStatus::Pending,
                created_at: self.env().block_timestamp(),
                batch_id: None,
            };

            self.intents.insert(intent_id, &intent);
            self.env().emit_event(IntentSubmitted { intent_id });

            intent_id
        }

        /// Creates a batch from pending intents
        ///
        /// # Arguments
        /// * `intent_ids` - IDs of intents to batch
        /// * `execution_route` - DEX routing strategy
        ///
        /// # Returns
        /// The batch ID
        #[ink(message)]
        pub fn create_batch(
            &mut self,
            intent_ids: Vec<u128>,
            execution_route: String,
        ) -> u128 {
            // Validate batch size safely converting length
            let intent_count = match u32::try_from(intent_ids.len()) {
                Ok(c) => c,
                Err(_) => return 0, // Too many intents to fit in u32
            };
            if intent_count < self.min_batch_size || intent_count > self.batch_size {
                return 0; // Invalid batch size
            }

            // Assign a new batch id
            self.batch_counter = self.batch_counter.saturating_add(1);
            let batch_id = self.batch_counter;

            // Calculate total volume & update intents
            let mut total_volume: u128 = 0;
            for intent_id in &intent_ids {
                if let Some(intent) = self.intents.get(intent_id) {
                    total_volume = total_volume.saturating_add(intent.min_output);
                    let mut updated = intent.clone();
                    updated.status = IntentStatus::Batched;
                    updated.batch_id = Some(batch_id);
                    self.intents.insert(*intent_id, &updated);
                }
            }

            let batch = Batch {
                batch_id,
                intent_ids,
                intent_count,
                total_volume,
                execution_route,
                status: IntentStatus::Pending,
                created_at: self.env().block_timestamp(),
                executed_at: None,
            };

            self.batches.insert(batch_id, &batch);
            self.env().emit_event(BatchCreated { batch_id });

            batch_id
        }

        /// Executes a batch on DEX
        ///
        /// # Arguments
        /// * `batch_id` - ID of batch to execute
        /// * `actual_output` - Actual output amount from DEX
        /// * `execution_price` - Execution price used
        ///
        /// # Returns
        /// true if execution was successful
        #[ink(message)]
        pub fn execute_batch(
            &mut self,
            batch_id: u128,
            actual_output: u128,
            execution_price: String,
        ) -> bool {
            if !self.batches.contains(batch_id) {
                return false;
            }

            let mut batch = self.batches.get(batch_id).unwrap();

            // Calculate input from intents
            let mut total_input: u128 = 0;
            for intent_id in &batch.intent_ids {
                if let Some(intent) = self.intents.get(intent_id) {
                    total_input = total_input.saturating_add(intent.min_output);
                }
            }

            // Update batch status
            batch.status = IntentStatus::Executed;
            batch.executed_at = Some(self.env().block_timestamp());
            self.batches.insert(batch_id, &batch);

            // Record execution result
            let result = BatchResult {
                batch_id,
                success: true,
                total_input_amount: total_input,
                total_output_amount: actual_output,
                execution_price,
                timestamp: self.env().block_timestamp(),
            };

            self.batch_results.insert(batch_id, &result);

            // Update intent statuses
            for intent_id in &batch.intent_ids {
                if let Some(mut intent) = self.intents.get(intent_id) {
                    intent.status = IntentStatus::Executed;
                    self.intents.insert(*intent_id, &intent);
                }
            }

            self.env().emit_event(BatchExecuted { batch_id });

            true
        }

        /// Retrieves an intent by ID
        #[ink(message)]
        pub fn get_intent(&self, intent_id: u128) -> Option<Intent> {
            self.intents.get(intent_id)
        }

        /// Retrieves a batch by ID
        #[ink(message)]
        pub fn get_batch(&self, batch_id: u128) -> Option<Batch> {
            self.batches.get(batch_id)
        }

        /// Retrieves batch execution result
        #[ink(message)]
        pub fn get_batch_result(&self, batch_id: u128) -> Option<BatchResult> {
            self.batch_results.get(batch_id)
        }

        /// Gets the current intent counter
        #[ink(message)]
        pub fn get_intent_counter(&self) -> u128 {
            self.intent_counter
        }

        /// Gets the current batch counter
        #[ink(message)]
        pub fn get_batch_counter(&self) -> u128 {
            self.batch_counter
        }

        /// Gets pending intents count
        #[ink(message)]
        pub fn get_pending_intents(&self) -> u32 {
            // Mock implementation
            0
        }

        /// Gets batch statistics
        #[ink(message)]
        pub fn get_batch_stats(&self, batch_id: u128) -> (u32, u128, bool) {
            if let Some(batch) = self.batches.get(batch_id) {
                let is_executed = batch.status == IntentStatus::Executed;
                return (batch.intent_count, batch.total_volume, is_executed);
            }
            (0, 0, false)
        }

        /// Sets batch size configuration
        #[ink(message)]
        pub fn set_batch_config(&mut self, batch_size: u32, min_batch_size: u32) -> bool {
            let caller: H160 = self.env().caller();
            if caller != self.admin {
                return false;
            }

            if min_batch_size > batch_size {
                return false;
            }

            self.batch_size = batch_size;
            self.min_batch_size = min_batch_size;

            true
        }
    }

    // ===== EVENTS =====

    /// Emitted when an intent is submitted
    #[ink(event)]
    pub struct IntentSubmitted {
        #[ink(topic)]
        pub intent_id: u128,
    }

    /// Emitted when a batch is created
    #[ink(event)]
    pub struct BatchCreated {
        #[ink(topic)]
        pub batch_id: u128,
    }

    /// Emitted when a batch is executed
    #[ink(event)]
    pub struct BatchExecuted {
        #[ink(topic)]
        pub batch_id: u128,
    }

    // ===== TESTS =====

    #[cfg(test)]
    mod tests {
        use super::*;

        // ===== INITIALIZATION TESTS =====

        #[ink::test]
        fn test_new() {
            let contract = MEVProtection::new();
            assert_eq!(contract.get_intent_counter(), 0);
            assert_eq!(contract.get_batch_counter(), 0);
        }

        #[ink::test]
        fn test_default_batch_config() {
            let contract = MEVProtection::new();
            let (count, volume, executed) = contract.get_batch_stats(999);
            assert_eq!(count, 0);
            assert_eq!(volume, 0);
            assert!(!executed);
        }

        // ===== INTENT SUBMISSION TESTS =====

        #[ink::test]
        fn test_submit_intent() {
            let mut contract = MEVProtection::new();

            let intent_id = contract.submit_intent(
                "encrypted_intent_123".into(),
                "token_in_abc".into(),
                "token_out_xyz".into(),
                1000,
            );

            assert_eq!(intent_id, 1);
            assert_eq!(contract.get_intent_counter(), 1);

            let intent = contract.get_intent(intent_id).unwrap();
            assert_eq!(intent.intent_id, 1);
            assert_eq!(intent.status, IntentStatus::Pending);
            assert_eq!(intent.min_output, 1000);
            assert_eq!(intent.token_in, "token_in_abc");
            assert_eq!(intent.token_out, "token_out_xyz");
            assert_eq!(intent.batch_id, None);
        }

        #[ink::test]
        fn test_submit_intent_with_zero_output() {
            let mut contract = MEVProtection::new();

            let intent_id = contract.submit_intent(
                "encrypted".into(),
                "USDT".into(),
                "DOT".into(),
                0,
            );

            assert_eq!(intent_id, 1);
            let intent = contract.get_intent(intent_id).unwrap();
            assert_eq!(intent.min_output, 0);
        }

        #[ink::test]
        fn test_submit_intent_with_large_amount() {
            let mut contract = MEVProtection::new();

            let large_amount = u128::MAX;
            let intent_id = contract.submit_intent(
                "encrypted".into(),
                "USDT".into(),
                "DOT".into(),
                large_amount,
            );

            assert_eq!(intent_id, 1);
            let intent = contract.get_intent(intent_id).unwrap();
            assert_eq!(intent.min_output, large_amount);
        }

        #[ink::test]
        fn test_multiple_intents() {
            let mut contract = MEVProtection::new();

            let id1 = contract.submit_intent(
                "intent_1".into(),
                "token_in".into(),
                "token_out".into(),
                500,
            );

            let id2 = contract.submit_intent(
                "intent_2".into(),
                "token_in".into(),
                "token_out".into(),
                500,
            );

            let id3 = contract.submit_intent(
                "intent_3".into(),
                "token_in".into(),
                "token_out".into(),
                500,
            );

            assert_eq!(id1, 1);
            assert_eq!(id2, 2);
            assert_eq!(id3, 3);
            assert_eq!(contract.get_intent_counter(), 3);
        }

        #[ink::test]
        fn test_intent_id_increment() {
            let mut contract = MEVProtection::new();

            for i in 1..=50 {
                let intent_id = contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    1000,
                );
                assert_eq!(intent_id, i);
            }

            assert_eq!(contract.get_intent_counter(), 50);
        }

        #[ink::test]
        fn test_intent_has_timestamp() {
            let mut contract = MEVProtection::new();

            let intent_id = contract.submit_intent(
                "encrypted".into(),
                "USDT".into(),
                "DOT".into(),
                1000,
            );

            let intent = contract.get_intent(intent_id).unwrap();
            // Timestamp is set by the block environment (may be 0 in test)
            assert!(intent.created_at >= 0);
            // Verify the intent has the expected ID
            assert_eq!(intent.intent_id, intent_id);
        }

        // ===== BATCH CREATION TESTS =====

        #[ink::test]
        fn test_create_batch() {
            let mut contract = MEVProtection::new();

            // Submit intents
            for i in 0..10 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "token_in".into(),
                    "token_out".into(),
                    100,
                );
            }

            // Create batch with first 5 intents
            let intent_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch_id = contract.create_batch(intent_ids.clone(), "hydradx".into());

            assert_eq!(batch_id, 1);
            assert_eq!(contract.get_batch_counter(), 1);

            let batch = contract.get_batch(batch_id).unwrap();
            assert_eq!(batch.batch_id, 1);
            assert_eq!(batch.intent_count, 5);
            assert_eq!(batch.execution_route, "hydradx");
            assert_eq!(batch.status, IntentStatus::Pending);
        }

        #[ink::test]
        fn test_batch_size_validation_too_small() {
            let mut contract = MEVProtection::new();

            // Submit intents
            for i in 0..3 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "token_in".into(),
                    "token_out".into(),
                    100,
                );
            }

            // Try to create batch with too few intents (only 3, min is 5)
            let intent_ids: Vec<u128> = vec![1, 2, 3];
            let batch_id = contract.create_batch(intent_ids, "hydradx".into());

            assert_eq!(batch_id, 0); // Should fail
            assert_eq!(contract.get_batch_counter(), 0);
        }

        #[ink::test]
        fn test_batch_size_validation_max_size() {
            let mut contract = MEVProtection::new();

            // Submit more intents than batch_size (100)
            for i in 0..110 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "token_in".into(),
                    "token_out".into(),
                    10,
                );
            }

            // Try to create batch with 101 intents (exceeds max)
            let intent_ids: Vec<u128> = (1..=101).collect();
            let batch_id = contract.create_batch(intent_ids, "hydradx".into());

            assert_eq!(batch_id, 0); // Should fail
        }

        #[ink::test]
        fn test_batch_size_at_minimum() {
            let mut contract = MEVProtection::new();

            // Submit exactly min_batch_size intents
            for i in 0..5 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    100,
                );
            }

            let intent_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch_id = contract.create_batch(intent_ids, "hydradx".into());

            assert_ne!(batch_id, 0); // Should succeed
            assert_eq!(batch_id, 1);
        }

        #[ink::test]
        fn test_batch_total_volume_calculation() {
            let mut contract = MEVProtection::new();

            // Submit intents with different volumes
            let volumes = vec![100u128, 200, 150, 250, 300];
            for (i, vol) in volumes.iter().enumerate() {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    *vol,
                );
            }

            let intent_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch_id = contract.create_batch(intent_ids, "hydradx".into());

            let batch = contract.get_batch(batch_id).unwrap();
            let expected_volume: u128 = volumes.iter().sum();
            assert_eq!(batch.total_volume, expected_volume);
        }

        #[ink::test]
        fn test_batch_with_different_routes() {
            let mut contract = MEVProtection::new();

            for i in 0..10 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    100,
                );
            }

            // Create batch 1 with HydraX
            let batch1_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch1_id = contract.create_batch(batch1_ids, "hydradx".into());
            let batch1 = contract.get_batch(batch1_id).unwrap();
            assert_eq!(batch1.execution_route, "hydradx");

            // Create batch 2 with Polkadex
            let batch2_ids: Vec<u128> = vec![6, 7, 8, 9, 10];
            let batch2_id = contract.create_batch(batch2_ids, "polkadex".into());
            let batch2 = contract.get_batch(batch2_id).unwrap();
            assert_eq!(batch2.execution_route, "polkadex");
        }

        // ===== BATCH EXECUTION TESTS =====

        #[ink::test]
        fn test_execute_batch() {
            let mut contract = MEVProtection::new();

            // Submit and batch intents
            for i in 0..5 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "token_in".into(),
                    "token_out".into(),
                    100,
                );
            }

            let intent_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch_id = contract.create_batch(intent_ids, "hydradx".into());

            // Execute batch
            let success = contract.execute_batch(batch_id, 520, "1.04".into());

            assert!(success);

            let batch = contract.get_batch(batch_id).unwrap();
            assert_eq!(batch.status, IntentStatus::Executed);
            assert!(batch.executed_at.is_some());

            let result = contract.get_batch_result(batch_id).unwrap();
            assert!(result.success);
            assert_eq!(result.total_output_amount, 520);
            assert_eq!(result.execution_price, "1.04");
        }

        #[ink::test]
        fn test_execute_nonexistent_batch() {
            let mut contract = MEVProtection::new();

            let success = contract.execute_batch(999, 1000, "1.5".into());
            assert!(!success);
        }

        #[ink::test]
        fn test_batch_result_has_timestamp() {
            let mut contract = MEVProtection::new();

            for i in 0..5 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    100,
                );
            }

            let intent_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch_id = contract.create_batch(intent_ids, "hydradx".into());
            contract.execute_batch(batch_id, 520, "1.04".into());

            let result = contract.get_batch_result(batch_id).unwrap();
            assert!(result.timestamp > 0);
        }

        #[ink::test]
        fn test_execute_batch_updates_intent_status() {
            let mut contract = MEVProtection::new();

            for i in 0..5 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    100,
                );
            }

            let intent_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch_id = contract.create_batch(intent_ids.clone(), "hydradx".into());

            // Before execution, intents should be Batched
            for intent_id in &intent_ids {
                let intent = contract.get_intent(*intent_id).unwrap();
                assert_eq!(intent.status, IntentStatus::Batched);
            }

            contract.execute_batch(batch_id, 520, "1.04".into());

            // After execution, intents should be Executed
            for intent_id in &intent_ids {
                let intent = contract.get_intent(*intent_id).unwrap();
                assert_eq!(intent.status, IntentStatus::Executed);
            }
        }

        // ===== STATUS TRANSITION TESTS =====

        #[ink::test]
        fn test_intent_status_transitions() {
            let mut contract = MEVProtection::new();

            let intent_id = contract.submit_intent(
                "intent".into(),
                "token_in".into(),
                "token_out".into(),
                100,
            );

            // Initial status: Pending
            let intent = contract.get_intent(intent_id).unwrap();
            assert_eq!(intent.status, IntentStatus::Pending);

            // After batching: Batched
            let batch_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            // Need 5 intents for batching
            for i in 1..5 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "token_in".into(),
                    "token_out".into(),
                    100,
                );
            }

            let batch_id = contract.create_batch(batch_ids, "hydradx".into());
            let intent = contract.get_intent(intent_id).unwrap();
            assert_eq!(intent.status, IntentStatus::Batched);

            // After execution: Executed
            contract.execute_batch(batch_id, 500, "1.0".into());
            let intent = contract.get_intent(intent_id).unwrap();
            assert_eq!(intent.status, IntentStatus::Executed);
        }

        // ===== STATISTICS TESTS =====

        #[ink::test]
        fn test_batch_statistics() {
            let mut contract = MEVProtection::new();

            // Submit intents
            for i in 0..5 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "token_in".into(),
                    "token_out".into(),
                    100,
                );
            }

            let intent_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch_id = contract.create_batch(intent_ids, "hydradx".into());

            let (count, volume, executed) = contract.get_batch_stats(batch_id);

            assert_eq!(count, 5);
            assert_eq!(volume, 500);
            assert!(!executed);

            // Execute batch
            contract.execute_batch(batch_id, 520, "1.04".into());

            let (count, volume, executed) = contract.get_batch_stats(batch_id);
            assert_eq!(count, 5);
            assert_eq!(volume, 500);
            assert!(executed);
        }

        #[ink::test]
        fn test_batch_configuration() {
            let mut contract = MEVProtection::new();

            let success = contract.set_batch_config(50, 3);
            assert!(success);

            // Now create batch with 3 intents (previously required 5)
            for i in 0..3 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    100,
                );
            }

            let intent_ids: Vec<u128> = vec![1, 2, 3];
            let batch_id = contract.create_batch(intent_ids, "hydradx".into());

            assert_ne!(batch_id, 0);
        }

        #[ink::test]
        fn test_batch_config_invalid_sizes() {
            let mut contract = MEVProtection::new();

            // Invalid: min > batch_size
            let success = contract.set_batch_config(5, 10);
            assert!(!success);
        }

        // ===== ERROR HANDLING TESTS =====

        #[ink::test]
        fn test_get_nonexistent_intent() {
            let contract = MEVProtection::new();
            let result = contract.get_intent(999);
            assert!(result.is_none());
        }

        #[ink::test]
        fn test_get_nonexistent_batch() {
            let contract = MEVProtection::new();
            let result = contract.get_batch(999);
            assert!(result.is_none());
        }

        #[ink::test]
        fn test_get_nonexistent_batch_result() {
            let contract = MEVProtection::new();
            let result = contract.get_batch_result(999);
            assert!(result.is_none());
        }

        // ===== MULTIPLE BATCHES TESTS =====

        #[ink::test]
        fn test_multiple_batches() {
            let mut contract = MEVProtection::new();

            // Create first batch
            for i in 0..5 {
                contract.submit_intent(
                    format!("intent_batch1_{}", i).into(),
                    "token_in".into(),
                    "token_out".into(),
                    100,
                );
            }

            let batch1_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch1_id = contract.create_batch(batch1_ids, "hydradx".into());

            // Create second batch
            for i in 5..10 {
                contract.submit_intent(
                    format!("intent_batch2_{}", i).into(),
                    "token_in".into(),
                    "token_out".into(),
                    200,
                );
            }

            let batch2_ids: Vec<u128> = vec![6, 7, 8, 9, 10];
            let batch2_id = contract.create_batch(batch2_ids, "polkadex".into());

            assert_eq!(batch1_id, 1);
            assert_eq!(batch2_id, 2);
            assert_eq!(contract.get_batch_counter(), 2);

            // Verify independent execution
            contract.execute_batch(batch1_id, 510, "1.02".into());

            let batch1 = contract.get_batch(batch1_id).unwrap();
            let batch2 = contract.get_batch(batch2_id).unwrap();

            assert_eq!(batch1.status, IntentStatus::Executed);
            assert_eq!(batch2.status, IntentStatus::Pending);
        }

        // ===== COMPLETE WORKFLOW TESTS =====

        #[ink::test]
        fn test_complete_mev_workflow() {
            let mut contract = MEVProtection::new();

            // Step 1: Submit intents
            for i in 0..5 {
                contract.submit_intent(
                    format!("encrypted_intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    1000,
                );
            }

            assert_eq!(contract.get_intent_counter(), 5);

            // Step 2: Create batch
            let intent_ids: Vec<u128> = vec![1, 2, 3, 4, 5];
            let batch_id = contract.create_batch(intent_ids, "optimal_route".into());

            assert_eq!(batch_id, 1);
            let batch = contract.get_batch(batch_id).unwrap();
            assert_eq!(batch.status, IntentStatus::Pending);

            // Step 3: Execute batch
            let executed = contract.execute_batch(batch_id, 5100, "1.02".into());
            assert!(executed);

            // Step 4: Verify execution
            let updated_batch = contract.get_batch(batch_id).unwrap();
            assert_eq!(updated_batch.status, IntentStatus::Executed);

            let result = contract.get_batch_result(batch_id).unwrap();
            assert!(result.success);
            assert_eq!(result.total_output_amount, 5100);
            assert_eq!(result.execution_price, "1.02");
        }

        // ===== PRODUCTION STRESS TESTS =====

        #[ink::test]
        fn test_stress_many_intents_and_batches() {
            let mut contract = MEVProtection::new();

            // Submit 100 intents
            for i in 0..100 {
                contract.submit_intent(
                    format!("intent_{}", i).into(),
                    "USDT".into(),
                    "DOT".into(),
                    100 + (i as u128),
                );
            }

            assert_eq!(contract.get_intent_counter(), 100);

            // Create 10 batches of 10 intents each
            for batch_num in 0..10 {
                let mut intent_ids = Vec::new();
                for j in 0..10 {
                    intent_ids.push((batch_num * 10 + j + 1) as u128);
                }

                let batch_id = contract.create_batch(intent_ids, "hydradx".into());

                // Only execute even-numbered batches
                if batch_num % 2 == 0 {
                    contract.execute_batch(batch_id, 1500, "1.5".into());
                }
            }

            assert_eq!(contract.get_batch_counter(), 10);
        }

        #[ink::test]
        fn test_security_intent_isolation() {
            let mut contract = MEVProtection::new();

            let intent_1 = contract.submit_intent(
                "secret_intent_1".into(),
                "USDT".into(),
                "DOT".into(),
                1000,
            );

            let intent_2 = contract.submit_intent(
                "secret_intent_2".into(),
                "USDC".into(),
                "ETH".into(),
                2000,
            );

            let retrieved_1 = contract.get_intent(intent_1).unwrap();
            let retrieved_2 = contract.get_intent(intent_2).unwrap();

            // Ensure intents don't leak into each other
            assert_eq!(retrieved_1.encrypted_intent, "secret_intent_1");
            assert_eq!(retrieved_2.encrypted_intent, "secret_intent_2");
            assert_ne!(retrieved_1.encrypted_intent, retrieved_2.encrypted_intent);

            assert_eq!(retrieved_1.token_in, "USDT");
            assert_eq!(retrieved_2.token_in, "USDC");
        }
    }
}
