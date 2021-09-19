"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.connect = exports.throttle = exports.debounce = exports.Data = exports.unsubscribe = exports.subscribe = exports.remove = exports.beginsWith = exports.getRange = exports.getAll = exports.metadata = exports.allotize = exports.username = exports.upperCase = exports.BoundedChannel = exports.WatchChannel = void 0;
const allotize_core_1 = require("allotize-core");
const nanoid_1 = require("nanoid");
class WatchChannel {
    constructor(route) {
        this.route = route;
    }
    send(msg) {
        this.data.msg = msg;
    }
    read(msg) {
        return this.data.msg;
    }
    async connect() {
        let app = await exports.allotize;
        let channel = this;
        const sync = throttle(function (route, data) {
            app.tx().share(route, { "data": JSON.stringify(data) });
        }, channel.throttleInterval || 350);
        const handler = {
            get: function (obj, prop, receiver) {
                return Reflect.get(obj, prop, receiver);
            },
            set: function (newData, prop, value, receiver) {
                let status = Reflect.set(newData, prop, value, receiver);
                try {
                    channel.onMsg ? channel.onMsg(newData) : '';
                }
                catch (err) { }
                try {
                    channel.onLocalMsg ? channel.onLocalMsg(newData) : '';
                }
                catch (err) { }
                try {
                    channel.onMsgCallbacks ? channel.onMsgCallbacks.forEach((cb) => cb(newData)) : '';
                }
                catch (err) { }
                sync(channel.route, newData);
                return status;
            },
        };
        const onRemoteMsg = (event) => {
            let newData = event.detail.data ? JSON.parse(event.detail.data) : {};
            try {
                channel.onMsg ? channel.onMsg(newData) : '';
            }
            catch (err) { }
            try {
                channel.onRemoteMsg ? channel.onRemoteMsg(newData) : '';
            }
            catch (err) { }
            try {
                channel.onMsgCallbacks ? channel.onMsgCallbacks.forEach((cb) => cb(newData)) : '';
            }
            catch (err) { }
        };
        // Creates a proxy
        let proxy = app.connect(channel.route, {}, handler, onRemoteMsg);
        channel.data = proxy;
    }
}
exports.WatchChannel = WatchChannel;
class BoundedChannel {
    constructor(route, bound) {
        this.route = route;
        this.bound = bound;
        this.received = [];
    }
    send(msg) {
        this.data.msg = msg;
    }
    read() {
        return this.data.msg;
    }
    readAll() {
        return this.received;
    }
    async connect() {
        let app = await exports.allotize;
        let channel = this;
        const handler = {
            get: function (obj, prop, receiver) {
                return Reflect.get(obj, prop, receiver);
            },
            set: function (newData, prop, value, receiver) {
                let status = Reflect.set(newData, prop, value, receiver);
                try {
                    channel.onMsg ? channel.onMsg(newData) : '';
                }
                catch (err) { }
                try {
                    channel.onLocalMsg ? channel.onLocalMsg(newData) : '';
                }
                catch (err) { }
                try {
                    channel.onMsgCallbacks ? channel.onMsgCallbacks.forEach((cb) => cb(newData)) : '';
                }
                catch (err) { }
                app.tx().share(channel.route, { "data": JSON.stringify(newData) });
                return status;
            },
        };
        const onRemoteMsg = (event) => {
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
            }
            catch (err) { }
            // try {
            //   channel.onMsgCallbacks ? channel.onMsgCallbacks.forEach((cb) => cb(newData)) : '';
            // } catch (err) { }
        };
        // Creates a proxy
        let proxy = app.connect(channel.route, {}, handler, onRemoteMsg);
        channel.data = proxy;
    }
}
exports.BoundedChannel = BoundedChannel;
const upperCase = (str) => {
    return str.toUpperCase();
};
exports.upperCase = upperCase;
exports.username = localStorage.getItem("username") || (0, nanoid_1.nanoid)(12);
localStorage.setItem("username", exports.username);
exports.allotize = new allotize_core_1.App(exports.username, true);
async function metadata() {
    let app = await exports.allotize;
    let metadata = await app.metadata();
    return metadata;
}
exports.metadata = metadata;
async function getAll() {
    let app = await exports.allotize;
    let all = await app.tx().beginsWith("");
    return all.map(([key, value]) => {
        if (typeof value === "string") {
            let v = JSON.parse(value);
            return {
                key: key,
                clock: v.clock,
                value: v.data ? JSON.parse(v.data) : null,
            };
        }
        else {
            return {
                key: key,
                value: value,
            };
        }
    });
}
exports.getAll = getAll;
async function getRange(start, end) {
    let app = await exports.allotize;
    let all = await app.tx().getRange(start, end);
    return Object.entries(all).map(([key, value]) => {
        if (typeof value === "string") {
            let v = JSON.parse(value);
            return {
                key: key,
                clock: v.clock,
                value: v.data ? JSON.parse(v.data) : null,
            };
        }
        else {
            return {
                key: key,
                value: value
            };
        }
    });
}
exports.getRange = getRange;
async function beginsWith(prefix) {
    let app = await exports.allotize;
    let all = await app.tx().beginsWith(prefix);
    return all.map(([key, value]) => {
        if (typeof value === "string") {
            let v = JSON.parse(value);
            return {
                key: key,
                clock: v.clock,
                value: v.data ? JSON.parse(v.data) : null,
            };
        }
        else {
            return {
                key: key,
                value: value,
            };
        }
    });
}
exports.beginsWith = beginsWith;
async function remove(key) {
    let app = await exports.allotize;
    return await app.tx().remove(key);
}
exports.remove = remove;
function subscribe(key, callback) {
    let app = async () => await exports.allotize;
    let cb = (e) => {
        callback(JSON.parse(e.detail.data).state);
    };
    app().then((app) => app.subscribe(key, cb));
    return [key, cb];
}
exports.subscribe = subscribe;
function unsubscribe(key, callback) {
    let app = async () => await exports.allotize;
    let cb = (e) => {
        callback(JSON.parse(e.detail.data).state);
    };
    app().then((app) => app.unsubscribe(key, cb));
}
exports.unsubscribe = unsubscribe;
function Data(data) {
    connect(data);
    return data;
}
exports.Data = Data;
function debounce(func, ms = 350) {
    let timeout;
    return function (...args) {
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(this, args), ms);
    };
}
exports.debounce = debounce;
function throttle(func, limit = 350) {
    let inThrottle;
    let applyLast = () => { };
    return function (...args) {
        if (!inThrottle) {
            const res = func.apply(this, args);
            inThrottle = true;
            setTimeout(() => {
                inThrottle = false;
                applyLast();
                applyLast = () => { };
            }, limit);
            return res;
        }
        else {
            applyLast = () => func.apply(this, args);
        }
    };
}
exports.throttle = throttle;
async function connect(crate) {
    let app = await exports.allotize;
    const sync = throttle(function (route, data, persist) {
        if (persist == null || persist) {
            app.tx().crdtPut(route, JSON.stringify(data));
        }
        else {
            app.tx().share(route, data);
        }
    }, crate.throttleInterval || 350);
    const handler = {
        get: function (obj, prop, receiver) {
            return Reflect.get(obj, prop, receiver);
        },
        set: function (newData, prop, value, receiver) {
            let oldData = Object.assign({}, newData);
            let status = Reflect.set(newData, prop, value, receiver);
            try {
                crate.onChange ? crate.onChange(oldData, newData) : "";
            }
            catch (err) { }
            try {
                crate.onLocalChange ? crate.onLocalChange(oldData, newData) : "";
            }
            catch (err) { }
            try {
                crate.onChangeCallbacks
                    ? crate.onChangeCallbacks.forEach((cb) => cb(oldData, newData))
                    : "";
            }
            catch (err) { }
            sync(crate.route, newData, crate.persist);
            return status;
        },
    };
    const onRemoteChange = (event) => {
        let newData = event.detail ? JSON.parse(event.detail.data) : {};
        let oldData = Object.assign({}, crate.rawData);
        Object.assign(crate.rawData, newData);
        try {
            crate.onChange ? crate.onChange(oldData, newData) : "";
        }
        catch (err) { }
        try {
            crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : "";
        }
        catch (err) { }
        try {
            crate.onChangeCallbacks
                ? crate.onChangeCallbacks.forEach((cb) => cb(oldData, newData))
                : "";
        }
        catch (err) { }
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
            .then((answer) => {
            let newData = answer ? JSON.parse(answer) : {};
            let oldData = Object.assign({}, crate.rawData);
            Object.assign(crate.rawData, newData);
            try {
                crate.onChange ? crate.onChange(oldData, newData) : "";
            }
            catch (err) { }
            try {
                crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : "";
            }
            catch (err) { }
        }, (error) => { });
        app
            .tx()
            .crdtGet(crate.route)
            .then((answer) => {
            let newData = answer ? JSON.parse(answer) : {};
            let oldData = Object.assign({}, crate.rawData);
            Object.assign(crate.rawData, newData);
            try {
                crate.onChange ? crate.onChange(oldData, newData) : "";
            }
            catch (err) { }
            try {
                crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : "";
            }
            catch (err) { }
        }, (error) => { });
    }
    else {
        app
            .tx()
            .share(crate.route, crate.rawData)
            .then((answer) => { }, (error) => { });
    }
}
exports.connect = connect;
//# sourceMappingURL=index.js.map