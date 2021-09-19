import React, { useEffect, useState } from "react";
import { useAllotize } from "../hooks/allotize";
import { subscribe, BoundedChannel } from "allotize-js";
import { sectionStyle, rowStyle } from "../style/style";

const channel = new BoundedChannel("myChannel", 3);

export function Channel() {
    const [text, setText] = useState("");
    const [received, setReceived] = useState("");
    const [sent, setSent] = useState("");

    useEffect(() => {
        channel.onRemoteMsg = e => {
            setReceived(JSON.stringify(channel.readAll()));
        };
        channel.connect();
    }, []);

    const send = e => {
        e.preventDefault();
        channel.send(text);
        setSent(text);
        setText("");
    }

    return (
        <section style={sectionStyle}>
            <h5>Channel</h5>
            <p>Send Message</p>
            <div style={rowStyle}>
                <form onSubmit={send}>
                    <label>
                        <input type="text" name="name" value={text} onChange={(e) => setText(e.target.value)} />
                    </label>
                    <input type="submit" value="Submit" />
                </form>
            </div>

            <div style={rowStyle}>
                Sent Message: {sent}
            </div>

            <div style={rowStyle}>
                Received Message: {received}
            </div>
        </section >
    );
}
