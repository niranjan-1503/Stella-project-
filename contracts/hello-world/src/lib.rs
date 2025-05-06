#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, Address, symbol_short, String};

// Tip data structure
#[contracttype]
#[derive(Clone)]
pub struct Tip {
    pub tip_id: u64,
    pub tipper: Address,
    pub creator: Address,
    pub amount: u64,
    pub message: String,
    pub timestamp: u64,
}

// Creator profile data structure
#[contracttype]
#[derive(Clone)]
pub struct CreatorProfile {
    pub address: Address,
    pub total_tips: u64,
    pub total_received: u64,
    pub tip_count: u64,
}

// Contract storage keys
const TIP_COUNT: Symbol = symbol_short!("TIP_COUNT");

// Mapping tip ID to tip data
#[contracttype]
pub enum TipMap {
    Tip(u64)
}

// Mapping creator address to profile
#[contracttype]
pub enum CreatorMap {
    Profile(Address)
}

#[contract]
pub struct CryptoTipJarContract;

#[contractimpl]
impl CryptoTipJarContract {
    // Send a tip to a creator
    pub fn send_tip(env: Env, tipper: Address, creator: Address, amount: u64, message: String) -> u64 {
        // Get the current tip count
        let mut tip_count = env.storage().instance().get(&TIP_COUNT).unwrap_or(0);
        tip_count += 1;
        
        // Create the tip record
        let tip = Tip {
            tip_id: tip_count,
            tipper: tipper,
            creator: creator.clone(),
            amount: amount,
            message: message,
            timestamp: env.ledger().timestamp(),
        };
        
        // Store the tip record
        env.storage().instance().set(&TipMap::Tip(tip_count), &tip);
        env.storage().instance().set(&TIP_COUNT, &tip_count);
        
        // Update creator profile
        let mut profile = Self::get_creator_profile(env.clone(), creator.clone());
        profile.address = creator;
        profile.total_tips += amount;
        profile.tip_count += 1;
        env.storage().instance().set(&CreatorMap::Profile(profile.address.clone()), &profile);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Tip {} sent with amount {}", tip_count, amount);
        
        tip_count
    }
    
    // Get tip details
    pub fn get_tip(env: Env, tip_id: u64) -> Tip {
        env.storage().instance().get(&TipMap::Tip(tip_id))
            .unwrap_or_else(|| panic!("Tip not found"))
    }
    
    // Get creator profile
    pub fn get_creator_profile(env: Env, creator: Address) -> CreatorProfile {
        env.storage().instance().get(&CreatorMap::Profile(creator.clone())).unwrap_or(CreatorProfile {
            address: creator,
            total_tips: 0,
            total_received: 0,
            tip_count: 0,
        })
    }
    
    // Withdraw tips (simplified implementation)
    pub fn withdraw(env: Env, creator: Address) -> u64 {
        let mut profile = Self::get_creator_profile(env.clone(), creator.clone());
        
        // Calculate available amount (total_tips - total_received)
        let available = profile.total_tips - profile.total_received;
        
        if available <= 0 {
            log!(&env, "No funds available to withdraw");
            panic!("No funds available to withdraw");
        }
        
        // Update creator profile
        profile.total_received = profile.total_tips;
        env.storage().instance().set(&CreatorMap::Profile(creator), &profile);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Withdrew {} tokens", available);
        
        available
    }
}