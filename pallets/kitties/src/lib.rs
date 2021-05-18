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
    use sp_std::vec::Vec;

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
        StorageMap<_, Blake2_128Concat, T::AccountId, Option<Vec<[u8; 16]>>, ValueQuery>;

    #[pallet::storage]
    pub(super) type Kitties<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 16], Option<Kitty<T>>, ValueQuery>;

    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct Kitty<T: Config> {
        owner: T::AccountId,
        dna: [u8; 16],
        gender: Gender,
    }
    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub enum Gender {
        Male,
        Female,
    }

    impl<T: Config> Kitty<T> {
        pub fn new(owner: &T::AccountId) -> Result<Kitty<T>, Error<T>> {
            // Collect sources for random hash
            let payload = (&owner, T::RandomnessSource::random(&owner.encode()[..]));

            // Generate random dna source
            let dna = payload.using_encoded(blake2_128);

            return Ok(Kitty {
                dna,
                owner: owner.clone(),
                gender: <Kitty<T>>::get_gender(dna),
            });
        }

        fn check_duplicate(dna: [u8; 16]) -> Result<(), Error<T>> {
            if <Kitties<T>>::contains_key(dna) {
                Err(<Error<T>>::DuplicateKitty)
            } else {
                Ok(())
            }
        }

        fn save_kitty(
            dna: [u8; 16],
            kitty: Kitty<T>,
            owner: &T::AccountId,
        ) -> Result<(), Error<T>> {
            // Ensure no duplicate dna
            <Kitty<T>>::check_duplicate(dna)?;

            // Save Kitty struct
            <Kitties<T>>::insert(dna, Some(kitty));

            // Assign kitty dna to an owner
            match <KittyOwners<T>>::get(owner) {
                Some(mut result) => {
                    result.push(dna);
                    <KittyOwners<T>>::insert(owner, Some(result));
                }
                None => {
                    <KittyOwners<T>>::insert(owner, Some(sp_std::vec![dna]));
                }
            }
            Ok(())
        }

        fn get_gender(dna: [u8; 16]) -> Gender {
            let total: u8 = dna.iter().sum();
            if total >= 128 {
                Gender::Male
            } else {
                Gender::Female
            }
        }

        pub fn ensure_owner(&self, owner: &T::AccountId) -> Result<(), Error<T>> {
            return match *owner == self.owner {
                true => Ok(()),
                false => Err(<Error<T>>::KittyOwnerMismatch),
            };
        }

        pub fn ensure_different_kitty(&self, other: &Kitty<T>) -> Result<(), Error<T>> {
            return match *self != *other {
                true => Ok(()),
                false => Err(<Error<T>>::KittyPartnerMissing),
            };
        }

        pub fn ensure_different_gender(&self, other: &Kitty<T>) -> Result<(), Error<T>> {
            return match self.gender != other.gender {
                true => Ok(()),
                false => Err(<Error<T>>::KittyGendersNotCompatible),
            };
        }

        pub fn breed(
            &self,
            partner: &Kitty<T>,
            owner: &T::AccountId,
        ) -> Result<Kitty<T>, Error<T>> {
            // Combine parent DNAs as seed
            let payload = (
                self.dna,
                partner.dna,
                T::RandomnessSource::random(&self.dna),
            );

            // Generate dna
            let dna = payload.using_encoded(blake2_128);

            Ok(Kitty {
                dna,
                owner: owner.clone(),
                gender: <Kitty<T>>::get_gender(dna),
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
        /// [dna, owner]
        KittyCreated([u8; 16], T::AccountId, Gender),
        /// A Kitty has been bred.
        /// [dna, owner]
        KittyBred([u8; 16], T::AccountId, Gender),
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
        /// Only owners of a kitty can breed them
        KittyOwnerMismatch,
        /// Kitty not found
        KittyNotFound,
        /// Kitties need a valid (different from self) partner to breed
        KittyPartnerMissing,
        /// Kitties must have different genders to be able to breed
        KittyGendersNotCompatible,
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
            let my_kitty = <Kitty<T>>::new(&who)?;
            let dna = my_kitty.dna;
            let gender = my_kitty.gender.clone();

            // Insert the created kitty into storage
            <Kitty<T>>::save_kitty(dna, my_kitty, &who)?;

            // Emit an event.
            Self::deposit_event(Event::KittyCreated(dna, who, gender));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
        pub fn breed_kitty(
            origin: OriginFor<T>,
            first_parent: [u8; 16],
            second_parent: [u8; 16],
        ) -> DispatchResultWithPostInfo {
            // Ensure signed origin
            let who = ensure_signed(origin)?;

            // Ensure that kitties exist
            let first_parent_struct =
                <Kitties<T>>::get(&first_parent).ok_or_else(|| <Error<T>>::KittyNotFound)?;
            let second_parent_struct =
                <Kitties<T>>::get(&second_parent).ok_or_else(|| <Error<T>>::KittyNotFound)?;

            // Ensure that kitties are owned by the origin
            first_parent_struct.ensure_owner(&who)?;
            second_parent_struct.ensure_owner(&who)?;

            // Ensure parents are not the same
            first_parent_struct.ensure_different_kitty(&second_parent_struct)?;
            // Ensure parents have opposite genders
            first_parent_struct.ensure_different_gender(&second_parent_struct)?;

            // Breed kitty
            let child_kitty = first_parent_struct.breed(&second_parent_struct, &who)?;
            let dna = child_kitty.dna;
            let gender = child_kitty.gender.clone();

            // Insert the created kitty into storage
            <Kitty<T>>::save_kitty(dna, child_kitty, &who)?;

            // Emit an event.
            Self::deposit_event(Event::KittyBred(dna, who, gender));
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
