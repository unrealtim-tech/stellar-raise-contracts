//! Proptest Generator Boundary Contract
//!
//! This contract provides the source of truth for all boundary conditions and validation
//! constants used by the crowdfunding platform's property-based tests. Exposing these
//! via a contract allows off-chain scripts and other contracts to dynamically query
//! current safe operating limits.
//!
//! ## Security Model
//!
//! - **Immutable Boundaries**: Constants are defined at compile-time to ensure test stability.
//! - **Public Transparency**: All limits are publicly readable for auditability.
//! - **Safety Guards**: Includes logic to clamp and validate inputs against platform-wide floors and caps.
//! - **Overflow Protection**: Uses `saturating_mul` and range checks to prevent integer overflow.
//! - **Division Safety**: All divisions are guarded against zero denominators.

#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Symbol};

// ── Constants ────────────────────────────────────────────────────────────────

/// Minimum deadline offset in seconds (~17 minutes).
/// Prevents flaky tests and meaningless campaigns.
pub const DEADLINE_OFFSET_MIN: u64 = 1_000;

/// Maximum deadline offset in seconds (~11.5 days).
/// Avoids u64 overflow when added to ledger timestamps.
pub const DEADLINE_OFFSET_MAX: u64 = 1_000_000;

/// Minimum valid goal amount.
/// Prevents division-by-zero in progress calculations.
pub const GOAL_MIN: i128 = 1_000;

/// Maximum goal amount for test generations (10 XLM).
/// Keeps tests fast while covering large campaigns.
pub const GOAL_MAX: i128 = 100_000_000;

/// Absolute floor for contribution amounts.
/// Prevents zero-value contributions from polluting ledger state.
pub const MIN_CONTRIBUTION_FLOOR: i128 = 1;

/// Progress basis points cap (100%).
/// Ensures frontend never displays >100% funded.
pub const PROGRESS_BPS_CAP: u32 = 10_000;

/// Fee basis points cap (100%).
/// Ensures fees cannot exceed 100% of contribution.
pub const FEE_BPS_CAP: u32 = 10_000;

/// Minimum proptest case count.
/// Below this, boundary-adjacent values are rarely sampled.
pub const PROPTEST_CASES_MIN: u32 = 32;

/// Maximum proptest case count.
/// Balances coverage with CI execution time.
pub const PROPTEST_CASES_MAX: u32 = 256;

/// Maximum generator batch size.
/// Prevents worst-case memory/gas spikes in test scaffolds.
pub const GENERATOR_BATCH_MAX: u32 = 512;

#[contract]
pub struct ProptestGeneratorBoundary;

#[contractimpl]
impl ProptestGeneratorBoundary {
    /// Returns the minimum deadline offset in seconds.
    ///
    /// @notice Campaigns shorter than this may experience timing races in CI.
    pub fn deadline_offset_min(_env: Env) -> u64 {
        DEADLINE_OFFSET_MIN
    }

    /// Returns the maximum deadline offset in seconds.
    ///
    /// @notice Prevents timestamp overflow when added to ledger time.
    pub fn deadline_offset_max(_env: Env) -> u64 {
        DEADLINE_OFFSET_MAX
    }

    /// Returns the minimum valid goal amount.
    ///
    /// @notice Prevents division-by-zero in progress_bps calculations.
    pub fn goal_min(_env: Env) -> i128 {
        GOAL_MIN
    }

    /// Returns the maximum goal amount for test generations.
    ///
    /// @notice Represents 10 XLM; keeps tests fast while covering large campaigns.
    pub fn goal_max(_env: Env) -> i128 {
        GOAL_MAX
    }

    /// Returns the absolute floor for contribution amounts.
    ///
    /// @notice Prevents zero-value contributions from polluting ledger state.
    pub fn min_contribution_floor(_env: Env) -> i128 {
        MIN_CONTRIBUTION_FLOOR
    }

    /// Validates that a deadline offset is within safe bounds.
    ///
    /// @notice Returns true if offset ∈ [DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX].
    /// @dev Rejects values that cause timestamp overflow or campaigns too short
    ///      for meaningful frontend display.
    pub fn is_valid_deadline_offset(_env: Env, offset: u64) -> bool {
        (DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX).contains(&offset)
    }

    /// Validates that a goal is within safe bounds.
    ///
    /// @notice Returns true if goal ∈ [GOAL_MIN, GOAL_MAX].
    /// @dev Rejects zero and negative goals to prevent division-by-zero.
    pub fn is_valid_goal(_env: Env, goal: i128) -> bool {
        (GOAL_MIN..=GOAL_MAX).contains(&goal)
    }

    /// Validates that a minimum contribution is within safe bounds.
    ///
    /// @notice Returns true if min_contribution ∈ [MIN_CONTRIBUTION_FLOOR, goal].
    /// @dev min_contribution > goal would make it impossible to contribute.
    pub fn is_valid_min_contribution(_env: Env, min_contribution: i128, goal: i128) -> bool {
        min_contribution >= MIN_CONTRIBUTION_FLOOR && min_contribution <= goal
    }

    /// Validates that a contribution amount meets the minimum.
    ///
    /// @notice Returns true if amount >= min_contribution.
    pub fn is_valid_contribution_amount(_env: Env, amount: i128, min_contribution: i128) -> bool {
        amount >= min_contribution
    }

    /// Validates that a fee basis points value is within safe bounds.
    ///
    /// @notice Returns true if fee_bps <= FEE_BPS_CAP.
    /// @dev A fee above 10,000 bps would exceed 100% of the contribution.
    pub fn is_valid_fee_bps(_env: Env, fee_bps: u32) -> bool {
        fee_bps <= FEE_BPS_CAP
    }

    /// Validates that a generator batch size is within safe bounds.
    ///
    /// @notice Returns true if batch_size ∈ [1, GENERATOR_BATCH_MAX].
    /// @dev Prevents worst-case memory/gas spikes in test scaffolds.
    pub fn is_valid_generator_batch_size(_env: Env, batch_size: u32) -> bool {
        batch_size > 0 && batch_size <= GENERATOR_BATCH_MAX
    }

    /// Clamps a requested proptest case count into safe operating bounds.
    ///
    /// @notice Returns value clamped to [PROPTEST_CASES_MIN, PROPTEST_CASES_MAX].
    /// @dev Protects CI runtime cost while preserving boundary signal.
    pub fn clamp_proptest_cases(_env: Env, requested: u32) -> u32 {
        requested.clamp(PROPTEST_CASES_MIN, PROPTEST_CASES_MAX)
    }

    /// Clamps raw progress basis points to [0, PROGRESS_BPS_CAP].
    ///
    /// @notice Negative raw values are floored to 0; values above 10,000 are capped.
    /// @dev Ensures the frontend never displays >100% funded.
    pub fn clamp_progress_bps(_env: Env, raw: i128) -> u32 {
        if raw <= 0 {
            0
        } else if raw >= PROGRESS_BPS_CAP as i128 {
            PROGRESS_BPS_CAP
        } else {
            raw as u32
        }
    }

    /// Computes progress in basis points, capped at 10,000.
    ///
    /// @notice Returns (raised * 10_000) / goal, clamped to [0, PROGRESS_BPS_CAP].
    /// @dev Returns 0 when goal <= 0 to avoid division-by-zero.
    ///      Uses saturating_mul to prevent integer overflow.
    pub fn compute_progress_bps(_env: Env, raised: i128, goal: i128) -> u32 {
        if goal <= 0 {
            return 0;
        }
        let raw = raised.saturating_mul(10_000) / goal;
        Self::clamp_progress_bps(_env, raw)
    }

    /// Computes fee amount from a contribution and fee basis points.
    ///
    /// @notice Returns (amount * fee_bps) / 10_000 (integer floor).
    /// @dev Returns 0 when amount <= 0 or fee_bps == 0.
    ///      Uses saturating_mul to prevent integer overflow.
    pub fn compute_fee_amount(_env: Env, amount: i128, fee_bps: u32) -> i128 {
        if amount <= 0 || fee_bps == 0 {
            return 0;
        }
        amount.saturating_mul(fee_bps as i128) / 10_000
    }

    /// Returns a diagnostic tag for boundary log events.
    ///
    /// @notice Used to identify boundary-related log entries in contract events.
    pub fn log_tag(_env: Env) -> Symbol {
        Symbol::new(&_env, "boundary")
    }
}
