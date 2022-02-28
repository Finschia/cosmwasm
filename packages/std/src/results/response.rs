use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::Binary;

use super::{Attribute, CosmosMsg, Empty, Event, SubMsg};

/// A response of a contract entry point, such as `instantiate`, `execute` or `migrate`.
///
/// This type can be constructed directly at the end of the call. Alternatively a
/// mutable response instance can be created early in the contract's logic and
/// incrementally be updated.
///
/// ## Examples
///
/// Direct:
///
/// ```
/// # use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo};
/// # type InstantiateMsg = ();
/// #
/// use cosmwasm_std::{attr, Response, StdResult};
///
/// pub fn instantiate(
///     deps: DepsMut,
///     _env: Env,
///     _info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> StdResult<Response> {
///     // ...
///
///     Ok(Response::new().add_attribute("action", "instantiate"))
/// }
/// ```
///
/// Mutating:
///
/// ```
/// # use cosmwasm_std::{coins, BankMsg, Binary, DepsMut, Env, MessageInfo, SubMsg};
/// # type InstantiateMsg = ();
/// # type MyError = ();
/// #
/// use cosmwasm_std::Response;
///
/// pub fn instantiate(
///     deps: DepsMut,
///     _env: Env,
///     info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> Result<Response, MyError> {
///     let mut response = Response::new()
///         .add_attribute("Let the", "hacking begin")
///         .add_message(BankMsg::Send {
///             to_address: String::from("recipient"),
///             amount: coins(128, "uint"),
///         })
///         .add_attribute("foo", "bar")
///         .set_data(b"the result data");
///     Ok(response)
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[non_exhaustive]
pub struct Response<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// Optional list of messages to pass. These will be executed in order.
    /// If the ReplyOn variant matches the result (Always, Success on Ok, Error on Err),
    /// the runtime will invoke this contract's `reply` entry point
    /// after execution. Otherwise, they act like "fire and forget".
    /// Use `SubMsg::new` to create messages with the older "fire and forget" semantics.
    pub messages: Vec<SubMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event.
    ///
    /// More info about events (and their attributes) can be found in [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/v0.42/core/events.html
    pub attributes: Vec<Attribute>,
    /// Extra, custom events separate from the main `wasm` one. These will have
    /// `wasm-` prepended to the type.
    ///
    /// More info about events can be found in [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/v0.42/core/events.html
    pub events: Vec<Event>,
    /// The binary payload to include in the response.
    pub data: Option<Binary>,
}

impl<T> Default for Response<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        Response {
            messages: vec![],
            attributes: vec![],
            events: vec![],
            data: None,
        }
    }
}

impl<T> Response<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an attribute included in the main `wasm` event.
    #[must_use]
    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute::new(key, value));
        self
    }

    /// This creates a "fire and forget" message, by using `SubMsg::new()` to wrap it,
    /// and adds it to the list of messages to process.
    #[must_use]
    pub fn add_message(mut self, msg: impl Into<CosmosMsg<T>>) -> Self {
        self.messages.push(SubMsg::new(msg));
        self
    }

    /// This takes an explicit SubMsg (creates via eg. `reply_on_error`)
    /// and adds it to the list of messages to process.
    #[must_use]
    pub fn add_submessage(mut self, msg: SubMsg<T>) -> Self {
        self.messages.push(msg);
        self
    }

    /// Adds an extra event to the response, separate from the main `wasm` event
    /// that is always created.
    ///
    /// The `wasm-` prefix will be appended by the runtime to the provided type
    /// of event.
    #[must_use]
    pub fn add_event(mut self, event: impl Into<Event>) -> Self {
        self.events.push(event.into());
        self
    }

    /// Bulk add attributes included in the main `wasm` event.
    ///
    /// Anything that can be turned into an iterator and yields something
    /// that can be converted into an `Attribute` is accepted.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{attr, Response};
    ///
    /// let attrs = vec![
    ///     ("action", "reaction"),
    ///     ("answer", "42"),
    ///     ("another", "attribute"),
    /// ];
    /// let res: Response = Response::new().add_attributes(attrs.clone());
    /// assert_eq!(res.attributes, attrs);
    /// ```
    #[must_use]
    pub fn add_attributes<A: Into<Attribute>>(
        mut self,
        attrs: impl IntoIterator<Item = A>,
    ) -> Self {
        self.attributes.extend(attrs.into_iter().map(A::into));
        self
    }

    /// Bulk add "fire and forget" messages to the list of messages to process.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{CosmosMsg, Response};
    ///
    /// fn make_response_with_msgs(msgs: Vec<CosmosMsg>) -> Response {
    ///     Response::new().add_messages(msgs)
    /// }
    /// ```
    #[must_use]
    pub fn add_messages<M: Into<CosmosMsg<T>>>(self, msgs: impl IntoIterator<Item = M>) -> Self {
        self.add_submessages(msgs.into_iter().map(SubMsg::new))
    }

    /// Bulk add explicit SubMsg structs to the list of messages to process.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{SubMsg, Response};
    ///
    /// fn make_response_with_submsgs(msgs: Vec<SubMsg>) -> Response {
    ///     Response::new().add_submessages(msgs)
    /// }
    /// ```
    #[must_use]
    pub fn add_submessages(mut self, msgs: impl IntoIterator<Item = SubMsg<T>>) -> Self {
        self.messages.extend(msgs.into_iter());
        self
    }

    /// Bulk add custom events to the response. These are separate from the main
    /// `wasm` event.
    ///
    /// The `wasm-` prefix will be appended by the runtime to the provided types
    /// of events.
    #[must_use]
    pub fn add_events<S>(mut self, events: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<Event>,
    {
        self.events.extend(events.into_iter().map(|e| e.into()));
        self
    }

    /// Set the binary data included in the response.
    #[must_use]
    pub fn set_data(mut self, data: impl Into<Binary>) -> Self {
        self.data = Some(data.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::super::BankMsg;
    use super::*;
    use crate::results::submessages::{ReplyOn, UNUSED_MSG_ID};
    use crate::{attr, coins, from_slice, to_vec, Addr, Coin, Event, IntoEvent};

    #[test]
    fn can_serialize_and_deserialize_init_response() {
        let original = Response {
            messages: vec![
                SubMsg {
                    id: 12,
                    msg: BankMsg::Send {
                        to_address: String::from("checker"),
                        amount: coins(888, "moon"),
                    }
                    .into(),
                    gas_limit: Some(12345u64),
                    reply_on: ReplyOn::Always,
                },
                SubMsg {
                    id: UNUSED_MSG_ID,
                    msg: BankMsg::Send {
                        to_address: String::from("you"),
                        amount: coins(1015, "earth"),
                    }
                    .into(),
                    gas_limit: None,
                    reply_on: ReplyOn::Never,
                },
            ],
            attributes: vec![Attribute {
                key: "action".to_string(),
                value: "release".to_string(),
            }],
            events: vec![],
            data: Some(Binary::from([0xAA, 0xBB])),
        };
        let serialized = to_vec(&original).expect("encode contract result");
        let deserialized: Response = from_slice(&serialized).expect("decode contract result");
        assert_eq!(deserialized, original);
    }

    #[test]
    fn using_into_event() {
        // IntoEvent can be used only when cosmwasm_std is imported as `cosmwasm_std`
        use crate as cosmwasm_std;

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

        #[derive(Clone, IntoEvent)]
        struct TransferEvent {
            from: Addr,
            receiver: Addr,
            #[to_string_fn(coins_to_string)]
            amount: Vec<Coin>,
        }

        let transfer_event = TransferEvent {
            from: Addr::unchecked("alice"),
            receiver: Addr::unchecked("bob"),
            amount: coins(42, "link"),
        };
        let expected =
            Response::<Empty>::new().add_event(Event::new("transfer_event").add_attributes(vec![
                attr("from", "alice"),
                attr("receiver", "bob"),
                attr("amount", coins_to_string(coins(42, "link"))),
            ]));
        let actual = Response::<Empty>::new().add_event(transfer_event);
        assert_eq!(actual, expected);
    }

    #[test]
    fn using_into_event_add_events() {
        use crate as cosmwasm_std;

        fn u32_to_string(n: u32) -> String {
            n.to_string()
        }

        #[derive(IntoEvent)]
        struct Act {
            name: String,
            #[to_string_fn(u32_to_string)]
            amount: u32,
        }

        let act1 = Act {
            name: "mint".to_string(),
            amount: 42,
        };
        let act2 = Act {
            name: "burn".to_string(),
            amount: 21,
        };
        let event1 =
            Event::new("act").add_attributes(vec![attr("name", "mint"), attr("amount", "42")]);
        let event2 =
            Event::new("act").add_attributes(vec![attr("name", "burn"), attr("amount", "21")]);
        let expected = Response::<Empty>::new().add_events(vec![event1, event2]);
        let actual = Response::<Empty>::new().add_events(vec![act1, act2]);
        assert_eq!(actual, expected);
    }
}
