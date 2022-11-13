#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
// use core::time::Duration;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use scale_info::TypeInfo;
use sp_std::{collections::vec_deque::VecDeque, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Define the Event Data type
#[derive(Clone, Encode, Decode, TypeInfo)]
pub struct EventInfo {
	/// Event data
	data: Vec<u8>,
	inserted_at: u64,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Get, StorageVersion, UnixTime},
		weights::Weight,
	};
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TimeProvider: UnixTime;
		#[pallet::constant]
		type Period: Get<u64>;
		#[pallet::constant]
		type AuthorizedOracleAccount: Get<Self::AccountId>;
	}

	#[pallet::storage]
	pub type EventFeedData<T: Config> = StorageValue<_, VecDeque<Option<EventInfo>>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NewEventAdded(Vec<u8>, u64),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		UnAuthorizedAccount,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Remove the event data older than an hour
		fn on_finalize(_now: T::BlockNumber) {
			let now: u64 = T::TimeProvider::now().as_secs();
			// get the period, 3600 seconds (1 hour)
			let period: u64 = T::Period::get();
			// Iterate over the stored event feed and remove all the event data that was inserted
			// more than an hour ago
			// Break out of the loop once you find a valid event, i.e., that was inserted within an
			// hour window
			EventFeedData::<T>::mutate(|queue| {
				while queue.len() > 0 {
					let event = queue.front().unwrap().as_ref().unwrap();
					if now > event.inserted_at + period {
						queue.pop_front();
					} else {
						break
					}
				}
			});
		}
	}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_new_event_data(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			let oracle_account = T::AuthorizedOracleAccount::get();
			if oracle_account == who {
				// Get the current time
				let now: u64 = T::TimeProvider::now().as_secs();
				// Construct the event data to be added to the feed
				let event_data = EventInfo { data: data.clone(), inserted_at: now };
				// Insert the event_data into the feed.
				EventFeedData::<T>::mutate(|event_feed_buffer| {
					event_feed_buffer.push_back(Some(event_data))
				});
				// Emit an event
				Self::deposit_event(Event::NewEventAdded(data, now));

				Ok(())
			} else {
				return Err(Error::<T>::UnAuthorizedAccount.into())
			}
		}
	}
}
