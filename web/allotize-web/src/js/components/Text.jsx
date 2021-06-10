import React from "react";
import { useAllotize } from "../hooks/allotize";

const sectionStyle = {
  margin: "1em",
  background: "#292931",
  padding: "1rem",
};

const rowStyle = {
  display: "flex",
  flexDirection: "row",
  alignItems: "center",
};

export function Text() {
  const [state, setState] = useAllotize({
    route: `text`,
    data: {
      content: "",
    },
  });

  return (
    <section style={sectionStyle}>
      <h5>Live Text</h5>
      <div style={rowStyle}>
        <textarea
          value={state.content}
          onChange={(e) => setState({ ...state, content: e.target.value })}
        ></textarea>
      </div>
    </section>
  );
}
