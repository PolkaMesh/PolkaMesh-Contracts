#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod compute_provider_registry {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::primitives::{H160, U256}; use ink::env::DefaultEnvironment;

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
    pub struct ProviderProfile {
        pub provider: H160,
        pub endpoint: String,
        pub compute_units: u64,
        pub hourly_rate: u128,
        pub registered_at: u64,
        pub is_active: bool,
        pub stake: u128,
        pub reputation_score: u32,
    }

    #[ink(storage)]
    pub struct ComputeProviderRegistry {
        /// provider address -> profile
        providers: Mapping<H160, ProviderProfile>,
        /// optional stake requirement for registration
        min_stake: u128,
        /// admin for future controls
        admin: H160,
        /// provider count for enumeration or stats
        provider_count: u64,
    }

    impl ComputeProviderRegistry {
        #[ink(constructor)]
        pub fn new(min_stake: u128) -> Self {
            let caller = Self::env().caller();
            let caller_h160: H160 = caller.into();
            Self {
                providers: Mapping::default(),
                min_stake,
                admin: caller_h160,
                provider_count: 0,
            }
        }

        /// Register as a compute provider. Requires attached stake >= min_stake.
        #[ink(message, payable)]
        pub fn register_provider(&mut self, endpoint: String, compute_units: u64, hourly_rate: u128) -> bool {
            let caller: H160 = self.env().caller().into();
            let stake_u256 = self.env().transferred_value();
            let stake = stake_u256.as_u128();
            if stake < self.min_stake { return false; }
            if self.providers.contains(caller) { return false; }

            let profile = ProviderProfile {
                provider: caller,
                endpoint,
                compute_units,
                hourly_rate,
                registered_at: self.env().block_timestamp(),
                is_active: true,
                stake,
                reputation_score: 100,
            };
            self.providers.insert(caller, &profile);
            self.provider_count = self.provider_count.saturating_add(1);
            self.env().emit_event(ProviderRegistered { provider: caller, stake, compute_units });
            true
        }

        /// Update provider's endpoint and hourly rate.
        #[ink(message)]
        pub fn update_provider(&mut self, endpoint: String, hourly_rate: Balance) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(mut profile) = self.providers.get(caller) {
                profile.endpoint = endpoint.clone();
                profile.hourly_rate = hourly_rate;
                self.providers.insert(caller, &profile);
                self.env().emit_event(ProviderUpdated { provider: caller, endpoint, hourly_rate });
                true
            } else { false }
        }

        /// Set provider as active or inactive.
        #[ink(message)]
        pub fn set_active(&mut self, is_active: bool) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(mut profile) = self.providers.get(caller) {
                profile.is_active = is_active;
                self.providers.insert(caller, &profile);
                self.env().emit_event(ProviderActiveChanged { provider: caller, is_active });
                true
            } else { false }
        }

        /// Increase provider's stake (payable).
        #[ink(message, payable)]
        pub fn add_stake(&mut self) -> bool {
            let caller: H160 = self.env().caller().into();
            let amount_u256 = self.env().transferred_value();
            let amount = amount_u256.as_u128();
            if amount == 0 { return false; }
            if let Some(mut profile) = self.providers.get(caller) {
                profile.stake = profile.stake.saturating_add(amount);
                self.providers.insert(caller, &profile);
                self.env().emit_event(StakeAdded { provider: caller, amount });
                true
            } else { false }
        }

        /// Withdraw stake (only if provider inactive or by admin).
        #[ink(message)]
        pub fn withdraw_stake(&mut self, amount: u128) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(mut profile) = self.providers.get(caller) {
                if profile.is_active && caller != self.admin { return false; }
                if profile.stake < amount { return false; }
                let amount_u256 = U256::from(amount);
                if self.env().transfer(caller, amount_u256).is_err() { return false; }
                profile.stake = profile.stake.saturating_sub(amount);
                self.providers.insert(caller, &profile);
                self.env().emit_event(StakeWithdrawn { provider: caller, amount });
                true
            } else { false }
        }

        /// Admin adjusts reputation score.
        #[ink(message)]
        pub fn set_reputation(&mut self, provider: H160, score: u32) -> bool {
            let caller: H160 = self.env().caller().into();
            if caller != self.admin { return false; }
            if let Some(mut profile) = self.providers.get(provider) {
                profile.reputation_score = score;
                self.providers.insert(provider, &profile);
                self.env().emit_event(ReputationUpdated { provider, score });
                true
            } else { false }
        }

        /// Get provider profile.
        #[ink(message)]
        pub fn get_provider(&self, provider: H160) -> Option<ProviderProfile> { self.providers.get(provider) }

        /// Get admin address.
        #[ink(message)]
        pub fn get_admin(&self) -> H160 { self.admin }

        /// Get provider count.
        #[ink(message)]
        pub fn get_provider_count(&self) -> u64 { self.provider_count }

        /// Get min stake requirement.
        #[ink(message)]
        pub fn get_min_stake(&self) -> u128 { self.min_stake }

        /// Admin sets min stake.
        #[ink(message)]
        pub fn set_min_stake(&mut self, new_min_stake: u128) -> bool {
            let caller: H160 = self.env().caller().into();
            if caller != self.admin { return false; }
            self.min_stake = new_min_stake;
            true
        }
    }

    #[ink(event)]
    pub struct ProviderRegistered { #[ink(topic)] pub provider: H160, pub stake: u128, pub compute_units: u64 }
    #[ink(event)]
    pub struct ProviderUpdated { #[ink(topic)] pub provider: H160, pub endpoint: String, pub hourly_rate: u128 }
    #[ink(event)]
    pub struct ProviderActiveChanged { #[ink(topic)] pub provider: H160, pub is_active: bool }
    #[ink(event)]
    pub struct StakeAdded { #[ink(topic)] pub provider: H160, pub amount: u128 }
    #[ink(event)]
    pub struct StakeWithdrawn { #[ink(topic)] pub provider: H160, pub amount: u128 }
    #[ink(event)]
    pub struct ReputationUpdated { #[ink(topic)] pub provider: H160, pub score: u32 }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn alice() -> H160 { H160::from([0x1; 20]) }
        fn bob() -> H160 { H160::from([0x2; 20]) }
        fn charlie() -> H160 { H160::from([0x3; 20]) }

        fn set_caller(account: H160) {
            ink::env::test::set_caller(account.into());
        }

        fn set_value(amount: u128) {
            ink::env::test::set_value_transferred(amount.into());
        }

        #[ink::test]
        fn new_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let registry = ComputeProviderRegistry::new(min_stake);
            
            assert_eq!(registry.get_min_stake(), min_stake);
            assert_eq!(registry.get_admin(), alice());
            assert_eq!(registry.get_provider_count(), 0);
        }

        #[ink::test]
        fn register_provider_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            
            assert!(registry.register_provider(
                "http://provider.com".to_string(),
                100, // compute_units
                50u128 // hourly_rate
            ));
            
            assert_eq!(registry.get_provider_count(), 1);
            
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.provider, bob());
            assert_eq!(profile.endpoint, "http://provider.com");
            assert_eq!(profile.compute_units, 100);
            assert_eq!(profile.hourly_rate, 50u128);
            assert_eq!(profile.is_active, true);
            assert_eq!(profile.stake, 1000u128);
            assert_eq!(profile.reputation_score, 100);
        }

        #[ink::test]
        fn register_provider_insufficient_stake_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(500); // Insufficient stake
            
            assert!(!registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            ));
            
            assert_eq!(registry.get_provider_count(), 0);
            assert!(registry.get_provider(bob()).is_none());
        }

        #[ink::test]
        fn register_provider_already_registered_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            
            // First registration
            assert!(registry.register_provider(
                "http://provider1.com".to_string(),
                100,
                50u128
            ));
            
            // Attempt second registration
            set_value(1000);
            assert!(!registry.register_provider(
                "http://provider2.com".to_string(),
                200,
                75u128
            ));
            
            // Original data should remain unchanged
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.endpoint, "http://provider1.com");
            assert_eq!(profile.compute_units, 100);
        }

        #[ink::test]
        fn update_provider_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            registry.register_provider(
                "http://old.com".to_string(),
                100,
                50u128
            );
            
            assert!(registry.update_provider(
                "http://new.com".to_string(),
                75u128
            ));
            
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.endpoint, "http://new.com");
            assert_eq!(profile.hourly_rate, 75u128);
            assert_eq!(profile.compute_units, 100); // Unchanged
        }

        #[ink::test]
        fn update_provider_not_registered_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            assert!(!registry.update_provider(
                "http://new.com".to_string(),
                75u128
            ));
        }

        #[ink::test]
        fn set_active_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            );
            
            // Provider starts as active
            assert_eq!(registry.get_provider(bob()).unwrap().is_active, true);
            
            // Set inactive
            assert!(registry.set_active(false));
            assert_eq!(registry.get_provider(bob()).unwrap().is_active, false);
            
            // Set active again
            assert!(registry.set_active(true));
            assert_eq!(registry.get_provider(bob()).unwrap().is_active, true);
        }

        #[ink::test]
        fn set_active_not_registered_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            assert!(!registry.set_active(false));
        }

        #[ink::test]
        fn add_stake_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            );
            
            set_value(500);
            assert!(registry.add_stake());
            
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.stake, 1500u128);
        }

        #[ink::test]
        fn add_stake_zero_amount_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            );
            
            set_value(0);
            assert!(!registry.add_stake());
            
            // Stake should remain unchanged
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.stake, 1000u128);
        }

        #[ink::test]
        fn add_stake_not_registered_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(500);
            assert!(!registry.add_stake());
        }

        #[ink::test]
        fn withdraw_stake_by_inactive_provider_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(2000);
            registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            );
            
            // Set provider inactive
            registry.set_active(false);
            
            // Withdraw partial stake
            assert!(registry.withdraw_stake(500u128));
            
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.stake, 1500u128);
        }

        #[ink::test]
        fn withdraw_stake_by_admin_as_provider_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            // Admin registers as a provider
            set_value(2000);
            registry.register_provider(
                "http://admin-provider.com".to_string(),
                100,
                50u128
            );
            
            // Admin can withdraw from their own account even if active
            assert!(registry.withdraw_stake(500u128));
            
            let profile = registry.get_provider(alice()).unwrap();
            assert_eq!(profile.stake, 1500u128);
        }

        #[ink::test]
        fn withdraw_stake_active_provider_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(2000);
            registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            );
            
            // Active provider cannot withdraw stake
            assert!(!registry.withdraw_stake(500u128));
            
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.stake, 2000u128);
        }

        #[ink::test]
        fn withdraw_stake_insufficient_balance_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            );
            
            registry.set_active(false);
            
            // Try to withdraw more than staked
            assert!(!registry.withdraw_stake(1500u128));
            
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.stake, 1000u128);
        }

        #[ink::test]
        fn set_reputation_by_admin_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            );
            
            set_caller(alice());
            assert!(registry.set_reputation(bob(), 85));
            
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.reputation_score, 85);
        }

        #[ink::test]
        fn set_reputation_not_admin_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            set_value(1000);
            registry.register_provider(
                "http://provider.com".to_string(),
                100,
                50u128
            );
            
            // Non-admin trying to set reputation
            set_caller(charlie());
            assert!(!registry.set_reputation(bob(), 85));
            
            let profile = registry.get_provider(bob()).unwrap();
            assert_eq!(profile.reputation_score, 100); // Unchanged
        }

        #[ink::test]
        fn set_reputation_nonexistent_provider_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            assert!(!registry.set_reputation(bob(), 85));
        }

        #[ink::test]
        fn set_min_stake_by_admin_works() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            assert!(registry.set_min_stake(2000u128));
            assert_eq!(registry.get_min_stake(), 2000u128);
        }

        #[ink::test]
        fn set_min_stake_not_admin_fails() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            set_caller(bob());
            assert!(!registry.set_min_stake(2000u128));
            assert_eq!(registry.get_min_stake(), 1000u128);
        }

        #[ink::test]
        fn get_provider_nonexistent() {
            set_caller(alice());
            let min_stake = 1000u128;
            let registry = ComputeProviderRegistry::new(min_stake);
            
            assert!(registry.get_provider(bob()).is_none());
        }

        #[ink::test]
        fn multiple_providers_registration() {
            set_caller(alice());
            let min_stake = 1000u128;
            let mut registry = ComputeProviderRegistry::new(min_stake);
            
            // Register first provider
            set_caller(bob());
            set_value(1000);
            assert!(registry.register_provider(
                "http://bob.com".to_string(),
                100,
                50u128
            ));
            
            // Register second provider
            set_caller(charlie());
            set_value(1500);
            assert!(registry.register_provider(
                "http://charlie.com".to_string(),
                200,
                75u128
            ));
            
            assert_eq!(registry.get_provider_count(), 2);
            
            let bob_profile = registry.get_provider(bob()).unwrap();
            let charlie_profile = registry.get_provider(charlie()).unwrap();
            
            assert_eq!(bob_profile.endpoint, "http://bob.com");
            assert_eq!(bob_profile.stake, 1000u128);
            assert_eq!(charlie_profile.endpoint, "http://charlie.com");
            assert_eq!(charlie_profile.stake, 1500u128);
        }
    }
}
