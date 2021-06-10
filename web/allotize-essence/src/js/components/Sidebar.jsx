import React, { useState, useEffect } from "react";

const style = {
    background: "#f3f3f3",
    padding: "1rem",
    display: "block",
}

const sticky = {
    position: "sticky",
    top: "1rem"
}

export function Sidebar(props) {
  return (
      <div style={style}>
          <div style={sticky}>
            <h2>Essential Plan</h2>
                <ul>
                    <li>blah</li>
            </ul>
          </div>
    </div>
  );
}