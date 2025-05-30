#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn send_message() {
        let caller: T::AccountId = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, 0);
        let content_cid = vec![0u8; 32]; // 32 bytes IPFS CID
        
        #[extrinsic_call]
        send_message(RawOrigin::Signed(caller), recipient, content_cid);
    }
    
    #[benchmark]
    fn read_message() {
        let sender: T::AccountId = account("sender", 0, 0);
        let recipient: T::AccountId = whitelisted_caller();
        let content_cid = vec![0u8; 32]; // 32 bytes IPFS CID
        
        // Setup: Send a message first
        let _ = Pallet::<T>::send_message(
            RawOrigin::Signed(sender.clone()).into(),
            recipient.clone(),
            content_cid,
        );
        
        let message_id = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&(
            &sender,
            &recipient,
            &frame_system::Pallet::<T>::block_number()
        ));
        
        #[extrinsic_call]
        read_message(RawOrigin::Signed(recipient), message_id);
    }
    
    #[benchmark]
    fn delete_message() {
        let sender: T::AccountId = account("sender", 0, 0);
        let recipient: T::AccountId = whitelisted_caller();
        let content_cid = vec![0u8; 32]; // 32 bytes IPFS CID
        
        // Setup: Send a message first
        let _ = Pallet::<T>::send_message(
            RawOrigin::Signed(sender.clone()).into(),
            recipient.clone(),
            content_cid,
        );
        
        let message_id = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&(
            &sender,
            &recipient,
            &frame_system::Pallet::<T>::block_number()
        ));
        
        #[extrinsic_call]
        delete_message(RawOrigin::Signed(recipient), message_id);
    }
    
    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}