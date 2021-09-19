import React, { useState, useEffect } from "react";
import { username, metadata } from "allotize-js";

const labelStyle = {
  padding: "1px 6px",
  backgroundColor: "#0c0c0c",
  color: "white",
};

export function Meta() {
  const [meta, setMeta] = useState({
    pool: {
      rtc: {
        open: 0,
        closed: 0,
      },
      token: "",
    },
  });

  useEffect(() => {
    async function getMeta() {
      setMeta(await metadata());
      setInterval(async () => {
        setMeta(await metadata());
      }, 3000);
    }
    getMeta();
  }, []);

  return (
    <>
      <section>
        <h4>User</h4>
        <strong>{username}</strong>
      </section>
      <section>
        <h4>Token</h4>
        <span style={labelStyle}>{meta.token}</span>
      </section>
      <section>
        <h4>Connections</h4>
        <span style={labelStyle}>open {meta.pool.rtc.open}</span>{" "}
        <span style={labelStyle}>closed {meta.pool.rtc.closed}</span>
      </section>
    </>
  );
}
