#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::sp_runtime::{
	traits::{AtLeast32BitUnsigned, Bounded, CheckedAdd, MaybeSerializeDeserialize, One, Zero},
	RuntimeDebug,
};
use frame_support::{traits::BalanceStatus, Parameter};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use orml_utilities::with_transaction_result;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct Order<CurrencyId, Balance, AccountId> {
	pub base_currency_id: CurrencyId,
	#[codec(compact)]
	pub base_amount: Balance,
	pub target_currency_id: CurrencyId,
	#[codec(compact)]
	pub target_amount: Balance,
	pub owner: AccountId,
}

type BalanceOf<T> =
	<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
type CurrencyIdOf<T> =
	<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
type OrderOf<T> = Order<CurrencyIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::AccountId>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Currency
		type Currency: MultiReservableCurrency<Self::AccountId>;
		/// OrderId
		type OrderId: Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Bounded;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub type Orders<T: Config> = StorageMap<_, Twox64Concat, T::OrderId, OrderOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn next_orderid)]
	pub type NextOrderId<T: Config> = StorageValue<_, T::OrderId>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", T::OrderId = "OrderId", OrderOf<T> = "Order", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OrderCreated(T::OrderId, OrderOf<T>),
		OrderTaken(T::AccountId, T::OrderId, OrderOf<T>),
		OrderCancelled(T::OrderId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		OrderIdOverflow,
		InvalidOrderId,
		InsufficientBalance,
		NotOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn submit_order(
			origin: OriginFor<T>,
			base_currency_id: CurrencyIdOf<T>,
			base_amount: BalanceOf<T>,
			target_currency_id: CurrencyIdOf<T>,
			target_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			NextOrderId::<T>::try_mutate(|id| -> DispatchResultWithPostInfo {
				let order_id = id.unwrap_or_default();

				let order = Order {
					base_currency_id,
					base_amount,
					target_currency_id,
					target_amount,
					owner: who.clone(),
				};

				*id = Some(
					order_id
						.checked_add(&One::one())
						.ok_or(Error::<T>::OrderIdOverflow)?,
				);

				T::Currency::reserve(base_currency_id, &who, base_amount)?;

				Orders::<T>::insert(order_id, &order);

				Self::deposit_event(Event::OrderCreated(order_id, order));
				Ok(().into())
			})?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn take_order(
			origin: OriginFor<T>,
			order_id: T::OrderId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Orders::<T>::try_mutate_exists(order_id, |order| -> DispatchResultWithPostInfo {
				let order = order.take().ok_or(Error::<T>::InvalidOrderId)?;

				with_transaction_result(|| {
					T::Currency::transfer(
						order.target_currency_id,
						&who,
						&order.owner,
						order.target_amount,
					)?;
					let val = T::Currency::repatriate_reserved(
						order.base_currency_id,
						&order.owner,
						&who,
						order.base_amount,
						BalanceStatus::Free,
					)?;
					ensure!(val.is_zero(), Error::<T>::InsufficientBalance);

					Self::deposit_event(Event::OrderTaken(who, order_id, order));
					Ok(())
				})
				.unwrap_or(());
				Ok(().into())
			})?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_order(
			origin: OriginFor<T>,
			order_id: T::OrderId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Orders::<T>::try_mutate_exists(order_id, |order| -> DispatchResultWithPostInfo {
				let order = order.take().ok_or(Error::<T>::InvalidOrderId)?;

				ensure!(order.owner == who, Error::<T>::NotOwner);

				Self::deposit_event(Event::OrderCancelled(order_id));
				Ok(().into())
			})?;
			Ok(().into())
		}
	}
}
