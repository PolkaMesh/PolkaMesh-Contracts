#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod data_nft_registry {
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
    pub struct DataNFT {
        pub token_id: u128,
        pub owner: H160,
        pub data_uri: String,
        pub privacy_level: u8,
        pub minted_at: u64,
        pub access_price: u128,
        pub is_transferable: bool,
    }

    #[ink(storage)]
    pub struct DataNftRegistry {
        /// token_id -> DataNFT
        nfts: Mapping<u128, DataNFT>,
        /// owner -> list of token_ids (simplified: count only)
        owner_nft_count: Mapping<H160, u128>,
        /// token_id -> approved address
        approvals: Mapping<u128, H160>,
        /// token_id -> granted access addresses
        granted_access: Mapping<(u128, H160), bool>,
        /// total minted count
        total_supply: u128,
        /// admin for controls
        admin: H160,
    }

    impl DataNftRegistry {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let caller_h160: H160 = caller.into();
            Self {
                nfts: Mapping::default(),
                owner_nft_count: Mapping::default(),
                approvals: Mapping::default(),
                granted_access: Mapping::default(),
                total_supply: 0,
                admin: caller_h160,
            }
        }

        /// Mint a new data NFT with metadata and privacy settings.
        #[ink(message, payable)]
        pub fn mint(&mut self, data_uri: String, privacy_level: u8, access_price: u128, is_transferable: bool) -> u128 {
            let caller: H160 = self.env().caller().into();
            let token_id = self.total_supply.saturating_add(1);

            let nft = DataNFT {
                token_id,
                owner: caller,
                data_uri: data_uri.clone(),
                privacy_level,
                minted_at: self.env().block_timestamp(),
                access_price,
                is_transferable,
            };

            self.nfts.insert(token_id, &nft);
            let count = self.owner_nft_count.get(caller).unwrap_or(0).saturating_add(1);
            self.owner_nft_count.insert(caller, &count);
            self.total_supply = token_id;

            self.env().emit_event(NFTMinted { token_id, owner: caller, data_uri, privacy_level });
            token_id
        }

        /// Transfer NFT to a new owner (only if is_transferable).
        #[ink(message)]
        pub fn transfer(&mut self, token_id: u128, to: H160) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(mut nft) = self.nfts.get(token_id) {
                if nft.owner != caller { return false; }
                if !nft.is_transferable { return false; }

                // Update counts
                let from_count = self.owner_nft_count.get(caller).unwrap_or(0).saturating_sub(1);
                self.owner_nft_count.insert(caller, &from_count);
                let to_count = self.owner_nft_count.get(to).unwrap_or(0).saturating_add(1);
                self.owner_nft_count.insert(to, &to_count);

                nft.owner = to;
                self.nfts.insert(token_id, &nft);
                self.approvals.remove(token_id);

                self.env().emit_event(NFTTransferred { token_id, from: caller, to });
                true
            } else { false }
        }

        /// Approve another address to transfer the NFT.
        #[ink(message)]
        pub fn approve(&mut self, token_id: u128, approved: H160) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(nft) = self.nfts.get(token_id) {
                if nft.owner != caller { return false; }
                self.approvals.insert(token_id, &approved);
                self.env().emit_event(NFTApproved { token_id, owner: caller, approved });
                true
            } else { false }
        }

        /// Grant access to an NFT for a specific address.
        #[ink(message, payable)]
        pub fn grant_access(&mut self, token_id: u128, grantee: H160) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(nft) = self.nfts.get(token_id) {
                if nft.owner != caller { return false; }
                let payment_u256 = self.env().transferred_value();
                let payment = payment_u256.as_u128();
                if payment < nft.access_price { return false; }

                self.granted_access.insert((token_id, grantee), &true);
                self.env().emit_event(AccessGranted { token_id, grantee, payment });
                true
            } else { false }
        }

        /// Revoke access from an address.
        #[ink(message)]
        pub fn revoke_access(&mut self, token_id: u128, grantee: H160) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(nft) = self.nfts.get(token_id) {
                if nft.owner != caller && caller != self.admin { return false; }
                self.granted_access.remove((token_id, grantee));
                self.env().emit_event(AccessRevoked { token_id, grantee });
                true
            } else { false }
        }

        /// Update data URI (owner-only).
        #[ink(message)]
        pub fn update_data_uri(&mut self, token_id: u128, new_uri: String) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(mut nft) = self.nfts.get(token_id) {
                if nft.owner != caller { return false; }
                nft.data_uri = new_uri.clone();
                self.nfts.insert(token_id, &nft);
                self.env().emit_event(DataURIUpdated { token_id, new_uri });
                true
            } else { false }
        }

        /// Update access price (owner-only).
        #[ink(message)]
        pub fn update_access_price(&mut self, token_id: u128, new_price: Balance) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(mut nft) = self.nfts.get(token_id) {
                if nft.owner != caller { return false; }
                nft.access_price = new_price;
                self.nfts.insert(token_id, &nft);
                self.env().emit_event(AccessPriceUpdated { token_id, new_price });
                true
            } else { false }
        }

        /// Burn NFT (destroy it).
        #[ink(message)]
        pub fn burn(&mut self, token_id: u128) -> bool {
            let caller: H160 = self.env().caller().into();
            if let Some(nft) = self.nfts.get(token_id) {
                if nft.owner != caller && caller != self.admin { return false; }
                self.nfts.remove(token_id);
                let count = self.owner_nft_count.get(nft.owner).unwrap_or(0).saturating_sub(1);
                self.owner_nft_count.insert(nft.owner, &count);
                self.env().emit_event(NFTBurned { token_id, owner: nft.owner });
                true
            } else { false }
        }

        /// Get NFT metadata.
        #[ink(message)]
        pub fn get_nft(&self, token_id: u128) -> Option<DataNFT> { self.nfts.get(token_id) }

        /// Get NFT count for an owner.
        #[ink(message)]
        pub fn balance_of(&self, owner: H160) -> u128 { self.owner_nft_count.get(owner).unwrap_or(0) }

        /// Get approved address for a token.
        #[ink(message)]
        pub fn get_approved(&self, token_id: u128) -> Option<H160> { self.approvals.get(token_id) }

        /// Check if address has access to a token.
        #[ink(message)]
        pub fn has_access(&self, token_id: u128, account: H160) -> bool {
            if let Some(nft) = self.nfts.get(token_id) {
                if nft.owner == account { return true; }
            }
            self.granted_access.get((token_id, account)).unwrap_or(false)
        }

        /// Get total supply.
        #[ink(message)]
        pub fn total_supply(&self) -> u128 { self.total_supply }

        /// Get admin address.
        #[ink(message)]
        pub fn get_admin(&self) -> H160 { self.admin }
    }

    #[ink(event)]
    pub struct NFTMinted { #[ink(topic)] pub token_id: u128, #[ink(topic)] pub owner: H160, pub data_uri: String, pub privacy_level: u8 }
    #[ink(event)]
    pub struct NFTTransferred { #[ink(topic)] pub token_id: u128, #[ink(topic)] pub from: H160, #[ink(topic)] pub to: H160 }
    #[ink(event)]
    pub struct NFTApproved { #[ink(topic)] pub token_id: u128, pub owner: H160, pub approved: H160 }
    #[ink(event)]
    pub struct AccessGranted { #[ink(topic)] pub token_id: u128, #[ink(topic)] pub grantee: H160, pub payment: u128 }
    #[ink(event)]
    pub struct AccessRevoked { #[ink(topic)] pub token_id: u128, #[ink(topic)] pub grantee: H160 }
    #[ink(event)]
    pub struct DataURIUpdated { #[ink(topic)] pub token_id: u128, pub new_uri: String }
    #[ink(event)]
    pub struct AccessPriceUpdated { #[ink(topic)] pub token_id: u128, pub new_price: u128 }
    #[ink(event)]
    pub struct NFTBurned { #[ink(topic)] pub token_id: u128, pub owner: H160 }

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

        #[ink::test]
        fn new_works() {
            set_caller(alice());
            let registry = DataNftRegistry::new();
            assert_eq!(registry.total_supply(), 0);
            assert_eq!(registry.get_admin(), alice());
        }

        #[ink::test]
        fn mint_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());

            let token_id = registry.mint(
                "ipfs://example".to_string(),
                1, // privacy_level
                100u128,
                true // is_transferable
            );

            assert_eq!(token_id, 1);
            assert_eq!(registry.total_supply(), 1);
            assert_eq!(registry.balance_of(alice()), 1);

            let nft = registry.get_nft(1).unwrap();
            assert_eq!(nft.owner, alice());
            assert_eq!(nft.data_uri, "ipfs://example");
            assert_eq!(nft.privacy_level, 1);
            assert_eq!(nft.access_price, 100u128);
            assert_eq!(nft.is_transferable, true);
        }

        #[ink::test]
        fn mint_multiple_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());

            let token_id1 = registry.mint("uri1".to_string(), 0, 50u128, true);
            let token_id2 = registry.mint("uri2".to_string(), 2, 200u128, false);
            
            assert_eq!(token_id1, 1);
            assert_eq!(token_id2, 2);
            assert_eq!(registry.total_supply(), 2);
            assert_eq!(registry.balance_of(alice()), 2);
            
            let nft1 = registry.get_nft(1).unwrap();
            let nft2 = registry.get_nft(2).unwrap();
            assert_eq!(nft1.is_transferable, true);
            assert_eq!(nft2.is_transferable, false);
        }

        #[ink::test]
        fn transfer_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            assert!(registry.transfer(token_id, bob()));
            
            assert_eq!(registry.balance_of(alice()), 0);
            assert_eq!(registry.balance_of(bob()), 1);
            
            let nft = registry.get_nft(token_id).unwrap();
            assert_eq!(nft.owner, bob());
        }

        #[ink::test]
        fn transfer_non_transferable_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, false);
            assert!(!registry.transfer(token_id, bob()));
            
            let nft = registry.get_nft(token_id).unwrap();
            assert_eq!(nft.owner, alice());
        }

        #[ink::test]
        fn transfer_not_owner_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            
            set_caller(bob());
            assert!(!registry.transfer(token_id, charlie()));
            
            let nft = registry.get_nft(token_id).unwrap();
            assert_eq!(nft.owner, alice());
        }

        #[ink::test]
        fn transfer_nonexistent_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            assert!(!registry.transfer(999, bob()));
        }

        #[ink::test]
        fn approve_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            assert!(registry.approve(token_id, bob()));
            
            assert_eq!(registry.get_approved(token_id), Some(bob()));
        }

        #[ink::test]
        fn approve_not_owner_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            
            set_caller(bob());
            assert!(!registry.approve(token_id, charlie()));
            
            assert_eq!(registry.get_approved(token_id), None);
        }

        #[ink::test]
        fn approve_nonexistent_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            assert!(!registry.approve(999, bob()));
        }

        #[ink::test]
        fn grant_access_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let access_price = 100u128;
            let token_id = registry.mint("uri".to_string(), 1, access_price, true);
            
            set_value(100);
            assert!(registry.grant_access(token_id, bob()));
            assert!(registry.has_access(token_id, bob()));
        }

        #[ink::test]
        fn grant_access_insufficient_payment_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let access_price = 100u128;
            let token_id = registry.mint("uri".to_string(), 1, access_price, true);
            
            set_value(50); // Insufficient payment
            assert!(!registry.grant_access(token_id, bob()));
            assert!(!registry.has_access(token_id, bob()));
        }

        #[ink::test]
        fn grant_access_not_owner_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let access_price = 100u128;
            let token_id = registry.mint("uri".to_string(), 1, access_price, true);
            
            set_caller(bob());
            set_value(100);
            assert!(!registry.grant_access(token_id, charlie()));
            assert!(!registry.has_access(token_id, charlie()));
        }

        #[ink::test]
        fn revoke_access_by_owner_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 1, 100u128, true);
            
            set_value(100);
            assert!(registry.grant_access(token_id, bob()));
            assert!(registry.has_access(token_id, bob()));
            
            assert!(registry.revoke_access(token_id, bob()));
            assert!(!registry.has_access(token_id, bob()));
        }

        #[ink::test]
        fn revoke_access_by_admin_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice()); // Alice is admin
            
            let token_id = registry.mint("uri".to_string(), 1, 100u128, true);
            
            set_value(100);
            assert!(registry.grant_access(token_id, bob()));
            assert!(registry.has_access(token_id, bob()));
            
            // Admin can revoke access even if not owner
            set_caller(alice());
            assert!(registry.revoke_access(token_id, bob()));
            assert!(!registry.has_access(token_id, bob()));
        }

        #[ink::test]
        fn revoke_access_unauthorized_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 1, 100u128, true);
            
            set_value(100);
            assert!(registry.grant_access(token_id, bob()));
            
            set_caller(charlie()); // Not owner or admin
            assert!(!registry.revoke_access(token_id, bob()));
            assert!(registry.has_access(token_id, bob()));
        }

        #[ink::test]
        fn update_data_uri_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("old_uri".to_string(), 0, 100u128, true);
            assert!(registry.update_data_uri(token_id, "new_uri".to_string()));
            
            let nft = registry.get_nft(token_id).unwrap();
            assert_eq!(nft.data_uri, "new_uri");
        }

        #[ink::test]
        fn update_data_uri_not_owner_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            
            set_caller(bob());
            assert!(!registry.update_data_uri(token_id, "new_uri".to_string()));
            
            let nft = registry.get_nft(token_id).unwrap();
            assert_eq!(nft.data_uri, "uri");
        }

        #[ink::test]
        fn update_access_price_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            assert!(registry.update_access_price(token_id, 200u128));
            
            let nft = registry.get_nft(token_id).unwrap();
            assert_eq!(nft.access_price, 200u128);
        }

        #[ink::test]
        fn update_access_price_not_owner_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            
            set_caller(bob());
            assert!(!registry.update_access_price(token_id, 200u128));
            
            let nft = registry.get_nft(token_id).unwrap();
            assert_eq!(nft.access_price, 100u128);
        }

        #[ink::test]
        fn burn_by_owner_works() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            assert_eq!(registry.balance_of(alice()), 1);
            
            assert!(registry.burn(token_id));
            assert_eq!(registry.balance_of(alice()), 0);
            assert!(registry.get_nft(token_id).is_none());
        }

        #[ink::test]
        fn burn_by_admin_works() {
            set_caller(alice()); // Alice is admin
            let mut registry = DataNftRegistry::new();
            
            set_caller(bob());
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            assert_eq!(registry.balance_of(bob()), 1);
            
            set_caller(alice()); // Admin burns
            assert!(registry.burn(token_id));
            assert_eq!(registry.balance_of(bob()), 0);
            assert!(registry.get_nft(token_id).is_none());
        }

        #[ink::test]
        fn burn_unauthorized_fails() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 0, 100u128, true);
            
            set_caller(bob());
            assert!(!registry.burn(token_id));
            assert_eq!(registry.balance_of(alice()), 1);
            assert!(registry.get_nft(token_id).is_some());
        }

        #[ink::test]
        fn has_access_owner_always_true() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 1, 100u128, true);
            assert!(registry.has_access(token_id, alice()));
        }

        #[ink::test]
        fn has_access_non_owner_without_grant_false() {
            let mut registry = DataNftRegistry::new();
            set_caller(alice());
            
            let token_id = registry.mint("uri".to_string(), 1, 100u128, true);
            assert!(!registry.has_access(token_id, bob()));
        }

        #[ink::test]
        fn balance_of_empty_account() {
            let registry = DataNftRegistry::new();
            assert_eq!(registry.balance_of(alice()), 0);
        }

        #[ink::test]
        fn get_nft_nonexistent() {
            let registry = DataNftRegistry::new();
            assert!(registry.get_nft(999).is_none());
        }

        #[ink::test]
        fn get_approved_nonexistent() {
            let registry = DataNftRegistry::new();
            assert!(registry.get_approved(999).is_none());
        }
    }
}
