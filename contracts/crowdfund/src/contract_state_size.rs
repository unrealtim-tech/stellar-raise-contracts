//! # Contract State Size Limits
//!
//! This module enforces upper-bound limits on the size of unbounded collections
//! stored in contract state to prevent:
//!
//! - **DoS via state bloat**: an attacker flooding the contributors or roadmap
//!   lists until operations become too expensive to execute.
//! - **Gas exhaustion**: iteration over an unbounded `Vec` in `withdraw`,
//!   `refund`, or `collect_pledges` can exceed Soroban resource limits.
//! - **Ledger entry size violations**: Soroban enforces a hard cap on the
//!   serialised size of each ledger entry; exceeding it causes a host panic.
//!
//! ## Security Assumptions
//!
//! 1. `MAX_CONTRIBUTORS` caps the `Contributors` persistent list. Any `contribute`
//!    call that would push the list past this limit is rejected.
//! 2. `MAX_PLEDGERS` caps the `Pledgers` persistent list.
//! 3. `MAX_ROADMAP_ITEMS` caps the `Roadmap` instance list.
//! 4. `MAX_STRETCH_GOALS` caps the `StretchGoals` list.
//! 5. `MAX_TITLE_LENGTH`, `MAX_DESCRIPTION_LENGTH`, `MAX_SOCIAL_LINKS_LENGTH`,
//!    `MAX_BONUS_GOAL_DESCRIPTION_LENGTH`, and `MAX_ROADMAP_DESCRIPTION_LENGTH`
//!    cap user-supplied `String` fields to prevent oversized ledger entries.
//! 6. `MAX_METADATA_TOTAL_LENGTH` provides a combined budget for title +
//!    description + socials to prevent excessive total storage.
//!
//! ## Limits (rationale)
//!
//! | Constant                           | Value | Rationale                                      |
//! |------------------------------------|-------|------------------------------------------------|
//! | `MAX_CONTRIBUTORS`                 |   128 | Keeps `withdraw` / `refund` batch within gas   |
//! | `MAX_PLEDGERS`                     |   128 | Keeps `collect_pledges` iteration within gas  |
//! | `MAX_ROADMAP_ITEMS`                |    32 | Cosmetic list; no operational iteration needed |
//! | `MAX_STRETCH_GOALS`                |    32 | Small advisory list                            |
//! | `MAX_TITLE_LENGTH`                 |   128 | Prevents oversized instance-storage entries   |
//! | `MAX_DESCRIPTION_LENGTH`           |  2048 | Allows detailed campaign descriptions         |
//! | `MAX_SOCIAL_LINKS_LENGTH`          |   512 | Allows multiple social links                  |
//! | `MAX_BONUS_GOAL_DESCRIPTION_LENGTH`|   280 | Twitter-length limit for goal descriptions    |
//! | `MAX_ROADMAP_DESCRIPTION_LENGTH`   |   280 | Twitter-length limit for roadmap items        |
//! | `MAX_METADATA_TOTAL_LENGTH`        |  2304 | Combined budget: title + description + socials|
//!
//! ## NatSpec Documentation
//!
//! This module follows NatSpec conventions for all public constants and
//! validation functions to enable automated documentation generation and
//! static analysis.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::contract_state_size::{validate_title, MAX_TITLE_LENGTH};
//!
//! fn set_title(env: &Env, title: &String) {
//!     if let Err(e) = validate_title(title) {
//!         panic!("{}", e);
//!     }
//!     // ... store title
//! }
//! ```

#![allow(missing_docs)]

use soroban_sdk::{contracterror, String, Vec};

// ── Limits ───────────────────────────────────────────────────────────────────

/// Maximum number of unique contributors tracked on-chain.
///
/// @notice This limit prevents unbounded growth of the contributor index.
///         When reached, new contributors cannot contribute unless they have
///         already contributed previously.
pub const MAX_CONTRIBUTORS: u32 = 128;

/// Maximum number of unique pledgers tracked on-chain.
///
/// @notice This limit prevents unbounded growth of the pledger index.
///         When reached, new pledgers cannot pledge unless they have
///         already pledged previously.
pub const MAX_PLEDGERS: u32 = 128;

/// Maximum number of roadmap items stored in instance storage.
///
/// @notice Roadmap items are cosmetic and do not require iteration in
///         any operational flow, so a modest limit is sufficient.
pub const MAX_ROADMAP_ITEMS: u32 = 32;

/// Maximum number of stretch-goal milestones.
///
/// @notice Stretch goals are advisory milestones that do not require
///         iteration in operational flows.
pub const MAX_STRETCH_GOALS: u32 = 32;

/// Maximum byte length of a campaign title.
///
/// @notice Titles are displayed in UI and stored in instance storage.
///         The limit ensures consistent UI rendering and storage bounds.
pub const MAX_TITLE_LENGTH: u32 = 128;

/// Maximum byte length of a campaign description.
///
/// @notice Descriptions can contain detailed information about the campaign.
///         The limit allows rich content while preventing oversized entries.
pub const MAX_DESCRIPTION_LENGTH: u32 = 2048;

/// Maximum byte length of social links field.
///
/// @notice Social links can contain multiple URLs separated by delimiters.
///         The limit accommodates several links while preventing bloat.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = 512;

/// Maximum byte length of bonus goal description.
///
/// @notice Bonus goal descriptions are shown when stretch goals are met.
///         Twitter-length limit encourages concise, readable content.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum byte length of roadmap item description.
///
/// @notice Roadmap item descriptions outline milestones and timelines.
///         Twitter-length limit encourages concise, readable content.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum combined byte length of title + description + socials.
///
/// @notice This aggregate limit prevents campaigns from storing several
///         individually-valid but collectively excessive fields at once.
///         The sum of all three fields must not exceed this value.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = 2304;

/// Maximum byte length for any string field (legacy alias).
///
/// @deprecated Use MAX_TITLE_LENGTH instead for new code.
///             This constant is kept for backwards compatibility.
pub const MAX_STRING_LEN: u32 = 256;

// ── Validation helpers ────────────────────────────────────────────────────────

/// Validates that a title does not exceed MAX_TITLE_LENGTH bytes.
///
/// @param title The title string to validate.
/// @return Ok(()) if the title is within limits, Err with descriptive message otherwise.
/// @notice Callers should treat errors as permanent rejections; the limit
///         will not change without a contract upgrade.
pub fn validate_title(title: &String) -> Result<(), &'static str> {
    if title.len() > MAX_TITLE_LENGTH {
        return Err("title exceeds MAX_TITLE_LENGTH bytes");
    }
    Ok(())
}

/// Validates that a description does not exceed MAX_DESCRIPTION_LENGTH bytes.
///
/// @param description The description string to validate.
/// @return Ok(()) if the description is within limits, Err with descriptive message otherwise.
pub fn validate_description(description: &String) -> Result<(), &'static str> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err("description exceeds MAX_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

/// Validates that social links do not exceed MAX_SOCIAL_LINKS_LENGTH bytes.
///
/// @param socials The social links string to validate.
/// @return Ok(()) if within limits, Err with descriptive message otherwise.
pub fn validate_social_links(socials: &String) -> Result<(), &'static str> {
    if socials.len() > MAX_SOCIAL_LINKS_LENGTH {
        return Err("social links exceed MAX_SOCIAL_LINKS_LENGTH bytes");
    }
    Ok(())
}

/// Validates that bonus goal description does not exceed MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes.
///
/// @param description The bonus goal description to validate.
/// @return Ok(()) if within limits, Err with descriptive message otherwise.
pub fn validate_bonus_goal_description(description: &String) -> Result<(), &'static str> {
    if description.len() > MAX_BONUS_GOAL_DESCRIPTION_LENGTH {
        return Err("bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

/// Validates that roadmap description does not exceed MAX_ROADMAP_DESCRIPTION_LENGTH bytes.
///
/// @param description The roadmap description to validate.
/// @return Ok(()) if within limits, Err with descriptive message otherwise.
pub fn validate_roadmap_description(description: &String) -> Result<(), &'static str> {
    if description.len() > MAX_ROADMAP_DESCRIPTION_LENGTH {
        return Err("roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

/// Validates that the combined metadata (title + description + socials) does not exceed
/// MAX_METADATA_TOTAL_LENGTH bytes.
///
/// @param title_len Length of the title in bytes.
/// @param description_len Length of the description in bytes.
/// @param socials_len Length of the social links in bytes.
/// @return Ok(()) if the total is within limits, Err with descriptive message otherwise.
/// @notice This function uses saturating addition to prevent overflow attacks.
///         If the sum would overflow, it is treated as exceeding the limit.
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), &'static str> {
    // Use saturating_add to prevent integer overflow attacks
    let total = title_len.saturating_add(description_len).saturating_add(socials_len);
    if total > MAX_METADATA_TOTAL_LENGTH {
        return Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes");
    }
    Ok(())
}

/// Validates that adding a new contributor would not exceed MAX_CONTRIBUTORS.
///
/// @param current_count The current number of contributors.
/// @return Ok(()) if a new contributor can be added, Err with descriptive message otherwise.
/// @notice This validates the index capacity, not whether a specific address
///         has already contributed. Existing contributors can always contribute.
pub fn validate_contributor_capacity(current_count: u32) -> Result<(), &'static str> {
    if current_count >= MAX_CONTRIBUTORS {
        return Err("contributors exceed MAX_CONTRIBUTORS");
    }
    Ok(())
}

/// Validates that adding a new pledger would not exceed MAX_PLEDGERS.
///
/// @param current_count The current number of pledgers.
/// @return Ok(()) if a new pledger can be added, Err with descriptive message otherwise.
/// @notice This validates the index capacity, not whether a specific address
///         has already pledged. Existing pledgers can always pledge again.
pub fn validate_pledger_capacity(current_count: u32) -> Result<(), &'static str> {
    if current_count >= MAX_PLEDGERS {
        return Err("pledgers exceed MAX_PLEDGERS");
    }
    Ok(())
}

/// Validates that adding a new roadmap item would not exceed MAX_ROADMAP_ITEMS.
///
/// @param current_count The current number of roadmap items.
/// @return Ok(()) if a new item can be added, Err with descriptive message otherwise.
pub fn validate_roadmap_capacity(current_count: u32) -> Result<(), &'static str> {
    if current_count >= MAX_ROADMAP_ITEMS {
        return Err("roadmap exceeds MAX_ROADMAP_ITEMS");
    }
    Ok(())
}

/// Validates that adding a new stretch goal would not exceed MAX_STRETCH_GOALS.
///
/// @param current_count The current number of stretch goals.
/// @return Ok(()) if a new goal can be added, Err with descriptive message otherwise.
pub fn validate_stretch_goal_capacity(current_count: u32) -> Result<(), &'static str> {
    if current_count >= MAX_STRETCH_GOALS {
        return Err("stretch goals exceed MAX_STRETCH_GOALS");
    }
    Ok(())
}

// ── Legacy compatibility functions ────────────────────────────────────────────

use crate::DataKey;
use soroban_sdk::Env;

/// Legacy function for checking string length limit.
///
/// @param s The string to validate.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_title, validate_description, or validate_social_links instead.
pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Legacy function for checking contributor limit.
///
/// @param env Soroban environment reference.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_contributor_capacity instead.
pub fn check_contributor_limit(env: &Env) -> Result<(), StateSizeError> {
    let contributors: Vec<soroban_sdk::Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Contributors)
        .unwrap_or_else(|| Vec::new(env));

    if contributors.len() >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Legacy function for checking pledger limit.
///
/// @param env Soroban environment reference.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_pledger_capacity instead.
pub fn check_pledger_limit(env: &Env) -> Result<(), StateSizeError> {
    let pledgers: Vec<soroban_sdk::Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Pledgers)
        .unwrap_or_else(|| Vec::new(env));

    if pledgers.len() >= MAX_PLEDGERS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Legacy function for checking roadmap limit.
///
/// @param env Soroban environment reference.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_roadmap_capacity instead.
pub fn check_roadmap_limit(env: &Env) -> Result<(), StateSizeError> {
    let roadmap: Vec<crate::RoadmapItem> = env
        .storage()
        .instance()
        .get(&DataKey::Roadmap)
        .unwrap_or_else(|| Vec::new(env));

    if roadmap.len() >= MAX_ROADMAP_ITEMS {
        return Err(StateSizeError::RoadmapLimitExceeded);
    }
    Ok(())
}

/// Legacy function for checking stretch goal limit.
///
/// @param env Soroban environment reference.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_stretch_goal_capacity instead.
pub fn check_stretch_goal_limit(env: &Env) -> Result<(), StateSizeError> {
    let goals: Vec<i128> = env
        .storage()
        .instance()
        .get(&DataKey::StretchGoals)
        .unwrap_or_else(|| Vec::new(env));

    if goals.len() >= MAX_STRETCH_GOALS {
        return Err(StateSizeError::StretchGoalLimitExceeded);
    }
    Ok(())
}

// ── Error types ───────────────────────────────────────────────────────────────

/// Error returned when a state-size limit would be exceeded.
///
/// @notice Callers should treat this as a permanent rejection for the current
///         campaign state; the limit will not change without a contract upgrade.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StateSizeError {
    /// The contributors / pledgers list is full.
    ContributorLimitExceeded = 100,
    /// The roadmap list is full.
    RoadmapLimitExceeded = 101,
    /// The stretch-goals list is full.
    StretchGoalLimitExceeded = 102,
    /// A string field exceeds its maximum length.
    StringTooLong = 103,
}
