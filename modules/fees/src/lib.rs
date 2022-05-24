// This file is part of Acala.

// Copyright (C) 2020-2022 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! # Network Fee Distribution & Incentive Pools Module

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::traits::Imbalance;
use frame_support::{
	pallet_prelude::*,
	parameter_types,
	traits::{Currency, OnUnbalanced},
	transactional,
};
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrency;
use primitives::{Balance, CurrencyId, IncomeSource};
use sp_runtime::FixedPointNumber;
use sp_std::vec::Vec;
use support::{FeeToTreasuryPool, Rate};

pub use module::*;

// mod mock;
// mod tests;
pub mod weights;
pub use weights::WeightInfo;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct PoolPercent<AccountId> {
	pool: AccountId,
	rate: Rate,
}

// type PalletBalanceOf<T> = <<T as Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

pub type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

#[frame_support::pallet]
pub mod module {
	use super::*;

	parameter_types! {
		pub const MaxSize: u8 = 10;
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		// #[pallet::constant]
		// type NativeCurrencyId: Get<CurrencyId>;

		type Currency: Currency<Self::AccountId>;

		type Currencies: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		#[pallet::constant]
		type NetworkTreasuryPoolAccount: Get<Self::AccountId>;

		// type OnUnbalanced: OnUnbalanced<NegativeImbalanceOf<Self>>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidParams,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		IncomeFeeSet {
			income: IncomeSource,
			pools: Vec<PoolPercent<T::AccountId>>,
		},
		TreasuryPoolSet {
			treasury: T::AccountId,
			pools: Vec<PoolPercent<T::AccountId>>,
		},
	}

	/// Income fee source mapping to different treasury pools.
	///
	/// IncomeToTreasuries: map IncomeSource => Vec<PoolPercent>
	#[pallet::storage]
	#[pallet::getter(fn income_to_treasuries)]
	pub type IncomeToTreasuries<T: Config> =
		StorageMap<_, Twox64Concat, IncomeSource, BoundedVec<PoolPercent<T::AccountId>, MaxSize>, ValueQuery>;

	/// Treasury pool allocation mapping to different income pools.
	///
	/// TreasuryToIncentives: map AccountId => Vec<PoolPercent>
	#[pallet::storage]
	#[pallet::getter(fn treasury_to_incentives)]
	pub type TreasuryToIncentives<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<PoolPercent<T::AccountId>, MaxSize>, ValueQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// TODO: trigger transfer from treasury pool to incentive pools
			<T as Config>::WeightInfo::on_initialize()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set how much percentage of income fee go to different treasury pools
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn set_income_fee(
			origin: OriginFor<T>,
			income_source: IncomeSource,
			treasury_pool_rates: Vec<(T::AccountId, u32)>,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;

			Self::do_set_treasury_rate(income_source, treasury_pool_rates)
		}

		/// Set how much percentage of treasury pool go to different incentive pools
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn set_treasury_pool(
			origin: OriginFor<T>,
			treasury: T::AccountId,
			incentive_pools: Vec<(T::AccountId, u32)>,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;

			Self::do_set_incentive_rate(treasury, incentive_pools)
		}

		/// Force transfer token from treasury pool to incentive pool.
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn force_transfer_to_incentive(
			origin: OriginFor<T>,
			_treasury: T::AccountId,
			_incentive: T::AccountId,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_set_treasury_rate(
		income_source: IncomeSource,
		treasury_pool_rates: Vec<(T::AccountId, u32)>,
	) -> DispatchResult {
		let pools: Vec<PoolPercent<T::AccountId>> = treasury_pool_rates
			.into_iter()
			.map(|p| {
				let rate = Rate::saturating_from_rational(p.1, 100);
				PoolPercent { pool: p.0, rate }
			})
			.collect();

		IncomeToTreasuries::<T>::try_mutate(income_source, |rates| -> DispatchResult {
			let percents: BoundedVec<PoolPercent<T::AccountId>, MaxSize> =
				pools.clone().try_into().map_err(|_| Error::<T>::InvalidParams)?;
			*rates = percents;
			Ok(())
		})?;

		Self::deposit_event(Event::IncomeFeeSet {
			income: income_source,
			pools,
		});
		Ok(())
	}

	fn do_set_incentive_rate(treasury: T::AccountId, incentive_pools: Vec<(T::AccountId, u32)>) -> DispatchResult {
		let pools: Vec<PoolPercent<T::AccountId>> = incentive_pools
			.into_iter()
			.map(|p| {
				let rate = Rate::saturating_from_rational(p.1, 100);
				PoolPercent { pool: p.0, rate }
			})
			.collect();

		TreasuryToIncentives::<T>::try_mutate(&treasury, |rates| -> DispatchResult {
			let percents: BoundedVec<PoolPercent<T::AccountId>, MaxSize> =
				pools.clone().try_into().map_err(|_| Error::<T>::InvalidParams)?;
			*rates = percents;
			Ok(())
		})?;

		Self::deposit_event(Event::TreasuryPoolSet { treasury, pools });
		Ok(())
	}
}

impl<T: Config + Send + Sync> FeeToTreasuryPool<T::AccountId, CurrencyId, Balance> for Pallet<T> {
	// TODO: maybe use `Happened<(AccountId,CurrencyId,Balance)>` instead of new trait?
	fn on_fee_changed(account_id: &T::AccountId, currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		// TODO: use `IncomeSource` to determine destination
		T::Currencies::deposit(currency_id, account_id, amount)
	}
}

// Transaction payment module `OnTransactionPayment` distribution transaction fee
impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for Pallet<T> {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalanceOf<T>>) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}

			// Must resolve into existing but better to be safe.
			let _ = T::Currency::resolve_creating(&T::NetworkTreasuryPoolAccount::get(), fees);
			// TODO: deposit event
		}
	}
}
