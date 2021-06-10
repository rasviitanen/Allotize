use crate::com::com_traits::RtcMessage;
use crate::com::rtcconstructs::RtcConstructs;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;

use futures_channel::oneshot;

use std::cell::RefCell;
use std::rc::Rc;

/// A transaction for communication over a datachannel
pub struct RtcTxn {
    inner: Rc<RefCell<RtcConstructs>>,
    pub on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    rx: Option<oneshot::Receiver<RtcMessage>>,
}

impl RtcTxn {
    /// Creates a new `RtcTxn`
    pub fn new(rtc: Rc<RefCell<RtcConstructs>>) -> RtcTxn {
        RtcTxn {
            inner: Rc::clone(&rtc),
            on_message: None,
            rx: None,
        }
    }

    /// Get the result of the transaction.
    /// This will actually return the receiver end of a oneshot channel,
    /// so it is necessary to use `.await` in order to receive the true result.
    ///
    /// When calling this function, ownership of the Receiver is transfered,
    /// so you are able to scheule a new action for the transaction,
    /// and still be able to receive both values.
    /// (Given that you take care of the Receiver)
    pub fn result(&mut self) -> oneshot::Receiver<RtcMessage> {
        self.rx
            .take()
            .expect("You need to queue a transaction before you can get a result")
    }

    pub async fn broadcast(self, message: &RtcMessage) {
        self.inner.borrow().broadcast(message);
    }

    pub fn recv_cmd(mut self, command: &'static str) -> RtcTxn {
        let (cx, rx) = oneshot::channel();
        self.on_message = Some(Closure::once(move |e: MessageEvent| {
            let received_str: String =
                JsValue::as_string(&e.data()).expect("Invalid message format");
            let received_msg: RtcMessage =
                serde_json::from_str(&received_str).expect("Invalid message format");

            if received_msg.command == command {
                cx.send(received_msg).unwrap();
            }
        }));

        // TODO Remove event listener when the transaction is dropped
        for channel in &self.inner.borrow().channels {
            channel
                .add_event_listener_with_callback(
                    "message",
                    self.on_message
                        .as_ref()
                        .map(|c| c.as_ref().unchecked_ref())
                        .unwrap(),
                )
                .unwrap();
        }

        self.rx = Some(rx);
        self
    }

    pub fn recv(mut self, callback: Option<Box<dyn FnOnce(MessageEvent)>>) -> RtcTxn {
        let (cx, rx) = oneshot::channel();
        self.on_message = Some(Closure::once(move |e: MessageEvent| {
            let _data = e.data();
            let received_str: String =
                JsValue::as_string(&e.data()).expect("Invalid message format");
            let received_msg: RtcMessage =
                serde_json::from_str(&received_str).expect("Invalid message format");

            if let Some(cb) = callback {
                cb(e);
            };

            cx.send(received_msg).unwrap();
        }));

        for channel in &self.inner.borrow().channels {
            channel.set_onmessage(self.on_message.as_ref().map(|c| c.as_ref().unchecked_ref()));
        }

        self.rx = Some(rx);
        self
    }
}
