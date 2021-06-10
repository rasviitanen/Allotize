import React from "react";
import ReactDOM from "react-dom";

import { Endeavor } from './js/components/Endeavor.jsx'
import { Sidebar } from './js/components/Sidebar.jsx'
import { DATA } from "./data.js"

const mainStyle = {
  display: "flex",
  flexDirection: "row",
}

const contentStyle = {
  display: "flex",
  flexDirection: "column",
  padding: "1rem"
}

class App extends React.Component {
  render() {
    return (
      <div className="App">
        <main style={mainStyle}>
        <Sidebar />
        <section style={contentStyle}>
          <Endeavor title={ DATA.requirements.title } cards={ DATA.requirements.cards }/>
        </section>
        </main>
      </div>
    )
  }
}


ReactDOM.render(
  <App />,
  document.getElementById("app"));
