use cosmwasm_std::{entry_point, Attribute, DepsMut, Env, Event, MessageInfo, Response};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

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
