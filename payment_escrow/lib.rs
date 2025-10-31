#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod payment_escrow {
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
    pub struct Escrow {
        pub owner: H160,
        pub provider: Option<H160>,
        pub amount: U256,
        pub released: bool,
        pub refunded: bool,
    }

    #[ink(storage)]
    pub struct PaymentEscrow {
        /// job_id -> Escrow record
        escrows: Mapping<u128, Escrow>,
        /// optional admin for emergency actions
        admin: H160,
    }

    impl PaymentEscrow {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { escrows: Mapping::default(), admin: Self::env().caller() }
        }

        /// Deposits funds for a job and sets the intended provider.
        /// Must be called by the job owner and is payable.
        #[ink(message, payable)]
        pub fn deposit_for_job(&mut self, job_id: u128, provider: H160) -> bool {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();
            if amount == 0.into() { return false; }
            if let Some(existing) = self.escrows.get(job_id) {
                // Prevent overwriting an active escrow
                if !existing.released && !existing.refunded && existing.amount > 0.into() { return false; }
            }
            let escrow = Escrow { owner: caller, provider: Some(provider), amount, released: false, refunded: false };
            self.escrows.insert(job_id, &escrow);
            self.env().emit_event(Deposited { job_id, owner: caller, provider, amount });
            true
        }

        /// Sets/updates the provider for an existing job escrow. Only the owner can change it.
        #[ink(message)]
        pub fn set_provider(&mut self, job_id: u128, provider: H160) -> bool {
            let caller = self.env().caller();
            if let Some(mut e) = self.escrows.get(job_id) {
                if caller != e.owner || e.released || e.refunded { return false; }
                e.provider = Some(provider);
                self.escrows.insert(job_id, &e);
                self.env().emit_event(ProviderSet { job_id, provider });
                true
            } else { false }
        }

        /// Releases funds to the assigned provider. Only the owner can release.
        #[ink(message)]
        pub fn release_to_provider(&mut self, job_id: u128) -> bool {
            let caller = self.env().caller();
            if let Some(mut e) = self.escrows.get(job_id) {
                if caller != e.owner || e.released || e.refunded { return false; }
                let provider = match e.provider { Some(p) => p, None => return false };
                let amount = e.amount;
                if amount == 0.into() { return false; }
                if self.env().transfer(provider, amount).is_err() { return false; }
                e.released = true;
                e.amount = 0.into();
                self.escrows.insert(job_id, &e);
                self.env().emit_event(Released { job_id, provider, amount });
                true
            } else { false }
        }

        /// Refunds funds back to the owner. Only the owner can refund.
        #[ink(message)]
        pub fn refund_to_owner(&mut self, job_id: u128) -> bool {
            let caller = self.env().caller();
            if let Some(mut e) = self.escrows.get(job_id) {
                if caller != e.owner || e.released || e.refunded { return false; }
                let amount = e.amount;
                if amount == 0.into() { return false; }
                if self.env().transfer(e.owner, amount).is_err() { return false; }
                e.refunded = true;
                e.amount = 0.into();
                self.escrows.insert(job_id, &e);
                self.env().emit_event(Refunded { job_id, owner: e.owner, amount });
                true
            } else { false }
        }

        /// Returns the escrow record for a job, if any.
        #[ink(message)]
        pub fn get_escrow(&self, job_id: u128) -> Option<Escrow> { self.escrows.get(job_id) }

        /// Admin address (optional usage for future controls)
        #[ink(message)]
        pub fn get_admin(&self) -> H160 { self.admin }
    }

    #[ink(event)]
    pub struct Deposited { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub owner: H160, #[ink(topic)] pub provider: H160, pub amount: U256 }
    #[ink(event)]
    pub struct Released { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub provider: H160, pub amount: U256 }
    #[ink(event)]
    pub struct Refunded { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub owner: H160, pub amount: U256 }
    #[ink(event)]
    pub struct ProviderSet { #[ink(topic)] pub job_id: u128, #[ink(topic)] pub provider: H160 }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::primitives::{H160, U256};

        fn alice() -> H160 {
            H160::from([1u8; 20])
        }

        fn bob() -> H160 {
            H160::from([2u8; 20])
        }

        fn charlie() -> H160 {
            H160::from([3u8; 20])
        }

        #[ink::test]
        fn test_new() {
            let _escrow = PaymentEscrow::new();
            // Constructor works without errors
        }

        #[ink::test]
        fn test_deposit_for_job() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Set up payment transfer
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            
            let result = escrow.deposit_for_job(job_id, provider);
            assert!(result);
            
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.owner, alice());
            assert_eq!(stored_escrow.provider, Some(provider));
            assert_eq!(stored_escrow.amount, U256::from(1000u128));
            assert!(!stored_escrow.released);
            assert!(!stored_escrow.refunded);
        }

        #[ink::test]
        fn test_deposit_zero_amount_fails() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(0u128));
            
            let result = escrow.deposit_for_job(job_id, provider);
            assert!(!result);
            assert!(escrow.get_escrow(job_id).is_none());
        }

        #[ink::test]
        fn test_set_provider() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let initial_provider = bob();
            let new_provider = charlie();
            
            // First deposit
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, initial_provider);
            
            // Change provider
            let result = escrow.set_provider(job_id, new_provider);
            assert!(result);
            
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.provider, Some(new_provider));
        }

        #[ink::test]
        fn test_set_provider_wrong_owner_fails() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Alice deposits
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            
            // Bob tries to change provider (should fail)
            ink::env::test::set_caller(bob());
            let result = escrow.set_provider(job_id, charlie());
            assert!(!result);
        }

        #[ink::test]
        fn test_release_to_provider() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Deposit
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            
            // Release to provider
            let result = escrow.release_to_provider(job_id);
            assert!(result);
            
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert!(stored_escrow.released);
            assert!(!stored_escrow.refunded);
            assert_eq!(stored_escrow.amount, U256::from(0u128));
        }

        #[ink::test]
        fn test_refund_to_owner() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Deposit
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            
            // Refund to owner
            let result = escrow.refund_to_owner(job_id);
            assert!(result);
            
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert!(!stored_escrow.released);
            assert!(stored_escrow.refunded);
            assert_eq!(stored_escrow.amount, U256::from(0u128));
        }

        #[ink::test]
        fn test_cannot_release_after_refund() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Deposit
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            
            // Refund first
            escrow.refund_to_owner(job_id);
            
            // Try to release (should fail)
            let result = escrow.release_to_provider(job_id);
            assert!(!result);
        }

        #[ink::test]
        fn test_cannot_refund_after_release() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Deposit
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            
            // Release first
            escrow.release_to_provider(job_id);
            
            // Try to refund (should fail)
            let result = escrow.refund_to_owner(job_id);
            assert!(!result);
        }

        #[ink::test]
        fn test_deposit_overwrites_completed_escrow() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // First deposit and release
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            escrow.release_to_provider(job_id);
            
            // Second deposit should succeed (escrow was completed)
            ink::env::test::set_value_transferred(U256::from(2000u128));
            let result = escrow.deposit_for_job(job_id, charlie());
            assert!(result);
            
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.amount, U256::from(2000u128));
            assert_eq!(stored_escrow.provider, Some(charlie()));
            assert!(!stored_escrow.released);
            assert!(!stored_escrow.refunded);
        }

        #[ink::test]
        fn test_deposit_overwrites_refunded_escrow() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // First deposit and refund
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            escrow.refund_to_owner(job_id);
            
            // Second deposit should succeed (escrow was refunded)
            ink::env::test::set_value_transferred(U256::from(3000u128));
            let result = escrow.deposit_for_job(job_id, charlie());
            assert!(result);
            
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.amount, U256::from(3000u128));
            assert_eq!(stored_escrow.provider, Some(charlie()));
            assert!(!stored_escrow.released);
            assert!(!stored_escrow.refunded);
        }

        #[ink::test]
        fn test_cannot_overwrite_active_escrow() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // First deposit
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            
            // Try to deposit again for same job (should fail)
            ink::env::test::set_value_transferred(U256::from(2000u128));
            let result = escrow.deposit_for_job(job_id, charlie());
            assert!(!result);
            
            // Original escrow should be unchanged
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.amount, U256::from(1000u128));
            assert_eq!(stored_escrow.provider, Some(provider));
        }

        #[ink::test]
        fn test_release_fails_without_provider() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Deposit with provider
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            
            // Remove provider by setting to None manually (simulate edge case)
            let mut stored_escrow = escrow.get_escrow(job_id).unwrap();
            stored_escrow.provider = None;
            escrow.escrows.insert(job_id, &stored_escrow);
            
            // Try to release (should fail)
            let result = escrow.release_to_provider(job_id);
            assert!(!result);
        }

        #[ink::test]
        fn test_set_provider_after_release_fails() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Deposit and release
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            escrow.release_to_provider(job_id);
            
            // Try to change provider after release (should fail)
            let result = escrow.set_provider(job_id, charlie());
            assert!(!result);
        }

        #[ink::test]
        fn test_set_provider_after_refund_fails() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Deposit and refund
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            escrow.refund_to_owner(job_id);
            
            // Try to change provider after refund (should fail)
            let result = escrow.set_provider(job_id, charlie());
            assert!(!result);
        }

        #[ink::test]
        fn test_release_nonexistent_job_fails() {
            let mut escrow = PaymentEscrow::new();
            ink::env::test::set_caller(alice());
            
            // Try to release for job that doesn't exist
            let result = escrow.release_to_provider(999);
            assert!(!result);
        }

        #[ink::test]
        fn test_refund_nonexistent_job_fails() {
            let mut escrow = PaymentEscrow::new();
            ink::env::test::set_caller(alice());
            
            // Try to refund for job that doesn't exist
            let result = escrow.refund_to_owner(999);
            assert!(!result);
        }

        #[ink::test]
        fn test_set_provider_nonexistent_job_fails() {
            let mut escrow = PaymentEscrow::new();
            ink::env::test::set_caller(alice());
            
            // Try to set provider for job that doesn't exist
            let result = escrow.set_provider(999, bob());
            assert!(!result);
        }

        #[ink::test]
        fn test_get_escrow_nonexistent_returns_none() {
            let escrow = PaymentEscrow::new();
            
            // Try to get escrow for job that doesn't exist
            let result = escrow.get_escrow(999);
            assert!(result.is_none());
        }

        #[ink::test]
        fn test_multiple_different_jobs() {
            let mut escrow = PaymentEscrow::new();
            let job1 = 1;
            let job2 = 2;
            let provider1 = bob();
            let provider2 = charlie();
            
            ink::env::test::set_caller(alice());
            
            // Deposit for job 1
            ink::env::test::set_value_transferred(U256::from(1000u128));
            let result1 = escrow.deposit_for_job(job1, provider1);
            assert!(result1);
            
            // Deposit for job 2
            ink::env::test::set_value_transferred(U256::from(2000u128));
            let result2 = escrow.deposit_for_job(job2, provider2);
            assert!(result2);
            
            // Both escrows should exist independently
            let escrow1 = escrow.get_escrow(job1).unwrap();
            let escrow2 = escrow.get_escrow(job2).unwrap();
            
            assert_eq!(escrow1.amount, U256::from(1000u128));
            assert_eq!(escrow1.provider, Some(provider1));
            assert_eq!(escrow2.amount, U256::from(2000u128));
            assert_eq!(escrow2.provider, Some(provider2));
            
            // Release job 1, refund job 2
            escrow.release_to_provider(job1);
            escrow.refund_to_owner(job2);
            
            let final_escrow1 = escrow.get_escrow(job1).unwrap();
            let final_escrow2 = escrow.get_escrow(job2).unwrap();
            
            assert!(final_escrow1.released);
            assert!(!final_escrow1.refunded);
            assert!(!final_escrow2.released);
            assert!(final_escrow2.refunded);
        }

        #[ink::test]
        fn test_provider_address_validation() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            
            // Test with zero address as provider
            let zero_provider = H160::from([0u8; 20]);
            
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            
            // Should allow zero address (contract doesn't validate this)
            let result = escrow.deposit_for_job(job_id, zero_provider);
            assert!(result);
            
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.provider, Some(zero_provider));
        }

        #[ink::test]
        fn test_sequential_operations_same_job() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider1 = bob();
            let provider2 = charlie();
            
            ink::env::test::set_caller(alice());
            
            // First cycle: deposit -> change provider -> release
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider1);
            escrow.set_provider(job_id, provider2);
            escrow.release_to_provider(job_id);
            
            // Second cycle: new deposit -> refund
            ink::env::test::set_value_transferred(U256::from(2000u128));
            escrow.deposit_for_job(job_id, provider1);
            escrow.refund_to_owner(job_id);
            
            let final_escrow = escrow.get_escrow(job_id).unwrap();
            assert!(final_escrow.refunded);
            assert!(!final_escrow.released);
            assert_eq!(final_escrow.amount, U256::from(0u128));
        }

        #[ink::test]
        fn test_wrong_caller_operations() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();
            
            // Alice deposits
            ink::env::test::set_caller(alice());
            ink::env::test::set_value_transferred(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);
            
            // Bob tries to perform owner operations (should all fail)
            ink::env::test::set_caller(bob());
            
            assert!(!escrow.release_to_provider(job_id));
            assert!(!escrow.refund_to_owner(job_id));
            assert!(!escrow.set_provider(job_id, charlie()));
            
            // Charlie tries same operations (should all fail)
            ink::env::test::set_caller(charlie());
            
            assert!(!escrow.release_to_provider(job_id));
            assert!(!escrow.refund_to_owner(job_id));
            assert!(!escrow.set_provider(job_id, bob()));
            
            // Original escrow should be unchanged
            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.owner, alice());
            assert_eq!(stored_escrow.provider, Some(provider));
            assert!(!stored_escrow.released);
            assert!(!stored_escrow.refunded);
        }
    }
}
