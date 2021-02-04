#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
	decl_error,
	ensure,
	StorageValue,
	StorageMap,
	Parameter,
	dispatch,
	debug
};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::DispatchError;
use sp_std::result::Result;
use sp_runtime::traits::{AtLeast32Bit, Bounded, Member};
use frame_support::traits::{Get, Currency, ReservableCurrency, Randomness};
use sp_std::vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Randomness: Randomness<Self::Hash>;

	type KittyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	type NewKittyReserve: Get<BalanceOf<Self>>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {

		debug::info!("map: {:?}", map);

		pub Kitties get(fn kitties): map hasher(blake2_128_concat) <T as Trait>::KittyIndex => Option<Kitty>;
		pub KittiesCount get(fn kitties_count): <T as Trait>::KittyIndex;
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) <T as Trait>::KittyIndex => Option<T::AccountId>;

		// map: AccountId -> [KittyIndex1, KittyIndex2, ...]
		pub KittyTotal get(fn kitty_total): map hasher(blake2_128_concat) T::AccountId => vec::Vec<T::KittyIndex>;

		// map: KittyIndex -> (Parent1, Parent2)
		pub KittiesParents get(fn kitty_parents): map hasher(blake2_128_concat) T::KittyIndex => (T::KittyIndex, T::KittyIndex);

		// map: KittyIndex -> [Children1, Children2, ...]
		pub KittiesChildren get(fn kitty_children): double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;

		// map: KittyIndex -> [Sibling1, Sibling2, ...]
		pub KittiesSibling get(fn kitty_sibling): map hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;

		// map: KittyIndex -> [Partner1, Partner2, ...]
		pub KittiesPartner get(fn kitty_partner): map hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;
	}
}

decl_event!(
	pub enum Event<T> where <T as frame_system::Trait>::AccountId,  <T as Trait>::KittyIndex {
		Created(AccountId, KittyIndex),
		Transferred(AccountId, AccountId, KittyIndex),
		Breed(AccountId, KittyIndex),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverflow,
		RequireDifferentParent,
		InvalidKittyId,
		MoneyNotEnough,
		KittyNotExists,
		NotKittyOwner,
		TransferToSelf,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		// 创建kitty
		#[weight = 0]
		pub fn create(origin) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?;

			let dna = Self::random_value(&sender);

			let kitty = Kitty(dna);

			T::Currency::reserve(&sender, T::NewKittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough)?;

			Self::insert_kitty(&sender, kitty_id, kitty);

			Self::deposit_event(RawEvent::Created(sender, kitty_id));

			Ok(())
		}

		// 转移kitty
		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			let owner = Self::kitty_owner(&kitty_id).ok_or(Error::<T>::KittyNotExists)?;

			// 判断是否是该kitty的owner
			ensure!(owner == sender, Error::<T>::NotKittyOwner);

			// 不能转移给自己
			ensure!(sender != to, Error::<T>::TransferToSelf);

			<KittyOwners<T>>::insert(kitty_id, to.clone());

			// 删除原来map里的值
			KittyTotal::<T>::mutate(&sender, |val| val.retain(|&temp| temp == kitty_id));
			KittyTotal::<T>::mutate(&to, |val| val.push(kitty_id));

			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));

			Ok(())
		}

		// 繁殖
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;

			Self::deposit_event(RawEvent::Breed(sender, new_kitty_id));

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	// 获取下一个kitty的id
	fn next_kitty_id() -> Result<T::KittyIndex, DispatchError> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
	}

	// 随机得到kitty
	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}

	// 存储kitty
	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
		Kitties::<T>::insert(kitty_id, kitty);
		KittiesCount::<T>::put(kitty_id + (1 as u32).into());
		KittyOwners::<T>::insert(kitty_id, owner);
	}

	// 繁殖kitty
	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> Result<T::KittyIndex, DispatchError> {
		let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
		let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

		ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

		let kitty_id = Self::next_kitty_id()?;
		let kitty_1_dna = kitty1.0;
		let kitty_2_dna = kitty2.0;

		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		for i in 0..kitty_1_dna.len() {
			new_dna[i] = combine_dna(kitty_1_dna[i], kitty_2_dna[i], selector[i]);
		}

		Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

		Ok(kitty_id)
	}
}

// 随机DNA
fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	(selector & dna1) | (!selector & dna2)
}

