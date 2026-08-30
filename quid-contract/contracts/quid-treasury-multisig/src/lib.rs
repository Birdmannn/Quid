#![no_std]

use soroban_sdk::{contract, contractclient, contractevent, contractimpl, Address, Env};

mod error;
mod types;

pub use error::TreasuryError;
pub use types::TransferProposal;
use types::{DataKey, Signers};

const PROPOSAL_TTL_LEDGERS: u32 = 5_184_000;

/// The subset of `quid-fee-collector` used to release protocol fees.
#[contractclient(name = "FeeCollectorClient")]
pub trait FeeCollector {
    fn withdraw_fees(env: Env, caller: Address, token: Address, to: Address, amount: i128);
}

#[contractevent(topics = ["treasury", "proposed"])]
pub struct TransferProposedEvent {
    pub proposal_id: u64,
    pub fee_collector: Address,
    pub token: Address,
    pub to: Address,
    pub amount: i128,
    pub expires_at: u64,
}

#[contractevent(topics = ["treasury", "approved"])]
pub struct ProposalApprovedEvent {
    pub proposal_id: u64,
    pub signer: Address,
    pub approvals: u32,
}

#[contractevent(topics = ["treasury", "executed"])]
pub struct TransferExecutedEvent {
    pub proposal_id: u64,
}

#[contractevent(topics = ["treasury", "threshold"], data_format = "single-value")]
pub struct ThresholdUpdatedEvent {
    pub threshold: u32,
}

/// Threshold-controlled administrator for one or more protocol fee collectors.
#[contract]
pub struct QuidTreasuryMultisigContract;

#[contractimpl]
impl QuidTreasuryMultisigContract {
    /// Initialize once. Every initial signer must authorize the signer set.
    pub fn initialize(env: Env, signers: Signers, threshold: u32) -> Result<(), TreasuryError> {
        if env.storage().instance().has(&DataKey::Signers) {
            return Err(TreasuryError::AlreadyInitialized);
        }
        Self::validate_signers(&signers, threshold)?;

        for signer in signers.iter() {
            signer.require_auth();
        }

        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        Ok(())
    }

    pub fn get_signers(env: Env) -> Result<Signers, TreasuryError> {
        env.storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(TreasuryError::NotInitialized)
    }

    pub fn get_threshold(env: Env) -> Result<u32, TreasuryError> {
        env.storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(TreasuryError::NotInitialized)
    }

    /// Update the threshold. All configured signers must authorize this change.
    pub fn set_threshold(env: Env, threshold: u32) -> Result<(), TreasuryError> {
        let signers = Self::get_signers(env.clone())?;
        Self::validate_threshold(&signers, threshold)?;

        for signer in signers.iter() {
            signer.require_auth();
        }

        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        ThresholdUpdatedEvent { threshold }.publish(&env);
        Ok(())
    }

    pub fn propose(
        env: Env,
        proposer: Address,
        fee_collector: Address,
        token: Address,
        to: Address,
        amount: i128,
        expires_at: u64,
    ) -> Result<u64, TreasuryError> {
        Self::require_signer(&env, &proposer)?;
        if amount <= 0 {
            return Err(TreasuryError::InvalidAmount);
        }
        if expires_at <= env.ledger().timestamp() {
            return Err(TreasuryError::InvalidExpiry);
        }

        let proposal_id = Self::proposal_count(env.clone())
            .checked_add(1)
            .ok_or(TreasuryError::Overflow)?;
        let proposal = TransferProposal {
            id: proposal_id,
            fee_collector: fee_collector.clone(),
            token: token.clone(),
            to: to.clone(),
            amount,
            expires_at,
            approvals: 0,
            executed: false,
        };

        Self::store_proposal(&env, &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &proposal_id);
        TransferProposedEvent {
            proposal_id,
            fee_collector,
            token,
            to,
            amount,
            expires_at,
        }
        .publish(&env);
        Ok(proposal_id)
    }

    pub fn approve(env: Env, signer: Address, proposal_id: u64) -> Result<(), TreasuryError> {
        Self::require_signer(&env, &signer)?;
        let mut proposal = Self::get_proposal(env.clone(), proposal_id)?;
        Self::require_pending(&env, &proposal)?;

        let approval_key = DataKey::Approval(proposal_id, signer.clone());
        if env.storage().persistent().has(&approval_key) {
            return Err(TreasuryError::AlreadyApproved);
        }

        proposal.approvals = proposal
            .approvals
            .checked_add(1)
            .ok_or(TreasuryError::Overflow)?;
        Self::store_proposal(&env, &proposal);
        env.storage().persistent().set(&approval_key, &true);
        env.storage().persistent().extend_ttl(
            &approval_key,
            PROPOSAL_TTL_LEDGERS,
            PROPOSAL_TTL_LEDGERS,
        );

        ProposalApprovedEvent {
            proposal_id,
            signer,
            approvals: proposal.approvals,
        }
        .publish(&env);
        Ok(())
    }

    /// Anyone may submit a sufficiently approved proposal for execution.
    pub fn execute_transfer(env: Env, proposal_id: u64) -> Result<(), TreasuryError> {
        let mut proposal = Self::get_proposal(env.clone(), proposal_id)?;
        Self::require_pending(&env, &proposal)?;
        if proposal.approvals < Self::get_threshold(env.clone())? {
            return Err(TreasuryError::InsufficientApprovals);
        }

        let treasury = env.current_contract_address();
        FeeCollectorClient::new(&env, &proposal.fee_collector).withdraw_fees(
            &treasury,
            &proposal.token,
            &proposal.to,
            &proposal.amount,
        );

        proposal.executed = true;
        Self::store_proposal(&env, &proposal);
        TransferExecutedEvent { proposal_id }.publish(&env);
        Ok(())
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<TransferProposal, TreasuryError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(TreasuryError::ProposalNotFound)
    }

    pub fn is_approved(env: Env, proposal_id: u64, signer: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Approval(proposal_id, signer))
    }

    fn proposal_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }

    fn require_signer(env: &Env, signer: &Address) -> Result<(), TreasuryError> {
        signer.require_auth();
        if !Self::get_signers(env.clone())?.contains(signer) {
            return Err(TreasuryError::NotSigner);
        }
        Ok(())
    }

    fn require_pending(env: &Env, proposal: &TransferProposal) -> Result<(), TreasuryError> {
        if proposal.executed {
            return Err(TreasuryError::ProposalExecuted);
        }
        if env.ledger().timestamp() >= proposal.expires_at {
            return Err(TreasuryError::ProposalExpired);
        }
        Ok(())
    }

    fn validate_signers(signers: &Signers, threshold: u32) -> Result<(), TreasuryError> {
        if signers.is_empty() {
            return Err(TreasuryError::InvalidSigners);
        }
        for signer in signers.iter() {
            let mut occurrences = 0u32;
            for candidate in signers.iter() {
                if signer == candidate {
                    occurrences += 1;
                }
            }
            if occurrences > 1 {
                return Err(TreasuryError::InvalidSigners);
            }
        }
        Self::validate_threshold(signers, threshold)
    }

    fn validate_threshold(signers: &Signers, threshold: u32) -> Result<(), TreasuryError> {
        if threshold == 0 || threshold > signers.len() {
            return Err(TreasuryError::InvalidThreshold);
        }
        Ok(())
    }

    fn store_proposal(env: &Env, proposal: &TransferProposal) {
        let key = DataKey::Proposal(proposal.id);
        env.storage().persistent().set(&key, proposal);
        env.storage()
            .persistent()
            .extend_ttl(&key, PROPOSAL_TTL_LEDGERS, PROPOSAL_TTL_LEDGERS);
    }
}

mod test;
