use super::*;
use crate::mock::*;
use frame_support::{assert_ok, traits::Hooks};
use frame_system::RawOrigin;

#[test]
fn authorized_oracle_event_insertion_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let data = "moshimoshi".as_bytes().to_vec();
		assert!(pallet::EventFeedData::<Test>::get().len() == 0);
		assert_ok!(EventFeedModule::add_new_event_data(RawOrigin::Signed(KAMISAMA).into(), data));
		// Read pallet storage and assert an expected result.
		assert!(pallet::EventFeedData::<Test>::get().len() > 0);
	});
}

#[should_panic]
#[test]
fn unauthorized_oracle_does_not_work() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		// Dispatch a signed extrinsic.
		const SPAMMER: AccountId = AccountId::new([0u8; 32]);
		let data = "moshimoshi".as_bytes().to_vec();
		assert_ok!(EventFeedModule::add_new_event_data(RawOrigin::Signed(SPAMMER).into(), data));
		// Read pallet storage and assert an expected result.
		assert!(pallet::EventFeedData::<Test>::get().len() == 0);
	});
}

#[test]
fn older_event_date_removed() {
	new_test_ext().execute_with(|| {
		let data = "moshimoshi".as_bytes().to_vec();
		assert_ok!(EventFeedModule::add_new_event_data(
			RawOrigin::Signed(KAMISAMA).into(),
			data.clone()
		));
		// Check if the event data is being included in the chain storage
		let data_from_chain =
			EventFeedData::<Test>::get().back().unwrap().as_ref().unwrap().data.clone();
		assert_eq!(data, data_from_chain);
		// Move 5 minutes forward, 5 * 60 * 1000 millisecs
		// pallet_timestamp::Now::<Test>::mutate(|val| val.saturating_add(5 * 60 * 1000));
		pallet_timestamp::Now::<Test>::mutate(|val| *val += 5 * 60 * 1000);
		// NOTE: For the purposes of this test, manually call the on_finalize() hook for the
		// EventFeedModule
		// NOTE: We do not need to call the hooks for Timestamp module as we are directly updating
		// the timestamp value to simulate our desired conditions
		// EventFeedModule::on_finalize(10); // The blocknumber doesn't matter for our purposes here

		// add a new event
		let new_data = "gm fam!".as_bytes().to_vec();
		assert_ok!(EventFeedModule::add_new_event_data(
			RawOrigin::Signed(KAMISAMA).into(),
			new_data.clone()
		));
		// Check if the new event data is being included in the chain storage
		let new_data_from_chain =
			EventFeedData::<Test>::get().back().unwrap().as_ref().unwrap().data.clone();
		assert_eq!(new_data, new_data_from_chain);

		EventFeedModule::on_finalize(10); // Trigger the on_finalize() hook

		// Check if the front of the VecDeque is the older event data ("moshimoshi")
		let front_event_feed_data =
			EventFeedData::<Test>::get().front().unwrap().as_ref().unwrap().data.clone();
		assert_eq!(front_event_feed_data, data);
		// Now 56 minutes forward, 56 * 60 * 1000 millisecs
		pallet_timestamp::Now::<Test>::mutate(|val| *val += 56 * 60 * 1000);
		// NOTE: For the purposes of this test, manually call the on_finalize() hook for the
		// EventFeedModule
		// NOTE: We do not need to call the hooks for Timestamp module as we are directly updating
		// the timestamp value to simulate our desired conditions
		EventFeedModule::on_finalize(10); // The blocknumber doesn't matter for our purposes here

		// Check if the event data ("moshimoshi") still exists in storage or is cleaned away as
		// intended
		let newer_front_event_feed_data =
			EventFeedData::<Test>::get().front().unwrap().as_ref().unwrap().data.clone();

		// Ideally, the older event feed data should have gotten purged
		assert_ne!(data, newer_front_event_feed_data);
		// It should be equal to the
		assert_eq!(newer_front_event_feed_data, new_data);

		// Move 5 minutes forward adn check if the one remaining event has been cleared or not
		pallet_timestamp::Now::<Test>::mutate(|val| *val += 5 * 60 * 1000);
		EventFeedModule::on_finalize(10);

		let total_event_feed_items = EventFeedData::<Test>::get().len();
		assert_eq!(total_event_feed_items, 0);
	});
}
