extern crate cosmwasm_event_derive;

use cosmwasm_event_derive::IntoEvent;
use cosmwasm_std::{attr, coins, Addr, Coin, Event};

fn coins_to_string(coins: Vec<Coin>) -> String {
    format!(
        "[{}]",
        coins
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    )
}

#[test]
fn basic() {
    #[derive(IntoEvent)]
    struct TransferEvent {
        from: Addr,
        receiver: Addr,
        #[to_string_fn(coins_to_string)]
        amount: Vec<Coin>,
    }

    let transfer = TransferEvent {
        from: Addr::unchecked("alice"),
        receiver: Addr::unchecked("bob"),
        amount: coins(42, "link"),
    };
    let expected = Event::new("transfer_event").add_attributes(vec![
        attr("from", "alice"),
        attr("receiver", "bob"),
        attr("amount", coins_to_string(coins(42, "link"))),
    ]);
    let transfer_event: Event = transfer.into();
    assert_eq!(transfer_event, expected);
}

#[test]
fn with_non_related_attribute() {
    #[derive(IntoEvent)]
    struct TransferEvent {
        #[rustfmt::skip]
        from: Addr,
        #[rustfmt::skip]
        receiver: Addr,
        #[rustfmt::skip]
        #[to_string_fn(coins_to_string)]
        amount: Vec<Coin>,
    }

    let transfer = TransferEvent {
        from: Addr::unchecked("alice"),
        receiver: Addr::unchecked("bob"),
        amount: coins(42, "link"),
    };
    let expected = Event::new("transfer_event").add_attributes(vec![
        attr("from", "alice"),
        attr("receiver", "bob"),
        attr("amount", coins_to_string(coins(42, "link"))),
    ]);
    let transfer_event: Event = transfer.into();
    assert_eq!(transfer_event, expected);
}

#[test]
fn no_fields() {
    #[derive(IntoEvent)]
    struct A {}

    let a = A {};
    let expected = Event::new("a");

    let a_event: Event = a.into();
    assert_eq!(a_event, expected);
}
