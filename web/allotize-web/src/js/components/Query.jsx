import React, { useState, useEffect } from "react";
import { beginsWith, remove } from "allotize-js";
import { sectionStyle } from "../style/style";

const tableStyle = {
  width: "100%",
  border: "1px solid white",
};

const oddTableStyle = {
  background: "#f3f3f313",
};

const tdStyle = {
  padding: "12px",
};

const preStyle = {
  color: "white",
};

export function Query() {
  const [items, setItems] = useState([]);
  const [ready, setReady] = useState(false);
  const [prefix, setPrefix] = useState("queryExample#");
  const getJsonIndented = (obj) =>
    obj ? JSON.stringify(obj, null, 2).replace(/["{[\}\]]/g, "") : "";
  useEffect(() => {
    async function getPrefixed() {
      setItems(await beginsWith(prefix));
      setReady(true);
    }
    getPrefixed();
  }, []);

  return (
    <section style={sectionStyle}>
      <p>
        <strong>Prefix:</strong>
        <form
          onSubmit={async (e) => {
            e.preventDefault();
            setItems(await beginsWith(prefix));
          }}
        >
          <input
            value={prefix}
            onChange={(e) => setPrefix(e.target.value)}
          ></input>
          <button type="submit">Query</button>
        </form>
      </p>

      <table style={tableStyle}>
        <tr>
          <th>Key</th>
          <th>Value</th>
          <th>Clock</th>
          <th>Actions</th>
        </tr>
        {ready
          ? items.map((item, idx) => {
            return (
              <tr key={idx} style={idx % 2 ? oddTableStyle : null}>
                <td style={tdStyle}>{item.key}</td>
                <td style={tdStyle}>
                  <pre style={preStyle}>{getJsonIndented(item.value)}</pre>
                </td>
                <td style={tdStyle}>
                  <pre style={preStyle}>
                    {getJsonIndented(item.clock.dots)}
                  </pre>
                </td>
                <td style={tdStyle}>
                  <button onClick={async () => {
                    remove(item.key);
                    setItems(await beginsWith(prefix));
                  }}>remove</button>
                </td>
              </tr>
            );
          })
          : "loading"}
      </table>
    </section>
  );
}
