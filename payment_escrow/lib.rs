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
            let caller = Self::env().caller();
            let caller_h160: H160 = caller.into();

            Self {
                escrows: Mapping::default(),
                admin: caller_h160,
            }
        }

        /// Deposits funds for a job and sets the intended provider.
        /// Must be called by the job owner and is payable.
        #[ink(message, payable)]
        pub fn deposit_for_job(&mut self, job_id: u128, provider: H160) -> bool {
            let caller: H160 = self.env().caller().into();
            let amount = self.env().transferred_value();

            if amount == 0.into() {
                return false;
            }

            if let Some(existing) = self.escrows.get(job_id) {
                // Prevent overwriting an active escrow
                if !existing.released && !existing.refunded && existing.amount > 0.into() {
                    return false;
                }
            }

            let escrow = Escrow {
                owner: caller,
                provider: Some(provider),
                amount,
                released: false,
                refunded: false,
            };

            self.escrows.insert(job_id, &escrow);
            self.env().emit_event(Deposited {
                job_id,
                owner: caller,
                provider,
                amount,
            });
            true
        }

        /// Sets/updates the provider for an existing job escrow. Only the owner can change it.
        #[ink(message)]
        pub fn set_provider(&mut self, job_id: u128, provider: H160) -> bool {
            let caller: H160 = self.env().caller().into();

            if let Some(mut e) = self.escrows.get(job_id) {
                if caller != e.owner || e.released || e.refunded {
                    return false;
                }
                e.provider = Some(provider);
                self.escrows.insert(job_id, &e);
                self.env().emit_event(ProviderSet { job_id, provider });
                true
            } else {
                false
            }
        }

        /// Releases funds to the assigned provider. Only the owner can release.
        #[ink(message)]
        pub fn release_to_provider(&mut self, job_id: u128) -> bool {
            let caller: H160 = self.env().caller().into();

            if let Some(mut e) = self.escrows.get(job_id) {
                if caller != e.owner || e.released || e.refunded {
                    return false;
                }

                let provider = match e.provider {
                    Some(p) => p,
                    None => return false,
                };

                let amount = e.amount;
                if amount == 0.into() {
                    return false;
                }

                if self.env().transfer(provider, amount).is_err() {
                    return false;
                }

                e.released = true;
                e.amount = 0.into();
                self.escrows.insert(job_id, &e);

                self.env()
                    .emit_event(Released { job_id, provider, amount });
                true
            } else {
                false
            }
        }

        /// Refunds funds back to the owner. Only the owner can refund.
        #[ink(message)]
        pub fn refund_to_owner(&mut self, job_id: u128) -> bool {
            let caller: H160 = self.env().caller().into();

            if let Some(mut e) = self.escrows.get(job_id) {
                if caller != e.owner || e.released || e.refunded {
                    return false;
                }

                let amount = e.amount;
                if amount == 0.into() {
                    return false;
                }

                if self.env().transfer(e.owner, amount).is_err() {
                    return false;
                }

                e.refunded = true;
                e.amount = 0.into();
                self.escrows.insert(job_id, &e);

                self.env()
                    .emit_event(Refunded { job_id, owner: e.owner, amount });
                true
            } else {
                false
            }
        }

        /// Returns the escrow record for a job, if any.
        #[ink(message)]
        pub fn get_escrow(&self, job_id: u128) -> Option<Escrow> {
            self.escrows.get(job_id)
        }

        /// Admin address (optional usage for future controls)
        #[ink(message)]
        pub fn get_admin(&self) -> H160 {
            self.admin
        }
    }

    #[ink(event)]
    pub struct Deposited {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub owner: H160,
        #[ink(topic)]
        pub provider: H160,
        pub amount: U256,
    }

    #[ink(event)]
    pub struct Released {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub provider: H160,
        pub amount: U256,
    }

    #[ink(event)]
    pub struct Refunded {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub owner: H160,
        pub amount: U256,
    }

    #[ink(event)]
    pub struct ProviderSet {
        #[ink(topic)]
        pub job_id: u128,
        #[ink(topic)]
        pub provider: H160,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn alice() -> H160 {
            H160::from([0x1; 20])
        }

        fn bob() -> H160 {
            H160::from([0x2; 20])
        }

        fn charlie() -> H160 {
            H160::from([0x3; 20])
        }

        #[ink::test]
        fn test_new() {
            let _escrow = PaymentEscrow::new();
        }

        #[ink::test]
        fn test_deposit_for_job() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice().into());
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(1000u128));

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

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice().into());
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(0u128));

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

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice().into());
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(1000u128));
            escrow.deposit_for_job(job_id, initial_provider);

            let result = escrow.set_provider(job_id, new_provider);
            assert!(result);

            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.provider, Some(new_provider));
        }

        #[ink::test]
        fn test_release_to_provider() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice().into());
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);

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

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice().into());
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);

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

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice().into());
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);

            escrow.refund_to_owner(job_id);

            let result = escrow.release_to_provider(job_id);
            assert!(!result);
        }

        #[ink::test]
        fn test_set_provider_wrong_owner_fails() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice().into());
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(bob().into());
            let result = escrow.set_provider(job_id, charlie());
            assert!(!result);
        }

        #[ink::test]
        fn test_cannot_overwrite_active_escrow() {
            let mut escrow = PaymentEscrow::new();
            let job_id = 1;
            let provider = bob();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice().into());
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(1000u128));
            escrow.deposit_for_job(job_id, provider);

            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(U256::from(2000u128));
            let result = escrow.deposit_for_job(job_id, charlie());
            assert!(!result);

            let stored_escrow = escrow.get_escrow(job_id).unwrap();
            assert_eq!(stored_escrow.amount, U256::from(1000u128));
            assert_eq!(stored_escrow.provider, Some(provider));
        }
    }
}
