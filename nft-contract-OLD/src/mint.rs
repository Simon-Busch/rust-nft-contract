use crate::*;
  /*
    -token_id: the ID of the token you're minting (as a string).
    -metadata: the metadata for the token that you're minting (of type TokenMetadata which is found in the metadata.rs file).
    -receiver_id: specifies who the owner of the token will be.
    -Behind the scenes, the function will:

    -Calculate the initial storage before adding anything to the contract
    -Create a Token object with the owner ID
    -Link the token ID to the newly created token object by inserting them into the tokens_by_id field.
    -Link the token ID to the passed in metadata by inserting them into the token_metadata_by_id field.
    -Add the token ID to the list of tokens that the owner owns by calling the internal_add_token_to_owner function.
    -Calculate the final and net storage to make sure that the user has attached enough NEAR to the call in order to cover those costs.
  */
#[near_bindgen]
impl Contract {
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