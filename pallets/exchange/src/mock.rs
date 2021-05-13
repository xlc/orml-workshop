use crate as pallet_exchange;
use frame_support::parameter_types;
use frame_support::sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, Zero},
};
use frame_system as system;
use sp_core::H256;

use orml_traits::parameter_type_with_key;

use frame_support::traits::GenesisBuild;

pub type Amount = i128;
pub type AccountId = u64;
pub type Balance = u128;
pub type CurrencyId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub const DOT: CurrencyId = 1;
//pub const KSM: CurrencyId = 2;
pub const BTC: CurrencyId = 3;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        ExchangeModule: pallet_exchange::{Module, Call, Storage, Event<T>},
        Currencies: orml_currencies::{Module, Call, Event<T>},
        Tokens: orml_tokens::{Module, Storage, Event<T>, Config<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Zero::zero()
    };
}

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = ();
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = 0;
}

impl orml_currencies::Config for Test {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency =
        orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl pallet_exchange::Config for Test {
    type Event = Event;
    type Currency = Currencies;
    type OrderId = u32;
}

// Build genesis storage according to the mock runtime.
// pub fn new_test_ext() -> sp_io::TestExternalities {
// 	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
// }

pub struct ExtBuilder {
    endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            endowed_accounts: vec![
                (ALICE, DOT, 1000_000_000_000_000u128),
                (BOB, DOT, 1000_000_000_000_000u128),
                (ALICE, BTC, 1000_000_000_000_000u128),
                (BOB, BTC, 1000_000_000_000_000u128),
            ],
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        orml_tokens::GenesisConfig::<Test> {
            endowed_accounts: self.endowed_accounts,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        t.into()
    }
}
