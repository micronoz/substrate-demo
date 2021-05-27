#![cfg_attr(not(feature = "std"), no_std)]

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

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Randomness,
    };
    use frame_system::pallet_prelude::*;
    use sp_core::H256;
    use sp_io::hashing::blake2_128;
    use sp_std::boxed::Box;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type RandomnessSource: Randomness<H256>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    #[pallet::storage]
    // Learn more about declaring storage items:
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
    pub(super) type Kitties<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        Option<Kitty>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub(super) type NextKittyId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct Kitty {
        dna: [u8; 16],
        gender: Gender,
    }
    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub enum Gender {
        Male,
        Female,
    }

    impl Kitty {
        fn new<T: Config>(owner: T::AccountId, index: u32) -> Result<Kitty, Error<T>> {
            // Collect sources for random hash
            let payload = (
                owner.clone(),
                T::RandomnessSource::random(&owner.encode()[..]),
                index,
            );

            // Generate random dna source
            let dna = payload.using_encoded(blake2_128);

            return Ok(Kitty {
                dna,
                gender: Kitty::get_gender_from_dna(dna),
            });
        }

        fn get_gender_from_dna(dna: [u8; 16]) -> Gender {
            let total: u8 = dna.iter().sum();
            if total >= 128 {
                Gender::Male
            } else {
                Gender::Female
            }
        }

        fn save_kitty<T: Config>(
            owner: &T::AccountId,
            generator: Box<dyn FnOnce(u32) -> Result<Kitty, Error<T>>>,
        ) -> Result<Kitty, Error<T>> {
            NextKittyId::<T>::try_mutate(|id| -> Result<Kitty, Error<T>> {
                let current_id = *id;
                *id = id.checked_add(1).ok_or(Error::<T>::KittyIdOverflow)?;
                let kitty = generator(current_id)?;
                Kitties::<T>::insert(owner, current_id, Some(kitty.clone()));
                Ok(kitty)
            })
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
            match first.gender != second.gender {
                true => Ok(()),
                false => Err(Error::<T>::KittyGendersNotCompatible),
            }
        }

        fn breed<T: Config>(first: Kitty, second: Kitty, index: u32) -> Result<Kitty, Error<T>> {
            // Ensure parents are not the same
            Kitty::ensure_different_kitty(&first, &second)?;
            // Ensure parents have opposite genders
            Kitty::ensure_different_gender(&first, &second)?;
            // Combine parent DNAs as seed
            let payload = (
                first.dna,
                second.dna,
                T::RandomnessSource::random(&index.encode()[..]),
                index,
            );

            // Generate dna
            let dna = payload.using_encoded(blake2_128);

            Ok(Kitty {
                dna,
                gender: Kitty::get_gender_from_dna(dna),
            })
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
        KittyCreated(Kitty, T::AccountId),
        /// A Kitty has been bred.
        /// [kitty, owner]
        KittyBred(Kitty, T::AccountId),
        /// A Kitty has been transfered.
        /// [kitty, from, to]
        KittyTransfer(Kitty, T::AccountId, T::AccountId),
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
        /// Only owners of a kitty can interact with them
        KittyOwnerMismatch,
        /// Kitty not found
        KittyNotFound,
        /// Kitties need a valid (different from self) partner to breed
        KittyPartnerMissing,
        /// Kitties must have different genders to be able to breed
        KittyGendersNotCompatible,
        /// Kitty Id has overflowed u32
        KittyIdOverflow,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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
            let my_kitty =
                Kitty::save_kitty::<T>(&who, Box::new(|index| Kitty::new::<T>(who_backup, index)))?;

            // Emit an event.
            Self::deposit_event(Event::KittyCreated(my_kitty, who));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,2))]
        pub fn breed_kitty(
            origin: OriginFor<T>,
            first_parent: u32,
            second_parent: u32,
        ) -> DispatchResultWithPostInfo {
            // Ensure signed origin
            let who = ensure_signed(origin)?;

            // Ensure that kitties exist
            let first_parent_struct =
                Kitties::<T>::get(&who, first_parent).ok_or_else(|| Error::<T>::KittyNotFound)?;
            let second_parent_struct =
                Kitties::<T>::get(&who, second_parent).ok_or_else(|| Error::<T>::KittyNotFound)?;

            // Insert the created kitty into storage
            let child_kitty = Kitty::save_kitty::<T>(
                &who,
                Box::new(|index| {
                    Kitty::breed::<T>(first_parent_struct, second_parent_struct, index)
                }),
            )?;

            // Emit an event.
            Self::deposit_event(Event::KittyBred(child_kitty, who));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        /// An example dispatchable that may throw a custom error.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn transfer_kitty(
            origin: OriginFor<T>,
            receiver: T::AccountId,
            kitty_id: u32,
        ) -> DispatchResultWithPostInfo {
            // Ensure signed origin
            let who = ensure_signed(origin)?;

            // Transfer ownership
            let kitty =
                Kitties::<T>::take(&who, kitty_id).ok_or_else(|| Error::<T>::KittyNotFound)?;
            Kitties::<T>::insert(&receiver, kitty_id, Some(kitty.clone()));

            // Emit event
            Self::deposit_event(Event::KittyTransfer(kitty, who, receiver));

            Ok(().into())
        }
    }
}
