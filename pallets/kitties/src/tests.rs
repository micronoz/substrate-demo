use frame_support::{assert_noop, assert_ok};

use crate::{mock::*, Error, Gender, Kitty};

fn last_event() -> Event {
    System::events().last().unwrap().event.clone()
}

#[test]
fn can_create() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(100)));

        let kitty = Kitty {
            dna: [
                214, 209, 234, 245, 69, 67, 6, 171, 41, 106, 181, 116, 218, 245, 185, 201,
            ],
        };
        assert_eq!(KittiesModule::kitties(100, 0), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(), 1);

        assert_eq!(
            last_event(),
            Event::pallet_kitties(crate::Event::<Test>::KittyCreated(kitty, 0, 100))
        );
    });
}

#[test]
fn gender() {
    assert_eq!(Kitty { dna: [0; 16] }.gender(), Gender::Male);
    assert_eq!(
        Kitty {
            dna: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        }
        .gender(),
        Gender::Female
    );
}

#[test]
fn can_breed() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(100)));

        System::set_extrinsic_index(3);

        assert_ok!(KittiesModule::create_kitty(Origin::signed(100)));

        assert_noop!(
            KittiesModule::breed_kitty(Origin::signed(100), 0, 11),
            Error::<Test>::KittyNotFound
        );
        assert_noop!(
            KittiesModule::breed_kitty(Origin::signed(100), 0, 0),
            Error::<Test>::KittyPartnerMissing
        );
        assert_noop!(
            KittiesModule::breed_kitty(Origin::signed(101), 0, 1),
            Error::<Test>::KittyNotFound
        );

        assert_ok!(KittiesModule::breed_kitty(Origin::signed(100), 0, 1));

        let kitty = Kitty {
            dna: [
                145, 236, 235, 229, 18, 100, 83, 204, 176, 115, 244, 197, 48, 106, 46, 45,
            ],
        };

        assert_eq!(KittiesModule::kitties(100, 2), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(), 3);

        assert_eq!(
            last_event(),
            Event::pallet_kitties(crate::Event::<Test>::KittyBred(kitty, 2, 100))
        );
    });
}

#[test]
fn test_overflow() {
    new_test_ext().execute_with(|| {
        crate::pallet::NextKittyId::<Test>::mutate(|x| *x = u32::MAX - 3);
        assert_ok!(KittiesModule::create_kitty(Origin::signed(100)));
        assert_ok!(KittiesModule::create_kitty(Origin::signed(100)));
        assert_ok!(KittiesModule::breed_kitty(
            Origin::signed(100),
            u32::MAX - 2,
            u32::MAX - 3
        ));
        assert_noop!(
            KittiesModule::create_kitty(Origin::signed(100)),
            Error::<Test>::KittyIdOverflow
        );
        assert_noop!(
            KittiesModule::create_kitty(Origin::signed(100)),
            Error::<Test>::KittyIdOverflow
        );
        assert_eq!(KittiesModule::next_kitty_id(), u32::MAX);
    });
}
