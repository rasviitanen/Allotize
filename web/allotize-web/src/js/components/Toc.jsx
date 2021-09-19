import React, { useState, useEffect } from "react";
import { getAll } from "allotize-js";
import { sectionStyle } from "../style/style";

const tableStyle = {
  width: "100%",
};

const oddTableStyle = {
  background: "#f3f3f313",
  borderRadius: "5px"
};

const tdStyle = {
  padding: "12px",
};

const preStyle = {
  color: "white",
};

export function Toc() {
  const [toc, setToc] = useState([]);
  const [ready, setReady] = useState(false);
  const getJsonIndented = (obj) => obj ? JSON.stringify(obj, null, 2).replace(/["{[\}\]]/g, "") : "";

  useEffect(() => {
    async function getToc() {
      setToc(await getAll());
      setReady(true);
      setInterval(async () => {
        setToc(await getAll());
      }, 3000);
    }
    getToc();
  }, []);

  return (
    <>
      <section style={sectionStyle}>
        <table style={tableStyle}>
          <tr>
            <th>Key</th>
            <th>Value</th>
            <th>Clock</th>
          </tr>
          {ready
            ? toc.map((item, idx) => {
              return (
                <tr key={idx} style={idx % 2 ? oddTableStyle : null}>
                  <td style={tdStyle}>{item.key}</td>
                  <td style={tdStyle}>
                    <pre style={preStyle}>{getJsonIndented(item.value)}</pre>
                  </td>
                  <td style={tdStyle}>
                    <pre style={preStyle}>{getJsonIndented(item.clock.dots)}</pre>
                  </td>
                </tr>
              );
            })
            : "loading"}
        </table>
      </section>
    </>
  );
}
