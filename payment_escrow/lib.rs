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
    mod tests {}
}
