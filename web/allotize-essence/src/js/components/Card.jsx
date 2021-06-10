import React, { useState, useEffect, useMemo } from "react";
import { useAllotize } from "../hooks/allotize";
import { beginsWith, subscribe, unsubscribe } from "allotize-js";

const cardStyle = {
  padding: "1rem",
  transition: "0.25s"
}

export function CardField(props) {
  const [state, setState] = useAllotize({
    route: `card#${props.parent}#${props.title}`,
      data: {
        done: false,
    },

    config: {
      onChange: (old, newData) => {
        if (state !== newData.done) {
          props.countDone();
        }
      }
    }
  });

  const toggle = () => {
    setState({...state, done: !state.done})
  };

  return (
    <section>
      <label>
        <input type="checkbox" onChange={() => toggle()} checked={ state.done ? true : false}/>
        &nbsp;
        { props.title }
      </label>
    </section>
  );
}

export function Card(props) {
  const [doneCount, setDone] = useState(0);
  const [allDone, setAllDone] = useState(false);
  async function countDone() {
    let items = await beginsWith(`card#${props.title}`);
    const done = items.filter(it => it.value.state.done).length;
    setAllDone(done == props.checklist.length);
    setDone(done);
  }

  useEffect(() => {
    countDone();
  }, []);

  const checklist = props.checklist.map((item, idx) => {
    return (
      <CardField key={idx}
        countDone={countDone}
        parent={props.title}
        title={item} />) });

  const doneStyle = allDone ? {
    background: "lightgreen",
    opacity: "0.4"
  } : {};
  return (
    <div style={{...cardStyle, ...doneStyle }}>
      <h2>{props.title}</h2>
      <span>{doneCount}/{props.checklist.length}</span>
      {checklist}
    </div>
  );
}
