import React, { useState, useEffect } from "react";
import { Card } from "./Card.jsx";

const style = {
  display: "flex",
  flexDirection: "row",
  justifyContent: "space-between",
  background: "#f3f3f3",
  color: "#030303",
  padding: "1rem",
  overflowX: "scroll",
}

export function Endeavor(props) {
  const [done, setDone] = useState(0);

  const cards = props.cards.map((card, idx) => {
    return (
      <Card key={idx} title={card.title} checklist={card.checklist} />
    );
  });

  return (
    <div>
      <h2>{props.title}</h2>
      <section style={style}>
      { cards }
      </section>
    </div>
  );
}