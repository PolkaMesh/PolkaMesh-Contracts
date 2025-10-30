#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = ink::env::DefaultEnvironment)]
mod ai_job_queue {
    use ink::prelude::string::String;
    use ink::storage::Mapping;

    #[derive(
        ink::storage::traits::Storable,
        ink::storage::traits::StorageKey,
        ink::scale::Encode,
        ink::scale::Decode,
        #![cfg_attr(not(feature = "std"), no_std, no_main)]

        #[ink::contract(env = ink::env::DefaultEnvironment)]
        mod ai_job_queue {
            use ink::prelude::string::String;
            use ink::storage::Mapping;

            #[derive(
            pub enum JobStatus { Registered, Assigned, InProgress, Completed, Cancelled }
                Eq,
            )]
            #[cfg_attr(
                feature = "std",
                derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
            )]
            pub enum JobStatus { Registered, Assigned, InProgress, Completed, Cancelled }

            #[derive(
                ink::storage::traits::Storable,
                ink::storage::traits::StorageKey,
                ink::scale::Encode,
                ink::scale::Decode,
                Clone,
            pub struct Job {
                pub id: u128,
                pub owner: AccountId,
                pub model_ref: String,
                pub data_ref: String,
                pub budget: Balance,
                pub status: JobStatus,
                pub assigned_provider: Option<AccountId>,
                pub deadline: u32,
                pub privacy_required: bool,
            }
                pub model_ref: String,
                pub data_ref: String,
                pub budget: Balance,
                pub status: JobStatus,
                pub assigned_provider: Option<AccountId>,
                pub deadline: u32,
                pub privacy_required: bool,
            }

            #[ink(storage)]
            pub struct AiJobQueue {
                jobs: Mapping<u128, Job>,
                job_counter: u128,
                min_budget: Balance,
                owner: AccountId,
            }

            impl AiJobQueue {
                #[ink(constructor)]
                pub fn new(min_budget: Balance) -> Self {
                    Self { jobs: Mapping::default(), job_counter: 0, min_budget, owner: Self::env().caller() }
                }

                #[ink(message, payable)]
                pub fn submit_job(&mut self, model_ref: String, data_ref: String, deadline: u32, privacy_required: bool) -> u128 {
                    let caller = self.env().caller();
                    let payment = self.env().transferred_value();
                    assert!(payment >= self.min_budget, "Insufficient payment");
                    assert!(deadline > self.env().block_number(), "Invalid deadline");
                    self.job_counter = self.job_counter.saturating_add(1);
                    let job_id = self.job_counter;
                    let job = Job { id: job_id, owner: caller, model_ref, data_ref, budget: payment, status: JobStatus::Registered, assigned_provider: None, deadline, privacy_required };
                    self.jobs.insert(job_id, &job);
                    self.env().emit_event(JobSubmitted { job_id, owner: caller, budget: payment });
                    job_id
                }

                #[ink(message)]
                pub fn get_job(&self, job_id: u128) -> Option<Job> { self.jobs.get(job_id) }

                #[ink(message)]
                pub fn assign_provider(&mut self, job_id: u128, provider: AccountId) -> bool {
                    let caller = self.env().caller();
                    if let Some(mut job) = self.jobs.get(job_id) {
                        if caller != job.owner || job.status != JobStatus::Registered { return false; }
                        job.assigned_provider = Some(provider);
                        job.status = JobStatus::Assigned;
                        self.jobs.insert(job_id, &job);
                        self.env().emit_event(JobAssigned { job_id, provider });
                        true
                    } else { false }
                }

                #[ink(message)]
                pub fn mark_in_progress(&mut self, job_id: u128) -> bool {
                    let caller = self.env().caller();
                    if let Some(mut job) = self.jobs.get(job_id) {
                        if job.assigned_provider != Some(caller) || job.status != JobStatus::Assigned { return false; }
                        job.status = JobStatus::InProgress;
                        self.jobs.insert(job_id, &job);
                        self.env().emit_event(JobStatusChanged { job_id, new_status: JobStatus::InProgress });
                        true
                    } else { false }
                }

                #[ink(message)]
                pub fn mark_completed(&mut self, job_id: u128, result_hash: String) -> bool {
                    let caller = self.env().caller();
                    if let Some(mut job) = self.jobs.get(job_id) {
                        if job.assigned_provider != Some(caller) || job.status != JobStatus::InProgress { return false; }
                        job.status = JobStatus::Completed;
                        self.jobs.insert(job_id, &job);
                        self.env().emit_event(JobCompleted { job_id, provider: caller, result_hash });
                        true
                    } else { false }
                }

                #[ink(message)]
                pub fn cancel_job(&mut self, job_id: u128) -> bool {
                    let caller = self.env().caller();
                    if let Some(mut job) = self.jobs.get(job_id) {
                        if caller != job.owner || job.status == JobStatus::Completed { return false; }
                        job.status = JobStatus::Cancelled;
                        self.jobs.insert(job_id, &job);
                        self.env().emit_event(JobCancelled { job_id });
                        true
                    } else { false }
                }

                #[ink(message)]
                pub fn get_job_counter(&self) -> u128 { self.job_counter }
                #[ink(message)]
                pub fn get_min_budget(&self) -> Balance { self.min_budget }
                #[ink(message)]
                pub fn set_min_budget(&mut self, new_min_budget: Balance) -> bool {
                    if self.env().caller() != self.owner { return false; }
                    self.min_budget = new_min_budget; true
                }
            }

            #[ink(event)]
            pub struct JobSubmitted { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub owner: AccountId, pub budget: Balance }
            #[ink(event)]
            pub struct JobAssigned { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub provider: AccountId }
            #[ink(event)]
            pub struct JobStatusChanged { #[ink(topic)] pub job_id: u128, pub new_status: JobStatus }
            #[ink(event)]
            pub struct JobCompleted { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub provider: AccountId, pub result_hash: String }
            #[ink(event)]
            pub struct JobCancelled { #[ink(topic)] pub job_id: u128 }

            #[cfg(test)]
            mod tests {
                use super::*;
                #[ink::test]
                fn new_works() { let c = AiJobQueue::new(1000); assert_eq!(c.get_min_budget(), 1000); assert_eq!(c.get_job_counter(), 0); }
                #[ink::test]
                fn get_job_works() { let c = AiJobQueue::new(1000); assert_eq!(c.get_job(1), None); }
            }
        }
#[ink::contract(env = ink::env::DefaultEnvironment)]
mod ai_job_queue {
    use ink::prelude::string::String;
    use ink::storage::Mapping;

    /// Job status lifecycle
    #[derive(
        ink::storage::traits::Storable,
        ink::storage::traits::StorageKey,
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
    pub enum JobStatus {
        Registered,
        Assigned,
        InProgress,
        Completed,
        Cancelled,
    }

*/
    impl Default for JobStatus {
        fn default() -> Self {
            JobStatus::Registered
        }
    }

*/

    /// Simple Job structure
    #[derive(
        ink::storage::traits::Storable,
        ink::storage::traits::StorageKey,
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
    pub struct Job {
        pub id: u128,
        pub owner: AccountId,
        pub model_ref: String,
        pub data_ref: String,
        pub budget: Balance,
        pub status: JobStatus,
        pub assigned_provider: Option<AccountId>,
        pub deadline: u32,
        pub privacy_required: bool,
    }

    impl Default for Job {
        fn default() -> Self {
            Job {
                id: 0,
                owner: [0u8; 32].into(),
                model_ref: String::new(),
                data_ref: String::new(),
                budget: 0,
                status: JobStatus::default(),
                assigned_provider: None,
                deadline: 0,
                privacy_required: false,
            }
        }
    }

    #[ink(storage)]
    pub struct AiJobQueue {
        /// Mapping from job ID to Job
        jobs: Mapping<u128, Job>,
        /// Current job counter for unique IDs
        job_counter: u128,
        /// Minimum budget required for job submission
        min_budget: Balance,
        /// Contract owner for governance
        owner: AccountId,
    }

    impl AiJobQueue {
        /// Constructor
        #[ink(constructor)]
        pub fn new(min_budget: Balance) -> Self {
            Self {
                jobs: Mapping::default(),
                job_counter: 0,
                min_budget,
                owner: Self::env().caller(),
            }
        }

        /// Submit a new AI job with payment
        #[ink(message, payable)]
        pub fn submit_job(
            &mut self,
            model_ref: String,
            data_ref: String,
            deadline: u32,
            privacy_required: bool,
        ) -> u128 {
            let caller = self.env().caller();
            let payment = self.env().transferred_value();

            // Basic validation
            assert!(payment >= self.min_budget, "Insufficient payment");
            assert!(deadline > self.env().block_number(), "Invalid deadline");

            // Generate new job ID
            self.job_counter = self.job_counter.saturating_add(1);
            let job_id = self.job_counter;

            // Create job
            let job = Job {
                id: job_id,
                owner: caller,
                model_ref,
                data_ref,
                budget: payment,
                status: JobStatus::Registered,
                assigned_provider: None,
                deadline,
                privacy_required,
            };

            // Store job
            self.jobs.insert(job_id, &job);

            // Emit event
            self.env().emit_event(JobSubmitted {
                job_id,
                owner: caller,
                budget: payment,
            });

            job_id
        }

        /// Get job details by ID
        #[ink(message)]
        pub fn get_job(&self, job_id: u128) -> Option<Job> {
            self.jobs.get(job_id)
        }

        /// Assign a provider to a job (called by job owner)
        #[ink(message)]
        pub fn assign_provider(&mut self, job_id: u128, provider: AccountId) -> bool {
            let caller = self.env().caller();

            if let Some(mut job) = self.jobs.get(job_id) {
                if caller != job.owner {
                    return false;
                }
                if job.status != JobStatus::Registered {
                    return false;
                }

                job.assigned_provider = Some(provider);
                job.status = JobStatus::Assigned;
                self.jobs.insert(job_id, &job);

                self.env().emit_event(JobAssigned { job_id, provider });
                true
            } else {
                false
            }
        }

        /// Mark job as in progress
        #[ink(message)]
        pub fn mark_in_progress(&mut self, job_id: u128) -> bool {
            let caller = self.env().caller();

            if let Some(mut job) = self.jobs.get(job_id) {
                if job.assigned_provider != Some(caller) {
                    return false;
                }
                if job.status != JobStatus::Assigned {
                    return false;
                }

                job.status = JobStatus::InProgress;
                self.jobs.insert(job_id, &job);

                self.env().emit_event(JobStatusChanged {
                    job_id,
                    new_status: JobStatus::InProgress,
                });
                true
            } else {
                false
            }
        }

        /// Mark job as completed
        #[ink(message)]
        pub fn mark_completed(&mut self, job_id: u128, result_hash: String) -> bool {
            let caller = self.env().caller();

            if let Some(mut job) = self.jobs.get(job_id) {
                if job.assigned_provider != Some(caller) {
                    return false;
                }
                if job.status != JobStatus::InProgress {
                    return false;
                }

                job.status = JobStatus::Completed;
                self.jobs.insert(job_id, &job);

                self.env().emit_event(JobCompleted {
                    job_id,
                    provider: caller,
                    result_hash,
                });
                true
            } else {
                false
            }
        }

        /// Cancel a job
        #[ink(message)]
        pub fn cancel_job(&mut self, job_id: u128) -> bool {
            let caller = self.env().caller();

            if let Some(mut job) = self.jobs.get(job_id) {
                if caller != job.owner {
                    return false;
                }
                if job.status == JobStatus::Completed {
                    return false;
                }

                job.status = JobStatus::Cancelled;
                self.jobs.insert(job_id, &job);
                self.env().emit_event(JobCancelled { job_id });
                true
            } else {
                false
            }
        }

        /// Get current job counter
        #[ink(message)]
        pub fn get_job_counter(&self) -> u128 {
            self.job_counter
        }

        /// Get minimum budget requirement
        #[ink(message)]
        pub fn get_min_budget(&self) -> Balance {
            self.min_budget
        }

        /// Set minimum budget (owner only)
        #[ink(message)]
        pub fn set_min_budget(&mut self, new_min_budget: Balance) -> bool {
            if self.env().caller() != self.owner {
                return false;
            }
            self.min_budget = new_min_budget;
            true
        }
    }

    // Events
    #[ink(event)]
    pub struct JobSubmitted {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub owner: AccountId,
        pub budget: Balance,
    }

    #[ink(event)]
    pub struct JobAssigned {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub provider: AccountId,
    }

    #[ink(event)]
    pub struct JobStatusChanged {
        #[ink(topic)]
        pub job_id: u128,
        pub new_status: JobStatus,
    }

    #[ink(event)]
    pub struct JobCompleted {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub provider: AccountId,
        pub result_hash: String,
    }

    #[ink(event)]
    pub struct JobCancelled {
        #[ink(topic)]
        pub job_id: u128,
    }

    // Unit tests
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let contract = AiJobQueue::new(1000);
            assert_eq!(contract.get_min_budget(), 1000);
            assert_eq!(contract.get_job_counter(), 0);
        }

        #[ink::test]
        fn get_job_works() {
            let contract = AiJobQueue::new(1000);
            let result = contract.get_job(1);
            assert_eq!(result, None);
        }
    }
}
/*

            
        }

        /// Get all bids for a job
        #[ink(message)]
        pub fn get_job_bids(&self, job_id: u128) -> Vec<Bid> {
            self.job_bids.get(job_id).unwrap_or_default()
        }

        /// Get current job counter
        #[ink(message)]
        pub fn get_job_counter(&self) -> u128 {
            self.job_counter
        }

        /// Get minimum budget requirement
        #[ink(message)]
        pub fn get_min_budget(&self) -> Balance {
            self.min_budget
        }

        // Governance functions (owner only)
        
        /// Set minimum budget (owner only)
        #[ink(message)]
        pub fn set_min_budget(&mut self, new_min_budget: Balance) -> Result<()> {
            self.ensure_owner()?;
            self.min_budget = new_min_budget;
            Ok(())
        }

        /// Pause/unpause contract (owner only)
        #[ink(message)]
        pub fn set_paused(&mut self, paused: bool) -> Result<()> {
            self.ensure_owner()?;
            self.paused = paused;
            Ok(())
        }

        /// Transfer ownership (owner only)
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.owner = new_owner;
            Ok(())
        }

        // Private helper functions
        
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }
            Ok(())
        }

        fn ensure_not_paused(&self) -> Result<()> {
            if self.paused {
                return Err(Error::Unauthorized);
            }
            Ok(())
        }
    }

    // Events
    
    #[ink(event)]
    pub struct JobSubmitted {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub owner: AccountId,
        pub budget: Balance,
        pub compute_type: ComputeType,
        pub deadline: u64,
        pub privacy_required: bool,
    }

    #[ink(event)]
    pub struct BidSubmitted {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub provider: AccountId,
        pub price: Balance,
        pub estimated_completion_time: u64,
    }

    #[ink(event)]
    pub struct JobAssigned {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub provider: AccountId,
        pub price: Balance,
    }

    #[ink(event)]
    pub struct JobStatusChanged {
        #[ink(topic)]
        pub job_id: u128,
        pub new_status: JobStatus,
    }

    #[ink(event)]
    pub struct JobCompleted {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub provider: AccountId,
        pub result_hash: String,
    }

    #[ink(event)]
    pub struct JobDisputed {
        #[ink(topic)]
        pub job_id: u128,
        pub reason: String,
    }

    #[ink(event)]
    pub struct JobCancelled {
        #[ink(topic)]
        pub job_id: u128,
    }

    // Unit tests
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let contract = AiJobQueue::new(1000);
            assert_eq!(contract.get_min_budget(), 1000);
            assert_eq!(contract.get_job_counter(), 0);
        }

        #[ink::test]
        fn submit_job_works() {
            let mut contract = AiJobQueue::new(1000);
            
            // Mock environment setup would be here in a real test
            // For now, this shows the structure
            
            // This would test successful job submission
            // let job_id = contract.submit_job(...).unwrap();
            // assert_eq!(job_id, 1);
        }

        #[ink::test]
        fn submit_bid_works() {
            // Test bid submission
        }

        #[ink::test]
        fn assign_provider_works() {
            // Test provider assignment
        }

        #[ink::test]
        fn job_lifecycle_works() {
            // Test complete job lifecycle
        }

        #[ink::test]
        fn unauthorized_access_fails() {
            // Test access control
        }
    }
}

*/

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = AiJobQueueRef::new(false);
            let contract = client
                .instantiate("ai_job_queue", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<AiJobQueue>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
