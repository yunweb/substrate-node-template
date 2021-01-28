#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, dispatch::Vec, ensure};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as PoeModule {
		Proofs get(fn proofs): map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		ClaimCreated(AccountId, Vec<u8>),
		ClaimRevoked(AccountId, Vec<u8>),
		ClaimTransfer(AccountId, Vec<u8>, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		ClaimTooShort,
		ClaimTooLong,
		ClaimAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
		NotTransferToSelf
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		// 创建存证
		#[weight = 0]
		pub fn create_claim(origin, claim: Vec<u8>) -> dispatch::DispatchResult {
			const MIN_CLAIM_LEN:usize = 2;
			const MAX_CLAIM_LEN:usize = 10;

			let sender = ensure_signed(origin)?;

			// 存证长度是否溢出
			ensure!(claim.len() >= MIN_CLAIM_LEN, Error::<T>::ClaimTooShort);
			ensure!(claim.len() <= MAX_CLAIM_LEN, Error::<T>::ClaimTooLong);

			// 检查存证是否存在
			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ClaimAlreadyExist);

			// 存储存证
			Proofs::<T>::insert(&claim, (sender.clone(), system::Module::<T>::block_number()));

			// 发送创建事件
			Self::deposit_event(RawEvent::ClaimCreated(sender, claim));

			Ok(())
		}

		// 撤销存证
		#[weight = 0]
		pub fn revoke_claim(origin, claim: Vec<u8>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			// 检查存证是否存在
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ClaimNotExist);

			let (owner, _block_number) = Proofs::<T>::get(&claim);

			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 移除存证
			Proofs::<T>::remove(&claim);

			// 发送移除事件
			Self::deposit_event(RawEvent::ClaimRevoked(sender, claim));

			Ok(())
		}

		// 转移存证
		#[weight = 0]
		pub fn transfer_claim(origin, claim: Vec<u8>, receiver: T::AccountId) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;

			// 检查存证是否存在
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ClaimNotExist);

			let (owner, _block_number) = Proofs::<T>::get(&claim);

			// 检查自己是拥有存证的所有权
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 不能发送给自己
			ensure!(sender != receiver, Error::<T>::NotTransferToSelf);

			// 转移到新账户
			Proofs::<T>::insert(&claim, (receiver.clone(), system::Module::<T>::block_number()));

			// 发送转移事件
			Self::deposit_event(RawEvent::ClaimTransfer(sender, claim, receiver));

			Ok(())
		}
	}
}
