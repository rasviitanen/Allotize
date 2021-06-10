use crate::com::com_traits::{
    IceCandidate, PoolMetadata, Protocol, SignalingAction, SignalingMessage,
};
use crate::com::rtcconstructs::RtcConstructs;
use crate::com::rtctransaction::RtcTxn;
use crate::com::timed_event::TimedEvent;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    ErrorEvent, MessageEvent, RtcDataChannelState, RtcIceCandidateInit, RtcSdpType,
    RtcSessionDescriptionInit, WebSocket,
};

use crate::identity::Identity;

use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Indicator for a Pool. Used to understand when we can send messages
#[derive(PartialEq, Debug)]
enum PoolStatus {
    Disconnected,
    GotOffer,
    SentOffer,
    Connected,
}

struct AwaitRequirements {
    open_channels: u64,
}

/// A P2P pool that is used to share information between users using `WebRTC`
/// This currently leaks some closures, but it is ok because
/// we expect them to have a static lifetime.
pub struct RtcPool {
    ws: Rc<WebSocket>,
    pool_name: String,
    status: Rc<RefCell<PoolStatus>>,
    identity: Identity,
    rtc: Rc<RefCell<RtcConstructs>>,
    await_requirements: Rc<RefCell<Option<AwaitRequirements>>>,

    #[allow(dead_code)] // Used to not drop the value when passed to JS
    ws_on_open: Option<Closure<dyn FnMut(JsValue)>>,
    #[allow(dead_code)] // Used to not drop the value when passed to JS
    ws_on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    #[allow(dead_code)] // Used to not drop the value when passed to JS
    ws_on_error: Option<Closure<dyn FnMut(ErrorEvent)>>,

    #[allow(dead_code)] // Used to not drop the value when passed to JS
    heartbeat_worker: Option<TimedEvent>,
}

impl std::future::Future for &RtcPool {
    type Output = Result<(), ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let rtc = self.rtc.borrow_mut();

        if let Some(requirements) = &*self.await_requirements.borrow() {
            let mut open_channels = 0;
            for channel in rtc.channels.iter() {
                if let RtcDataChannelState::Open = channel.ready_state() {
                    open_channels += 1;
                }
            }
            if open_channels >= requirements.open_channels {
                return Poll::Ready(Ok(()));
            }
        }

        let task = cx.waker().clone();
        rtc.task.replace(Some(task));
        Poll::Pending
    }
}

impl RtcPool {
    /// Creates a new Pool that connects to our `Signaling Server`
    pub fn new(pool_name: &str, identity: Identity) -> RtcPool {
        let ws = Rc::new(
            WebSocket::new(&format!(
                // "wss://dry-brushlands-13605.herokuapp.com?user={}&room={}",
                "wss://allotize-signal.herokuapp.com/connect/{}/{}/123",
                pool_name, identity.username,
            ))
            .expect("WS not supported"),
        );

        RtcPool {
            ws: Rc::clone(&ws),
            pool_name: pool_name.into(),
            status: Rc::new(RefCell::new(PoolStatus::Disconnected)),
            identity: identity.clone(),
            rtc: Rc::new(RefCell::new(RtcConstructs::new(Rc::clone(&ws), identity))),
            await_requirements: Rc::new(RefCell::new(None)),

            ws_on_message: None,
            ws_on_open: None,
            ws_on_error: None,

            heartbeat_worker: None,
        }
    }

    pub fn metadata(&self) -> PoolMetadata {
        PoolMetadata {
            rtc: self.rtc.borrow().metadata(),
        }
    }

    pub fn set_onmessage(&self, onmessage: Option<Closure<dyn FnMut(MessageEvent)>>) {
        self.rtc.borrow_mut().set_onmessage(onmessage);
    }

    pub fn txn(&self) -> RtcTxn {
        RtcTxn::new(Rc::clone(&self.rtc))
    }

    pub fn require_channels(&self, open_channels: u64) -> &RtcPool {
        self.await_requirements
            .replace(Some(AwaitRequirements { open_channels }));
        self
    }

    fn attach_heartbeat_worker(&mut self) {
        let ws = Rc::clone(&self.ws);
        let thrity_second_period = 30000;
        let message = SignalingMessage {
            protocol: Protocol::OneToSelf,
            room: self.pool_name.clone(),
            from: "".to_string(),
            endpoint: None,
            action: SignalingAction::Heartbeat,
            data: None,
        };

        let heartbeat = serde_json::to_string(&message).expect("Invalid message serialization");

        let callback = Closure::wrap(Box::new(move || {
            if ws.send_with_str(&heartbeat).is_err() {
                info!(
                    "HEARTBEAT",
                    "Hearbeat failure", "Could not send heartbeat to server"
                );
            } else {
                info!("HEARTBEAT", "Hearbeat Success", "...");
            }
        }) as Box<dyn FnMut()>);

        let sender = TimedEvent::new(callback, thrity_second_period);
        self.heartbeat_worker = Some(sender);
    }

    /// Sets up necessary listeners for the WS
    pub fn setup(&mut self, send_on_setup: bool) -> &RtcPool {
        // Setup connection and datachannel listeners
        let cloned_identity = self.identity.clone();
        let status = Rc::clone(&self.status);
        let _cloned_ws = Rc::clone(&self.ws);
        let rtc = Rc::clone(&self.rtc);

        self.attach_heartbeat_worker();
        // Setup websocket listeners
        let onmessage_callback = Some(Closure::wrap(Box::new(move |e: MessageEvent| {
            // Respond to the message
            let response = e.data();
            let response: String = response.into_serde().expect("Could not serailize message");
            let message: SignalingMessage =
                serde_json::from_str(&response).expect("Could not serailize message");

            if let SignalingAction::Heartbeat = message.action {
                return;
            }

            if message.from != cloned_identity.username {
                match &message.action {
                    SignalingAction::Offer | SignalingAction::ReconnectOffer => {
                        // Newest peer executes this
                        info!(
                            "PROCESS OFFER",
                            "Processing offer", cloned_identity.username
                        );
                        RtcConstructs::setup_to_process_offer(Rc::clone(&rtc), message);
                        status.replace(PoolStatus::GotOffer);
                    }
                    SignalingAction::HandleConnection => {
                        info!("CREATE OFFER", "Creating offer", cloned_identity.username);
                        RtcConstructs::setup_to_create_offer(Rc::clone(&rtc), message.from);
                        status.replace(PoolStatus::SentOffer);
                    }
                    SignalingAction::Answer if *status.borrow() == PoolStatus::SentOffer => {
                        // Answering peer executes this as step 2
                        info!("ANSWER OFFER", "Answering offer", cloned_identity.username);

                        let data = &message
                            .data
                            .expect("No data provided in answer (got offer)");
                        let mut description: RtcSessionDescriptionInit =
                            RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                        description.sdp(&data);

                        rtc.borrow()
                            .set_remote_description(message.from, &description);
                    }
                    SignalingAction::Candidate => {
                        // Answering peer executes this as step 3
                        let data = &message
                            .data
                            .expect("No data provided in answer (got candidate)");
                        info!(
                            "CANDIDATE DESCRIPTION",
                            "Processing candidate",
                            format!("{:#?}", data)
                        );
                        let ice_candidate: IceCandidate =
                            serde_json::from_str(&data).expect("Ice Candidate has bad format");
                        let mut rtc_ice_candidate =
                            RtcIceCandidateInit::new(&ice_candidate.candidate);
                        rtc_ice_candidate.sdp_mid(ice_candidate.sdp_mid.as_ref().map(|sdp| &**sdp));
                        rtc.borrow()
                            .process_ice(message.from, Some(&rtc_ice_candidate));
                    }
                    _ => {}
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>));

        self.ws.set_onmessage(
            onmessage_callback
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );
        self.ws_on_message = onmessage_callback;

        let onerror_callback = Some(Closure::wrap(Box::new(move |_e: ErrorEvent| {
            panic!("Could not contact signaling server");
        }) as Box<dyn FnMut(ErrorEvent)>));

        self.ws.set_onerror(
            onerror_callback
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );
        self.ws_on_error = onerror_callback;

        if send_on_setup {
            let cloned_ws = self.ws.clone();
            let cloned_identity = self.identity.clone();
            let status = Rc::clone(&self.status);
            let pool_name = self.pool_name.clone();
            let onopen_callback = Some(Closure::wrap(Box::new(move |_| {
                // Send a negotiation when the channel is opened
                let message = SignalingMessage {
                    protocol: Protocol::OneToRoom,
                    room: pool_name.clone(),
                    from: cloned_identity.username.clone(),
                    endpoint: Some("any".to_string()),
                    action: SignalingAction::HandleConnection,
                    data: None,
                };

                match cloned_ws.send_with_str(
                    &serde_json::to_string(&message).expect("Invalid message serialization"),
                ) {
                    Ok(_) => {}
                    Err(_err) => panic!(),
                }

                status.replace(PoolStatus::Connected);
            }) as Box<dyn FnMut(JsValue)>));

            self.ws
                .set_onopen(onopen_callback.as_ref().map(|c| c.as_ref().unchecked_ref()));
            self.ws_on_open = onopen_callback;
        }

        self
    }
}

impl AsRef<RtcPool> for RtcPool {
    fn as_ref(&self) -> &Self {
        self
    }
}
