// Re-export pallet items so that they can be accessed from the crate namespace.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::{*, ValueQuery};
    use frame_support::traits::{Currency, Randomness};
    use frame_system::pallet_prelude::*;
    // use sp_std::vec::Vec;

    use sp_runtime::{
        traits::{CheckedAdd, Saturating, Zero},
        DispatchResult
    };

    type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
        type RandomnessSource: Randomness<Self::Hash, Self::BlockNumber>;


        #[pallet::constant]
        type MaximumSupply: Get<BalanceOf<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config>
    {
        UserCoinsIssued(T::AccountId, BalanceOf<T>),
        SignerRemoved(T::AccountId),
        SignerAdded(T::AccountId),

        WalletAdded(T::AccountId),
        WalletRemoved(T::AccountId),
    }

    #[pallet::error]   // <-- Step 4. code block will replace this.
    pub enum Error<T> {
        WalletNotMatched,

        AlreadySigner,

        NotSigner,

        LowBalanceToBurn,

        TooManyCoinsToAllocate,

        // for WalletFoundation
        AlreadyWalletAdded,
        WalletAdded,
        WalletRemoved
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);


    #[pallet::storage]
    #[pallet::getter(fn balance)]
    pub(super) type CoinsAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn main_account)]
    pub(super) type MainAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn signers)]
    //pub(super) type Signers<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;
    pub(super) type Signers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

    #[pallet::storage]
    #[pallet::getter(fn foundation_wallet)]
    pub(super) type FoundationWallet<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        // pub balance: u128,
        pub balance: BalanceOf<T>,
        pub main_account: Vec<(T::AccountId, BalanceOf<T>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                balance: Default::default(),
                main_account: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <CoinsAmount<T>>::put(&self.balance);
            for (a, b) in &self.main_account {
                log::info!("New signer with ID: {:?}.", a);
                <MainAccount<T>>::insert(a, b);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_signer(origin: OriginFor<T>, signer: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(!Signers::<T>::contains_key(&signer),Error::<T>::AlreadySigner);
            Signers::<T>::insert(&signer, true);
            log::info!("insert successful");
            Self::deposit_event(Event::SignerAdded(signer));
            Ok(())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_signer(origin: OriginFor<T>, signer: T::AccountId) -> DispatchResult {
            log::info!("removing signer...");
            ensure_root(origin)?;
            ensure!(Signers::<T>::contains_key(&signer), Error::<T>::NotSigner);
            Signers::<T>::remove(&signer);
            Self::deposit_event(Event::SignerRemoved(signer));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_wallet(origin: OriginFor<T>, signer: T::AccountId) -> DispatchResult {
            ensure_root(origin)?;          
            FoundationWallet::<T>::set(core::prelude::v1::Some(signer));
            log::info!("insert add_wallet successful");
            // Self::deposit_event(Event::WalletAdded(signer));
            Ok(())
        }
    }
}