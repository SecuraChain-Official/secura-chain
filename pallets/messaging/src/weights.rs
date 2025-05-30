#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_messaging.
pub trait WeightInfo {
    fn send_message() -> Weight;
    fn read_message() -> Weight;
    fn delete_message() -> Weight;
    fn create_group() -> Weight;
    fn add_member() -> Weight;
    fn remove_member() -> Weight;
    fn send_group_message() -> Weight;
    fn on_initialize() -> Weight;
}

/// Weights for pallet_messaging using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn send_message() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    
    fn read_message() -> Weight {
        Weight::from_parts(5_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn delete_message() -> Weight {
        Weight::from_parts(5_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    
    fn create_group() -> Weight {
        Weight::from_parts(15_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(52)) // 1 group + up to 50 memberships + 1 event
    }
    
    fn add_member() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    
    fn remove_member() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    
    fn send_group_message() -> Weight {
        Weight::from_parts(15_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    
    fn on_initialize() -> Weight {
        Weight::from_parts(2_000, 0)
    }
}

// For tests
impl WeightInfo for () {
    fn send_message() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn read_message() -> Weight {
        Weight::from_parts(5_000, 0)
    }
    
    fn delete_message() -> Weight {
        Weight::from_parts(5_000, 0)
    }
    
    fn create_group() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    
    fn add_member() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn remove_member() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn send_group_message() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    
    fn on_initialize() -> Weight {
        Weight::from_parts(2_000, 0)
    }
}