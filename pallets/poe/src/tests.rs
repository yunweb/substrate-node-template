use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// test cases for create_claim
#[test]
fn create_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
        assert_eq!(
            Proofs::<Test>::get(&claim),
            (1, system::Module::<Test>::block_number())
        );
    })
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_noop!(PoeModule::create_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimAlreadyExist);
    })
}

#[test]
fn create_claim_failed_when_claim_is_too_long() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        assert_noop!(PoeModule::create_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimTooLong);
    })
}

// test cases for revoke_claim
#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
        assert_eq!(
            Proofs::<Test>::get(&claim),
            (0, system::Module::<Test>::block_number())
        );
    })
}
#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_noop!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimNotExist);
    })
}

#[test]
fn revoke_claim_failed_with_wrong_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_noop!(PoeModule::revoke_claim(Origin::signed(2), claim.clone()), Error::<Test>::NotClaimOwner);
    })
}

// test cases for transfer_claim
#[test]
fn transfer_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));
        assert_eq!(
            Proofs::<Test>::get(&claim),
            (2, system::Module::<Test>::block_number())
        );
    })
}

#[test]
fn transfer_claim_failed_when_claim_is_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_noop!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2), Error::<Test>::ClaimNotExist);
    })
}

#[test]
fn transfer_claim_failed_with_wrong_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_noop!(PoeModule::transfer_claim(Origin::signed(2), claim.clone(), 2), Error::<Test>::NotClaimOwner);
    })
}
