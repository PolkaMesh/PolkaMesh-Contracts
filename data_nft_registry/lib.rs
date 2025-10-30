#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod data_nft_registry {
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
    pub struct DataNFT {
        pub token_id: u128,
        pub owner: H160,
        pub data_uri: String,
        pub privacy_level: u8,
        pub minted_at: u64,
        pub access_price: U256,
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
            Self {
                nfts: Mapping::default(),
                owner_nft_count: Mapping::default(),
                approvals: Mapping::default(),
                granted_access: Mapping::default(),
                total_supply: 0,
                admin: Self::env().caller(),
            }
        }

        /// Mint a new data NFT with metadata and privacy settings.
        #[ink(message, payable)]
        pub fn mint(&mut self, data_uri: String, privacy_level: u8, access_price: U256, is_transferable: bool) -> u128 {
            let caller = self.env().caller();
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
            let caller = self.env().caller();
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
            let caller = self.env().caller();
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
            let caller = self.env().caller();
            if let Some(nft) = self.nfts.get(token_id) {
                if nft.owner != caller { return false; }
                let payment = self.env().transferred_value();
                if payment < nft.access_price { return false; }

                self.granted_access.insert((token_id, grantee), &true);
                self.env().emit_event(AccessGranted { token_id, grantee, payment });
                true
            } else { false }
        }

        /// Revoke access from an address.
        #[ink(message)]
        pub fn revoke_access(&mut self, token_id: u128, grantee: H160) -> bool {
            let caller = self.env().caller();
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
            let caller = self.env().caller();
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
        pub fn update_access_price(&mut self, token_id: u128, new_price: U256) -> bool {
            let caller = self.env().caller();
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
            let caller = self.env().caller();
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
    pub struct AccessGranted { #[ink(topic)] pub token_id: u128, #[ink(topic)] pub grantee: H160, pub payment: U256 }
    #[ink(event)]
    pub struct AccessRevoked { #[ink(topic)] pub token_id: u128, #[ink(topic)] pub grantee: H160 }
    #[ink(event)]
    pub struct DataURIUpdated { #[ink(topic)] pub token_id: u128, pub new_uri: String }
    #[ink(event)]
    pub struct AccessPriceUpdated { #[ink(topic)] pub token_id: u128, pub new_price: U256 }
    #[ink(event)]
    pub struct NFTBurned { #[ink(topic)] pub token_id: u128, pub owner: H160 }

    #[cfg(test)]
    mod tests {}
}
