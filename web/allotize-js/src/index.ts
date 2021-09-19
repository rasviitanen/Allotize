import { App } from "allotize-core";
import { nanoid } from "nanoid";

interface AllotizeData {
  route: string;
  onChange?: (arg0: any, arg1: any) => void;
  onLocalChange?: (arg0: any, arg1: any) => void;
  onRemoteChange?: (arg0: any, arg1: any) => void;
  onChangeCallbacks?: ((oldData: any, newData: any) => void)[];
  persist?: boolean;
  data: any;
  throttleInterval?: number;
  rawData?: any;
}

interface Channel {
  route: string;
  onMsg?: (arg0: any, arg1: any) => void;
  onLocalMsg?: (arg0: any, arg1: any) => void;
  onRemoteMsg?: (arg0: any, arg1: any) => void;
  onMsgCallbacks?: ((oldData: any, newData: any) => void)[];
  data?: any;

  throttleInterval?: number;

  send(msg: any): void;
}

export class WatchChannel implements Channel {
  onMsg?: (arg0: any) => void;
  onLocalMsg?: (arg0: any) => void;
  onRemoteMsg?: (arg0: any) => void;
  onMsgCallbacks?: ((newData: any) => void)[];
  data?: any;

  throttleInterval?: number;

  constructor(public route: string) { }

  send(msg: any) {
    this.data.msg = msg;
  }

  read(msg: any) {
    return this.data.msg;
  }

  async connect() {
    let app = await allotize;
    let channel = this;

    const sync = throttle(function (route: string, data: any) {
      app.tx().share(route, { "data": JSON.stringify(data) });
    }, channel.throttleInterval || 350);

    const handler = {
      get: function (obj: any, prop: PropertyKey, receiver?: any) {
        return Reflect.get(obj, prop, receiver);
      },
      set: function (newData: any, prop: PropertyKey, value: any, receiver?: any) {
        let status = Reflect.set(newData, prop, value, receiver);
        try {
          channel.onMsg ? channel.onMsg(newData) : '';
        } catch (err) { }
        try {
          channel.onLocalMsg ? channel.onLocalMsg(newData) : '';
        } catch (err) { }
        try {
          channel.onMsgCallbacks ? channel.onMsgCallbacks.forEach((cb) => cb(newData)) : '';
        } catch (err) { }

        sync(channel.route, newData);

        return status;
      },
    };

    const onRemoteMsg = (event: any) => {
      let newData = event.detail.data ? JSON.parse(event.detail.data) : {};
      try {
        channel.onMsg ? channel.onMsg(newData) : '';
      } catch (err) { }
      try {
        channel.onRemoteMsg ? channel.onRemoteMsg(newData) : '';
      } catch (err) { }
      try {
        channel.onMsgCallbacks ? channel.onMsgCallbacks.forEach((cb) => cb(newData)) : '';
      } catch (err) { }
    };

    // Creates a proxy
    let proxy = app.connect(
      channel.route,
      {},
      handler,
      onRemoteMsg
    );
    channel.data = proxy;
  }
}

export class BoundedChannel implements Channel {
  onMsg?: (arg0: any) => void;
  onLocalMsg?: (arg0: any) => void;
  onRemoteMsg?: (arg0: any) => void;
  onMsgCallbacks?: ((newData: any) => void)[];
  data?: any;
  received: any[];

  throttleInterval?: number;

  constructor(public route: string, public bound: number) {
    this.received = []
  }

  send(msg: any) {
    this.data.msg = msg;
  }

  read() {
    return this.data.msg;
  }

  readAll() {
    return this.received;
  }

  async connect() {
    let app = await allotize;
    let channel = this;


    const handler = {
      get: function (obj: any, prop: PropertyKey, receiver?: any) {
        return Reflect.get(obj, prop, receiver);
      },
      set: function (newData: any, prop: PropertyKey, value: any, receiver?: any) {
        let status = Reflect.set(newData, prop, value, receiver);

        try {
          channel.onMsg ? channel.onMsg(newData) : '';
        } catch (err) { }
        try {
          channel.onLocalMsg ? channel.onLocalMsg(newData) : '';
        } catch (err) { }
        try {
          channel.onMsgCallbacks ? channel.onMsgCallbacks.forEach((cb) => cb(newData)) : '';
        } catch (err) { }

        app.tx().share(channel.route, { "data": JSON.stringify(newData) })

        return status;
      },
    };

    const onRemoteMsg = (event: any) => {
      let newData = event.detail.data ? JSON.parse(event.detail.data) : {};

      if (this.received.length >= this.bound) {
        this.received.shift();
      }
      this.received.push(newData);

      // try {
      //   channel.onMsg ? channel.onMsg(newData) : '';
      // } catch (err) { }
      try {
        channel.onRemoteMsg ? channel.onRemoteMsg(newData) : '';
      } catch (err) { }
      // try {
      //   channel.onMsgCallbacks ? channel.onMsgCallbacks.forEach((cb) => cb(newData)) : '';
      // } catch (err) { }
    };

    // Creates a proxy
    let proxy = app.connect(
      channel.route,
      {},
      handler,
      onRemoteMsg
    );
    channel.data = proxy;
  }
}

interface VersionedComponent {
  clock: any;
  data?: any;
}

interface Entry {
  key: string;
  value: VersionedComponent;
}

export const upperCase = (str: string): string => {
  return str.toUpperCase();
};

export const username: string = localStorage.getItem("username") || nanoid(12);
localStorage.setItem("username", username);

export const allotize: App = new App(username, true);

export async function metadata() {
  let app = await allotize;
  let metadata = await app.metadata();
  return metadata;
}

export async function getAll() {
  let app = await allotize;
  let all: string[][] = await app.tx().beginsWith("");
  return all.map(([key, value]) => {
    if (typeof value === "string") {
      let v = JSON.parse(value);
      return {
        key: key,
        clock: v.clock,
        value: v.data ? JSON.parse(v.data) : null,
      };
    } else {
      return {
        key: key,
        value: value,
      };
    }
  });
}

export async function getRange(start: string, end?: string) {
  let app = await allotize;
  let all = await app.tx().getRange(start, end);
  return Object.entries(all).map(([key, value]) => {
    if (typeof value === "string") {
      let v = JSON.parse(value);
      return {
        key: key,
        clock: v.clock,
        value: v.data ? JSON.parse(v.data) : null,
      };
    } else {
      return {
        key: key,
        value: value
      }
    }
  });
}

export async function beginsWith(prefix: string) {
  let app = await allotize;
  let all: string[][] = await app.tx().beginsWith(prefix);
  return all.map(([key, value]) => {
    if (typeof value === "string") {
      let v = JSON.parse(value);
      return {
        key: key,
        clock: v.clock,
        value: v.data ? JSON.parse(v.data) : null,
      };
    } else {
      return {
        key: key,
        value: value,
      };
    }
  });
}

export async function remove(key: string) {
  let app = await allotize;
  return await app.tx().remove(key);
}

export function subscribe(key: string, callback: (arg0: any) => void) {
  let app = async () => await allotize;
  let cb = (e: any) => {
    callback(JSON.parse(e.detail.data).state);
  };
  app().then((app) => app.subscribe(key, cb));

  return [key, cb];
}

export function unsubscribe(key: string, callback: (arg0: any) => void) {
  let app = async () => await allotize;
  let cb = (e: any) => {
    callback(JSON.parse(e.detail.data).state);
  };
  app().then((app) => app.unsubscribe(key, cb));
}

export function Data(data: AllotizeData) {
  connect(data);
  return data;
}

export function debounce(func: Function, ms = 350) {
  let timeout: ReturnType<typeof setTimeout>;
  return function (this: any, ...args: any[]) {
    clearTimeout(timeout);
    timeout = setTimeout(() => func.apply(this, args), ms);
  };
}

export function throttle(func: Function, limit = 350) {
  let inThrottle: boolean;
  let applyLast = () => { };
  return function (this: any, ...args: any[]) {
    if (!inThrottle) {
      const res = func.apply(this, args);
      inThrottle = true;
      setTimeout(() => {
        inThrottle = false;
        applyLast();
        applyLast = () => { };
      }, limit);
      return res;
    } else {
      applyLast = () => func.apply(this, args);
    }
  };
}

export async function connect(crate: AllotizeData) {
  let app = await allotize;

  const sync = throttle(function (route: string, data: any, persist: boolean) {
    if (persist == null || persist) {
      app.tx().crdtPut(route, JSON.stringify(data));
    } else {
      app.tx().share(route, data);
    }
  }, crate.throttleInterval || 350);

  const handler = {
    get: function (obj: any, prop: PropertyKey, receiver?: any) {
      return Reflect.get(obj, prop, receiver);
    },
    set: function (
      newData: any,
      prop: PropertyKey,
      value: any,
      receiver?: any
    ) {
      let oldData = { ...newData };
      let status = Reflect.set(newData, prop, value, receiver);
      try {
        crate.onChange ? crate.onChange(oldData, newData) : "";
      } catch (err) { }
      try {
        crate.onLocalChange ? crate.onLocalChange(oldData, newData) : "";
      } catch (err) { }
      try {
        crate.onChangeCallbacks
          ? crate.onChangeCallbacks.forEach((cb) => cb(oldData, newData))
          : "";
      } catch (err) { }

      sync(crate.route, newData, crate.persist);

      return status;
    },
  };

  const onRemoteChange = (event: any) => {
    let newData = event.detail ? JSON.parse(event.detail.data) : {};
    let oldData = { ...crate.rawData };
    Object.assign(crate.rawData, newData);
    try {
      crate.onChange ? crate.onChange(oldData, newData) : "";
    } catch (err) { }
    try {
      crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : "";
    } catch (err) { }
    try {
      crate.onChangeCallbacks
        ? crate.onChangeCallbacks.forEach((cb) => cb(oldData, newData))
        : "";
    } catch (err) { }
  };

  // Creates a proxy
  crate.rawData = crate.data;
  let proxy = app.connect(crate.route, crate.data, handler, onRemoteChange);
  crate.data = proxy;

  // Does not block to wait for a connection
  // i.e. this is faster and works if we are browsing alone/offline
  // Only persist if crate.persist != false
  if (crate.persist == null || crate.persist) {
    // Blocks until we get a connection
    app
      .tx()
      .syncWithPeers(crate.route)
      .then(
        (answer) => {
          let newData = answer ? JSON.parse(answer) : {};
          let oldData = { ...crate.rawData };
          Object.assign(crate.rawData, newData);
          try {
            crate.onChange ? crate.onChange(oldData, newData) : "";
          } catch (err) { }
          try {
            crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : "";
          } catch (err) { }
        },
        (error) => { }
      );
    app
      .tx()
      .crdtGet(crate.route)
      .then(
        (answer) => {
          let newData = answer ? JSON.parse(answer) : {};
          let oldData = { ...crate.rawData };
          Object.assign(crate.rawData, newData);
          try {
            crate.onChange ? crate.onChange(oldData, newData) : "";
          } catch (err) { }
          try {
            crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : "";
          } catch (err) { }
        },
        (error) => { }
      );
  } else {
    app
      .tx()
      .share(crate.route, crate.rawData)
      .then(
        (answer) => { },
        (error) => { }
      );
  }
}
