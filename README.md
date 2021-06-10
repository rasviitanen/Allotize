<div align="center">
  <img align="center" height="64px" src="https://allotize.com/img/logo.svg">
  <br></br>
  
  <strong>Allotize is a platform for building web apps without
  traditional server infrastructure.</strong>
  <br></br>

  <p>
  The purpose of this project is to enable frictionless creation of web applications.
  For example, you are able to build collaboration services such as a text editor without requiring servers or back-end code.
  </p>

  Allotize runs natively in typical @edge hosted environments to keep latencies to a minimum
  It's possible to let users act as host nodes; keeping server costs down.

  [Web Page][allotize-page] | [Demo][allotize-demo] | [Book][allotize-tutorial]

  <sub>Built with 🦀🕸 by the Allotize Team</sub>
</div>

## About
Allotize is an attempt of building a system for distributing web apps without traditional server infrastructure
The project is composed of two major modules, a database and a P2P networking solution.

The database that is optimized to run in edge environments
and replicates freely between nodes using conflict-free strategies.

To make development as frictionless as possible, the system handles
JS-values directly. We call this principle `remote code`.
This enables you do to things such as: `votes += 1`, but where
mutations propagates to all connected users instead of just locally.
*i.e. you write traditional JavaScript, but operations can take place
remotely if you connect your variables/objects/items to Allotize*

[**📚 Read the tutorial! 📚**][allotize-tutorial]

This tutorial is designed for kickstarting your first Allotize application.

[allotize-page]: https://allotize.com
[allotize-demo]: https://app.allotize.com
[allotize-tutorial]: https://docs.allotize.com

## Demo
Here is an example of Allotize in action! We leverage the fact
that Allotize can run with Users as nodes. So this could be deployed
as a static file, yet allow real-time changes.
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
