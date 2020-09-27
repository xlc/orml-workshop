#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure,
	Parameter, traits::BalanceStatus,
};
use frame_system::ensure_signed;
use sp_runtime::{
	DispatchResult, RuntimeDebug,
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Bounded, One, CheckedAdd, Zero},
};
use orml_traits::{MultiReservableCurrency, MultiCurrency};
use orml_utilities::with_transaction_result;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: MultiReservableCurrency<Self::AccountId>;
	type OrderId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
}

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

type BalanceOf<T> = <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
type CurrencyIdOf<T> = <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::CurrencyId;
type OrderOf<T> = Order<CurrencyIdOf<T>, BalanceOf<T>, <T as frame_system::Trait>::AccountId>;

decl_storage! {
	trait Store for Module<T: Trait> as Exchange {
		pub Orders: map hasher(twox_64_concat) T::OrderId => Option<OrderOf<T>>;
		pub NextOrderId: T::OrderId;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as Trait>::OrderId,
		Order = OrderOf<T>,
		<T as frame_system::Trait>::AccountId,
	{
		OrderCreated(OrderId, Order),
		OrderTaken(AccountId, OrderId, Order),
		OrderCancelled(OrderId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		OrderIdOverflow,
		InvalidOrderId,
		InsufficientBalance,
		NotOwner,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 1000]
		fn submit_order(
			origin,
			base_currency_id: CurrencyIdOf<T>,
			base_amount: BalanceOf<T>,
			target_currency_id: CurrencyIdOf<T>,
			target_amount: BalanceOf<T>,
		) {
			let who = ensure_signed(origin)?;
			NextOrderId::<T>::try_mutate(|id| -> DispatchResult {
				let order_id = *id;

				let order = Order {
					base_currency_id,
					base_amount,
					target_currency_id,
					target_amount,
					owner: who.clone(),
				};

				*id = id.checked_add(&One::one()).ok_or(Error::<T>::OrderIdOverflow)?;
				
				T::Currency::reserve(base_currency_id, &who, base_amount)?;

				Orders::<T>::insert(order_id, &order);

				Self::deposit_event(RawEvent::OrderCreated(order_id, order));
				Ok(())
			})?;
		}

		#[weight = 1000]
		fn take_order(origin, order_id: T::OrderId) {
			let who = ensure_signed(origin)?;

			Orders::<T>::try_mutate_exists(order_id, |order| -> DispatchResult {
				let order = order.take().ok_or(Error::<T>::InvalidOrderId)?;

				with_transaction_result(|| {
					T::Currency::transfer(order.target_currency_id, &who, &order.owner, order.target_amount)?;
					let val = T::Currency::repatriate_reserved(order.base_currency_id, &order.owner, &who, order.base_amount, BalanceStatus::Free)?;
					ensure!(val.is_zero(), Error::<T>::InsufficientBalance);

					Self::deposit_event(RawEvent::OrderTaken(who, order_id, order));

					Ok(())
				})
			})?;
		}

		#[weight = 1000]
		fn cancel_order(origin, order_id: T::OrderId) {
			let who = ensure_signed(origin)?;

			Orders::<T>::try_mutate_exists(order_id, |order| -> DispatchResult {
				let order = order.take().ok_or(Error::<T>::InvalidOrderId)?;

				ensure!(order.owner == who, Error::<T>::NotOwner);

				Self::deposit_event(RawEvent::OrderCancelled(order_id));

				Ok(())
			})?;
		}
	}
}

impl<T: Trait> Module<T> {}
