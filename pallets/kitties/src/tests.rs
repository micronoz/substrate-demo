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
                229, 32, 78, 248, 81, 2, 246, 121, 208, 232, 58, 118, 70, 78, 137, 103,
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

        System::set_extrinsic_index(1);

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
                228, 103, 81, 96, 30, 52, 60, 84, 78, 8, 205, 247, 24, 133, 211, 217,
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
    new_test_ext().execute_with(|| {});
}
