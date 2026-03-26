//! Comprehensive tests for the ProptestGeneratorBoundary contract.
//!
//! @title   ProptestGeneratorBoundary Tests
//! @notice  Validates correct return of boundary constants and logic for clamping/validation.
//! @dev     Includes both unit tests and property-based tests for boundary safety.
//!          Target coverage: ≥95% line coverage with 256 property test cases.

#[cfg(test)]
mod tests {
    use soroban_sdk::{Env, Symbol};
    use proptest::prelude::*;
    use crate::proptest_generator_boundary::{
        ProptestGeneratorBoundary, ProptestGeneratorBoundaryClient,
        DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX, GOAL_MIN, GOAL_MAX,
        MIN_CONTRIBUTION_FLOOR, PROGRESS_BPS_CAP, FEE_BPS_CAP,
        PROPTEST_CASES_MIN, PROPTEST_CASES_MAX, GENERATOR_BATCH_MAX,
    };

    /// Setup a fresh test environment with the boundary contract registered.
    fn setup() -> (Env, ProptestGeneratorBoundaryClient<'static>) {
        let env = Env::default();
        let contract_id = env.register(ProptestGeneratorBoundary, ());
        let client = ProptestGeneratorBoundaryClient::new(&env, &contract_id);
        (env, client)
    }

    // ── Constant Sanity Checks ────────────────────────────────────────────────

    #[test]
    fn test_constants_return_correct_values() {
        let (_env, client) = setup();
        assert_eq!(client.deadline_offset_min(), DEADLINE_OFFSET_MIN);
        assert_eq!(client.deadline_offset_max(), DEADLINE_OFFSET_MAX);
        assert_eq!(client.goal_min(), GOAL_MIN);
        assert_eq!(client.goal_max(), GOAL_MAX);
        assert_eq!(client.min_contribution_floor(), MIN_CONTRIBUTION_FLOOR);
    }

    #[test]
    fn test_constants_are_ordered_correctly() {
        assert!(DEADLINE_OFFSET_MIN < DEADLINE_OFFSET_MAX);
        assert!(GOAL_MIN < GOAL_MAX);
        assert!(PROPTEST_CASES_MIN < PROPTEST_CASES_MAX);
        assert!(PROGRESS_BPS_CAP > 0);
        assert!(FEE_BPS_CAP > 0);
        assert!(GENERATOR_BATCH_MAX > 0);
    }

    #[test]
    fn test_constants_have_reasonable_values() {
        // Deadline offsets should be in seconds
        assert!(DEADLINE_OFFSET_MIN >= 60);
        assert!(DEADLINE_OFFSET_MAX <= 100_000_000);
        
        // Goals should be positive
        assert!(GOAL_MIN > 0);
        assert!(GOAL_MAX > 0);
        
        // Basis points should be <= 10,000
        assert!(PROGRESS_BPS_CAP <= 10_000);
        assert!(FEE_BPS_CAP <= 10_000);
    }

    // ── is_valid_deadline_offset Tests ────────────────────────────────────────

    #[test]
    fn test_is_valid_deadline_offset_boundary_values() {
        let (_env, client) = setup();
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MIN));
        assert!(client.is_valid_deadline_offset(&DEADLINE_OFFSET_MAX));
        assert!(!client.is_valid_deadline_offset(&(DEADLINE_OFFSET_MIN - 1)));
        assert!(!client.is_valid_deadline_offset(&(DEADLINE_OFFSET_MAX + 1)));
    }

    #[test]
    fn test_is_valid_deadline_offset_midrange() {
        let (_env, client) = setup();
        assert!(client.is_valid_deadline_offset(&500_000));
        assert!(client.is_valid_deadline_offset(&100_000));
        assert!(client.is_valid_deadline_offset(&10_000));
    }

    #[test]
    fn test_is_valid_deadline_offset_zero_and_negative() {
        let (_env, client) = setup();
        assert!(!client.is_valid_deadline_offset(&0));
        // Note: u64 cannot be negative, so we test the lower bound
    }

    // ── is_valid_goal Tests ──────────────────────────────────────────────────

    #[test]
    fn test_is_valid_goal_boundary_values() {
        let (_env, client) = setup();
        assert!(client.is_valid_goal(&GOAL_MIN));
        assert!(client.is_valid_goal(&GOAL_MAX));
        assert!(!client.is_valid_goal(&(GOAL_MIN - 1)));
        assert!(!client.is_valid_goal(&(GOAL_MAX + 1)));
    }

    #[test]
    fn test_is_valid_goal_midrange() {
        let (_env, client) = setup();
        assert!(client.is_valid_goal(&50_000_000));
        assert!(client.is_valid_goal(&10_000_000));
        assert!(client.is_valid_goal(&1_000_000));
    }

    #[test]
    fn test_is_valid_goal_zero_and_negative() {
        let (_env, client) = setup();
        assert!(!client.is_valid_goal(&0));
        assert!(!client.is_valid_goal(&-1));
        assert!(!client.is_valid_goal(&-1_000_000));
    }

    // ── is_valid_min_contribution Tests ──────────────────────────────────────

    #[test]
    fn test_is_valid_min_contribution_valid_cases() {
        let (_env, client) = setup();
        assert!(client.is_valid_min_contribution(&1, &1_000));
        assert!(client.is_valid_min_contribution(&500, &1_000));
        assert!(client.is_valid_min_contribution(&1_000, &1_000));
        assert!(client.is_valid_min_contribution(&1, &100_000_000));
    }

    #[test]
    fn test_is_valid_min_contribution_invalid_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_min_contribution(&0, &1_000));
        assert!(!client.is_valid_min_contribution(&-1, &1_000));
        assert!(!client.is_valid_min_contribution(&1_001, &1_000));
        assert!(!client.is_valid_min_contribution(&100_000_001, &100_000_000));
    }

    // ── is_valid_contribution_amount Tests ───────────────────────────────────

    #[test]
    fn test_is_valid_contribution_amount_valid_cases() {
        let (_env, client) = setup();
        assert!(client.is_valid_contribution_amount(&1, &1));
        assert!(client.is_valid_contribution_amount(&100, &50));
        assert!(client.is_valid_contribution_amount(&1_000, &1));
        assert!(client.is_valid_contribution_amount(&100_000_000, &1));
    }

    #[test]
    fn test_is_valid_contribution_amount_invalid_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_contribution_amount(&0, &1));
        assert!(!client.is_valid_contribution_amount(&-1, &1));
        assert!(!client.is_valid_contribution_amount(&50, &100));
        assert!(!client.is_valid_contribution_amount(&1, &100));
    }

    // ── is_valid_fee_bps Tests ───────────────────────────────────────────────

    #[test]
    fn test_is_valid_fee_bps_valid_cases() {
        let (_env, client) = setup();
        assert!(client.is_valid_fee_bps(&0));
        assert!(client.is_valid_fee_bps(&1));
        assert!(client.is_valid_fee_bps(&5_000));
        assert!(client.is_valid_fee_bps(&10_000));
    }

    #[test]
    fn test_is_valid_fee_bps_invalid_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_fee_bps(&10_001));
        assert!(!client.is_valid_fee_bps(&20_000));
        assert!(!client.is_valid_fee_bps(&u32::MAX));
    }

    // ── is_valid_generator_batch_size Tests ──────────────────────────────────

    #[test]
    fn test_is_valid_generator_batch_size_valid_cases() {
        let (_env, client) = setup();
        assert!(client.is_valid_generator_batch_size(&1));
        assert!(client.is_valid_generator_batch_size(&256));
        assert!(client.is_valid_generator_batch_size(&512));
    }

    #[test]
    fn test_is_valid_generator_batch_size_invalid_cases() {
        let (_env, client) = setup();
        assert!(!client.is_valid_generator_batch_size(&0));
        assert!(!client.is_valid_generator_batch_size(&513));
        assert!(!client.is_valid_generator_batch_size(&1_000));
    }

    // ── clamp_proptest_cases Tests ───────────────────────────────────────────

    #[test]
    fn test_clamp_proptest_cases_below_min() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_proptest_cases(&0), PROPTEST_CASES_MIN);
        assert_eq!(client.clamp_proptest_cases(&1), PROPTEST_CASES_MIN);
        assert_eq!(client.clamp_proptest_cases(&31), PROPTEST_CASES_MIN);
    }

    #[test]
    fn test_clamp_proptest_cases_within_range() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_proptest_cases(&32), 32);
        assert_eq!(client.clamp_proptest_cases(&100), 100);
        assert_eq!(client.clamp_proptest_cases(&256), 256);
    }

    #[test]
    fn test_clamp_proptest_cases_above_max() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_proptest_cases(&257), PROPTEST_CASES_MAX);
        assert_eq!(client.clamp_proptest_cases(&1_000), PROPTEST_CASES_MAX);
        assert_eq!(client.clamp_proptest_cases(&u32::MAX), PROPTEST_CASES_MAX);
    }

    // ── clamp_progress_bps Tests ─────────────────────────────────────────────

    #[test]
    fn test_clamp_progress_bps_negative_values() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&-1_000), 0);
        assert_eq!(client.clamp_progress_bps(&-1), 0);
    }

    #[test]
    fn test_clamp_progress_bps_zero() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&0), 0);
    }

    #[test]
    fn test_clamp_progress_bps_within_range() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&1), 1);
        assert_eq!(client.clamp_progress_bps(&5_000), 5_000);
        assert_eq!(client.clamp_progress_bps(&10_000), 10_000);
    }

    #[test]
    fn test_clamp_progress_bps_above_cap() {
        let (_env, client) = setup();
        assert_eq!(client.clamp_progress_bps(&10_001), PROGRESS_BPS_CAP);
        assert_eq!(client.clamp_progress_bps(&20_000), PROGRESS_BPS_CAP);
        assert_eq!(client.clamp_progress_bps(&i128::MAX), PROGRESS_BPS_CAP);
    }

    // ── compute_progress_bps Tests ───────────────────────────────────────────

    #[test]
    fn test_compute_progress_bps_zero_goal() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&0, &0), 0);
        assert_eq!(client.compute_progress_bps(&1_000, &0), 0);
        assert_eq!(client.compute_progress_bps(&100_000_000, &0), 0);
    }

    #[test]
    fn test_compute_progress_bps_negative_goal() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&1_000, &-1), 0);
        assert_eq!(client.compute_progress_bps(&100_000, &-1_000), 0);
    }

    #[test]
    fn test_compute_progress_bps_zero_raised() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&0, &1_000), 0);
        assert_eq!(client.compute_progress_bps(&0, &100_000_000), 0);
    }

    #[test]
    fn test_compute_progress_bps_negative_raised() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&-1_000, &1_000), 0);
        assert_eq!(client.compute_progress_bps(&-100_000_000, &1_000), 0);
    }

    #[test]
    fn test_compute_progress_bps_partial_progress() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&500, &1_000), 5_000);
        assert_eq!(client.compute_progress_bps(&250, &1_000), 2_500);
        assert_eq!(client.compute_progress_bps(&1, &1_000), 10);
    }

    #[test]
    fn test_compute_progress_bps_full_progress() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&1_000, &1_000), 10_000);
        assert_eq!(client.compute_progress_bps(&100_000_000, &100_000_000), 10_000);
    }

    #[test]
    fn test_compute_progress_bps_over_goal() {
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&2_000, &1_000), 10_000);
        assert_eq!(client.compute_progress_bps(&200_000_000, &100_000_000), 10_000);
    }

    // ── compute_fee_amount Tests ─────────────────────────────────────────────

    #[test]
    fn test_compute_fee_amount_zero_amount() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&0, &1_000), 0);
        assert_eq!(client.compute_fee_amount(&0, &10_000), 0);
    }

    #[test]
    fn test_compute_fee_amount_negative_amount() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&-1_000, &1_000), 0);
        assert_eq!(client.compute_fee_amount(&-100_000_000, &5_000), 0);
    }

    #[test]
    fn test_compute_fee_amount_zero_fee() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&1_000, &0), 0);
        assert_eq!(client.compute_fee_amount(&100_000_000, &0), 0);
    }

    #[test]
    fn test_compute_fee_amount_valid_calculations() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&1_000, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_000, &5_000), 500);
        assert_eq!(client.compute_fee_amount(&1_000, &10_000), 1_000);
        assert_eq!(client.compute_fee_amount(&10_000, &1_000), 1_000);
    }

    #[test]
    fn test_compute_fee_amount_large_values() {
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&100_000_000, &1_000), 10_000_000);
        assert_eq!(client.compute_fee_amount(&100_000_000, &5_000), 50_000_000);
    }

    // ── log_tag Tests ────────────────────────────────────────────────────────

    #[test]
    fn test_log_tag() {
        let (env, client) = setup();
        assert_eq!(client.log_tag(), Symbol::new(&env, "boundary"));
    }

    // ── Property-Based Tests ──────────────────────────────────────────────────

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Property: All valid deadline offsets pass validation.
        #[test]
        fn prop_deadline_offset_validity(offset in DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_deadline_offset(&offset));
        }

        /// Property: All invalid deadline offsets fail validation.
        #[test]
        fn prop_deadline_offset_invalidity(offset in 0u64..DEADLINE_OFFSET_MIN) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_deadline_offset(&offset));
        }

        /// Property: All valid goals pass validation.
        #[test]
        fn prop_goal_validity(goal in GOAL_MIN..=GOAL_MAX) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_goal(&goal));
        }

        /// Property: All invalid goals fail validation.
        #[test]
        fn prop_goal_invalidity(goal in i128::MIN..GOAL_MIN) {
            let (_env, client) = setup();
            prop_assert!(!client.is_valid_goal(&goal));
        }

        /// Property: Progress BPS is always bounded by PROGRESS_BPS_CAP.
        #[test]
        fn prop_progress_bps_bounds(
            raised in -1_000_000_000i128..=200_000_000i128,
            goal in GOAL_MIN..=GOAL_MAX
        ) {
            let (_env, client) = setup();
            let bps = client.compute_progress_bps(&raised, &goal);
            prop_assert!(bps <= PROGRESS_BPS_CAP);
        }

        /// Property: Clamped progress BPS is always bounded.
        #[test]
        fn prop_clamped_progress_bps_bounds(raw in i128::MIN..=i128::MAX) {
            let (_env, client) = setup();
            let clamped = client.clamp_progress_bps(&raw);
            prop_assert!(clamped <= PROGRESS_BPS_CAP);
        }

        /// Property: Proptest cases are always within bounds after clamping.
        #[test]
        fn prop_clamped_cases_bounds(requested in 0u32..=u32::MAX) {
            let (_env, client) = setup();
            let clamped = client.clamp_proptest_cases(&requested);
            prop_assert!(clamped >= PROPTEST_CASES_MIN);
            prop_assert!(clamped <= PROPTEST_CASES_MAX);
        }

        /// Property: Fee amounts are always non-negative.
        #[test]
        fn prop_fee_amount_non_negative(
            amount in 0i128..=100_000_000i128,
            fee_bps in 0u32..=FEE_BPS_CAP
        ) {
            let (_env, client) = setup();
            let fee = client.compute_fee_amount(&amount, &fee_bps);
            prop_assert!(fee >= 0);
        }

        /// Property: Fee amount never exceeds the original amount.
        #[test]
        fn prop_fee_amount_not_exceeds_original(
            amount in 1i128..=100_000_000i128,
            fee_bps in 0u32..=FEE_BPS_CAP
        ) {
            let (_env, client) = setup();
            let fee = client.compute_fee_amount(&amount, &fee_bps);
            prop_assert!(fee <= amount);
        }

        /// Property: Valid min contributions are always >= MIN_CONTRIBUTION_FLOOR.
        #[test]
        fn prop_valid_min_contribution_floor(
            min_contrib in MIN_CONTRIBUTION_FLOOR..=GOAL_MAX,
            goal in GOAL_MIN..=GOAL_MAX
        ) {
            let (_env, client) = setup();
            if min_contrib <= goal {
                prop_assert!(client.is_valid_min_contribution(&min_contrib, &goal));
            }
        }

        /// Property: Valid contribution amounts are >= min_contribution.
        #[test]
        fn prop_valid_contribution_amount(
            amount in MIN_CONTRIBUTION_FLOOR..=100_000_000i128,
            min_contrib in MIN_CONTRIBUTION_FLOOR..=100_000_000i128
        ) {
            let (_env, client) = setup();
            if amount >= min_contrib {
                prop_assert!(client.is_valid_contribution_amount(&amount, &min_contrib));
            }
        }

        /// Property: Valid fee BPS are always <= FEE_BPS_CAP.
        #[test]
        fn prop_valid_fee_bps(fee_bps in 0u32..=FEE_BPS_CAP) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_fee_bps(&fee_bps));
        }

        /// Property: Valid batch sizes are always > 0 and <= GENERATOR_BATCH_MAX.
        #[test]
        fn prop_valid_batch_size(batch_size in 1u32..=GENERATOR_BATCH_MAX) {
            let (_env, client) = setup();
            prop_assert!(client.is_valid_generator_batch_size(&batch_size));
        }
    }

    // ── Regression Tests ──────────────────────────────────────────────────────

    #[test]
    fn regression_deadline_offset_minimum_1000() {
        // Regression: Deadline offset minimum was previously 100, causing flaky tests.
        // This test ensures it's now 1,000 (17 minutes).
        let (_env, client) = setup();
        assert_eq!(client.deadline_offset_min(), 1_000);
        assert!(!client.is_valid_deadline_offset(&100));
        assert!(client.is_valid_deadline_offset(&1_000));
    }

    #[test]
    fn regression_progress_bps_never_exceeds_cap() {
        // Regression: Progress BPS should never exceed 10,000 (100%).
        let (_env, client) = setup();
        let bps = client.compute_progress_bps(&1_000_000_000, &1);
        assert_eq!(bps, PROGRESS_BPS_CAP);
    }

    #[test]
    fn regression_fee_calculation_precision() {
        // Regression: Fee calculation should use integer floor division.
        let (_env, client) = setup();
        assert_eq!(client.compute_fee_amount(&1_000, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_001, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_009, &1_000), 100);
        assert_eq!(client.compute_fee_amount(&1_010, &1_000), 101);
    }

    #[test]
    fn regression_zero_goal_division_safety() {
        // Regression: Division by zero should be prevented.
        let (_env, client) = setup();
        assert_eq!(client.compute_progress_bps(&1_000, &0), 0);
        assert_eq!(client.compute_progress_bps(&100_000_000, &0), 0);
    }
}
