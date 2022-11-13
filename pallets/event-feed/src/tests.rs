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
fn event_data_removed_after_an_hour() {
	new_test_ext().execute_with(|| {
		let data = "moshimoshi".as_bytes().to_vec();
		assert_ok!(EventFeedModule::add_new_event_data(
			RawOrigin::Signed(KAMISAMA).into(),
			data.clone()
		));
		// Check if the event data was posted to the chain
		let data_from_chain =
			EventFeedData::<Test>::get().back().unwrap().as_ref().unwrap().data.clone();
		assert_eq!(data, data_from_chain);
		// Move 5 minutes forward, 5 * 60 * 1000 millisecs
		pallet_timestamp::Now::<Test>::mutate(|val| *val += 5 * 60 * 1000);
		// NOTE: For the purposes of this test, we manually call the on_finalize() hook of the
		// EventFeedModule
		// NOTE: We do not need to call the hooks for Timestamp module as we are directly updating
		// the timestamp value to simulate our desired conditions
		EventFeedModule::on_finalize(10); // The blocknumber doesn't matter for our purposes here

		// post a new event
		let new_data = "sob sob... screw that mfer sbf".as_bytes().to_vec();
		assert_ok!(EventFeedModule::add_new_event_data(
			RawOrigin::Signed(KAMISAMA).into(),
			new_data.clone()
		));
		// Check if the new event data was being posted
		let new_data_from_chain =
			EventFeedData::<Test>::get().back().unwrap().as_ref().unwrap().data.clone();
		assert_eq!(new_data, new_data_from_chain);

		EventFeedModule::on_finalize(10); // Trigger the on_finalize() hook

		// Check if the front of the Event Feed VecDeque points to the older event ("moshimoshi")
		let front_event_feed_data =
			EventFeedData::<Test>::get().front().unwrap().as_ref().unwrap().data.clone();
		assert_eq!(front_event_feed_data, data);
		// Move 56 minutes forward so that the first event is purged, 56 * 60 * 1000 millisecs
		pallet_timestamp::Now::<Test>::mutate(|val| *val += 56 * 60 * 1000);
		EventFeedModule::on_finalize(10); // The blocknumber doesn't matter for our purposes here

		// Check if the first event got purged away as it doesn't fall in our 1 hour window anymore
		let newer_front_event_feed_data =
			EventFeedData::<Test>::get().front().unwrap().as_ref().unwrap().data.clone();

		// Ideally, the older event feed data should have gotten purged
		assert_ne!(data, newer_front_event_feed_data);
		// It should be equal to the second event posted
		assert_eq!(newer_front_event_feed_data, new_data);

		// Move 5 minutes forward and check if the one remaining event has been cleared or not
		pallet_timestamp::Now::<Test>::mutate(|val| *val += 5 * 60 * 1000);
		EventFeedModule::on_finalize(10);

		// There should not be any events left
		let total_event_feed_items = EventFeedData::<Test>::get().len();
		assert_eq!(total_event_feed_items, 0);
	});
}
