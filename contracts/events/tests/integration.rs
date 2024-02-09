use cosmwasm_std::{to_json_vec, Attribute, Event, Response};
use cosmwasm_vm::testing::{
    execute, instantiate, mock_env, mock_info, Contract, MockApi, MockInstanceOptions, MockQuerier,
    MockStorage,
};
use cosmwasm_vm::Instance;
use events::msg::{ExecuteMsg, InstantiateMsg};

static CONTRACT: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/events.wasm");

fn instantiate_instance() -> Instance<MockApi, MockStorage, MockQuerier> {
    let options = MockInstanceOptions::default();
    let env = to_json_vec(&mock_env()).unwrap();
    let api = MockApi::default();
    let querier = MockQuerier::new(&[]);
    let contract = Contract::from_code(CONTRACT, &env, &options, None).unwrap();
    let mut instance = contract.generate_instance(api, querier, &options).unwrap();

    let _res: Response = instantiate(
        &mut instance,
        mock_env(),
        mock_info("creator", &[]),
        InstantiateMsg {},
    )
    .unwrap();

    instance
}

#[test]
fn event_works() {
    let mut instance = instantiate_instance();
    let ty = String::from("type");
    let attributes = vec![Attribute::new("foo", "Alice"), Attribute::new("bar", "Bob")];
    let msg = ExecuteMsg::Event {
        ty: ty.clone(),
        attributes: attributes.clone(),
    };
    let _res: Response =
        execute(&mut instance, mock_env(), mock_info("executor", &[]), msg).unwrap();

    let (events, attrs) = instance.get_events_attributes();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].ty, ty);
    assert_eq!(events[0].attributes, attributes);
    assert_eq!(attrs.len(), 0);
}

#[test]
fn events_works() {
    let mut instance = instantiate_instance();
    let event1 = Event::new("e1")
        .add_attribute("foo", "Alice")
        .add_attribute("bar", "Bob");
    let event2 = Event::new("e2")
        .add_attribute("foobar", "Bob")
        .add_attribute("barfoo", "Alice");
    let msg = ExecuteMsg::Events {
        events: vec![event1.clone(), event2.clone()],
    };
    let _res: Response =
        execute(&mut instance, mock_env(), mock_info("executor", &[]), msg).unwrap();

    let (events, attrs) = instance.get_events_attributes();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0], event1);
    assert_eq!(events[1], event2);
    assert_eq!(attrs.len(), 0);
}

#[test]
fn attribute_works() {
    let mut instance = instantiate_instance();
    let key = String::from("foo");
    let value = String::from("Alice");
    let msg = ExecuteMsg::Attribute {
        key: key.clone(),
        value: value.clone(),
    };
    let _res: Response =
        execute(&mut instance, mock_env(), mock_info("executor", &[]), msg).unwrap();

    let (events, attrs) = instance.get_events_attributes();

    assert_eq!(events.len(), 0);
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].key, key);
    assert_eq!(attrs[0].value, value);
}

#[test]
fn attributes_works() {
    let mut instance = instantiate_instance();
    let attr1 = Attribute::new("foo", "Alice");
    let attr2 = Attribute::new("bar", "Bob");
    let msg = ExecuteMsg::Attributes {
        attributes: vec![attr1.clone(), attr2.clone()],
    };
    let _res: Response =
        execute(&mut instance, mock_env(), mock_info("executor", &[]), msg).unwrap();

    let (events, attrs) = instance.get_events_attributes();

    assert_eq!(events.len(), 0);
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0], attr1);
    assert_eq!(attrs[1], attr2);
}
