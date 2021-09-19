<div align="center">
  <img align="center" height="64px" src="https://allotize.com/img/logo.svg">
  <br></br>
  
  <strong>Automatic Infrastructure from Code</strong>
  <br></br>

  <p>
  Recent developments have made Infrastructure as Code very popular, as it's declarative and expressive.
  However, typical Infrastructure as Code still defines your infrastructure in a separate system, which requires you to still set up integrations.
  Allotize insteads attempts to let your application define the infrastrucutre through dependency inference. This allows you to quickly implement complex applications with dynamic content without having to think about infrastructure - only client side code.
  </p>
  
  [Web Page][allotize-page] | [Demo][allotize-demo] | [Book][allotize-tutorial]

<sub>Built with 🦀🕸 by the Allotize Team</sub>

</div>

```JavaScript
import { useAllotize } from "allotize-js"

export function Counter() {
    const [state, setState] = useAllotize({
        route: `store#counter`,
        data: {
            count: 0,
        },
    });

    const increment = () => {
        setState({
            count: state.count + 1,
        });
    };

    return (
        <button onClick={increment} />
    );
}
```

## About

Allotize is a system for creating collaborative and dynamic web apps without traditional server infrastructure.
The project is composed of two major modules, a database and a P2P networking solution.

The database is optimized to run in edge environments
and replicates freely between nodes using conflict-free strategies.

To make development as frictionless as possible, the system handles
JS-values directly and establishes proxies to them.
This enables you do to things such as: `votes += 1`, but where
mutations propagates to all connected users instead of just locally via the proxy.
_i.e. you write regular JavaScript, but operations can replicate
remotely if you connect your variables/objects/items to Allotize_

[**📚 Read the tutorial! 📚**][allotize-tutorial]

This tutorial is designed for kickstarting your first Allotize app.

[allotize-page]: https://allotize.com
[allotize-demo]: https://app.allotize.com
[allotize-tutorial]: https://docs.allotize.com

## Demo

Here is an example of Allotize in action! As users can act as hosts, you could serve this
as a static file, yet allow your app to show real-time changes.
You don't need complex frameworks either, here is an example with regular JavaScript:

<div align="center">
<img align="center" src="https://allotize.com/img/cubea.gif" >
</div>

```JavaScript
const upvotes = document.getElementById("upvotes");
const downvote = document.getElementById("downvote");
const upvote = document.getElementById("upvote");

const votes = Allotize.Data({
    route: "cube/votes",

    data: {
        upvotes: 0,
    },

    onChange: (old, new) => {
        upvotes.innerHTML = new.upvotes;
    },
});

upvote.onclick = () => {
    votes.data.upvotes += 1;
};

downvote.onclick = () => {
    votes.data.upvotes -= 1;
};
```
