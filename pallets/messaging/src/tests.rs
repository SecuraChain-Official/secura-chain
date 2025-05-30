use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::Hash;

#[test]
fn send_message_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let sender = 1;
        let recipient = 2;
        let content_cid = vec![1, 2, 3, 4]; // IPFS CID
        
        // Send a message
        assert_ok!(Messaging::send_message(RuntimeOrigin::signed(sender), recipient, content_cid.clone()));
        
        // Check event
        System::assert_last_event(Event::MessageSent(
            BlakeTwo256::hash_of(&(sender, recipient, 1)),
            sender,
            recipient,
        ).into());
        
        // Check storage
        let message_id = BlakeTwo256::hash_of(&(sender, recipient, 1));
        assert!(Messaging::messages(message_id).is_some());
        assert!(Messaging::inbox(recipient).contains(&message_id));
        assert!(Messaging::outbox(sender).contains(&message_id));
    });
}

#[test]
fn read_message_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let sender = 1;
        let recipient = 2;
        let content_cid = vec![1, 2, 3, 4]; // IPFS CID
        
        // Send a message
        assert_ok!(Messaging::send_message(RuntimeOrigin::signed(sender), recipient, content_cid.clone()));
        let message_id = BlakeTwo256::hash_of(&(sender, recipient, 1));
        
        // Read the message
        assert_ok!(Messaging::read_message(RuntimeOrigin::signed(recipient), message_id));
        
        // Check event
        System::assert_last_event(Event::MessageRead(message_id, recipient).into());
        
        // Check message is marked as read
        let message = Messaging::messages(message_id).unwrap();
        assert!(message.read);
    });
}

#[test]
fn delete_message_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let sender = 1;
        let recipient = 2;
        let content_cid = vec![1, 2, 3, 4]; // IPFS CID
        
        // Send a message
        assert_ok!(Messaging::send_message(RuntimeOrigin::signed(sender), recipient, content_cid.clone()));
        let message_id = BlakeTwo256::hash_of(&(sender, recipient, 1));
        
        // Delete the message as recipient
        assert_ok!(Messaging::delete_message(RuntimeOrigin::signed(recipient), message_id));
        
        // Check event
        System::assert_last_event(Event::MessageDeleted(message_id).into());
        
        // Check message is deleted
        assert!(Messaging::messages(message_id).is_none());
        assert!(!Messaging::inbox(recipient).contains(&message_id));
    });
}

#[test]
fn invalid_cid() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let sender = 1;
        let recipient = 2;
        let content_cid = vec![0; 100]; // CID too long
        
        // Attempt to send a message with invalid CID
        assert_noop!(
            Messaging::send_message(RuntimeOrigin::signed(sender), recipient, content_cid),
            Error::<Test>::InvalidCID
        );
    });
}

#[test]
fn unauthorized_read() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let sender = 1;
        let recipient = 2;
        let unauthorized = 3;
        let content_cid = vec![1, 2, 3, 4]; // IPFS CID
        
        // Send a message
        assert_ok!(Messaging::send_message(RuntimeOrigin::signed(sender), recipient, content_cid.clone()));
        let message_id = BlakeTwo256::hash_of(&(sender, recipient, 1));
        
        // Attempt to read by unauthorized user
        assert_noop!(
            Messaging::read_message(RuntimeOrigin::signed(unauthorized), message_id),
            Error::<Test>::NotAuthorized
        );
    });
}

// Group messaging tests

#[test]
fn create_group_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let owner = 1;
        let member1 = 2;
        let member2 = 3;
        let name = b"test group".to_vec();
        
        // Create a group
        assert_ok!(Messaging::create_group(
            RuntimeOrigin::signed(owner),
            name.clone(),
            vec![member1, member2]
        ));
        
        // Check group exists
        let group_id = BlakeTwo256::hash_of(&(owner, name.clone(), 1));
        let group = Messaging::groups(group_id).unwrap();
        
        assert_eq!(group.owner, owner);
        assert_eq!(group.members.len(), 3); // owner + 2 members
        assert!(group.members.contains(&owner));
        assert!(group.members.contains(&member1));
        assert!(group.members.contains(&member2));
        
        // Check memberships
        assert!(Messaging::group_membership(owner).contains(&group_id));
        assert!(Messaging::group_membership(member1).contains(&group_id));
        assert!(Messaging::group_membership(member2).contains(&group_id));
    });
}

#[test]
fn add_member_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let owner = 1;
        let member1 = 2;
        let new_member = 3;
        let name = b"test group".to_vec();
        
        // Create a group
        assert_ok!(Messaging::create_group(
            RuntimeOrigin::signed(owner),
            name.clone(),
            vec![member1]
        ));
        
        let group_id = BlakeTwo256::hash_of(&(owner, name.clone(), 1));
        
        // Add a new member
        assert_ok!(Messaging::add_member(
            RuntimeOrigin::signed(owner),
            group_id,
            new_member
        ));
        
        // Check member was added
        let group = Messaging::groups(group_id).unwrap();
        assert!(group.members.contains(&new_member));
        assert!(Messaging::group_membership(new_member).contains(&group_id));
    });
}

#[test]
fn remove_member_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let owner = 1;
        let member1 = 2;
        let member2 = 3;
        let name = b"test group".to_vec();
        
        // Create a group
        assert_ok!(Messaging::create_group(
            RuntimeOrigin::signed(owner),
            name.clone(),
            vec![member1, member2]
        ));
        
        let group_id = BlakeTwo256::hash_of(&(owner, name.clone(), 1));
        
        // Remove a member
        assert_ok!(Messaging::remove_member(
            RuntimeOrigin::signed(owner),
            group_id,
            member1
        ));
        
        // Check member was removed
        let group = Messaging::groups(group_id).unwrap();
        assert!(!group.members.contains(&member1));
        assert!(!Messaging::group_membership(member1).contains(&group_id));
    });
}

#[test]
fn send_group_message_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let owner = 1;
        let member = 2;
        let name = b"test group".to_vec();
        let content_cid = vec![1, 2, 3, 4]; // IPFS CID
        
        // Create a group
        assert_ok!(Messaging::create_group(
            RuntimeOrigin::signed(owner),
            name.clone(),
            vec![member]
        ));
        
        let group_id = BlakeTwo256::hash_of(&(owner, name.clone(), 1));
        
        // Send a group message
        assert_ok!(Messaging::send_group_message(
            RuntimeOrigin::signed(member),
            group_id,
            content_cid.clone()
        ));
        
        // Check message was stored
        let message_id = BlakeTwo256::hash_of(&(member, group_id, 1));
        assert!(Messaging::messages(message_id).is_some());
        assert!(Messaging::group_messages(group_id).contains(&message_id));
        assert!(Messaging::outbox(member).contains(&message_id));
    });
}

#[test]
fn non_member_cannot_send_group_message() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let owner = 1;
        let member = 2;
        let non_member = 3;
        let name = b"test group".to_vec();
        let content_cid = vec![1, 2, 3, 4]; // IPFS CID
        
        // Create a group
        assert_ok!(Messaging::create_group(
            RuntimeOrigin::signed(owner),
            name.clone(),
            vec![member]
        ));
        
        let group_id = BlakeTwo256::hash_of(&(owner, name.clone(), 1));
        
        // Attempt to send a group message as non-member
        assert_noop!(
            Messaging::send_group_message(
                RuntimeOrigin::signed(non_member),
                group_id,
                content_cid.clone()
            ),
            Error::<Test>::NotGroupMember
        );
    });
}