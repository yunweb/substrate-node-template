use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// 测试创建kitty
#[test]
fn create_kitty() {
	kitty_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
	})
}

// 测试转移kitty
