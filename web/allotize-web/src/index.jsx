import React from "react";
import ReactDOM from "react-dom";

import { Counter } from './js/components/Counter.jsx'
import { Meta } from './js/components/Meta.jsx'
import { Text } from './js/components/Text.jsx'
import { Toc } from './js/components/Toc.jsx'
import { Query } from './js/components/Query.jsx'

class App extends React.Component {
  render() {
    return (
      <div className="App">
        <Meta></Meta>
        <br></br>
        <h2>App</h2>
        <Counter></Counter>
        <Text></Text>
        <h2>Query</h2>
        <Query></Query>
        <h2>Store</h2>
        <Toc></Toc>
      </div>
    )
  }
}


ReactDOM.render(
  <App />,
  document.getElementById("app"));