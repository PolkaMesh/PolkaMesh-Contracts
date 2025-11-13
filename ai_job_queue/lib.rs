#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ai_job_queue {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::primitives::H160;

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
    pub enum JobStatus {
        Registered,
        Assigned,
        InProgress,
        Completed,
        Cancelled,
    }

    impl Default for JobStatus { fn default() -> Self { JobStatus::Registered } }

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
    pub struct Job {
        pub id: u128,
        pub owner: H160,
        pub model_ref: String,
        pub data_ref: String,
        pub budget: u128,
        pub status: JobStatus,
        pub assigned_provider: Option<H160>,
        pub deadline: u32,
        pub privacy_required: bool,
    }

    #[ink(storage)]
    pub struct AiJobQueue {
        jobs: Mapping<u128, Job>,
        job_counter: u128,
        min_budget: u128,
        owner: H160,
    }

    impl AiJobQueue {
        #[ink(constructor)]
        pub fn new(min_budget: u128) -> Self {
            let caller = Self::env().caller();
            let caller_h160: H160 = caller.into();
            Self { jobs: Mapping::default(), job_counter: 0, min_budget, owner: caller_h160 }
        }

        #[ink(message, payable)]
        pub fn submit_job(&mut self, model_ref: String, data_ref: String, deadline: u32, privacy_required: bool) -> u128 {
            let caller: H160 = self.env().caller().into();
            let payment: u128 = self.env().transferred_value().as_u128();
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
        pub fn assign_provider(&mut self, job_id: u128, provider: H160) -> bool {
            let caller: H160 = self.env().caller().into();
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
            let caller: H160 = self.env().caller().into();
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
            let caller: H160 = self.env().caller().into();
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
            let caller: H160 = self.env().caller().into();
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
        pub fn get_min_budget(&self) -> u128 { self.min_budget }
        #[ink(message)]
        pub fn set_min_budget(&mut self, new_min_budget: u128) -> bool {
            let caller: H160 = self.env().caller().into();
            if caller != self.owner { return false; }
            self.min_budget = new_min_budget;
            true
        }
    }

    #[ink(event)]
    pub struct JobSubmitted { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub owner: H160, pub budget: u128 }
    #[ink(event)]
    pub struct JobAssigned { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub provider: H160 }
    #[ink(event)]
    pub struct JobStatusChanged { #[ink(topic)] pub job_id: u128, pub new_status: JobStatus }
    #[ink(event)]
    pub struct JobCompleted { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub provider: H160, pub result_hash: String }
    #[ink(event)]
    pub struct JobCancelled { #[ink(topic)] pub job_id: u128 }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn alice() -> H160 { H160::from([0x1; 20]) }
        fn bob() -> H160 { H160::from([0x2; 20]) }
        fn charlie() -> H160 { H160::from([0x3; 20]) }

        fn set_caller(account: H160) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(account.into());
        }

        fn set_value(amount: u128) {
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(amount);
        }

        fn set_block_number(block: u32) {
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(block);
        }

        #[ink::test]
        fn new_works() {
            let contract = AiJobQueue::new(1000u128);
            assert_eq!(contract.get_min_budget(), 1000u128);
            assert_eq!(contract.get_job_counter(), 0);
        }

        #[ink::test]
        fn submit_job_works() {
            set_caller(alice());
            set_block_number(100);
            set_block_number(100);
            set_value(1500);

            let mut contract = AiJobQueue::new(1000u128);
            let job_id = contract.submit_job("model_uri".into(), "dataset_uri".into(), 500, true);

            assert_eq!(job_id, 1);
            assert_eq!(contract.get_job_counter(), 1);

            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.owner, alice());
            assert_eq!(job.model_ref, "model_uri");
            assert_eq!(job.data_ref, "dataset_uri");
            assert_eq!(job.deadline, 500);
            assert_eq!(job.privacy_required, true);
            assert_eq!(job.budget, 1500u128);
            assert_eq!(job.status, JobStatus::Registered);
        }

        #[ink::test]
        #[should_panic(expected = "Insufficient payment")]
        fn submit_job_insufficient_budget_fails() {
            set_caller(alice());
            set_block_number(100);
            set_block_number(100);
            set_value(500); // Below minimum budget of 1000

            let mut contract = AiJobQueue::new(1000u128);
            contract.submit_job("model".into(), "data".into(), 200, false);
        }

        #[ink::test]
        fn submit_multiple_jobs_works() {
            set_caller(alice());
            set_block_number(100);
            set_block_number(100);

            let mut contract = AiJobQueue::new(500u128);

            set_value(1000);
            let job_id1 = contract.submit_job("model1".into(), "data1".into(), 300, true);

            set_value(2000);
            let job_id2 = contract.submit_job("model2".into(), "data2".into(), 400, false);

            assert_eq!(job_id1, 1);
            assert_eq!(job_id2, 2);
            assert_eq!(contract.get_job_counter(), 2);

            let job1 = contract.get_job(job_id1).unwrap();
            let job2 = contract.get_job(job_id2).unwrap();

            assert_eq!(job1.budget, 1000u128);
            assert_eq!(job2.budget, 2000u128);
            assert_eq!(job1.privacy_required, true);
            assert_eq!(job2.privacy_required, false);
        }

        #[ink::test]
        fn assign_provider_works() {
            set_caller(alice());
            set_block_number(100);
            set_block_number(100);
            set_value(1000);

            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);

            assert!(contract.assign_provider(job_id, bob()));

            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.assigned_provider, Some(bob()));
            assert_eq!(job.status, JobStatus::Assigned);
        }

        #[ink::test]
        fn assign_provider_not_owner_fails() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            
            set_caller(bob()); // Different caller
            assert!(!contract.assign_provider(job_id, charlie()));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.assigned_provider, None);
            assert_eq!(job.status, JobStatus::Registered);
        }

        #[ink::test]
        fn assign_provider_nonexistent_job_fails() {
            set_caller(alice());
            set_block_number(100);
            let mut contract = AiJobQueue::new(500u128);
            
            assert!(!contract.assign_provider(999, bob()));
        }

        #[ink::test]
        fn assign_provider_already_assigned_fails() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            
            assert!(contract.assign_provider(job_id, bob()));
            assert!(!contract.assign_provider(job_id, charlie())); // Already assigned
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.assigned_provider, Some(bob())); // Should remain bob
        }

        #[ink::test]
        fn mark_in_progress_works() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            contract.assign_provider(job_id, bob());
            
            set_caller(bob()); // Provider marks in progress
            assert!(contract.mark_in_progress(job_id));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::InProgress);
        }

        #[ink::test]
        fn mark_in_progress_not_provider_fails() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            contract.assign_provider(job_id, bob());
            
            set_caller(charlie()); // Not the assigned provider
            assert!(!contract.mark_in_progress(job_id));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Assigned);
        }

        #[ink::test]
        fn mark_in_progress_wrong_status_fails() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            
            set_caller(bob()); // Try to mark in progress without assignment
            assert!(!contract.mark_in_progress(job_id));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Registered);
        }

        #[ink::test]
        fn mark_completed_works() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            set_block_number(100);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            contract.assign_provider(job_id, bob());
            
            set_caller(bob());
            contract.mark_in_progress(job_id);
            
            set_block_number(200);
            assert!(contract.mark_completed(job_id, "result_hash".into()));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Completed);
        }

        #[ink::test]
        fn mark_completed_not_provider_fails() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            contract.assign_provider(job_id, bob());
            
            set_caller(bob());
            contract.mark_in_progress(job_id);
            
            set_caller(charlie()); // Not the provider
            assert!(!contract.mark_completed(job_id, "result".into()));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::InProgress);
        }

        #[ink::test]
        fn mark_completed_wrong_status_fails() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            contract.assign_provider(job_id, bob());
            
            set_caller(bob());
            // Skip mark_in_progress, try to complete directly
            assert!(!contract.mark_completed(job_id, "result".into()));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Assigned);
        }

        #[ink::test]
        fn cancel_job_by_owner_works() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            
            assert!(contract.cancel_job(job_id));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Cancelled);
        }

        #[ink::test]
        fn cancel_job_assigned_by_owner_works() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            contract.assign_provider(job_id, bob());
            
            assert!(contract.cancel_job(job_id));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Cancelled);
        }

        #[ink::test]
        fn cancel_job_in_progress_by_owner_works() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 200, false);
            contract.assign_provider(job_id, bob());
            
            set_caller(bob());
            contract.mark_in_progress(job_id);
            
            set_caller(alice()); // Owner cancels even when in progress
            set_block_number(100);
            assert!(contract.cancel_job(job_id));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Cancelled);
        }

        #[ink::test]
        fn cancel_job_unauthorized_fails() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            contract.assign_provider(job_id, bob());
            
            set_caller(charlie()); // Not owner or provider
            assert!(!contract.cancel_job(job_id));
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Assigned);
        }

        #[ink::test]
        fn cancel_job_completed_fails() {
            set_caller(alice());
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            contract.assign_provider(job_id, bob());
            
            set_caller(bob());
            contract.mark_in_progress(job_id);
            contract.mark_completed(job_id, "result".into());
            
            set_caller(alice());
            set_block_number(100);
            assert!(!contract.cancel_job(job_id)); // Cannot cancel completed job
            
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Completed);
        }

        #[ink::test]
        fn cancel_job_nonexistent_fails() {
            set_caller(alice());
            set_block_number(100);
            let mut contract = AiJobQueue::new(500u128);
            
            assert!(!contract.cancel_job(999));
        }

        #[ink::test]
        fn get_job_nonexistent_returns_none() {
            let contract = AiJobQueue::new(500u128);
            assert!(contract.get_job(999).is_none());
        }

        #[ink::test]
        fn complete_job_lifecycle() {
            set_caller(alice());
            set_block_number(100);
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            
            // Submit job
            let job_id = contract.submit_job("model_uri".into(), "dataset_uri".into(), 300, false);
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Registered);
            
            // Assign provider
            assert!(contract.assign_provider(job_id, bob()));
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Assigned);
            assert_eq!(job.assigned_provider, Some(bob()));
            
            // Mark in progress
            set_caller(bob());
            assert!(contract.mark_in_progress(job_id));
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::InProgress);
            
            // Complete job
            set_block_number(200);
            assert!(contract.mark_completed(job_id, "final_result_hash".into()));
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Completed);
        }

        #[ink::test]
        fn job_lifecycle_with_cancellation() {
            set_caller(alice());
            set_block_number(100);
            set_block_number(100);
            set_value(1000);
            
            let mut contract = AiJobQueue::new(500u128);
            let job_id = contract.submit_job("model".into(), "data".into(), 300, false);
            
            // Assign and start job
            contract.assign_provider(job_id, bob());
            set_caller(bob());
            contract.mark_in_progress(job_id);
            
            // Owner cancels (only owner can cancel)
            set_caller(alice());
            set_block_number(100);
            assert!(contract.cancel_job(job_id));
            let job = contract.get_job(job_id).unwrap();
            assert_eq!(job.status, JobStatus::Cancelled);
            
            // Cannot complete cancelled job
            set_caller(bob());
            assert!(!contract.mark_completed(job_id, "result".into()));
        }

        #[ink::test] 
        fn different_users_different_jobs() {
            set_block_number(100);
            let mut contract = AiJobQueue::new(500u128);
            
            // Alice submits job
            set_caller(alice());
            set_value(1000);
            let alice_job = contract.submit_job("alice_model".into(), "alice_data".into(), 300, true);
            
            // Bob submits job  
            set_caller(bob());
            set_value(1500);
            let bob_job = contract.submit_job("bob_model".into(), "bob_data".into(), 400, false);
            
            assert_eq!(alice_job, 1);
            assert_eq!(bob_job, 2);
            
            let alice_job_data = contract.get_job(alice_job).unwrap();
            let bob_job_data = contract.get_job(bob_job).unwrap();
            
            assert_eq!(alice_job_data.owner, alice());
            assert_eq!(bob_job_data.owner, bob());
            assert_eq!(alice_job_data.privacy_required, true);
            assert_eq!(bob_job_data.privacy_required, false);
            assert_eq!(alice_job_data.budget, 1000u128);
            assert_eq!(bob_job_data.budget, 1500u128);
        }
    }
}
