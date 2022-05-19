// This file is part of Acala.

// Copyright (C) 2022 Acala Foundation.
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

//! Unit tests for the Aggregated DEX module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_runtime::FixedPointNumber;

#[test]
fn dex_swap_get_swap_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, AUSD, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);

		assert_ok!(inject_liquidity(
			DOT,
			AUSD,
			100_000_000_000u128,
			200_000_000_000_000u128
		));
		assert_ok!(inject_liquidity(
			LDOT,
			AUSD,
			1_000_000_000_000u128,
			200_000_000_000_000u128
		));

		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, AUSD, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 1_980_198_019_801u128))
		);

		set_dex_swap_joint_list(vec![vec![AUSD]]);
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 9_803_921_568u128))
		);

		assert_ok!(inject_liquidity(DOT, LDOT, 100_000_000_000u128, 1_000_000_000_000u128));
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 9_900_990_099u128))
		);
	});
}

#[test]
fn dex_swap_swap_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(inject_liquidity(
			DOT,
			AUSD,
			100_000_000_000u128,
			200_000_000_000_000u128
		));
		assert_ok!(inject_liquidity(
			LDOT,
			AUSD,
			1_000_000_000_000u128,
			200_000_000_000_000u128
		));

		assert_noop!(
			DexSwap::<Runtime>::swap(&ALICE, DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Error::<Runtime>::CannotSwap
		);

		set_dex_swap_joint_list(vec![vec![AUSD]]);
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 100_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 0);

		assert_noop!(
			DexSwap::<Runtime>::swap(
				&ALICE,
				DOT,
				LDOT,
				SwapLimit::ExactSupply(1_000_000_000u128, 10_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);
		assert_ok!(DexSwap::<Runtime>::swap(
			&ALICE,
			DOT,
			LDOT,
			SwapLimit::ExactSupply(1_000_000_000u128, 5_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 99_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 9_803_921_568u128);

		assert_noop!(
			DexSwap::<Runtime>::swap(
				&ALICE,
				LDOT,
				DOT,
				SwapLimit::ExactTarget(9_803_921_568u128, 1_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);
		assert_ok!(DexSwap::<Runtime>::swap(
			&ALICE,
			LDOT,
			DOT,
			SwapLimit::ExactTarget(9_803_921_568u128, 500_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 99_500_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 4_950_495_048u128);
	});
}

#[test]
fn taiga_swap_get_swap_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)),
			None
		);

		set_taiga_swap(DOT, LDOT, ExchangeRate::saturating_from_rational(10, 1));
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, AUSD, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 10_000_000_000u128))
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(
				DOT,
				LDOT,
				SwapLimit::ExactSupply(1_000_000_000u128, 10_000_000_001u128)
			),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, AUSD, SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)),
			Some((1_000_000_000u128, 10_000_000_000u128))
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(
				DOT,
				LDOT,
				SwapLimit::ExactTarget(999_999_999u128, 10_000_000_000u128)
			),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(
				LDOT,
				DOT,
				SwapLimit::ExactTarget(100_000_000_000u128, 1_000_000_001u128)
			),
			Some((10_000_000_010u128, 1_000_000_001u128))
		);
	});
}

#[test]
fn taiga_swap_swap_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			TaigaSwap::<Runtime>::swap(
				&ALICE,
				LDOT,
				DOT,
				SwapLimit::ExactSupply(1_000_000_000u128, 10_000_000_001u128)
			),
			Error::<Runtime>::CannotSwap
		);
		assert_noop!(
			TaigaSwap::<Runtime>::swap(
				&ALICE,
				LDOT,
				DOT,
				SwapLimit::ExactTarget(1_000_000_000u128, 10_000_000_001u128)
			),
			Error::<Runtime>::CannotSwap
		);

		set_taiga_swap(DOT, LDOT, ExchangeRate::saturating_from_rational(10, 1));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 100_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 0);

		assert_ok!(TaigaSwap::<Runtime>::swap(
			&ALICE,
			DOT,
			LDOT,
			SwapLimit::ExactSupply(1_000_000_000u128, 10_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 99_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 10_000_000_000u128);

		assert_noop!(
			TaigaSwap::<Runtime>::swap(
				&ALICE,
				DOT,
				LDOT,
				SwapLimit::ExactSupply(1_000_000_000u128, 10_000_000_001u128)
			),
			Error::<Runtime>::CannotSwap
		);
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 99_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 10_000_000_000u128);

		assert_ok!(TaigaSwap::<Runtime>::swap(
			&ALICE,
			DOT,
			LDOT,
			SwapLimit::ExactTarget(10_000_000_000u128, 10_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 98_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 20_000_000_000u128);

		assert_noop!(
			TaigaSwap::<Runtime>::swap(
				&ALICE,
				DOT,
				LDOT,
				SwapLimit::ExactTarget(999_999_999u128, 10_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);
	});
}

#[test]
fn either_dex_or_taiga_swap_get_swap_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);
		assert_eq!(
			EitherDexOrTaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)),
			None
		);
		assert_eq!(
			EitherDexOrTaigaSwap::<Runtime>::get_swap_amount(
				DOT,
				LDOT,
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			None
		);

		set_taiga_swap(DOT, LDOT, ExchangeRate::saturating_from_rational(10, 1));
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 10_000_000_000u128))
		);
		assert_eq!(
			EitherDexOrTaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 10_000_000_000u128))
		);
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)),
			Some((1_000_000_000u128, 10_000_000_000u128))
		);
		assert_eq!(
			EitherDexOrTaigaSwap::<Runtime>::get_swap_amount(
				DOT,
				LDOT,
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			Some((1_000_000_000u128, 10_000_000_000u128))
		);

		assert_ok!(inject_liquidity(DOT, LDOT, 1_000_000_000u128, 30_000_000_000u128));
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 15_000_000_000u128))
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 10_000_000_000u128))
		);
		assert_eq!(
			EitherDexOrTaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Some((1_000_000_000u128, 15_000_000_000u128))
		);
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(
				DOT,
				LDOT,
				SwapLimit::ExactTarget(1_000_000_000u128, 10_000_000_000u128)
			),
			Some((500_000_001u128, 10_000_000_000u128))
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(
				DOT,
				LDOT,
				SwapLimit::ExactTarget(1_000_000_000u128, 10_000_000_000u128)
			),
			Some((1_000_000_000u128, 10_000_000_000u128))
		);
		assert_eq!(
			EitherDexOrTaigaSwap::<Runtime>::get_swap_amount(
				DOT,
				LDOT,
				SwapLimit::ExactTarget(1_000_000_000u128, 10_000_000_000u128)
			),
			Some((500_000_001u128, 10_000_000_000u128))
		);

		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(10_000_000_000u128, 0)),
			Some((10_000_000_000u128, 27_272_727_272u128))
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(10_000_000_000u128, 0)),
			Some((10_000_000_000u128, 100_000_000_000u128))
		);
		assert_eq!(
			EitherDexOrTaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactSupply(10_000_000_000u128, 0)),
			Some((10_000_000_000u128, 100_000_000_000u128))
		);
		assert_eq!(
			DexSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactTarget(u128::MAX, 30_000_000_000u128)),
			None
		);
		assert_eq!(
			TaigaSwap::<Runtime>::get_swap_amount(DOT, LDOT, SwapLimit::ExactTarget(u128::MAX, 30_000_000_000u128)),
			Some((3_000_000_000u128, 30_000_000_000u128))
		);
		assert_eq!(
			EitherDexOrTaigaSwap::<Runtime>::get_swap_amount(
				DOT,
				LDOT,
				SwapLimit::ExactTarget(u128::MAX, 30_000_000_000u128)
			),
			Some((3_000_000_000u128, 30_000_000_000u128))
		);
	});
}

#[test]
fn either_dex_or_taiga_swap_swap_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EitherDexOrTaigaSwap::<Runtime>::swap(&ALICE, DOT, LDOT, SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Error::<Runtime>::CannotSwap
		);
		assert_noop!(
			EitherDexOrTaigaSwap::<Runtime>::swap(
				&ALICE,
				DOT,
				LDOT,
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);

		set_taiga_swap(DOT, LDOT, ExchangeRate::saturating_from_rational(10, 1));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 100_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 0);

		assert_noop!(
			EitherDexOrTaigaSwap::<Runtime>::swap(
				&ALICE,
				DOT,
				LDOT,
				SwapLimit::ExactSupply(1_000_000_000u128, 10_000_000_001u128)
			),
			Error::<Runtime>::CannotSwap
		);
		assert_ok!(EitherDexOrTaigaSwap::<Runtime>::swap(
			&ALICE,
			DOT,
			LDOT,
			SwapLimit::ExactSupply(1_000_000_000u128, 10_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 99_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 10_000_000_000u128);

		assert_noop!(
			EitherDexOrTaigaSwap::<Runtime>::swap(
				&ALICE,
				DOT,
				LDOT,
				SwapLimit::ExactTarget(999_999_999u128, 10_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);
		assert_ok!(EitherDexOrTaigaSwap::<Runtime>::swap(
			&ALICE,
			DOT,
			LDOT,
			SwapLimit::ExactTarget(1_000_000_000u128, 10_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 98_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 20_000_000_000u128);

		assert_ok!(inject_liquidity(DOT, LDOT, 100_000_000_000u128, 2_000_000_000_000u128));
		assert_ok!(EitherDexOrTaigaSwap::<Runtime>::swap(
			&ALICE,
			DOT,
			LDOT,
			SwapLimit::ExactSupply(1_000_000_000u128, 10_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 97_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 39_801_980_198u128);

		assert_ok!(EitherDexOrTaigaSwap::<Runtime>::swap(
			&ALICE,
			DOT,
			LDOT,
			SwapLimit::ExactTarget(1_000_000_000u128, 10_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 964_873_611_73u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 49_801_980_198u128);
	});
}

#[test]
fn do_aggregated_swap_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AggregatedDex::do_aggregated_swap(&ALICE, vec![], SwapLimit::ExactSupply(1_000_000_000u128, 0)),
			Error::<Runtime>::InvalidSwapPath
		);
		assert_noop!(
			AggregatedDex::do_aggregated_swap(&ALICE, vec![], SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)),
			Error::<Runtime>::InvalidSwapPath
		);

		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Dex(vec![])],
				SwapLimit::ExactSupply(1_000_000_000u128, 0)
			),
			Error::<Runtime>::InvalidSwapPath
		);
		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Dex(vec![])],
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			Error::<Runtime>::InvalidSwapPath
		);

		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Dex(vec![LDOT])],
				SwapLimit::ExactSupply(1_000_000_000u128, 0)
			),
			module_dex::Error::<Runtime>::InvalidTradingPathLength
		);
		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Dex(vec![LDOT])],
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);

		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Dex(vec![LDOT, AUSD])],
				SwapLimit::ExactSupply(1_000_000_000u128, 0)
			),
			module_dex::Error::<Runtime>::MustBeEnabled
		);
		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Dex(vec![LDOT, AUSD])],
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);

		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Taiga(0, 0, 1)],
				SwapLimit::ExactSupply(1_000_000_000u128, 0)
			),
			Error::<Runtime>::InvalidPoolId
		);
		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Taiga(0, 0, 1)],
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			Error::<Runtime>::InvalidPoolId
		);

		set_taiga_swap(DOT, LDOT, ExchangeRate::saturating_from_rational(10, 1));

		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Taiga(0, 0, 1), SwapPath::Dex(vec![LDOT, AUSD]),],
				SwapLimit::ExactSupply(1_000_000_000u128, 0)
			),
			module_dex::Error::<Runtime>::MustBeEnabled
		);
		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Taiga(0, 0, 1), SwapPath::Dex(vec![AUSD, LDOT]),],
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);

		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Taiga(0, 0, 2)],
				SwapLimit::ExactSupply(1_000_000_000u128, 0)
			),
			Error::<Runtime>::InvalidTokenIndex
		);
		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Taiga(0, 0, 2)],
				SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
			),
			Error::<Runtime>::InvalidTokenIndex
		);

		assert_eq!(Tokens::free_balance(DOT, &ALICE), 100_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 0);

		assert_ok!(AggregatedDex::do_aggregated_swap(
			&ALICE,
			vec![SwapPath::Taiga(0, 0, 1)],
			SwapLimit::ExactSupply(1_000_000_000u128, 0)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 99_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 10_000_000_000u128);
		assert_eq!(Tokens::free_balance(AUSD, &ALICE), 0);

		assert_ok!(AggregatedDex::do_aggregated_swap(
			&ALICE,
			vec![SwapPath::Taiga(0, 0, 1)],
			SwapLimit::ExactTarget(u128::MAX, 10_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 98_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 20_000_000_000u128);
		assert_eq!(Tokens::free_balance(AUSD, &ALICE), 0);

		assert_ok!(inject_liquidity(
			LDOT,
			AUSD,
			100_000_000_000u128,
			20_000_000_000_000u128
		));

		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Dex(vec![LDOT, AUSD])],
				SwapLimit::ExactSupply(1_000_000_000u128, 200_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);
		assert_noop!(
			AggregatedDex::do_aggregated_swap(
				&ALICE,
				vec![SwapPath::Dex(vec![LDOT, AUSD])],
				SwapLimit::ExactTarget(1_000_000_000u128, 200_000_000_000u128)
			),
			Error::<Runtime>::CannotSwap
		);

		assert_ok!(AggregatedDex::do_aggregated_swap(
			&ALICE,
			vec![SwapPath::Dex(vec![LDOT, AUSD])],
			SwapLimit::ExactSupply(1_000_000_000u128, 0)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 98_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 19_000_000_000u128);
		assert_eq!(Tokens::free_balance(AUSD, &ALICE), 198_019_801_980u128);

		assert_ok!(AggregatedDex::do_aggregated_swap(
			&ALICE,
			vec![SwapPath::Dex(vec![LDOT, AUSD])],
			SwapLimit::ExactTarget(1_000_000_000u128, 10_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 98_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 18_948_969_229u128);
		// actually swap by ExactSupply, actual target amount may be slightly more than exact target amount
		// of limit
		assert_eq!(Tokens::free_balance(AUSD, &ALICE), 208_019_802_070u128);

		assert_ok!(AggregatedDex::do_aggregated_swap(
			&ALICE,
			vec![SwapPath::Taiga(0, 0, 1), SwapPath::Dex(vec![LDOT, AUSD]),],
			SwapLimit::ExactSupply(1_000_000_000u128, 0)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 97_000_000_000u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 18_948_969_229u128);
		assert_eq!(Tokens::free_balance(AUSD, &ALICE), 1_990_261_719_188u128);

		assert_ok!(AggregatedDex::do_aggregated_swap(
			&ALICE,
			vec![SwapPath::Taiga(0, 0, 1), SwapPath::Dex(vec![LDOT, AUSD]),],
			SwapLimit::ExactTarget(1_000_000_000_000u128, 1_000_000_000_000u128)
		));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), 96_347_132_631u128);
		assert_eq!(Tokens::free_balance(LDOT, &ALICE), 18_948_969_229u128);
		// actually swap by ExactSupply, actual target amount may be slightly more than exact target amount
		// of limit
		assert_eq!(Tokens::free_balance(AUSD, &ALICE), 2_990_261_719_330u128);
	});
}
