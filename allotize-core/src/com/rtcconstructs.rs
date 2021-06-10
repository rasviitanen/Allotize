use crate::com::com_traits::{
    IceCandidate, PeerConnectionStatus, Protocol, RtcMessage, RtcMetadata, SignalingAction,
    SignalingMessage,
};
use crate::identity::Identity;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    MessageEvent, RtcAnswerOptions, RtcConfiguration, RtcDataChannel, RtcDataChannelEvent,
    RtcDataChannelState, RtcIceCandidateInit, RtcIceServer, RtcOfferOptions, RtcPeerConnection,
    RtcPeerConnectionIceEvent, RtcSdpType, RtcSessionDescription, RtcSessionDescriptionInit,
    WebSocket,
};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::task::Waker;

use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;

type WASMClosure<E> = Closure<dyn FnMut(E)>;
/// Handles an RtcSession, by separating this from `pool`
/// it is possible to create different scopes in parallel
pub struct RtcConstructs {
    pub channels: Vec<RtcDataChannel>,
    pub status: Rc<RefCell<PeerConnectionStatus>>,
    pub task: Rc<RefCell<Option<Waker>>>,

    ws: Rc<WebSocket>,
    peer_connections: HashMap<String, Rc<RefCell<RtcPeerConnection>>>,
    identity: Identity,

    conn_on_icecandidate: Option<WASMClosure<RtcPeerConnectionIceEvent>>,
    conn_on_datachannel: Option<WASMClosure<RtcDataChannelEvent>>,
    ch_on_message: Option<WASMClosure<MessageEvent>>,
    ch_on_open: Option<WASMClosure<MessageEvent>>,
    ch_on_close: Option<WASMClosure<MessageEvent>>,

    // FIXME:(rasviitanen) We can create a wrapper for RtcPeerConnection
    // and store callbacks there instead and follow RAII to free the closure.
    gc_ice: Vec<Option<WASMClosure<RtcPeerConnectionIceEvent>>>,
    gc_dc: Vec<Option<WASMClosure<RtcDataChannelEvent>>>,
}

fn ice_configuration() -> RtcConfiguration {
    let mut configuration = RtcConfiguration::new();
    let mut stun_server = RtcIceServer::new();
    let stun_servers = js_sys::Array::new();
    stun_servers.push(&JsValue::from("stun:stun.l.google.com:19302"));
    stun_server.urls(&stun_servers);
    let mut turn_server = RtcIceServer::new();
    turn_server.url("turn:ec2-18-219-105-207.us-east-2.compute.amazonaws.com:80");
    turn_server.username("rasviitanen");
    turn_server.credential(env!("TURNPWD"));
    let ice_servers = js_sys::Array::new();
    ice_servers.push(&JsValue::from(&stun_server));
    ice_servers.push(&JsValue::from(&turn_server));
    configuration.ice_servers(&ice_servers);
    configuration
}

impl RtcConstructs {
    /// Creates a new `RtcConstructs`
    pub fn new(ws: Rc<WebSocket>, identity: Identity) -> RtcConstructs {
        let task: Rc<RefCell<Option<Waker>>> = Rc::new(RefCell::new(None));
        let status = Rc::new(RefCell::new(PeerConnectionStatus::Connecting));

        let task_clone = Rc::clone(&task);
        let status_clone = Rc::clone(&status);
        let ch_on_open = Some(Closure::wrap(Box::new(move |_e: MessageEvent| {
            status_clone.replace(PeerConnectionStatus::Open);
            if let Some(waker) = task_clone.borrow_mut().take() {
                waker.wake();
            };
        }) as Box<dyn FnMut(MessageEvent)>));

        let task_clone = Rc::clone(&task);
        let status_clone = Rc::clone(&status);
        let ch_on_close = Some(Closure::wrap(Box::new(move |_e: MessageEvent| {
            status_clone.replace(PeerConnectionStatus::Closed);
            if let Some(waker) = task_clone.borrow_mut().take() {
                waker.wake();
            }
        }) as Box<dyn FnMut(MessageEvent)>));

        let ws_clone = Rc::clone(&ws);
        let conn_on_icecandidate = Some(Closure::wrap(Box::new(
            move |e: RtcPeerConnectionIceEvent| {
                if let Some(candidate) = e.candidate() {
                    let candidate_description = IceCandidate {
                        candidate: candidate.candidate(),
                        sdp_mid: candidate.sdp_mid(),
                    };

                    let message = SignalingMessage {
                        protocol: Protocol::OneToOne,
                        room: "test-room".to_string(),
                        from: "".to_string(),
                        endpoint: Some("any".to_string()),
                        action: SignalingAction::Candidate,
                        data: serde_json::to_string(&candidate_description).ok(),
                    };

                    match ws_clone.send_with_str(
                        &serde_json::to_string(&message).expect("Invalid message serialization"),
                    ) {
                        Ok(_) => {}
                        Err(_err) => panic!("Ice candidate message error"),
                    }
                }
            },
        )
            as Box<dyn FnMut(RtcPeerConnectionIceEvent)>));

        RtcConstructs {
            ws,
            channels: Vec::new(),
            identity,
            status,
            task,

            peer_connections: HashMap::new(),

            ch_on_open,
            ch_on_message: None,
            ch_on_close,
            conn_on_datachannel: None,
            conn_on_icecandidate,

            gc_ice: Vec::new(),
            gc_dc: Vec::new(),
        }
    }

    pub fn metadata(&self) -> RtcMetadata {
        let mut connecting = 0;
        let mut open = 0;
        let mut closing = 0;
        let mut closed = 0;

        for channel in &self.channels {
            match channel.ready_state() {
                RtcDataChannelState::Connecting => connecting += 1,
                RtcDataChannelState::Open => open += 1,
                RtcDataChannelState::Closing => closing += 1,
                RtcDataChannelState::Closed => closed += 1,
                _ => {}
            }
        }

        RtcMetadata {
            connecting,
            open,
            closing,
            closed,
        }
    }

    pub fn set_onmessage(&mut self, onmessage: Option<WASMClosure<MessageEvent>>) {
        self.ch_on_message = onmessage;
    }

    pub fn close_channels(&self) {
        for channel in &self.channels {
            channel.close();
        }
    }

    /// Processes an offer request that is sent from a peer
    pub fn setup_to_process_offer(rtc: Rc<RefCell<RtcConstructs>>, message: SignalingMessage) {
        let configuration = ice_configuration();

        let new_connection = RtcPeerConnection::new_with_configuration(&configuration)
            .expect("Can't create RTCPeerConnection");

        let identity_clone = rtc.borrow().identity.clone();
        let ws_clone = Rc::clone(&rtc.borrow().ws);
        let requestee_clone = message.from.clone();
        let conn_on_icecandidate = Closure::wrap(Box::new(move |e: RtcPeerConnectionIceEvent| {
            info!(
                "PROCESS OFFER - ON ICE CANDIDATE",
                "ON ICE CANDIDATE", "ON ICE CANDIDATE"
            );

            if let Some(candidate) = e.candidate() {
                let candidate_description = IceCandidate {
                    candidate: candidate.candidate(),
                    sdp_mid: candidate.sdp_mid(),
                };

                let message = SignalingMessage {
                    protocol: Protocol::OneToOne,
                    room: "test_room".into(),
                    from: identity_clone.username.clone(),
                    endpoint: Some((&requestee_clone).to_string()),
                    action: SignalingAction::Candidate,
                    data: serde_json::to_string(&candidate_description).ok(),
                };

                match ws_clone.send_with_str(
                    &serde_json::to_string(&message).expect("Invalid message serialization"),
                ) {
                    Ok(_) => {}
                    Err(_err) => panic!("Ice candidate message error"),
                }
            }
        })
            as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);

        let old_ice_cb = rtc
            .borrow_mut()
            .conn_on_icecandidate
            .replace(conn_on_icecandidate);
        rtc.borrow_mut().gc_ice.push(old_ice_cb);

        new_connection.set_onicecandidate(
            rtc.borrow()
                .conn_on_icecandidate
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );

        let cloned_rtc = Rc::clone(&rtc);
        let on_datachannel = Closure::wrap(Box::new(move |e: RtcDataChannelEvent| {
            cloned_rtc.borrow_mut().on_datachannel(e)
        }) as Box<dyn FnMut(RtcDataChannelEvent)>);
        let old_dc_cb = rtc.borrow_mut().conn_on_datachannel.replace(on_datachannel);
        rtc.borrow_mut().gc_dc.push(old_dc_cb);

        new_connection.set_ondatachannel(
            rtc.borrow()
                .conn_on_datachannel
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );

        spawn_local(RtcConstructs::create_answer(rtc, new_connection, message));
    }

    /// Creates an offer request that is sent to a peer
    pub fn setup_to_create_offer(rtc: Rc<RefCell<RtcConstructs>>, sender: String) {
        let configuration = ice_configuration();

        let new_connection = RtcPeerConnection::new_with_configuration(&configuration)
            .expect("Can't create RTCPeerConnection");

        let send_channel = new_connection.create_data_channel("hadal-default-channel");

        let identity_clone = rtc.borrow().identity.clone();
        let ws_clone = Rc::clone(&rtc.borrow().ws);
        let requestee_clone = sender.clone();
        let conn_on_icecandidate = Closure::wrap(Box::new(move |e: RtcPeerConnectionIceEvent| {
            info!(
                "CRETE OFFER - ON ICE CANDIDATE",
                "ON ICE CANDIDATE", "ON ICE CANDIDATE"
            );
            if let Some(candidate) = e.candidate() {
                let candidate_description = IceCandidate {
                    candidate: candidate.candidate(),
                    sdp_mid: candidate.sdp_mid(),
                };

                let message = SignalingMessage {
                    protocol: Protocol::OneToOne,
                    room: "test_room".into(),
                    from: identity_clone.username.clone(),
                    endpoint: Some((&requestee_clone).to_string()),
                    action: SignalingAction::Candidate,
                    data: serde_json::to_string(&candidate_description).ok(),
                };

                match ws_clone.send_with_str(
                    &serde_json::to_string(&message).expect("Invalid message serialization"),
                ) {
                    Ok(_) => {}
                    Err(_err) => panic!("Ice candidate message error"),
                }
            }
        })
            as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);
        let old_ice_cb = rtc
            .borrow_mut()
            .conn_on_icecandidate
            .replace(conn_on_icecandidate);
        rtc.borrow_mut().gc_ice.push(old_ice_cb);

        new_connection.set_onicecandidate(
            rtc.borrow()
                .conn_on_icecandidate
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );

        send_channel.set_onopen(
            rtc.borrow()
                .ch_on_open
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );

        send_channel.set_onmessage(
            rtc.borrow()
                .ch_on_message
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );

        send_channel.set_onclose(
            rtc.borrow()
                .ch_on_close
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );

        rtc.borrow_mut().channels.push(send_channel);

        // TODO enable reconnection by setting false to true in some cases
        spawn_local(RtcConstructs::create_offer(
            rtc,
            sender,
            new_connection,
            false,
        ));
    }

    /// Creates an offer using the contained `RtcPeerConnection`
    async fn create_offer(
        rtc: Rc<RefCell<RtcConstructs>>,
        sender: String,
        new_connection: RtcPeerConnection,
        reconnect: bool,
    ) {
        let mut options = RtcOfferOptions::new();
        options.ice_restart(reconnect);
        let offer_promise = new_connection.create_offer_with_rtc_offer_options(&options);
        let offer = JsFuture::from(offer_promise)
            .await
            .expect("Could not create offer");
        rtc.borrow_mut()
            .on_offer(offer, sender, new_connection, reconnect);
    }

    /// Creates an answer using the contained `RtcPeerConnection`
    async fn create_answer(
        rtc: Rc<RefCell<RtcConstructs>>,
        new_connection: RtcPeerConnection,
        offer: SignalingMessage,
    ) {
        let mut options = RtcOfferOptions::new();
        if let SignalingAction::ReconnectOffer = offer.action {
            options.ice_restart(true);
        } else {
            options.ice_restart(false);
        }
        let answer_options: RtcAnswerOptions = options.unchecked_into();

        let new_connection = Rc::new(RefCell::new(new_connection));
        rtc.borrow_mut()
            .peer_connections
            .insert(offer.from.clone(), Rc::clone(&new_connection));

        let data = &offer.data.expect("No data provided in answer (got offer)");
        let mut description: RtcSessionDescriptionInit =
            RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        description.sdp(&data);

        JsFuture::from(new_connection.borrow().set_remote_description(&description))
            .await
            .expect("Could not process offer");

        let answer_promise = new_connection
            .borrow()
            .create_answer_with_rtc_answer_options(&answer_options);
        let answer = JsFuture::from(answer_promise)
            .await
            .expect("Could not create answer");

        rtc.borrow_mut().on_answer(answer, offer.from);
    }

    /// Sends a message over referenced `WS`, likely being the same as the WS used in `Pool`
    fn send_message(&self, message: &SignalingMessage) {
        match self
            .ws
            .send_with_str(&serde_json::to_string(&message).expect("Invalid message serialization"))
        {
            Ok(_) => {}
            Err(_err) => panic!(),
        }
    }

    /// Ands an Ice Candidate to the current connection
    pub fn process_ice(&self, sender: String, ice_candiate: Option<&RtcIceCandidateInit>) {
        info!("Processing ice", "Processing ice", "");

        wasm_bindgen_futures::JsFuture::from(
            self.peer_connections
                .get(&sender)
                .expect("Requires connection before adding Ice candidate")
                .borrow()
                .add_ice_candidate_with_opt_rtc_ice_candidate_init(ice_candiate),
        );
    }

    /// Ands an Ice Candidate to the current connection
    pub fn set_remote_description(&self, sender: String, description: &RtcSessionDescriptionInit) {
        wasm_bindgen_futures::JsFuture::from(
            self.peer_connections
                .get(&sender)
                .expect("Requires connection before adding remote description")
                .borrow()
                .set_remote_description(&description),
        );
    }

    /// Callback to run when an answer has been created.
    /// This sets the local description for the contained `RtcPeerConnection`
    /// and sends an `answer` over the WS.
    fn on_answer(&mut self, answer: JsValue, sender: String) {
        let description: RtcSessionDescriptionInit = answer.unchecked_into();
        let session = RtcSessionDescription::new_with_description_init_dict(&description)
            .expect("Could not create sdp description");

        info!("SETTING LOCAL DESCRIPTION", "SETTING LOCAL", "LOCAL");

        wasm_bindgen_futures::JsFuture::from(
            self.peer_connections
                .get_mut(&sender)
                .expect("No peer connection")
                .borrow()
                .set_local_description(&description),
        );

        // self.peer_connections.insert(sender.clone(), new_connection);

        let message = SignalingMessage {
            protocol: Protocol::OneToOne,
            room: "test_room".to_string(),
            from: self.identity.username.clone(),
            endpoint: Some(sender),
            action: SignalingAction::Answer,
            data: Some(session.sdp()),
        };

        self.send_message(&message);
    }

    /// Callback to run when a datachannel is established
    /// includes callbacks for `onopen`, `onclose` and `onmessage`
    fn on_datachannel(&mut self, e: RtcDataChannelEvent) {
        info!("RUNNING ON DATACHNL", "on data ch", "on datach");
        let channel = e.channel();
        // Set callback functions for channel
        channel.set_onopen(self.ch_on_open.as_ref().map(|c| c.as_ref().unchecked_ref()));
        channel.set_onmessage(
            self.ch_on_message
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );
        channel.set_onclose(
            self.ch_on_close
                .as_ref()
                .map(|c| c.as_ref().unchecked_ref()),
        );

        self.channels.push(channel);
    }

    /// Callback to run when an offer is created
    /// This sends the offer back to the remote peer
    fn on_offer(
        &mut self,
        offer: JsValue,
        sender: String,
        new_connection: RtcPeerConnection,
        reconnect: bool,
    ) {
        let description: RtcSessionDescriptionInit = offer.unchecked_into();
        let desc_string = RtcSessionDescription::new_with_description_init_dict(&description)
            .expect("Could not create sdp description");

        let mut action = SignalingAction::Offer;
        if reconnect {
            action = SignalingAction::ReconnectOffer;
        }

        wasm_bindgen_futures::JsFuture::from(new_connection.set_local_description(&description));

        self.peer_connections
            .insert(sender.clone(), Rc::new(RefCell::new(new_connection)));

        let message = SignalingMessage {
            protocol: Protocol::OneToOne,
            room: "test_room".to_string(),
            from: self.identity.username.clone(),
            endpoint: Some(sender),
            action,
            data: Some(desc_string.sdp()),
        };
        self.send_message(&message);
    }

    /// Sends a message to all establised `RtcDataChannels`
    pub fn broadcast(&self, message: &RtcMessage) {
        let msg = &serde_json::to_string(message).expect("Invalid RtcMessage serialization");

        for channel in &self.channels {
            if channel.send_with_str(&msg).is_err() {
                info!(
                    "RTCConstructs",
                    "Broadcast", "Could not send message to channel"
                );
            }
        }
    }
}
