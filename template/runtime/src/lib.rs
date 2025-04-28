//! The Elysium Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#![allow(clippy::new_without_default, clippy::or_fun_call)]
#![cfg_attr(feature = "runtime-benchmarks", warn(unused_crate_dependencies))]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use scale_codec::{Decode, Encode};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use sp_core::{
    crypto::{ByteArray, KeyTypeId},
    OpaqueMetadata, H160, H256, U256,
};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{
        AccountIdLookup, BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable, Get, IdentifyAccount,
        NumberFor, One, PostDispatchInfoOf, UniqueSaturatedInto, Verify, OpaqueKeys, ConstU128,
    },
    transaction_validity::{
        TransactionSource, TransactionValidity, TransactionValidityError}, MultiSignature,
    ApplyExtrinsicResult, ConsensusEngineId, Perbill, Permill, Perquintill, ExtrinsicInclusionMode, FixedPointNumber,
};
use sp_std::{marker::PhantomData, prelude::*};
use sp_version::RuntimeVersion;
#[cfg(feature = "with-rocksdb-weights")]
use frame_support::weights::constants::RocksDbWeight as RuntimeDbWeight;
use pallet_transaction_payment::{TargetedFeeAdjustment, Multiplier, FungibleAdapter};
use sp_genesis_builder::PresetId;
// use fp_evm::weight_per_gas;
use fp_rpc::TransactionStatus;
use pallet_ethereum::{
    Call::transact, PostLogContent, Transaction as EthereumTransaction,
};
use pallet_evm::{Account as EVMAccount, EnsureAddressTruncated, FeeCalculator, HashedAddressMapping, Runner, EVMCurrencyAdapter, GasWeightMapping, OnChargeEVMTransaction, AddressMapping};
use frame_system::EnsureRoot;
use smallvec::smallvec;
pub use frame_support::{
    derive_impl,
    genesis_builder_helper::{build_state, get_preset},
    parameter_types,
    traits::{ConstBool, ConstU32, ConstU8, FindAuthor, OnFinalize, OnTimestampSet},
    weights::{
        constants::{WEIGHT_REF_TIME_PER_MILLIS, WEIGHT_REF_TIME_PER_SECOND}, Weight, ConstantMultiplier,
        WeightToFeePolynomial, WeightToFeeCoefficients, WeightToFeeCoefficient,
    },
    dispatch::{DispatchClass, GetDispatchInfo, PostDispatchInfo},
};
use frame_support::traits::{Currency, Imbalance};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
mod precompiles;
mod impls;
use precompiles::FrontierPrecompiles;

// ==============================
// @@ Define Types @@
// Purpose:
// Properties:
// ==============================
pub type BlockNumber = u32;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type AccountIndex = u32;
pub type Balance = u128;
pub type Nonce = u32;
pub type Index = u32;
pub type Hash = H256;
pub type Hashing = BlakeTwo256;
pub type DigestItem = generic::DigestItem;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type SignedBlock = generic::SignedBlock<Block>;
pub type BlockId = generic::BlockId<Block>;

// ==============================
// @@ Define Constants @@
// Purpose:
// Properties:
// ==============================
pub const MILLISECOND_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECOND_PER_BLOCK;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECOND_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
pub const WEIGHT_MILLISECOND_PER_BLOCK: u64 = 2000;
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(WEIGHT_MILLISECOND_PER_BLOCK * WEIGHT_REF_TIME_PER_MILLIS, u64::MAX);
pub const MAXIMUM_BLOCK_LENGTH: u32 = 5 * 1024 * 1024;
pub const BLOCK_GAS_LIMIT: u64 = 75_000_000;
pub const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
pub const CHAIN_ID: u64 = 1569;
pub const GAS_PER_SECOND: u64 = 40_000_000;
const WEIGHT_PER_GAS: u64 = WEIGHT_REF_TIME_PER_SECOND / GAS_PER_SECOND;
// ==============================
// @@ Currency @@
// Purpose:
// Properties:
// ==============================
pub mod currency {
    use super::Balance;

    pub const SUPPLY_FACTOR: Balance = 1;
    pub const WEI: Balance = 1;
    pub const KIL_O_WEI: Balance = 1_000;
    pub const MEGA_WEI: Balance = 1_000_000;
    pub const GIGA_WEI: Balance = 1_000_000_000;
    pub const MICRO_LAVA: Balance = 1_000_000_000_000;
    pub const MILLI_LAVA: Balance = 1_000_000_000_000_000;
    pub const LAVA: Balance = 1_000_000_000_000_000_000;
    pub const KILO_LAVA: Balance = 1_000_000_000_000_000_000_000;
    pub const TRANSACTION_BYTE_FEE: Balance = 10 * MICRO_LAVA * SUPPLY_FACTOR;
    pub const STORAGE_BYTE_FEE: Balance = 100 * MICRO_LAVA * SUPPLY_FACTOR;
    pub const WEIGHT_FEE: Balance = 50 * KIL_O_WEI * SUPPLY_FACTOR;
    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 100 * MILLI_LAVA * SUPPLY_FACTOR + (bytes as Balance) * STORAGE_BYTE_FEE
    }
}

// ==============================
// @@ Runtime Version @@
// Purpose:
// Properties:
// ==============================
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("elysium"),
    impl_name: create_runtime_str!("elysium"),
    authoring_version: 1,
    spec_version: 9,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

// ==============================
// @@ Opaque @@
// Purpose:
// Properties:
// ==============================
pub mod opaque {
    use super::*;
    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    pub type BlockId = generic::BlockId<Block>;
    impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

// ==============================
// @@ Native Version of Chain @@
// Purpose:
// Properties:
// ==============================
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
    sp_version::NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

// ==============================
// @@ Pallet Frame System @@
// Purpose
// Properties
// ==============================
// Here we assume Ethereum's base fee of 21000 gas and convert to weight, but we
// subtract roughly the cost of a balance transfer from it (about 1/3 the cost)
// and some cost to account for per-byte-fee.
// TODO: we should use benchmarking's overhead feature to measure this
pub const EXTRINSIC_BASE_WEIGHT: Weight = Weight::from_parts(10000 * WEIGHT_PER_GAS, 0);
pub const NORMAL_WEIGHT: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_mul(3).saturating_div(4);
pub struct RuntimeBlockWeights;
impl Get<frame_system::limits::BlockWeights> for RuntimeBlockWeights {
    fn get() -> frame_system::limits::BlockWeights {
        frame_system::limits::BlockWeights::builder()
            .for_class(DispatchClass::Normal, |weights| {
                weights.base_extrinsic = EXTRINSIC_BASE_WEIGHT;
                weights.max_total = NORMAL_WEIGHT.into();
            })
            .for_class(DispatchClass::Operational, |weights| {
                weights.max_total = MAXIMUM_BLOCK_WEIGHT.into();
                weights.reserved = (MAXIMUM_BLOCK_WEIGHT - NORMAL_WEIGHT).into();
            })
            .avg_block_initialization(Perbill::from_percent(10))
            .build()
            .expect("Provided BlockWeight definitions are valid, qed")
    }
}
parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 256;
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(MAXIMUM_BLOCK_LENGTH, NORMAL_DISPATCH_RATIO); // allow 5 MB block
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = BlockLength;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type RuntimeTask = RuntimeTask;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = Hashing;
    type AccountId = AccountId;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Block = Block;
    type BlockHashCount = BlockHashCount;
    type DbWeight = RuntimeDbWeight;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

// ==============================
// @@ Pallet Aura @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
	pub const MaxAuthorities: u32 = 100;
}
impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type MaxAuthorities = MaxAuthorities;
    type DisabledValidators = ();
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

// ==============================
// @@ Pallet Grandpa @@
// Purpose:
// Properties:
// ==============================
impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ();
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

// ==============================
// @@ Pallet Authorship @@
// Purpose:
// Properties:
// ==============================
impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type EventHandler = ();
}

// ==============================
// @@ Pallet Timestamp @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
	pub storage EnableManualSeal: bool = false;
}
pub struct ConsensusOnTimestampSet<T>(PhantomData<T>);
impl<T: pallet_aura::Config> OnTimestampSet<T::Moment> for ConsensusOnTimestampSet<T> {
    fn on_timestamp_set(moment: T::Moment) {
        if EnableManualSeal::get() {
            return;
        }
        <pallet_aura::Pallet<T> as OnTimestampSet<T::Moment>>::on_timestamp_set(moment)
    }
}
impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = ConsensusOnTimestampSet<Self>;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// ==============================
// @@ Pallet Balance @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Self>;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type MaxFreezes = ConstU32<1>;
}

// ==============================
// @@ Pallet Transaction Payment @@
// Purpose:
// Properties:
// ==============================
pub struct LengthToFee;
impl WeightToFeePolynomial for LengthToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec![
			WeightToFeeCoefficient {
				degree: 1,
				coeff_frac: Perbill::zero(),
				coeff_integer: currency::TRANSACTION_BYTE_FEE,
				negative: false,
			},
			WeightToFeeCoefficient {
				degree: 3,
				coeff_frac: Perbill::zero(),
				coeff_integer: 1 * currency::SUPPLY_FACTOR,
				negative: false,
			},
		]
    }
}
parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
    pub const TransactionByteFee: Balance = currency::TRANSACTION_BYTE_FEE;
    /// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
    /// than this will decrease the weight and more will increase.
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(35);
    /// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
    /// change the fees more rapidly. This low value causes changes to occur slowly over time.
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(4, 1_000);
    /// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
    /// that combined with `AdjustmentVariable`, we can recover from the minimum.
    /// See `multiplier_can_grow_from_zero` in integration_tests.rs.
    /// This value is currently only used by pallet-transaction-payment as an assertion that the
    /// next multiplier is always > min value.
	pub MinimumMultiplier: Multiplier = Multiplier::from(1u128);
	/// Maximum multiplier. We pick a value that is expensive but not impossibly so; it should act
    /// as a safety net.
	pub MaximumMultiplier: Multiplier = Multiplier::from(100_000u128);
}
pub type SlowAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
    R,
    TargetBlockFullness,
    AdjustmentVariable,
    MinimumMultiplier,
    MaximumMultiplier,
>;

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, crate::impls::DealWithFees<Runtime>>;
    type WeightToFee = ConstantMultiplier<Balance, ConstU128<{ currency::WEIGHT_FEE }>>;
    type LengthToFee = LengthToFee;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Runtime>;
    type OperationalFeeMultiplier = ConstU8<5>;
}

// ==============================
// @@ Pallet Sudo @@
// Purpose:
// Properties:
// ==============================
impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Self>;
}

// ==============================
// @@ Pallet EVM Chain ID @@
// Purpose:
// Properties:
// ==============================
impl pallet_evm_chain_id::Config for Runtime {}

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
    fn find_author<'a, I>(digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item=(ConsensusEngineId, &'a [u8])>,
    {
        if let Some(author_index) = F::find_author(digests) {
            let authority_id = pallet_aura::Authorities::<Runtime>::get()[author_index as usize].clone();
            return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
        }
        None
    }
}

// ==============================
// @@ Pallet EVM @@
// Purpose:
// Properties:
// ==============================
pub struct TransactionPaymentAsGasPrice;
impl FeeCalculator for TransactionPaymentAsGasPrice {
    fn min_gas_price() -> (U256, Weight) {
        // let min_gas_price = TransactionPayment::next_fee_multiplier().saturating_mul_int(currency::WEIGHT_FEE.saturating_mul(WEIGHT_PER_GAS as u128));
        let min_gas_price = U256::from(3_000_000_000u64); // Increase from 1.25 Gwei to 3 Gwei
        (
            min_gas_price.into(),
            <Runtime as frame_system::Config>::DbWeight::get().reads(1),
        )
    }
}

// Custom OnChargeTransaction for sponsor fees
pub struct SponsorFeeAdapter<C, D>(sp_std::marker::PhantomData<(C, D)>);

impl<T, C, D> OnChargeEVMTransaction<T> for SponsorFeeAdapter<C, D>
where
    T: pallet_evm::Config,
    C: Currency<T::AccountId>,
    D: OnChargeEVMTransaction<T, LiquidityInfo=Option<C::NegativeImbalance>>,
    C::NegativeImbalance: Imbalance<C::Balance>,
    C::Balance: TryFrom<U256> + TryInto<u128>,
{
    type LiquidityInfo = Option<C::NegativeImbalance>;

    fn withdraw_fee(
        sender: &H160,
        receiver: Option<&H160>,
        fee: U256,
    ) -> Result<Self::LiquidityInfo, pallet_evm::Error<T>> {
        let sponsor_wallet = pallet_sponsor::SponsoredWallets::<Runtime>::iter()
            .find(|(_sponsor, wallets)| wallets.contains(sender) || receiver.map_or(false, |r| wallets.contains(r)))
            .map(|(sponsor, _)| sponsor);

        if let Some(sponsor_wallet) = sponsor_wallet {
            // Deduct fee from sponsor (Address C)
            let sponsor_account = T::AddressMapping::into_account_id(sponsor_wallet);
            let fee_balance = fee
                .try_into()
                .map_err(|_| { pallet_evm::Error::<T>::BalanceLow })?;

            // Attempt to withdraw fee from Wallet C
            match C::withdraw(
                &sponsor_account,
                fee_balance,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            ) {
                Ok(negative) => { Ok(Some(negative)) }
                Err(_e) => { Err(pallet_evm::Error::<T>::BalanceLow) }
            }
        } else if fee.is_zero() {
            // Zero fee: skip withdrawal
            Ok(None)
        } else {
            D::withdraw_fee(sender, None, fee)
        }
    }

    fn correct_and_deposit_fee(
        sender: &H160,
        corrected_fee: U256,
        base_fee: U256,
        already_withdrawn: Self::LiquidityInfo,
        receiver: Option<&H160>,
    ) -> Self::LiquidityInfo {
        let sponsor_wallet = pallet_sponsor::SponsoredWallets::<Runtime>::iter()
            .find(|(_sponsor, wallets)| wallets.contains(sender) || receiver.map_or(false, |r| wallets.contains(r)))
            .map(|(sponsor, _)| sponsor);

        if let Some(_sponsor_wallet) = sponsor_wallet {
            already_withdrawn
        }else{
            D::correct_and_deposit_fee(sender, corrected_fee, base_fee, already_withdrawn, receiver)
        }

    }

    fn pay_priority_fee(tip: Self::LiquidityInfo) {
        D::pay_priority_fee(tip);
    }
}

parameter_types! {
    pub BlockGasLimit: U256 = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT.ref_time() / WEIGHT_PER_GAS);
	pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::new();
	// pub WeightPerGas: Weight = Weight::from_parts(weight_per_gas(BLOCK_GAS_LIMIT, NORMAL_DISPATCH_RATIO, WEIGHT_MILLISECOND_PER_BLOCK), 0);
    pub WeightPerGas: Weight = Weight::from_parts(WEIGHT_PER_GAS, 0);
	pub SuicideQuickClearLimit: u32 = 0;
    pub const ChainId: u64 = CHAIN_ID;
    pub const GasLimitPovSizeRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(MAX_POV_SIZE);
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = TransactionPaymentAsGasPrice;
    type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
    type WeightPerGas = WeightPerGas;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressTruncated;
    type WithdrawOrigin = EnsureAddressTruncated;
    type AddressMapping = HashedAddressMapping<BlakeTwo256>;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type PrecompilesType = FrontierPrecompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = EVMChainId;
    type BlockGasLimit = BlockGasLimit;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    // type OnChargeTransaction = EVMCurrencyAdapter<Balances, crate::impls::DealWithEVMFees>;
    type OnChargeTransaction = SponsorFeeAdapter<
        Balances,
        EVMCurrencyAdapter<Balances, crate::impls::DealWithEVMFees>,
    >;
    type OnCreate = ();
    type FindAuthor = FindAuthorTruncated<Aura>;
    type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
    type SuicideQuickClearLimit = SuicideQuickClearLimit;
    type Timestamp = Timestamp;
    type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

// ==============================
// @@ Pallet Ethereum @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}
impl pallet_ethereum::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
    type PostLogContent = PostBlockAndTxnHashes;
    type ExtraDataLength = ();
}

// ==============================
// @@ Pallet Dynamic Fee @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
	pub BoundDivision: U256 = U256::from(1024);
}
impl pallet_dynamic_fee::Config for Runtime {
    type MinGasPriceBoundDivisor = BoundDivision;
}

// ==============================
// @@ Pallet Base Fee @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
    pub DefaultBaseFeePerGas: U256 = currency::GIGA_WEI.saturating_mul(currency::SUPPLY_FACTOR as u128).into();
	pub DefaultElasticity: Permill = Permill::from_parts(125_000);
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
    fn lower() -> Permill {
        Permill::zero()
    }
    fn ideal() -> Permill {
        Permill::from_parts(500_000)
    }
    fn upper() -> Permill {
        Permill::from_parts(1_000_000)
    }
}
impl pallet_base_fee::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Threshold = BaseFeeThreshold;
    type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
    type DefaultElasticity = DefaultElasticity;
}

// ==============================
// @@ Pallet HotFix Sufficient  @@
// Purpose:
// Properties:
// ==============================
impl pallet_hotfix_sufficients::Config for Runtime {
    type AddressMapping = HashedAddressMapping<BlakeTwo256>;
    type WeightInfo = pallet_hotfix_sufficients::weights::SubstrateWeight<Runtime>;
}

// ==============================
// @@ Pallet Substrate Validator Set  @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
	pub const MinAuthorities: u32 = 1;
}
impl substrate_validator_set::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddRemoveOrigin = EnsureRoot<AccountId>;
    type MinAuthorities = MinAuthorities;
    type WeightInfo = substrate_validator_set::weights::SubstrateWeight<Runtime>;
}

// ==============================
// @@ Pallet Session  @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
	pub const Period: u32 = 2 * MINUTES;
	pub const Offset: u32 = 0;
}
impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = substrate_validator_set::ValidatorOf<Self>;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = ValidatorSet;
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = opaque::SessionKeys;
    type WeightInfo = ();
}

// ==============================
// @@ Pallet Insure Randomness Collective Flip @@
// Purpose:
// Properties:
// ==============================
impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

// ==============================
// @@ Pallet Node Authorization @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
    pub const MaxWellKnownNodes: u32 = 512;
    pub const MaxPeerIdLength: u32 = 128;
}
impl pallet_node_authorization::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxWellKnownNodes = MaxWellKnownNodes;
    type MaxPeerIdLength = MaxPeerIdLength;
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}

// ==============================
// @@ Pallet MultiSig @@
// Purpose:
// Properties:
// ==============================
parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = currency::deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = currency::deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}
impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

// ==============================
// @@ Pallet Manual Seal @@
// Purpose:
// Properties:
// ==============================
#[frame_support::pallet]
pub mod pallet_manual_seal {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        pub enable: bool,
        #[serde(skip)]
        pub _config: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            EnableManualSeal::set(&self.enable);
        }
    }
}

impl pallet_manual_seal::Config for Runtime {}

impl pallet_sponsor::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxSponsoredWallets = ConstU32<100>;
}

// ==============================
// @@ Construct Runtime  @@
// Purpose:
// Properties:
// ==============================

#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeEvent,
        RuntimeCall,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask
    )]
    pub struct Runtime;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;

    #[runtime::pallet_index(1)]
    pub type Timestamp = pallet_timestamp;

    #[runtime::pallet_index(2)]
    pub type RandomnessCollectiveFlip = pallet_insecure_randomness_collective_flip;

    #[runtime::pallet_index(3)]
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(4)]
    pub type ValidatorSet = substrate_validator_set;

    #[runtime::pallet_index(5)]
    pub type Session = pallet_session;

    #[runtime::pallet_index(6)]
    pub type Aura = pallet_aura;

    #[runtime::pallet_index(7)]
    pub type Grandpa = pallet_grandpa;

    #[runtime::pallet_index(8)]
    pub type TransactionPayment = pallet_transaction_payment;

    #[runtime::pallet_index(9)]
    pub type Sudo = pallet_sudo;

    #[runtime::pallet_index(10)]
    pub type Ethereum = pallet_ethereum;

    #[runtime::pallet_index(11)]
    pub type EVM = pallet_evm;

    #[runtime::pallet_index(12)]
    pub type EVMChainId = pallet_evm_chain_id;

    #[runtime::pallet_index(13)]
    pub type DynamicFee = pallet_dynamic_fee;

    #[runtime::pallet_index(14)]
    pub type BaseFee = pallet_base_fee;

    #[runtime::pallet_index(15)]
    pub type HotfixSufficients = pallet_hotfix_sufficients;

    #[runtime::pallet_index(16)]
    pub type NodeAuthorization = pallet_node_authorization;

    #[runtime::pallet_index(17)]
    pub type Authorship = pallet_authorship;

    #[runtime::pallet_index(18)]
    pub type Multisig = pallet_multisig;

    #[runtime::pallet_index(19)]
    pub type ManualSeal = pallet_manual_seal;

    #[runtime::pallet_index(20)]
    pub type Sponsor = pallet_sponsor;
}

#[derive(Clone)]
pub struct TransactionConverter<B>(PhantomData<B>);

impl<B> Default for TransactionConverter<B> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<B: BlockT> fp_rpc::ConvertTransaction<<B as BlockT>::Extrinsic> for TransactionConverter<B> {
    fn convert_transaction(
        &self,
        transaction: pallet_ethereum::Transaction,
    ) -> <B as BlockT>::Extrinsic {
        let extrinsic = UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        );
        let encoded = extrinsic.encode();
        <B as BlockT>::Extrinsic::decode(&mut &encoded[..])
            .expect("Encoded extrinsic is always valid")
    }
}

pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
pub type UncheckedExtrinsic =
fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
pub type CheckedExtrinsic =
fp_self_contained::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra, H160>;
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

impl fp_self_contained::SelfContainedCall for RuntimeCall {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            RuntimeCall::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            RuntimeCall::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<RuntimeCall>,
        len: usize,
    ) -> Option<TransactionValidity> {
        match self {
            RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<RuntimeCall>,
        len: usize,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            RuntimeCall::Ethereum(call) => {
                call.pre_dispatch_self_contained(info, dispatch_info, len)
            }
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
        match self {
            call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
                Some(call.dispatch(RuntimeOrigin::from(
                    pallet_ethereum::RawOrigin::EthereumTransaction(info),
                )))
            }
            _ => None,
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;
#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_timestamp, Timestamp]
		[pallet_sudo, Sudo]
		[pallet_evm, EVM]
	);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			vec![]
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			_authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}

		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}

		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<Runtime as pallet_evm::Config>::ChainId::get()
		}

		fn account_basic(address: H160) -> EVMAccount {
			let (account, _) = pallet_evm::Pallet::<Runtime>::account_basic(&address);
			account
		}

		fn gas_price() -> U256 {
			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
			gas_price
		}

        fn check_sponsor_balance(sender: H160, receiver: Option<H160>) -> Option<U256> {
            // Iterate over SponsoredWallets to find a sponsor whose wallets include sender or receiver
            let sponsor_wallet = pallet_sponsor::SponsoredWallets::<Runtime>::iter()
                .find(|(_sponsor, wallets)| {
                    wallets.contains(&sender) || receiver.map_or(false, |r| wallets.contains(&r))
                })
                .map(|(sponsor, _)| sponsor);

            // If a sponsor wallet is found, return its EVM balance
            sponsor_wallet.map(|sponsor| {
                pallet_evm::Pallet::<Runtime>::account_basic(&sponsor).0.balance
            })
        }

		fn account_code_at(address: H160) -> Vec<u8> {
			pallet_evm::AccountCodes::<Runtime>::get(address)
		}

		fn author() -> H160 {
			<pallet_evm::Pallet<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];
			index.to_big_endian(&mut tmp);
			pallet_evm::AccountStorages::<Runtime>::get(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {

			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};
            let without_base_extrinsic_weight = true;
            let mut estimated_transaction_len = data.len() + 258;
            if access_list.is_some() {
                estimated_transaction_len += access_list.encoded_size();
            }

            let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();
            let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
            };

			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				false,
				true,
				weight_limit,
				proof_size_base_cost,
                config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.error.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};
            let mut estimated_transaction_len = data.len() + 190;
            if max_fee_per_gas.is_some() {
                estimated_transaction_len += 32;
            }
            if max_priority_fee_per_gas.is_some() {
                estimated_transaction_len += 32;
            }
            if access_list.is_some() {
                estimated_transaction_len += access_list.encoded_size();
            }
            let gas_limit = if gas_limit > U256::from(u64::MAX) {
                u64::MAX
            } else {
                gas_limit.low_u64()
            };

            let without_base_extrinsic_weight = true;

            let (weight_limit, proof_size_base_cost) =
                match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
                    gas_limit,
                    without_base_extrinsic_weight
                ) {
                weight_limit if weight_limit.proof_size() > 0 => {
                    (Some(weight_limit), Some(estimated_transaction_len as u64))
                }
                _ => (None, None),
            };

			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				false,
				true,
				weight_limit,
				proof_size_base_cost,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.error.into())
		}

		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			pallet_ethereum::CurrentBlock::<Runtime>::get()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			pallet_ethereum::CurrentReceipts::<Runtime>::get()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentReceipts::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn extrinsic_filter(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> Vec<EthereumTransaction> {
			xts.into_iter().filter_map(|xt| match xt.0.function {
				RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
				_ => None
			}).collect::<Vec<EthereumTransaction>>()
		}

		fn elasticity() -> Option<Permill> {
			Some(pallet_base_fee::Elasticity::<Runtime>::get())
		}

		fn gas_limit_multiplier_support() {}

		fn pending_block(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> (Option<pallet_ethereum::Block>, Option<Vec<TransactionStatus>>) {
			for ext in xts.into_iter() {
				let _ = Executive::apply_extrinsic(ext);
			}

			Ethereum::on_finalize(System::block_number() + 1);

			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

        fn initialize_pending_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header);
		}
	}

	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
			)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			use baseline::Pallet as BaselineBench;
			use frame_system_benchmarking::Pallet as SystemBench;
			use pallet_hotfix_sufficients::Pallet as PalletHotfixSufficients;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);
			list_benchmark!(list, extra, pallet_hotfix_sufficients, PalletHotfixSufficients::<Runtime>);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, add_benchmark};
			use frame_support::traits::TrackedStorageKey;

			use pallet_evm::Pallet as PalletEvmBench;
			use pallet_hotfix_sufficients::Pallet as PalletHotfixSufficientsBench;

			impl baseline::Config for Runtime {}
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, pallet_evm, PalletEvmBench::<Runtime>);
			add_benchmark!(params, batches, pallet_hotfix_sufficients, PalletHotfixSufficientsBench::<Runtime>);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

#[cfg(test)]
mod tests {
    use super::{Runtime, WeightPerGas};
    #[test]
    fn configured_base_extrinsic_weight_is_evm_compatible() {
        let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
        let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
            .get(frame_support::dispatch::DispatchClass::Normal)
            .base_extrinsic;
        assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
    }
}