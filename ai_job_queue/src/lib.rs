#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ai_job_queue {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::primitives::{H160, U256};

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
    pub enum JobStatus { Registered, Assigned, InProgress, Completed, Cancelled }

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
        pub budget: U256,
        pub status: JobStatus,
        pub assigned_provider: Option<H160>,
        pub deadline: u32,
        pub privacy_required: bool,
    }

    #[ink(storage)]
    pub struct AiJobQueue {
        jobs: Mapping<u128, Job>,
        job_counter: u128,
        min_budget: U256,
        owner: H160,
    }

    impl AiJobQueue {
        #[ink(constructor)]
        pub fn new(min_budget: U256) -> Self {
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
        pub fn assign_provider(&mut self, job_id: u128, provider: H160) -> bool {
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
        pub fn get_min_budget(&self) -> U256 { self.min_budget }
        #[ink(message)]
        pub fn set_min_budget(&mut self, new_min_budget: U256) -> bool {
            if self.env().caller() != self.owner { return false; }
            self.min_budget = new_min_budget; true
        }
    }

    #[ink(event)]
    pub struct JobSubmitted { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub owner: H160, pub budget: U256 }
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
    #[ink::test]
    fn new_works() { let c = AiJobQueue::new(U256::from(1000u128)); assert_eq!(c.get_min_budget(), U256::from(1000u128)); assert_eq!(c.get_job_counter(), 0); }
        #[ink::test]
        fn get_job_works() { let c = AiJobQueue::new(1000); assert_eq!(c.get_job(1), None); }
    }
}
