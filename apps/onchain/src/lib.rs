#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[derive(Clone)]
#[contracttype]
pub enum EscrowStatus {
    Pending,
    Funded,
    Completed,
    Disputed,
}

#[derive(Clone)]
#[contracttype]
pub struct Escrow {
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub token: Address,
    pub status: EscrowStatus,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Create a new escrow and lock funds
    pub fn create_escrow(
        env: Env,
        escrow_id: u64,
        buyer: Address,
        seller: Address,
        amount: i128,
        token: Address,
    ) {
        buyer.require_auth();

        let escrow = Escrow {
            buyer: buyer.clone(),
            seller,
            amount,
            token: token.clone(),
            status: EscrowStatus::Pending,
        };

        env.storage().instance().set(&escrow_id, &escrow);

        // Transfer funds from buyer to contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        // Update status to Funded
        let mut updated_escrow = escrow;
        updated_escrow.status = EscrowStatus::Funded;
        env.storage().instance().set(&escrow_id, &updated_escrow);
    }

    /// Buyer confirms delivery and releases funds to seller
    pub fn confirm_delivery(env: Env, escrow_id: u64, buyer: Address) {
        // Security Check: Verify buyer authorization
        buyer.require_auth();

        // Retrieve escrow from storage
        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&escrow_id)
            .expect("Escrow not found");

        // Security Check: Ensure caller is the assigned buyer
        if escrow.buyer != buyer {
            panic!("Only the buyer can confirm delivery");
        }

        // State Check: Ensure status is Funded
        match escrow.status {
            EscrowStatus::Funded => {},
            EscrowStatus::Completed => panic!("Escrow already completed"),
            EscrowStatus::Disputed => panic!("Escrow is disputed"),
            EscrowStatus::Pending => panic!("Escrow not funded yet"),
        }

        // Execute token transfer from contract to seller
        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.seller,
            &escrow.amount,
        );

        // Update and save the new status
        escrow.status = EscrowStatus::Completed;
        env.storage().instance().set(&escrow_id, &escrow);
    }

    /// Get escrow details
    pub fn get_escrow(env: Env, escrow_id: u64) -> Escrow {
        env.storage()
            .instance()
            .get(&escrow_id)
            .expect("Escrow not found")
    }
}

#[cfg(test)]
mod test;
