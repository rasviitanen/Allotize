<div style="display: flex; align-items: center; flex-direction: row; border-bottom: 2px solid #e3e3e3; padding: 1%; justify-content: space-between">
<img width="25%" src="./assets/logo.svg"></img>
<div>
    <a style="text-decoration: none; margin: 10px" href="https://allotize.com">Page</a>
    <a style="text-decoration: none; margin: 10px" href="https://www.npmjs.com/package/allotize-js">NPM</a>
</div>
</div>

> 🎉🎊 Thank you for checking out Allotize.

# Introduction
This is the docs for Allotize - A Platform for building collaborative web apps.
In it you will find a tutorial that allows you to kickstart your new project.

## What is Allotize?
Allotize provides a platform to build modern web applications without backend code, servers or even databases.
Yet, you are able to build real-time collaborative apps with dynamic content.

Think of Allotize as something that gives your traditional web app superpowers and allows data to teleport and sync between devices!

## How it works
You connect variables, objects, or any other JavaScript code to Allotize.
Your users will then be able to detect updates and changes on each others' machines.
The state is managed in an offline-first approach and typical data conflicts that
happen in distributed systems are handled automatically by Allotize.

<div align="center">
<img align="center" src="./assets/cubea.gif" >
</div>

Let's take a look at the upvoting code.
We simply create an object `votes` that holds our information and register it to Allotize via `Allotize.Crate(votes)`.

```JavaScript
const votes = Allotize.Crate({
    route: "cube/votes",

    data: {
        upvotes: 0,
    },

    onChange: (oldData, newData) => {
        document.getElementById("upvotes").innerHTML = newData.upvotes;
    },
});

document.getElementById("upvote").addEventListener('click', (e) => {
    votes.data.upvotes += 1;
});

document.getElementById("downvote").addEventListener('click', (e) => {
    votes.data.upvotes -= 1;
});
```

That's it! Everyone who loads our website will have a synced number of upvotes that changes in real-time.
Proceed to the next chapter to find out how you can create your own app!
