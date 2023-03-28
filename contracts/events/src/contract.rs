use cosmwasm_std::{
    callable_point, dynamic_link, entry_point, Addr, Attribute, Contract, DepsMut, Env, Event,
    MessageInfo, Response,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

#[derive(Contract)]
struct EventsContract {
    address: Addr,
}

#[dynamic_link(EventsContract)]
trait Events: Contract {
    fn add_event_dyn(&self, ty: String, attributes: Vec<Attribute>);
    fn add_events_dyn(&self, events: Vec<Event>);
    fn add_attribute_dyn(&self, key: String, value: String);
    fn add_attributes_dyn(&self, attributes: Vec<Attribute>);
}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Event { ty, attributes } => handle_event(deps, ty, attributes),
        ExecuteMsg::Events { events } => handle_events(deps, events),
        ExecuteMsg::Attribute { key, value } => handle_attribute(deps, key, value),
        ExecuteMsg::Attributes { attributes } => handle_attributes(deps, attributes),
        ExecuteMsg::EventDyn {
            address,
            ty,
            attributes,
        } => handle_event_dyn(deps, address, ty, attributes),
        ExecuteMsg::EventsDyn { address, events } => handle_events_dyn(deps, address, events),
        ExecuteMsg::AttributeDyn {
            address,
            key,
            value,
        } => handle_attribute_dyn(deps, address, key, value),
        ExecuteMsg::AttributesDyn {
            address,
            attributes,
        } => handle_attributes_dyn(deps, address, attributes),
    }
}

fn handle_event(
    deps: DepsMut,
    ty: String,
    attributes: Vec<Attribute>,
) -> Result<Response, ContractError> {
    let event = Event::new(ty).add_attributes(attributes);
    deps.api.add_event(&event)?;
    Ok(Response::default())
}

fn handle_events(deps: DepsMut, events: Vec<Event>) -> Result<Response, ContractError> {
    deps.api.add_events(&events)?;
    Ok(Response::default())
}

fn handle_attribute(deps: DepsMut, key: String, value: String) -> Result<Response, ContractError> {
    deps.api.add_attribute(&key, &value)?;
    Ok(Response::default())
}

fn handle_attributes(deps: DepsMut, attributes: Vec<Attribute>) -> Result<Response, ContractError> {
    deps.api.add_attributes(&attributes)?;
    Ok(Response::default())
}

/// This issues the given event three times in different ways
fn handle_event_dyn(
    deps: DepsMut,
    address: Addr,
    ty: String,
    attributes: Vec<Attribute>,
) -> Result<Response, ContractError> {
    let contract = EventsContract { address };

    // issue event via dynamic link
    contract.add_event_dyn(ty.clone(), attributes.clone());

    let event = Event::new(ty).add_attributes(attributes);

    // issue event via api
    deps.api.add_event(&event)?;

    // issue event via response
    Ok(Response::default().add_event(event))
}

/// This issues given events three times in different ways
fn handle_events_dyn(
    deps: DepsMut,
    address: Addr,
    events: Vec<Event>,
) -> Result<Response, ContractError> {
    let contract = EventsContract { address };

    // issue events via dynamic link
    contract.add_events_dyn(events.clone());

    // issue events via api
    deps.api.add_events(&events)?;

    // issue events via response
    Ok(Response::default().add_events(events))
}

/// This issues the given attribute three times in different ways
fn handle_attribute_dyn(
    deps: DepsMut,
    address: Addr,
    key: String,
    value: String,
) -> Result<Response, ContractError> {
    let contract = EventsContract { address };

    // issue attribute via dynamic link
    contract.add_attribute_dyn(key.clone(), value.clone());

    // issue attribute via api
    deps.api.add_attribute(&key, &value)?;

    // issue attribute via response
    Ok(Response::default().add_attribute(key, value))
}

/// This issues the given attributes three times in different ways
fn handle_attributes_dyn(
    deps: DepsMut,
    address: Addr,
    attributes: Vec<Attribute>,
) -> Result<Response, ContractError> {
    let contract = EventsContract { address };

    // issue attributes via dynamic link
    contract.add_attributes_dyn(attributes.clone());

    // issue attributes via api
    deps.api.add_attributes(&attributes)?;

    // issue attributes via response
    Ok(Response::default().add_attributes(attributes))
}

#[callable_point]
fn add_event_dyn(deps: DepsMut, _env: Env, ty: String, attributes: Vec<Attribute>) {
    let event = Event::new(ty).add_attributes(attributes);
    deps.api.add_event(&event).unwrap();
}

#[callable_point]
fn add_events_dyn(deps: DepsMut, _env: Env, events: Vec<Event>) {
    deps.api.add_events(&events).unwrap();
}

#[callable_point]
fn add_attribute_dyn(deps: DepsMut, _env: Env, key: String, value: String) {
    deps.api.add_attribute(&key, &value).unwrap();
}

#[callable_point]
fn add_attributes_dyn(deps: DepsMut, _env: Env, attributes: Vec<Attribute>) {
    deps.api.add_attributes(&attributes).unwrap();
}
