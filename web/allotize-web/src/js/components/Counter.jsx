import React, { useEffect } from "react";
import { useAllotize } from "../hooks/allotize";
import { subscribe } from "allotize-js";

const sectionStyle= {
    margin: "1em",
    background: "#8675A2",
    padding: "1rem"
}

const btnStyle= {
    border: "none",
    background: "none"
}

const counterStyle = {
    fontSize: "3em",
    marginLeft: "0.5em"
}

const columnStyle = {
    display: "flex",
    flexDirection: "column",
}


const rowStyle = {
    display: "flex",
    flexDirection: "row",
    alignItems: "center"
}

export function Counter() {
    const [state, setState] = useAllotize({
        route: `counter`,
        data: {
            count: 0,
        },
    });

    useEffect(() => {
        subscribe(`counter`, (e) => {
            console.log("event", e);
        });
    }, []);

    const increment = () => {
        setState({
            ...state,
            count: state.count + 1,
        });
    };

    const decrement = () => {
        setState({
            ...state,
            count: state.count - 1,
        });
    };



    return (
        <section style={sectionStyle}>
            <h5>Counter</h5>
            <div style={rowStyle}>
                <div>
                    <div style={columnStyle}>
                        <button onClick={increment} style={btnStyle}>▲</button>
                        <button onClick={decrement} style={btnStyle}>▼</button>
                    </div>
                </div>
                <span style={counterStyle}>{state.count}</span>
            </div>
        </section>
    );
}