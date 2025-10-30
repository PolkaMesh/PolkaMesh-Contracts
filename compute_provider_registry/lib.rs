#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod compute_provider_registry {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
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
    pub struct ProviderProfile {
        pub provider: H160,
        pub endpoint: String,
        pub compute_units: u64,
        pub hourly_rate: U256,
        pub registered_at: u64,
        pub is_active: bool,
        pub stake: U256,
        pub reputation_score: u32,
    }

    #[ink(storage)]
    pub struct ComputeProviderRegistry {
        /// provider address -> profile
        providers: Mapping<H160, ProviderProfile>,
        /// optional stake requirement for registration
        min_stake: U256,
        /// admin for future controls
        admin: H160,
        /// provider count for enumeration or stats
        provider_count: u64,
    }

    impl ComputeProviderRegistry {
        #[ink(constructor)]
        pub fn new(min_stake: U256) -> Self {
            Self {
                providers: Mapping::default(),
                min_stake,
                admin: Self::env().caller(),
                provider_count: 0,
            }
        }

        /// Register as a compute provider. Requires attached stake >= min_stake.
        #[ink(message, payable)]
        pub fn register_provider(&mut self, endpoint: String, compute_units: u64, hourly_rate: U256) -> bool {
            let caller = self.env().caller();
            let stake = self.env().transferred_value();
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
        pub fn update_provider(&mut self, endpoint: String, hourly_rate: U256) -> bool {
            let caller = self.env().caller();
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
            let caller = self.env().caller();
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
            let caller = self.env().caller();
            let amount = self.env().transferred_value();
            if amount == 0.into() { return false; }
            if let Some(mut profile) = self.providers.get(caller) {
                profile.stake = profile.stake.saturating_add(amount);
                self.providers.insert(caller, &profile);
                self.env().emit_event(StakeAdded { provider: caller, amount });
                true
            } else { false }
        }

        /// Withdraw stake (only if provider inactive or by admin).
        #[ink(message)]
        pub fn withdraw_stake(&mut self, amount: U256) -> bool {
            let caller = self.env().caller();
            if let Some(mut profile) = self.providers.get(caller) {
                if profile.is_active && caller != self.admin { return false; }
                if profile.stake < amount { return false; }
                if self.env().transfer(caller, amount).is_err() { return false; }
                profile.stake = profile.stake.saturating_sub(amount);
                self.providers.insert(caller, &profile);
                self.env().emit_event(StakeWithdrawn { provider: caller, amount });
                true
            } else { false }
        }

        /// Admin adjusts reputation score.
        #[ink(message)]
        pub fn set_reputation(&mut self, provider: H160, score: u32) -> bool {
            if self.env().caller() != self.admin { return false; }
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
        pub fn get_min_stake(&self) -> U256 { self.min_stake }

        /// Admin sets min stake.
        #[ink(message)]
        pub fn set_min_stake(&mut self, new_min_stake: U256) -> bool {
            if self.env().caller() != self.admin { return false; }
            self.min_stake = new_min_stake;
            true
        }
    }

    #[ink(event)]
    pub struct ProviderRegistered { #[ink(topic)] pub provider: H160, pub stake: U256, pub compute_units: u64 }
    #[ink(event)]
    pub struct ProviderUpdated { #[ink(topic)] pub provider: H160, pub endpoint: String, pub hourly_rate: U256 }
    #[ink(event)]
    pub struct ProviderActiveChanged { #[ink(topic)] pub provider: H160, pub is_active: bool }
    #[ink(event)]
    pub struct StakeAdded { #[ink(topic)] pub provider: H160, pub amount: U256 }
    #[ink(event)]
    pub struct StakeWithdrawn { #[ink(topic)] pub provider: H160, pub amount: U256 }
    #[ink(event)]
    pub struct ReputationUpdated { #[ink(topic)] pub provider: H160, pub score: u32 }

    #[cfg(test)]
    mod tests {}
}
