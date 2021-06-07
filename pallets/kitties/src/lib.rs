#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Randomness},
    };
    use frame_system::pallet_prelude::*;
    use orml_utilities::with_transaction_result;
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};
    use sp_core::H256;
    use sp_io::hashing::blake2_128;

    use orml_nft::Pallet as NftModule;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        orml_nft::Config<TokenData = Kitty, ClassData = ()> + frame_system::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type RandomnessSource: Randomness<H256>;
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {}

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            // create a NTF class
            let class_id = NftModule::<T>::create_class(&Default::default(), Vec::new(), ())
                .expect("Cannot fail or invalid chain spec");
            ClassId::<T>::put(class_id);
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type KittyIndexOf<T> = <T as orml_nft::Config>::TokenId;

    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct Listing<T: Config>(T::AccountId, BalanceOf<T>);

    #[pallet::storage]
    #[pallet::getter(fn kitty_exchange)]
    pub(super) type KittyExchange<T: Config> =
        StorageMap<_, Blake2_128Concat, KittyIndexOf<T>, Option<Listing<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn class_id)]
    pub(super) type ClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(Encode, Decode, Clone, PartialEq, Debug, Eq, Copy)]
    pub struct Kitty(pub [u8; 16]);

    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub enum Gender {
        Male,
        Female,
    }

    impl Kitty {
        fn new<T: Config>(owner: T::AccountId) -> Result<Kitty, Error<T>> {
            // Collect sources for random hash
            let payload = (
                owner.clone(),
                T::RandomnessSource::random(&owner.encode()[..]),
                frame_system::Module::<T>::extrinsic_index(),
            );

            // Generate random dna source
            let dna = payload.using_encoded(blake2_128);

            return Ok(Kitty(dna));
        }

        fn get_gender_from_dna(dna: [u8; 16]) -> Gender {
            let total = dna.iter().max();
            match total {
                Some(total) => {
                    if total % 2 == 0 {
                        Gender::Male
                    } else {
                        Gender::Female
                    }
                }
                None => Gender::Male,
            }
        }

        fn ensure_different_kitty<T: Config>(
            first: &Kitty,
            second: &Kitty,
        ) -> Result<(), Error<T>> {
            match *first != *second {
                true => Ok(()),
                false => Err(Error::<T>::KittyPartnerMissing),
            }
        }

        fn ensure_different_gender<T: Config>(
            first: &Kitty,
            second: &Kitty,
        ) -> Result<(), Error<T>> {
            match first.gender() != second.gender() {
                true => Ok(()),
                false => Err(Error::<T>::KittyGendersNotCompatible),
            }
        }

        pub fn gender(&self) -> Gender {
            Kitty::get_gender_from_dna(self.0)
        }

        fn breed<T: Config>(first: Kitty, second: Kitty) -> Result<Kitty, Error<T>> {
            // Ensure parents are not the same
            Kitty::ensure_different_kitty(&first, &second)?;
            // Ensure parents have opposite genders
            Kitty::ensure_different_gender(&first, &second)?;
            // Combine parent DNAs as seed
            let payload = (
                first.0,
                second.0,
                T::RandomnessSource::random_seed(),
                frame_system::Module::<T>::extrinsic_index(),
            );

            // Generate dna
            let dna = payload.using_encoded(blake2_128);

            Ok(Kitty(dna))
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A Kitty has been generated for the owner with random dna.
        /// [kitty, owner]
        KittyCreated(Kitty, KittyIndexOf<T>, T::AccountId),
        /// A Kitty has been bred.
        /// [kitty, owner]
        KittyBred(Kitty, KittyIndexOf<T>, T::AccountId),
        /// A Kitty has been transfered.
        /// [kitty, from, to]
        KittyTransfer(KittyIndexOf<T>, T::AccountId, T::AccountId),
        /// A Kitty has been sold.
        /// [kitty, price, seller, buyer]
        KittySold(KittyIndexOf<T>, BalanceOf<T>, T::AccountId, T::AccountId),
        /// A Kitty's price has been updated
        /// [kitty, price, owner]
        KittyPriceUpdated(KittyIndexOf<T>, Option<BalanceOf<T>>, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        /// Kitty was generated with a DNA that already exists
        DuplicateKitty,
        /// Kitty not found
        KittyNotFound,
        /// Kitties need a valid (different from self) partner to breed
        KittyPartnerMissing,
        /// Kitties must have different genders to be able to breed
        KittyGendersNotCompatible,
        /// Kitty Id has overflowed KittyIndex
        KittyIdOverflow,
        /// Kitty is not listed on the exchange
        KittyNotForSale,
        /// Cannot buy own kitty
        CannotBuyOwnKitty,
        /// Could not create kitty
        CouldNotCreateKitty,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    impl<T: Config> Pallet<T> {
        fn kitties(owner: &T::AccountId, kitty_id: KittyIndexOf<T>) -> Option<Kitty> {
            NftModule::<T>::tokens(Self::class_id(), kitty_id).and_then(|x| {
                if x.owner == *owner {
                    Some(x.data)
                } else {
                    None
                }
            })
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;
            let who_backup = who.clone();
            // Insert the created kitty into storage

            let kitty = Kitty::new::<T>(who_backup)?;
            let current_id =
                NftModule::<T>::mint(&who, Self::class_id(), Default::default(), kitty.clone())?;

            // Emit an event.
            Self::deposit_event(Event::KittyCreated(kitty, current_id, who));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,2))]
        pub fn breed_kitty(
            origin: OriginFor<T>,
            first_parent: KittyIndexOf<T>,
            second_parent: KittyIndexOf<T>,
        ) -> DispatchResultWithPostInfo {
            // Ensure signed origin
            let who = ensure_signed(origin)?;

            // Ensure that kitties exist
            let first_parent_struct =
                Self::kitties(&who, first_parent).ok_or_else(|| Error::<T>::KittyNotFound)?;
            let second_parent_struct =
                Self::kitties(&who, second_parent).ok_or_else(|| Error::<T>::KittyNotFound)?;

            // Insert the created kitty into storage
            let kitty = Kitty::breed::<T>(first_parent_struct, second_parent_struct)?;
            let current_id =
                NftModule::<T>::mint(&who, Self::class_id(), Default::default(), kitty.clone())?;

            // Emit an event.
            Self::deposit_event(Event::KittyBred(kitty, current_id, who));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        /// An example dispatchable that may throw a custom error.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn transfer_kitty(
            origin: OriginFor<T>,
            receiver: T::AccountId,
            kitty_id: KittyIndexOf<T>,
        ) -> DispatchResultWithPostInfo {
            // Ensure signed origin
            let who = ensure_signed(origin)?;

            NftModule::<T>::transfer(&who, &receiver, (Self::class_id(), kitty_id))?;

            if who != receiver {
                KittyExchange::<T>::remove(kitty_id);
                Self::deposit_event(Event::KittyTransfer(kitty_id, who, receiver));
            }
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn set_price(
            origin: OriginFor<T>,
            kitty_id: KittyIndexOf<T>,
            new_price: Option<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(
                orml_nft::TokensByOwner::<T>::contains_key(&who, (Self::class_id(), kitty_id)),
                Error::<T>::KittyNotFound
            );

            match new_price {
                Some(new_price) => KittyExchange::<T>::mutate_exists(kitty_id, |price| {
                    *price = Some(Some(Listing::<T>(who.clone(), new_price)))
                }),
                None => KittyExchange::<T>::remove(kitty_id),
            }

            Self::deposit_event(Event::KittyPriceUpdated(kitty_id, new_price, who));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn buy_kitty(
            origin: OriginFor<T>,
            kitty_id: KittyIndexOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            KittyExchange::<T>::try_mutate(kitty_id, |listing_option| {
                let Listing::<T>(owner, price) =
                    listing_option.take().ok_or(Error::<T>::KittyNotForSale)?;
                ensure!(who != owner, Error::<T>::CannotBuyOwnKitty);

                with_transaction_result(|| {
                    NftModule::<T>::transfer(&owner, &who, (Self::class_id(), kitty_id))?;
                    T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;

                    Self::deposit_event(Event::KittySold(kitty_id, price, owner, who));

                    Ok(())
                })
            })?;
            Ok(().into())
        }
    }
}
