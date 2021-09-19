# Quick start
This tutorial helps you quick start an application.

## Data

```javascript
import { Allotize } from "allotize-js"

const user = Allotize.Data({
    route: "user/bob",
    data: {
        name: "Bob",
        posts: 0
    }
});
```

## Channel

```javascript
import { Allotize } from "allotize-js"

const chat = Allotize.BoundedChannel({
    route: "chat",
    size: 25 
});
```
