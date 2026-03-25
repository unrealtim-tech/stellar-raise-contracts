//! # campaign_goal_minimum
//!
//! @title   CampaignGoalMinimum — Enforces minimum campaign goal thresholds.
//!
//! @notice  This module provides the logic to prevent campaigns from being
//!          created with goals below a defined minimum, ensuring realistic
//!          fundraising targets and improving security.

use soroban_sdk::{Address, Env};

/// Minimum allowed campaign goal.
pub const MIN_CAMPAIGN_GOAL: u64 = 100;

/// Creates a new campaign with goal validation.
///
/// # Parameters
/// - creator: campaign owner
/// - goal: funding target
///
/// # Security
/// Ensures goal meets minimum threshold and creator is authenticated.
pub fn create_campaign(env: Env, creator: Address, goal: u64) {
    creator.require_auth();

    if goal < MIN_CAMPAIGN_GOAL {
        panic!("Minimum campaign goal not met");
    }

    if goal == 0 {
        panic!("Campaign goal must be non-zero");
    }

    // Example storage logic (placeholder)
    // env.storage().instance().set(&DataKey::Creator, &creator);
    // env.storage().instance().set(&DataKey::Goal, &goal);
    
    // Emit event as requested
    env.events().publish(("campaign", "created"), (creator, goal));
}

/// Validates if a goal meets the minimum threshold.
///
/// # Parameters
/// - goal: the proposed goal
///
/// # Returns
/// true if the goal is secure and valid.
pub fn validate_goal(goal: u64) -> bool {
    goal >= MIN_CAMPAIGN_GOAL
}
