use super::*;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    token, Address, Env, IntoVal,
};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &contract_address.address()),
        token::StellarAssetClient::new(env, &contract_address.address()),
    )
}

#[test]
fn test_create_escrow_and_confirm_delivery() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);

    // Create token contract and mint tokens to buyer
    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &1000);

    // Register escrow contract
    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let escrow_id = 1u64;
    let amount = 500i128;

    // Create escrow - funds locked
    client.create_escrow(&escrow_id, &buyer, &seller, &amount, &token_client.address);

    // Verify escrow state
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.buyer, buyer);
    assert_eq!(escrow.seller, seller);
    assert_eq!(escrow.amount, amount);
    assert!(matches!(escrow.status, EscrowStatus::Funded));

    // Check balances after escrow creation
    assert_eq!(token_client.balance(&buyer), 500); // 1000 - 500
    assert_eq!(token_client.balance(&contract_id), 500);
    assert_eq!(token_client.balance(&seller), 0);

    // Buyer confirms delivery
    client.confirm_delivery(&escrow_id, &buyer);

    // Verify final state
    let escrow = client.get_escrow(&escrow_id);
    assert!(matches!(escrow.status, EscrowStatus::Completed));

    // Check seller received funds
    assert_eq!(token_client.balance(&buyer), 500);
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(token_client.balance(&seller), 500);
}

#[test]
#[should_panic(expected = "Only the buyer can confirm delivery")]
fn test_non_buyer_cannot_confirm() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let non_buyer = Address::generate(&env);
    let admin = Address::generate(&env);

    // Create token contract and mint tokens to buyer
    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &1000);

    // Register escrow contract
    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let escrow_id = 1u64;
    let amount = 500i128;

    // Create escrow
    client.create_escrow(&escrow_id, &buyer, &seller, &amount, &token_client.address);

    // Non-buyer tries to confirm - should panic
    client.confirm_delivery(&escrow_id, &non_buyer);
}

#[test]
#[should_panic(expected = "Escrow already completed")]
fn test_cannot_confirm_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let admin = Address::generate(&env);

    // Create token contract and mint tokens to buyer
    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &1000);

    // Register escrow contract
    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let escrow_id = 1u64;
    let amount = 500i128;

    // Create escrow
    client.create_escrow(&escrow_id, &buyer, &seller, &amount, &token_client.address);

    // First confirmation succeeds
    client.confirm_delivery(&escrow_id, &buyer);

    // Second confirmation should panic
    client.confirm_delivery(&escrow_id, &buyer);
}
