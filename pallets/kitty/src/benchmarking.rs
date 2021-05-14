//! Benchmarking setup for pallet-template

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::{boxed::Box, vec, vec::Vec};

#[allow(unused)]
use crate::Kitty;

benchmarks! {
    create_kitty {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller))
    verify {
        assert_eq!(true, true);
    }
}

// impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test,);
