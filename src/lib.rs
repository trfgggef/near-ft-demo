use near_contract_standards::fungible_token::events::{FtBurn, FtMint};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::{impl_fungible_token_core, impl_fungible_token_storage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{
    env, near_bindgen, require, AccountId, Balance, BorshStorageKey, PanicOnDefault, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    tokens: FungibleToken,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FungibleToken,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn init(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            tokens: FungibleToken::new(StorageKey::FungibleToken),
        }
    }

    pub fn mint(&mut self, account_id: AccountId, amount: U128, memo: Option<String>) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Only contract owner can call this method."
        );
        self.internal_mint(&account_id, amount.0, memo);
    }

    pub fn burn(&mut self, account_id: AccountId, amount: U128, memo: Option<String>) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Only contract owner can call this method."
        );
        self.internal_burn(&account_id, amount.0, memo);
    }
}

// ft_transfer
// ft_transfer_call
// ft_total_supply
// ft_balance_of
// ft_resolve_transfer
impl_fungible_token_core!(Contract, tokens);

// storage_deposit
// storage_withdraw
// storage_unregister
// storage_balance_bounds
// storage_balance_of
impl_fungible_token_storage!(Contract, tokens);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Hello Fungible Token".to_string(),
            symbol: "HelloFT".to_string(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}

impl Contract {
    pub(crate) fn internal_mint(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        if !self.tokens.accounts.contains_key(account_id) {
            self.tokens.internal_register_account(account_id);
        }

        // mint
        self.tokens.internal_deposit(account_id, amount);

        FtMint {
            owner_id: account_id,
            amount: &U128(amount),
            memo: memo.as_deref(),
        }
        .emit();
    }

    pub(crate) fn internal_burn(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        // burn
        self.tokens.internal_withdraw(account_id, amount);

        FtBurn {
            owner_id: account_id,
            amount: &U128(amount),
            memo: memo.as_deref(),
        }
        .emit();
    }
}

#[cfg(test)]
mod test {
    use crate::Contract;
    use near_contract_standards::fungible_token::core::FungibleTokenCore;
    use near_contract_standards::storage_management::StorageManagement;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Balance, ONE_YOCTO};

    fn owner() -> AccountId {
        "owner.near".parse().unwrap()
    }

    fn alice() -> AccountId {
        "alice.near".parse().unwrap()
    }

    fn bob() -> AccountId {
        "bob.near".parse().unwrap()
    }

    const ONE_TOKEN: Balance = 1_000_000_000_000_000_000;

    #[test]
    fn test_mint_transfer_burn() {
        let mut contract = Contract::init(owner());

        testing_env!(VMContextBuilder::new()
            .predecessor_account_id(owner())
            .build());

        contract.mint(bob(), U128(1000 * ONE_TOKEN), None);

        assert_eq!(contract.ft_balance_of(bob()), U128(1000 * ONE_TOKEN));
        assert_eq!(contract.ft_total_supply(), U128(1000 * ONE_TOKEN));

        let storage_balance_bounds = contract.storage_balance_bounds();

        testing_env!(VMContextBuilder::new()
            .predecessor_account_id(bob())
            .attached_deposit(storage_balance_bounds.min.0)
            .build());

        contract.storage_deposit(Some(alice()), None);

        testing_env!(VMContextBuilder::new()
            .predecessor_account_id(bob())
            .attached_deposit(ONE_YOCTO)
            .build());

        contract.ft_transfer(alice(), U128(200 * ONE_TOKEN), None);

        assert_eq!(contract.ft_balance_of(bob()), U128(800 * ONE_TOKEN));
        assert_eq!(contract.ft_balance_of(alice()), U128(200 * ONE_TOKEN));
        assert_eq!(contract.ft_total_supply(), U128(1000 * ONE_TOKEN));


        testing_env!(VMContextBuilder::new()
            .predecessor_account_id(owner())
            .build());

        contract.burn(bob(), U128(300 * ONE_TOKEN), None);

        assert_eq!(contract.ft_balance_of(bob()), U128(500 * ONE_TOKEN));
        assert_eq!(contract.ft_balance_of(alice()), U128(200 * ONE_TOKEN));
        assert_eq!(contract.ft_total_supply(), U128(700 * ONE_TOKEN));
    }
}