use frame_system;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub enum EligibilityError {
	NotAllowed,
}

pub trait IsAuthorizedAccount<T: frame_system::Config> {
	pub fn is_valid(&self, account: AccountIdOf<T>) -> Result<(), EligibilityError>;
}
