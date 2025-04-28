#![cfg_attr(not(feature = "std"), no_std)]

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{BuildGenesisConfig, Get},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use sp_core::H160;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Maximum number of sponsored wallets per sponsor wallet.
        #[pallet::constant]
        type MaxSponsoredWallets: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Storage map that associates a sponsor wallet (H160) with a list of sponsored wallet addresses (BoundedVec<H160>).
    #[pallet::storage]
    #[pallet::getter(fn sponsored_wallets)]
    pub type SponsoredWallets<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H160, // Sponsor wallet address
        BoundedVec<H160, T::MaxSponsoredWallets>, // List of sponsored wallet addresses
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A sponsored wallet was added. [sponsor_wallet, sponsored_wallet]
        SponsoredWalletAdded(H160, H160),
        /// A sponsored wallet was removed. [sponsor_wallet, sponsored_wallet]
        SponsoredWalletRemoved(H160, H160),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The sponsored wallet is already associated with the sponsor.
        AlreadySponsored,
        /// The sponsored wallet is not associated with the sponsor.
        NotSponsored,
        /// The maximum number of sponsored wallets has been reached.
        TooManySponsoredWallets,
        /// The sponsor wallet and sponsored wallet cannot be the same.
        SameWallet,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a sponsored wallet address under a sponsor wallet.
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn add_sponsored_wallet(
            origin: OriginFor<T>,
            sponsor_wallet: H160,
            sponsored_wallet: H160,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(
                sponsor_wallet != sponsored_wallet,
                Error::<T>::SameWallet
            );
            let mut wallets = <SponsoredWallets<T>>::get(&sponsor_wallet);
            ensure!(
                !wallets.contains(&sponsored_wallet),
                Error::<T>::AlreadySponsored
            );
            wallets
                .try_push(sponsored_wallet)
                .map_err(|_| Error::<T>::TooManySponsoredWallets)?;
            <SponsoredWallets<T>>::insert(&sponsor_wallet, wallets);
            Self::deposit_event(Event::SponsoredWalletAdded(sponsor_wallet, sponsored_wallet));
            Ok(())
        }

        /// Remove a sponsored wallet address from a sponsor wallet.
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn remove_sponsored_wallet(
            origin: OriginFor<T>,
            sponsor_wallet: H160,
            sponsored_wallet: H160,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let mut wallets = <SponsoredWallets<T>>::get(&sponsor_wallet);
            let index = wallets
                .iter()
                .position(|w| w == &sponsored_wallet)
                .ok_or(Error::<T>::NotSponsored)?;
            wallets.remove(index);
            <SponsoredWallets<T>>::insert(&sponsor_wallet, wallets);
            Self::deposit_event(Event::SponsoredWalletRemoved(sponsor_wallet, sponsored_wallet));
            Ok(())
        }
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub sponsored_wallets: Vec<(H160, Vec<H160>)>,
        #[serde(skip)]
        pub _phantom: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (sponsor, wallets) in &self.sponsored_wallets {
                if let Ok(bounded_wallets) = BoundedVec::try_from(wallets.clone()) {
                    <SponsoredWallets<T>>::insert(sponsor, bounded_wallets);
                }
            }
        }
    }
}