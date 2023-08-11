// TODO add types here
use super::*;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub(super) type PoolIdOf<T> = (<T as Config>::AssetId, <T as Config>::AssetId);
pub(super) type AssetBalanceOf<T> = <T as Config>::AssetBalance;

/// Stores the lp_token asset id a particular pool has been assigned.
#[derive(Decode, Encode, Default, PartialEq, Eq, MaxEncodedLen, TypeInfo, Debug)]
pub struct PoolInfo<LpTokenId> {
	/// Liquidity pool asset
	pub lp_token: LpTokenId,
}
