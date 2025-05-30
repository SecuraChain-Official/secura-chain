#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use crate::weights::WeightInfo;
    use frame_support::weights::Weight;
    use frame_support::sp_runtime::traits::Hash;
    use frame_support::sp_runtime::Saturating;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Maximum message length
        #[pallet::constant]
        type MaxMessageLength: Get<u32>;
        
        /// Message time-to-live in blocks
        #[pallet::constant]
        type MessageTTL: Get<BlockNumberFor<Self>>;
        
        /// Weight information
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn messages)]
    pub type Messages<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        Message<T::AccountId, BlockNumberFor<T>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn inbox)]
    pub type Inbox<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<T::Hash, ConstU32<100>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn outbox)]
    pub type Outbox<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<T::Hash, ConstU32<100>>,
        ValueQuery,
    >;
    
    // Group definition
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct Group<AccountId> {
        // Group owner
        pub owner: AccountId,
        // Group members
        pub members: BoundedVec<AccountId, ConstU32<50>>,
        // Group name
        pub name: BoundedVec<u8, ConstU32<32>>,
    }

    // Storage for groups
    #[pallet::storage]
    #[pallet::getter(fn groups)]
    pub type Groups<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,  // Group ID
        Group<T::AccountId>,
    >;

    // Group membership index
    #[pallet::storage]
    #[pallet::getter(fn group_membership)]
    pub type GroupMembership<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<T::Hash, ConstU32<50>>,  // Groups the user belongs to
        ValueQuery,
    >;

    // Group messages index
    #[pallet::storage]
    #[pallet::getter(fn group_messages)]
    pub type GroupMessages<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,  // Group ID
        BoundedVec<T::Hash, ConstU32<1000>>,  // Message IDs
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Message sent [message_id, sender, recipient]
        MessageSent(T::Hash, T::AccountId, T::AccountId),
        /// Message read [message_id, recipient]
        MessageRead(T::Hash, T::AccountId),
        /// Message deleted [message_id]
        MessageDeleted(T::Hash),
        /// Group created [group_id, owner]
        GroupCreated(T::Hash, T::AccountId),
        /// Member added to group [group_id, member]
        MemberAdded(T::Hash, T::AccountId),
        /// Member removed from group [group_id, member]
        MemberRemoved(T::Hash, T::AccountId),
        /// Group message sent [message_id, group_id, sender]
        GroupMessageSent(T::Hash, T::Hash, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Message too long
        MessageTooLong,
        /// Message not found
        MessageNotFound,
        /// Not authorized
        NotAuthorized,
        /// Inbox full
        InboxFull,
        /// Outbox full
        OutboxFull,
        /// Invalid IPFS CID
        InvalidCID,
        /// Group not found
        GroupNotFound,
        /// Not a group member
        NotGroupMember,
        /// Not the group owner
        NotGroupOwner,
        /// Group is full
        GroupFull,
        /// User is already a member
        AlreadyMember,
        /// User is in too many groups
        TooManyGroups,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct Message<AccountId, BlockNumber> {
        /// Sender
        pub sender: AccountId,
        /// Recipient
        pub recipient: AccountId,
        /// IPFS CID of encrypted content
        pub content_cid: BoundedVec<u8, ConstU32<64>>,
        /// When sent
        pub timestamp: BlockNumber,
        /// When expires
        pub expires_at: BlockNumber,
        /// Read status
        pub read: bool,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Send a new encrypted message with IPFS CID
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::send_message())]
        pub fn send_message(
            origin: OriginFor<T>,
            recipient: T::AccountId,
            content_cid: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Check CID size
            ensure!(content_cid.len() <= 64, Error::<T>::InvalidCID);
            
            let bounded_cid = BoundedVec::<u8, ConstU32<64>>::try_from(content_cid)
                .map_err(|_| Error::<T>::InvalidCID)?;
            
            let now = frame_system::Pallet::<T>::block_number();
            let expires_at = now.saturating_add(T::MessageTTL::get());
            
            // Create message
            let message = Message {
                sender: sender.clone(),
                recipient: recipient.clone(),
                content_cid: bounded_cid,
                timestamp: now,
                expires_at,
                read: false,
            };
            
            // Generate ID
            let message_id = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&(
                &sender,
                &recipient,
                &now
            ));
            
            // Store message
            Messages::<T>::insert(message_id, message);
            
            // Update recipient's inbox
            Inbox::<T>::try_mutate(&recipient, |messages| {
                messages.try_push(message_id)
            }).map_err(|_| Error::<T>::InboxFull)?;
            
            // Update sender's outbox
            Outbox::<T>::try_mutate(&sender, |messages| {
                messages.try_push(message_id)
            }).map_err(|_| Error::<T>::OutboxFull)?;
            
            // Emit event
            Self::deposit_event(Event::MessageSent(message_id, sender, recipient));
            
            Ok(())
        }
        
        /// Mark message as read
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::read_message())]
        pub fn read_message(
            origin: OriginFor<T>,
            message_id: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            Messages::<T>::try_mutate(message_id, |maybe_message| -> DispatchResult {
                let message = maybe_message.as_mut().ok_or(Error::<T>::MessageNotFound)?;
                ensure!(message.recipient == who, Error::<T>::NotAuthorized);
                
                message.read = true;
                Ok(())
            })?;
            
            Self::deposit_event(Event::MessageRead(message_id, who));
            
            Ok(())
        }
        
        /// Delete message
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::delete_message())]
        pub fn delete_message(
            origin: OriginFor<T>,
            message_id: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let message = Messages::<T>::get(message_id).ok_or(Error::<T>::MessageNotFound)?;
            ensure!(
                message.sender == who || message.recipient == who,
                Error::<T>::NotAuthorized
            );
            
            // Remove message
            Messages::<T>::remove(message_id);
            
            // Clean up inbox/outbox
            if message.recipient == who {
                Inbox::<T>::mutate(&who, |messages| {
                    if let Some(pos) = messages.iter().position(|id| *id == message_id) {
                        messages.swap_remove(pos);
                    }
                });
            }
            
            if message.sender == who {
                Outbox::<T>::mutate(&who, |messages| {
                    if let Some(pos) = messages.iter().position(|id| *id == message_id) {
                        messages.swap_remove(pos);
                    }
                });
            }
            
            Self::deposit_event(Event::MessageDeleted(message_id));
            
            Ok(())
        }
        
        /// Create a new group
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::create_group())]
        pub fn create_group(
            origin: OriginFor<T>,
            name: Vec<u8>,
            initial_members: Vec<T::AccountId>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            
            // Validate name length
            ensure!(name.len() <= 32, Error::<T>::MessageTooLong);
            let bounded_name = BoundedVec::<u8, ConstU32<32>>::try_from(name)
                .map_err(|_| Error::<T>::MessageTooLong)?;
            
            // Validate members count
            ensure!(initial_members.len() < 50, Error::<T>::GroupFull);
            
            // Create unique member list including owner
            let mut members = Vec::with_capacity(initial_members.len() + 1);
            members.push(owner.clone());
            for member in initial_members {
                if !members.contains(&member) {
                    members.push(member);
                }
            }
            
            let bounded_members = BoundedVec::<T::AccountId, ConstU32<50>>::try_from(members)
                .map_err(|_| Error::<T>::GroupFull)?;
            
            // Create group
            let group = Group {
                owner: owner.clone(),
                members: bounded_members.clone(),
                name: bounded_name.clone(),
            };
            
            // Generate group ID
            let group_id = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&(
                &owner,
                &bounded_name,
                &frame_system::Pallet::<T>::block_number()
            ));
            
            // Store group
            Groups::<T>::insert(group_id, group);
            
            // Update membership for all members
            for member in bounded_members.iter() {
                GroupMembership::<T>::try_mutate(member, |groups| {
                    groups.try_push(group_id)
                }).map_err(|_| Error::<T>::TooManyGroups)?;
            }
            
            Self::deposit_event(Event::GroupCreated(group_id, owner));
            
            Ok(())
        }

        /// Add member to group
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::add_member())]
        pub fn add_member(
            origin: OriginFor<T>,
            group_id: T::Hash,
            new_member: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            Groups::<T>::try_mutate(group_id, |maybe_group| -> DispatchResult {
                let group = maybe_group.as_mut().ok_or(Error::<T>::GroupNotFound)?;
                
                // Only owner can add members
                ensure!(group.owner == who, Error::<T>::NotGroupOwner);
                
                // Check if already a member
                ensure!(!group.members.contains(&new_member), Error::<T>::AlreadyMember);
                
                // Add to group
                group.members.try_push(new_member.clone())
                    .map_err(|_| Error::<T>::GroupFull)?;
                
                // Update membership
                GroupMembership::<T>::try_mutate(&new_member, |groups| {
                    groups.try_push(group_id)
                }).map_err(|_| Error::<T>::TooManyGroups)?;
                
                Self::deposit_event(Event::MemberAdded(group_id, new_member));
                
                Ok(())
            })
        }

        /// Remove member from group
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::remove_member())]
        pub fn remove_member(
            origin: OriginFor<T>,
            group_id: T::Hash,
            member: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            Groups::<T>::try_mutate(group_id, |maybe_group| -> DispatchResult {
                let group = maybe_group.as_mut().ok_or(Error::<T>::GroupNotFound)?;
                
                // Only owner can remove members
                ensure!(group.owner == who, Error::<T>::NotGroupOwner);
                
                // Cannot remove owner
                ensure!(group.owner != member, Error::<T>::NotAuthorized);
                
                // Remove from group
                if let Some(pos) = group.members.iter().position(|m| *m == member) {
                    group.members.swap_remove(pos);
                } else {
                    return Err(Error::<T>::NotGroupMember.into());
                }
                
                // Update membership
                GroupMembership::<T>::mutate(&member, |groups| {
                    if let Some(pos) = groups.iter().position(|g| *g == group_id) {
                        groups.swap_remove(pos);
                    }
                });
                
                Self::deposit_event(Event::MemberRemoved(group_id, member));
                
                Ok(())
            })
        }
        
        /// Send message to group
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::send_group_message())]
        pub fn send_group_message(
            origin: OriginFor<T>,
            group_id: T::Hash,
            content_cid: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Check group exists and sender is a member
            let group = Groups::<T>::get(group_id).ok_or(Error::<T>::GroupNotFound)?;
            ensure!(group.members.contains(&sender), Error::<T>::NotGroupMember);
            
            // Validate CID
            ensure!(content_cid.len() <= 64, Error::<T>::InvalidCID);
            let bounded_cid = BoundedVec::<u8, ConstU32<64>>::try_from(content_cid)
                .map_err(|_| Error::<T>::InvalidCID)?;
            
            let now = frame_system::Pallet::<T>::block_number();
            let expires_at = now.saturating_add(T::MessageTTL::get());
            
            // Create message
            let message = Message {
                sender: sender.clone(),
                recipient: sender.clone(), // For group messages, sender is also marked as recipient
                content_cid: bounded_cid,
                timestamp: now,
                expires_at,
                read: false,
            };
            
            // Generate message ID
            let message_id = <<T as frame_system::Config>::Hashing as Hash>::hash_of(&(
                &sender,
                &group_id,
                &now
            ));
            
            // Store message
            Messages::<T>::insert(message_id, message);
            
            // Add to group messages
            GroupMessages::<T>::try_mutate(group_id, |messages| {
                messages.try_push(message_id)
            }).map_err(|_| Error::<T>::InboxFull)?;
            
            // Add to sender's outbox
            Outbox::<T>::try_mutate(&sender, |messages| {
                messages.try_push(message_id)
            }).map_err(|_| Error::<T>::OutboxFull)?;
            
            Self::deposit_event(Event::GroupMessageSent(message_id, group_id, sender));
            
            Ok(())
        }
    }
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            // Could implement message expiration cleanup here
            Weight::zero()
        }
    }
}