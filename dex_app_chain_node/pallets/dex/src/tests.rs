use crate::{mock::*, Config, Error, Event, PoolInfo};
use codec;
use frame_support::{
	assert_noop, assert_ok,
	traits::{
		fungible::{Inspect, Mutate},
		fungibles::InspectEnumerable,
		Get,
	},
};

fn setup_account(account_id: u64, assets: Vec<u32>) {
	for asset_id in assets {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			codec::Compact(asset_id),
			account_id,
			false,
			1
		));
	}
}

#[test]
fn can_create_pool() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;
		let pool_id = (asset1, asset2);
		let pool_account = Dex::get_pool_account(&pool_id);

		// create assets
		setup_account(user, vec![asset1, asset2]);

		let lp_token_id = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		// create pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));
		assert_eq!(lp_token_id + 1, Dex::next_lp_token_id().unwrap());

		let pool_set_up_deposit = <Test as Config>::PoolSetupDeposit::get();

		assert_eq!(<Test as Config>::NativeAsset::balance(&user), 1000 - pool_set_up_deposit);
		assert_eq!(<Test as Config>::NativeAsset::balance(&pool_account), pool_set_up_deposit);

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::PoolCreated {
				creator: user,
				pool_id,
				pool_account,
				lp_token: lp_token_id,
			}]
		);

		System::reset_events();

		assert_eq!(Dex::pools(pool_id).unwrap(), PoolInfo { lp_token: lp_token_id });
		let mut assets = Assets::asset_ids().collect::<Vec<_>>();
		assets.sort();
		assert_eq!(assets, vec![lp_token_id, asset1, asset2]);
	});
}

#[test]
fn create_pool_with_same_asset_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		// create assets
		setup_account(user, vec![asset1, asset2]);

		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		assert_noop!(
			Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset1, 100, 200, 10),
			Error::<Test>::CannotCreatePoolWithSameAsset
		);
		assert_noop!(
			Dex::create_pool(RuntimeOrigin::signed(user), asset2, asset2, 100, 200, 10),
			Error::<Test>::CannotCreatePoolWithSameAsset
		);
	});
}

#[test]
fn create_same_pool_twice_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;
		let pool_id = (asset1, asset2);
		let pool_account = Dex::get_pool_account(&pool_id);

		// create assets
		setup_account(user, vec![asset1, asset2]);

		let lp_token_id = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		// create pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));
		assert_eq!(lp_token_id + 1, Dex::next_lp_token_id().unwrap());

		let pool_set_up_deposit = <Test as Config>::PoolSetupDeposit::get();

		assert_eq!(<Test as Config>::NativeAsset::balance(&user), 1000 - pool_set_up_deposit);
		assert_eq!(<Test as Config>::NativeAsset::balance(&pool_account), pool_set_up_deposit);

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::PoolCreated {
				creator: user,
				pool_id,
				pool_account,
				lp_token: lp_token_id,
			}]
		);

		System::reset_events();

		assert_eq!(Dex::pools(pool_id).unwrap(), PoolInfo { lp_token: lp_token_id });
		let mut assets = Assets::asset_ids().collect::<Vec<_>>();
		assets.sort();
		assert_eq!(assets, vec![lp_token_id, asset1, asset2]);

		assert_noop!(
			Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 200, 400, 10),
			Error::<Test>::PoolAlreadyExists
		);
	});
}

#[test]
fn create_pool_twice_with_same_assets_but_different_ordering_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;
		let pool_id = (asset1, asset2);
		let pool_account = Dex::get_pool_account(&pool_id);

		// create assets
		setup_account(user, vec![asset1, asset2]);

		let lp_token_id = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		// create pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));
		assert_eq!(lp_token_id + 1, Dex::next_lp_token_id().unwrap());

		let pool_set_up_deposit = <Test as Config>::PoolSetupDeposit::get();

		assert_eq!(<Test as Config>::NativeAsset::balance(&user), 1000 - pool_set_up_deposit);
		assert_eq!(<Test as Config>::NativeAsset::balance(&pool_account), pool_set_up_deposit);

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::PoolCreated {
				creator: user,
				pool_id,
				pool_account,
				lp_token: lp_token_id,
			}]
		);

		System::reset_events();

		assert_eq!(Dex::pools(pool_id).unwrap(), PoolInfo { lp_token: lp_token_id });
		let mut assets = Assets::asset_ids().collect::<Vec<_>>();
		assets.sort();
		assert_eq!(assets, vec![lp_token_id, asset1, asset2]);

		assert_noop!(
			Dex::create_pool(RuntimeOrigin::signed(user), asset2, asset1, 200, 400, 10),
			Error::<Test>::PoolAlreadyExists
		);
	});
}

#[test]
fn create_pool_without_enough_native_asset_to_pay_for_deposit_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		// create assets
		setup_account(user, vec![asset1, asset2]);

		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1));

		// create pool
		assert_noop!(
			Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10),
			Error::<Test>::NotEnoughToPayForPoolSetupDeposit
		);
	});
}

#[test]
fn different_pools_have_different_lp_tokens() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;
		let pool_id = (asset1, asset2);
		let pool_account = Dex::get_pool_account(&pool_id);

		// create assets
		setup_account(user, vec![asset1, asset2]);

		let lp_token_id = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		// create pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));
		assert_eq!(lp_token_id + 1, Dex::next_lp_token_id().unwrap());

		let pool_set_up_deposit = <Test as Config>::PoolSetupDeposit::get();

		assert_eq!(<Test as Config>::NativeAsset::balance(&user), 1000 - pool_set_up_deposit);
		assert_eq!(<Test as Config>::NativeAsset::balance(&pool_account), pool_set_up_deposit);

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::PoolCreated {
				creator: user,
				pool_id,
				pool_account,
				lp_token: lp_token_id,
			}]
		);

		System::reset_events();

		assert_eq!(Dex::pools(pool_id).unwrap(), PoolInfo { lp_token: lp_token_id });
		let mut assets = Assets::asset_ids().collect::<Vec<_>>();
		assets.sort();
		assert_eq!(assets, vec![lp_token_id, asset1, asset2]);

		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset3 = 88;
		let asset4 = 99;
		let pool_id2 = (asset3, asset4);
		let pool_account2 = Dex::get_pool_account(&pool_id2);

		// create assets
		setup_account(user, vec![asset3, asset4]);

		let lp_token_id2 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset3, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset4, 1000, user));

		// create pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset3, asset4, 100, 200, 10));
		assert_eq!(lp_token_id2 + 1, Dex::next_lp_token_id().unwrap());

		assert_eq!(<Test as Config>::NativeAsset::balance(&user), 1000 - 2 * pool_set_up_deposit);
		assert_eq!(<Test as Config>::NativeAsset::balance(&pool_account), pool_set_up_deposit);

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::PoolCreated {
				creator: user,
				pool_id: pool_id2,
				pool_account: pool_account2,
				lp_token: lp_token_id2,
			}]
		);

		System::reset_events();

		assert_eq!(Dex::pools(pool_id2).unwrap(), PoolInfo { lp_token: lp_token_id2 });
		let mut assets = Assets::asset_ids().collect::<Vec<_>>();
		assets.sort();
		assert_eq!(assets, vec![lp_token_id, lp_token_id2, asset1, asset2, asset3, asset4]);
		assert_ne!(lp_token_id, lp_token_id2);
	});
}

#[test]
fn can_add_liquidity() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;
		let asset3 = 88;

		setup_account(user, vec![asset1, asset2, asset3]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset3, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		let lp_token_id1 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));
		let lp_token_id2 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset3, 100, 300, 10));
		System::reset_events();

		assert_eq!(Assets::balance(asset1, user), 1000 - 100 - 100);
		assert_eq!(Assets::balance(asset2, user), 1000 - 200);
		assert_eq!(Assets::balance(asset3, user), 1000 - 300);
		assert_eq!(Assets::balance(lp_token_id1, user), 141);
		assert_eq!(Assets::balance(lp_token_id2, user), 173);

		let pool_id1 = (asset1, asset2);
		let pool_id2 = (asset1, asset3);
		let pool_account1 = Dex::get_pool_account(&pool_id1);
		let pool_account2 = Dex::get_pool_account(&pool_id2);
		assert_eq!(Assets::balance(asset1, pool_account1), 100);
		assert_eq!(Assets::balance(asset2, pool_account1), 200);
		assert_eq!(Assets::balance(asset1, pool_account2), 100);
		assert_eq!(Assets::balance(asset3, pool_account2), 300);

		let user2 = 2;
		frame_system::Pallet::<Test>::inc_providers(&user2);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 2000, user2));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 2000, user2));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset3, 2000, user2));

		assert_ok!(Dex::add_liquidity(
			RuntimeOrigin::signed(user2),
			asset1,
			asset2,
			200,
			500,
			10,
			1000
		));

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::LiquidityAdded {
				liquidity_provider: user2,
				pool_id: pool_id1,
				asset1_amount_provided: 200,
				asset2_amount_provided: 400,
				lp_token: lp_token_id1,
				lp_token_amount_minted: 282
			}]
		);

		System::reset_events();

		assert_eq!(Assets::balance(asset1, user2), 2000 - 200);
		assert_eq!(Assets::balance(asset2, user2), 2000 - 400);
		assert_eq!(Assets::balance(asset1, pool_account1), 100 + 200);
		assert_eq!(Assets::balance(asset2, pool_account1), 200 + 400);
		assert_eq!(Assets::balance(lp_token_id1, user2), 282);

		assert_ok!(Dex::add_liquidity(
			RuntimeOrigin::signed(user2),
			asset1,
			asset3,
			200,
			600,
			10,
			1000
		));

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::LiquidityAdded {
				liquidity_provider: user2,
				pool_id: pool_id2,
				asset1_amount_provided: 200,
				asset2_amount_provided: 600,
				lp_token: lp_token_id2,
				lp_token_amount_minted: 346
			}]
		);

		assert_eq!(Assets::balance(asset1, user2), 1800 - 200);
		assert_eq!(Assets::balance(asset3, user2), 2000 - 600);
		assert_eq!(Assets::balance(asset1, pool_account2), 100 + 200);
		assert_eq!(Assets::balance(asset3, pool_account2), 300 + 600);
		assert_eq!(Assets::balance(lp_token_id2, user2), 346);
	});
}

#[test]
fn add_liquidity_with_same_asset_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		let lp_token_id1 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		System::reset_events();

		assert_eq!(Assets::balance(asset1, user), 1000 - 100);
		assert_eq!(Assets::balance(asset2, user), 1000 - 200);
		assert_eq!(Assets::balance(lp_token_id1, user), 141);

		assert_noop!(
			Dex::add_liquidity(RuntimeOrigin::signed(user), asset1, asset1, 200, 500, 10, 1000),
			Error::<Test>::CannotAddLiquidityWithSameAsset
		);
	});
}

#[test]
fn add_liquidity_passed_the_deadline_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		let lp_token_id1 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		System::reset_events();

		assert_eq!(Assets::balance(asset1, user), 1000 - 100);
		assert_eq!(Assets::balance(asset2, user), 1000 - 200);
		assert_eq!(Assets::balance(lp_token_id1, user), 141);

		assert_noop!(
			Dex::add_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 200, 500, 10, 1),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn add_liquidity_with_zero_asset_amount_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		let lp_token_id1 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		System::reset_events();

		assert_eq!(Assets::balance(asset1, user), 1000 - 100);
		assert_eq!(Assets::balance(asset2, user), 1000 - 200);
		assert_eq!(Assets::balance(lp_token_id1, user), 141);

		assert_noop!(
			Dex::add_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 0, 500, 10, 100),
			Error::<Test>::InvalidLiquidityAmount
		);
	});
}

#[test]
fn add_liquidity_to_pool_that_doesnt_exist_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		let lp_token_id1 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		System::reset_events();

		assert_eq!(Assets::balance(asset1, user), 1000 - 100);
		assert_eq!(Assets::balance(asset2, user), 1000 - 200);
		assert_eq!(Assets::balance(lp_token_id1, user), 141);

		assert_noop!(
			Dex::add_liquidity(RuntimeOrigin::signed(user), 33, 44, 200, 500, 10, 100),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn add_liquidity_leads_to_not_enough_liquidity_provided_error() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		let lp_token_id1 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		System::reset_events();

		assert_eq!(Assets::balance(asset1, user), 1000 - 100);
		assert_eq!(Assets::balance(asset2, user), 1000 - 200);
		assert_eq!(Assets::balance(lp_token_id1, user), 141);

		assert_noop!(
			Dex::add_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 200, 500, 1000, 100),
			Error::<Test>::NotEnoughLiquidityProvided
		);
	});
}

#[test]
fn can_remove_liquidity() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;
		let asset3 = 88;

		setup_account(user, vec![asset1, asset2, asset3]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset3, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		let lp_token_id1 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));
		let lp_token_id2 = Dex::next_lp_token_id().unwrap_or(0);
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset3, 100, 300, 10));
		System::reset_events();

		let pool_id1 = (asset1, asset2);
		let pool_id2 = (asset1, asset3);
		let pool_account1 = Dex::get_pool_account(&pool_id1);
		let pool_account2 = Dex::get_pool_account(&pool_id2);

		let user2 = 2;
		frame_system::Pallet::<Test>::inc_providers(&user2);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 2000, user2));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 2000, user2));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset3, 2000, user2));

		assert_ok!(Dex::add_liquidity(
			RuntimeOrigin::signed(user2),
			asset1,
			asset2,
			200,
			500,
			10,
			1000
		));

		System::reset_events();

		assert_ok!(Dex::remove_liquidity(
			RuntimeOrigin::signed(user),
			asset1,
			asset2,
			1,
			2,
			100,
			1000
		));

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::LiquidityRemoved {
				remover: user,
				pool_id: pool_id1,
				asset1_received_amount: 70,
				asset2_received_amount: 141,
				lp_token: lp_token_id1,
				lp_token_burned: 100,
			}]
		);

		System::reset_events();
		assert_eq!(Assets::balance(asset1, user), 1000 - 200 + 70);
		assert_eq!(Assets::balance(asset2, user), 1000 - 200 + 141);
		assert_eq!(Assets::balance(asset1, pool_account1), 100 + 200 - 70);
		assert_eq!(Assets::balance(asset2, pool_account1), 200 + 400 - 141);
		assert_eq!(Assets::balance(lp_token_id1, user), 141 - 100);

		assert_ok!(Dex::remove_liquidity(
			RuntimeOrigin::signed(user),
			asset1,
			asset3,
			1,
			1,
			100,
			1000
		));

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::LiquidityRemoved {
				remover: user,
				pool_id: pool_id2,
				asset1_received_amount: 57,
				asset2_received_amount: 173,
				lp_token: lp_token_id2,
				lp_token_burned: 100,
			}]
		);

		assert_eq!(Assets::balance(asset1, user), 1000 - 200 + 70 + 57);
		assert_eq!(Assets::balance(asset3, user), 1000 - 300 + 173);
		assert_eq!(Assets::balance(asset1, pool_account2), 100 - 57);
		assert_eq!(Assets::balance(asset3, pool_account2), 300 - 173);
		assert_eq!(Assets::balance(lp_token_id2, user), 173 - 100);
	});
}

#[test]
fn remove_liquidity_passed_the_deadline_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(10);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		assert_noop!(
			Dex::remove_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 1, 2, 100, 10),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn remove_liquidity_with_zero_lp_amount_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		assert_noop!(
			Dex::remove_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 1, 2, 0, 100),
			Error::<Test>::NotEnoughLiquidityToken
		);
	});
}

#[test]
fn remove_liquidity_leads_to_remove_liquidity_did_not_meet_min_amount_error() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		assert_noop!(
			Dex::remove_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 100, 2, 100, 100),
			Error::<Test>::RemoveLiquidityDidNotMeetMinimumAmount
		);

		assert_noop!(
			Dex::remove_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 1, 200, 100, 100),
			Error::<Test>::RemoveLiquidityDidNotMeetMinimumAmount
		);
	});
}

#[test]
fn redeem_more_lp_tokens_than_total_issuance_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));

		assert_noop!(
			Dex::remove_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 100, 2, 100, 100),
			Error::<Test>::RemoveLiquidityDidNotMeetMinimumAmount
		);

		assert_noop!(
			Dex::remove_liquidity(RuntimeOrigin::signed(user), asset1, asset2, 1, 200, 142, 100),
			Error::<Test>::AmountMoreThanBalance
		);
	});
}

#[test]
fn can_query_price() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;
		let asset3 = 88;

		setup_account(user, vec![asset1, asset2, asset3]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 1000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset3, 1000, user));
		assert_ok!(Balances::mint_into(&user, 1000));

		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 100, 200, 10));
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset3, 100, 300, 10));
		System::reset_events();

		assert_ok!(Dex::price_oracle(RuntimeOrigin::signed(user), asset1, asset2, 10));

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::PriceInfo {
				querier: user,
				pool_id: (asset1, asset2),
				asset_amount: 10,
				asset_pool_reserve: 100,
				price_unit_pool_reserve: 200,
				marginal_price: 5
			}]
		);

		System::reset_events();
		assert_ok!(Dex::price_oracle(RuntimeOrigin::signed(user), asset2, asset1, 20));
		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::PriceInfo {
				querier: user,
				pool_id: (asset1, asset2),
				asset_amount: 20,
				asset_pool_reserve: 200,
				price_unit_pool_reserve: 100,
				marginal_price: 40
			}]
		);
		System::reset_events();

		assert_ok!(Dex::price_oracle(RuntimeOrigin::signed(user), asset1, asset3, 10));

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::PriceInfo {
				querier: user,
				pool_id: (asset1, asset3),
				asset_amount: 10,
				asset_pool_reserve: 100,
				price_unit_pool_reserve: 300,
				marginal_price: 3
			}]
		);

		assert_noop!(
			Dex::price_oracle(RuntimeOrigin::signed(user), asset1, 4, 20),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn can_swap_exact_in_for_out_between_two_supported_assets() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		let pool_id = (asset_in, asset_out);
		let pool_account = Dex::get_pool_account(&pool_id);
		let balance_before_swap1 =
			Assets::balance(asset_in, pool_account) + Assets::balance(asset_in, user);
		let balance_before_swap2 =
			Assets::balance(asset_out, pool_account) + Assets::balance(asset_out, user);

		assert_ok!(Dex::swap_exact_in_for_out(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			300,
			290,
			100
		));

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::SwapSucceeded {
				user,
				asset_in,
				asset_out,
				amount_in: 300,
				amount_out: 290,
			}]
		);

		assert_eq!(Assets::balance(asset_in, user), 20000 - 10000 - 300);
		assert_eq!(Assets::balance(asset_out, user), 20000 - 10000 + 290);
		assert_eq!(Assets::balance(asset_in, pool_account), 10000 + 300);
		assert_eq!(Assets::balance(asset_out, pool_account), 10000 - 290);

		assert_eq!(
			balance_before_swap1,
			Assets::balance(asset_in, user) + Assets::balance(asset_in, pool_account)
		);
		assert_eq!(
			balance_before_swap2,
			Assets::balance(asset_out, user) + Assets::balance(asset_out, pool_account)
		);
	});
}

#[test]
fn swap_exact_in_for_out_with_same_assets_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_exact_in_for_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_in,
				300,
				290,
				100
			),
			Error::<Test>::CannotSwapSameAsset
		);
	});
}

#[test]
fn swap_exact_in_for_out_between_unsupported_assets_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_exact_in_for_out(RuntimeOrigin::signed(user), asset_in, 33, 300, 290, 100),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn swap_exact_in_for_out_after_deadline_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_exact_in_for_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_out,
				300,
				290,
				1
			),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn swap_exact_in_for_out_with_zero_amount_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_exact_in_for_out(RuntimeOrigin::signed(user), asset_in, asset_out, 300, 0, 1),
			Error::<Test>::DeadlinePassed
		);

		assert_noop!(
			Dex::swap_exact_in_for_out(RuntimeOrigin::signed(user), asset_in, asset_out, 0, 100, 1),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn swap_exact_in_for_out_close_to_empty_pool_should_not_panic() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			1000,
			1000,
			10
		));

		System::reset_events();

		assert_ok!(Dex::swap_exact_in_for_out(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			500,
			100
		));
	});
}

#[test]
fn swap_exact_in_for_out_should_fail_if_too_much_slippage() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_exact_in_for_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_out,
				100,
				100,
				100
			),
			Error::<Test>::InsufficientMinimumForSwap
		);
	});
}

#[test]
fn swap_exact_in_for_out_with_existential_deposit_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_exact_in_for_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_out,
				10001,
				1,
				100
			),
			Error::<Test>::AmountMoreThanBalance
		);
	});
}

#[test]
fn can_swap_in_for_exact_out_between_two_supported_assets() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		let pool_id = (asset_in, asset_out);
		let pool_account = Dex::get_pool_account(&pool_id);
		let balance_before_swap1 =
			Assets::balance(asset_in, pool_account) + Assets::balance(asset_in, user);
		let balance_before_swap2 =
			Assets::balance(asset_out, pool_account) + Assets::balance(asset_out, user);

		assert_ok!(Dex::swap_in_for_exact_out(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			300,
			290,
			100
		));

		assert_eq!(
			System::events()
				.into_iter()
				.map(|r| r.event)
				.filter_map(|e| {
					if let RuntimeEvent::Dex(inner) = e {
						Some(inner)
					} else {
						None
					}
				})
				.collect::<Vec<_>>(),
			[Event::<Test>::SwapSucceeded {
				user,
				asset_in,
				asset_out,
				amount_in: 299,
				amount_out: 290,
			}]
		);

		assert_eq!(Assets::balance(asset_in, user), 20000 - 10000 - 299);
		assert_eq!(Assets::balance(asset_out, user), 20000 - 10000 + 290);
		assert_eq!(Assets::balance(asset_in, pool_account), 10000 + 299);
		assert_eq!(Assets::balance(asset_out, pool_account), 10000 - 290);

		assert_eq!(
			balance_before_swap1,
			Assets::balance(asset_in, user) + Assets::balance(asset_in, pool_account)
		);
		assert_eq!(
			balance_before_swap2,
			Assets::balance(asset_out, user) + Assets::balance(asset_out, pool_account)
		);
	});
}

#[test]
fn swap_in_for_exact_out_with_same_assets_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_in_for_exact_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_in,
				300,
				290,
				100
			),
			Error::<Test>::CannotSwapSameAsset
		);
	});
}

#[test]
fn swap_in_for_exact_out_between_unsupported_assets_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_in_for_exact_out(RuntimeOrigin::signed(user), asset_in, 44, 300, 290, 100),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn swap_in_for_exact_out_after_deadline_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_in_for_exact_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_out,
				300,
				290,
				1
			),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn swap_in_for_exact_out_with_zero_amount_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_in_for_exact_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_in,
				0,
				290,
				100
			),
			Error::<Test>::CannotSwapZeroAmount
		);

		assert_noop!(
			Dex::swap_in_for_exact_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_in,
				300,
				0,
				100
			),
			Error::<Test>::CannotSwapZeroAmount
		);
	});
}

#[test]
fn swap_in_for_exact_out_close_to_empty_pool_should_not_panic() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			1000,
			1000,
			10
		));

		assert_ok!(Dex::swap_in_for_exact_out(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			19000,
			900,
			100
		));
	});
}

#[test]
fn swap_in_for_exact_out_should_fail_if_too_much_slippage() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_in_for_exact_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_out,
				100,
				100,
				100
			),
			Error::<Test>::InsufficientMaximumForSwap
		);
	});
}

#[test]
fn swap_in_for_exact_out_with_existential_deposit_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset_in = 66;
		let asset_out = 77;

		setup_account(user, vec![asset_in, asset_out]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_in, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset_out, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(
			RuntimeOrigin::signed(user),
			asset_in,
			asset_out,
			10000,
			10000,
			10
		));

		System::reset_events();

		assert_noop!(
			Dex::swap_in_for_exact_out(
				RuntimeOrigin::signed(user),
				asset_in,
				asset_out,
				10001,
				1,
				100
			),
			Error::<Test>::AmountMoreThanBalance
		);
	});
}

#[test]
fn destroy_non_empty_pool_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 10000, 10000, 10));

		assert_noop!(
			Dex::destroy_pool(RuntimeOrigin::signed(user), asset1, asset2),
			Error::<Test>::CannotDestroyPoolWithLiquidity
		);
	});
}

#[test]
fn destroy_pool_doesnt_exist_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let user = 1;
		frame_system::Pallet::<Test>::inc_providers(&user);
		let asset1 = 66;
		let asset2 = 77;

		setup_account(user, vec![asset1, asset2]);
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset1, 20000, user));
		assert_ok!(Dex::mint_asset(RuntimeOrigin::root(), asset2, 20000, user));
		assert_ok!(Balances::mint_into(&user, 1000));
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(user), asset1, asset2, 10000, 10000, 10));

		assert_noop!(
			Dex::destroy_pool(RuntimeOrigin::signed(user), 3, 4),
			Error::<Test>::PoolNotFound
		);
	});
}
