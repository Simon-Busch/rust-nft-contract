use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,
};

pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::approval::*;
pub use crate::royalty::*;

mod approval; 
mod enumeration; 
mod metadata; 
mod mint; 
mod nft_core; 
mod royalty; 
/*
  -Add the token ID into the set of tokens that the receiver owns. This will be done on the tokens_per_owner field.
  -Create a token object and map the token ID to that token object in the tokens_by_id field.
  -Map the token ID to it's metadata using the token_metadata_by_id.
*/
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
  //contract owner
  pub owner_id: AccountId,

  //keeps track of all the token IDs for a given account
  pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

  //keeps track of the token struct for a given token ID
  pub tokens_by_id: LookupMap<TokenId, Token>,

  //keeps track of the token metadata for a given token ID
  pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

  //keeps track of the metadata for the contract
  pub metadata: LazyOption<NFTContractMetadata>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
  TokensPerOwner,
  TokenPerOwnerInner { account_id_hash: CryptoHash },
  TokensById,
  TokenMetadataById,
  NFTContractMetadata,
  TokensPerType,
  TokensPerTypeInner { token_type_hash: CryptoHash },
  TokenTypesLocked,
}

#[near_bindgen]
impl Contract {
  /*
    initialization function (can only be called once).
    this initializes the contract with default metadata so the
    user doesn't have to manually type metadata.
  */
  #[init]
  pub fn new_default_meta(owner_id: AccountId) -> Self {
  //calls the other function "new: with some default metadata and the owner_id passed in 
  Self::new(
    owner_id,
    NFTContractMetadata {
      spec: "nft-1.0.0".to_string(),
      name: "NFT Tutorial Contract".to_string(),
      symbol: "GOTEAM".to_string(),
      icon: None,
      base_uri: None,
      reference: None,
      reference_hash: None,
    },
  )
}

  /*
    initialization function (can only be called once).
    this initializes the contract with metadata that was passed in and
    the owner_id. 
  */
  #[init]
  pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
    //create a variable of type Self with all the fields initialized. 
    let this = Self {
      //Storage keys are simply the prefixes used for the collections. This helps avoid data collision
      tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
      tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
      token_metadata_by_id: UnorderedMap::new(
        StorageKey::TokenMetadataById.try_to_vec().unwrap(),
      ),
      //set the owner_id field equal to the passed in owner_id. 
      owner_id,
      metadata: LazyOption::new(
        StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
        Some(&metadata),
      ),
    };

    //return the Contract object
    this
  }

  #[payable]
  pub fn nft_mint(
    &mut self,
    token_id: TokenId,
    metadata: TokenMetadata,
    receiver_id: AccountId,
  ) {
    //measure the initial storage being used on the contract
    let initial_storage_usage = env::storage_usage();

    //specify the token struct that contains the owner ID 
    let token = Token {
        //set the owner ID equal to the receiver ID passed into the function
        owner_id: receiver_id,
    };

    //insert the token ID and token struct and make sure that the token doesn't exist
    assert!(
        self.tokens_by_id.insert(&token_id, &token).is_none(),
        "Token already exists"
    );

    //insert the token ID and metadata
    self.token_metadata_by_id.insert(&token_id, &metadata);

    //call the internal method for adding the token to the owner
    self.internal_add_token_to_owner(&token.owner_id, &token_id);

    //calculate the required storage which was the used - initial
    let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

    //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
    refund_deposit(required_storage_in_bytes);
  }
}