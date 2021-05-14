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
    pub(super) type KittyOwners<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, sp_std::vec::Vec<[u8; 16]>, ValueQuery>;

    #[pallet::storage]
    pub(super) type Kitties<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 16], Kitty<T>, ValueQuery>;

    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct Kitty<T: Config> {
        owner: T::AccountId,
        dna: [u8; 16],
    }

    impl<T: Config> Default for Kitty<T> {
        fn default() -> Kitty<T> {
            return Kitty {
                owner: T::AccountId::default(),
                dna: Default::default(),
            };
        }
    }

    impl<T: Config> Kitty<T> {
        pub fn new(owner: T::AccountId) -> Result<Kitty<T>, Error<T>> {
            // Collect sources for random hash
            let payload = (
                owner.clone(),
                T::RandomnessSource::random(&owner.encode()[..]),
            );

            // Generate random dna source
            let dna = payload.using_encoded(sp_io::hashing::blake2_128);

            // Ensure that dna is unique
            if <Kitties<T>>::contains_key(dna) {
                return Err(<Error<T>>::DuplicateKitty);
            } else {
                return Ok(Kitty { dna, owner });
            }
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A Kitty has been generated for the owner with random dna.
        /// [dna, owner]
        KittyCreated([u8; 16], T::AccountId),
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
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;
            let my_kitty = <Kitty<T>>::new(who.clone())?;
            let dna = my_kitty.dna;

            // Insert the created kitty into storage
            <Kitties<T>>::insert(dna, &my_kitty);

            match <KittyOwners<T>>::try_get(&who) {
                Ok(mut result) => {
                    result.push(dna);
                    <KittyOwners<T>>::insert(&who, &result);
                }
                Err(_) => {
                    <KittyOwners<T>>::insert(&who, sp_std::vec![dna]);
                }
            }

            // Emit an event.
            Self::deposit_event(Event::KittyCreated(dna, who));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        /// An example dispatchable that may throw a custom error.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;

            // // Read a value from storage.
            // match <Something<T>>::get() {
            //     // Return an error if the value has not been set.
            //     None => Err(Error::<T>::NoneValue)?,
            //     Some(old) => {
            //         // Increment the value read from storage; will error in the event of overflow.
            //         let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
            //         // Update the value in storage with the incremented result.
            //         <Something<T>>::put(new);
            //         Ok(().into())
            //     }
            // }
            Ok(().into())
        }
    }
}
