extern crate cosmwasm_derive;

use cosmwasm_std::{Addr, Contract};

#[test]
fn basic() {
    #[derive(Contract)]
    struct Callee {
        address: Addr,
    }

    let mut callee = Callee {
        address: Addr::unchecked("foo"),
    };
    assert_eq!(callee.get_address(), Addr::unchecked("foo"));
    callee.set_address(Addr::unchecked("bar"));
    assert_eq!(callee.get_address(), Addr::unchecked("bar"));
}

#[test]
#[allow(dead_code)]
fn specify_field() {
    #[derive(Contract)]
    struct Callee {
        address: Addr,
        #[address]
        address_actually: Addr,
    }

    let mut callee = Callee {
        address: Addr::unchecked("dummy"),
        address_actually: Addr::unchecked("foo"),
    };
    assert_eq!(callee.get_address(), Addr::unchecked("foo"));
    callee.set_address(Addr::unchecked("bar"));
    assert_eq!(callee.get_address(), Addr::unchecked("bar"));
}
