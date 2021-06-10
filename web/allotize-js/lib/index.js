"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.connect = exports.throttle = exports.debounce = exports.Crate = exports.unsubscribe = exports.subscribe = exports.remove = exports.beginsWith = exports.getRange = exports.getAll = exports.metadata = exports.allotize = exports.username = exports.upperCase = void 0;
const allotize_core_1 = require("allotize-core");
const nanoid_1 = require("nanoid");
const upperCase = (str) => {
    return str.toUpperCase();
};
exports.upperCase = upperCase;
exports.username = localStorage.getItem('username') || nanoid_1.nanoid(12);
localStorage.setItem('username', exports.username);
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
        if (typeof value === 'string') {
            let v = JSON.parse(value);
            return {
                key: key,
                clock: v.clock,
                value: v.data ? JSON.parse(v.data) : null
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
exports.getAll = getAll;
async function getRange(start, end) {
    let app = await exports.allotize;
    let all = await app.tx().getRange(start, end);
    return Object.entries(all).map(([key, value]) => {
        if (typeof value === 'string') {
            let v = JSON.parse(value);
            return {
                key: key,
                clock: v.clock,
                value: v.data ? JSON.parse(v.data) : null
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
        if (typeof value === 'string') {
            let v = JSON.parse(value);
            return {
                key: key,
                clock: v.clock,
                value: v.data ? JSON.parse(v.data) : null
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
exports.beginsWith = beginsWith;
async function remove(key) {
    let app = await exports.allotize;
    return await app.tx().remove(key);
}
exports.remove = remove;
function subscribe(key, callback) {
    let app = async () => await exports.allotize;
    let cb = (e) => { callback(JSON.parse(e.detail.data).state); };
    app().then(app => app.subscribe(key, cb));
    return [key, cb];
}
exports.subscribe = subscribe;
function unsubscribe(key, callback) {
    let app = async () => await exports.allotize;
    let cb = (e) => { callback(JSON.parse(e.detail.data).state); };
    app().then(app => app.unsubscribe(key, cb));
}
exports.unsubscribe = unsubscribe;
function Crate(crate) {
    connect(crate);
    return crate;
}
exports.Crate = Crate;
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
            }, limit);
            return res;
        }
        else {
            applyLast = () => func.apply(this, args);
        }
        return true;
    };
}
exports.throttle = throttle;
async function connect(crate) {
    let app = await exports.allotize;
    const sync = throttle(function (route, data, persist) {
        if (persist == null || persist) {
            console.log("crdt_put", route, data);
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
                crate.onChange ? crate.onChange(oldData, newData) : '';
            }
            catch (err) { }
            try {
                crate.onLocalChange ? crate.onLocalChange(oldData, newData) : '';
            }
            catch (err) { }
            try {
                crate.onChangeCallbacks ? crate.onChangeCallbacks.forEach((cb) => cb(oldData, newData)) : '';
            }
            catch (err) { }
            sync(crate.route, newData, crate.persist);
            return status;
        },
    };
    const onRemoteChange = (event) => {
        console.log("NOOOOOH");
        let newData = event.detail ? JSON.parse(event.detail.data) : {};
        let oldData = Object.assign({}, crate.rawData);
        Object.assign(crate.rawData, newData);
        try {
            crate.onChange ? crate.onChange(oldData, newData) : '';
        }
        catch (err) { }
        try {
            crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : '';
        }
        catch (err) { }
        try {
            crate.onChangeCallbacks ? crate.onChangeCallbacks.forEach((cb) => cb(oldData, newData)) : '';
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
                crate.onChange ? crate.onChange(oldData, newData) : '';
            }
            catch (err) { }
            try {
                crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : '';
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
                crate.onChange ? crate.onChange(oldData, newData) : '';
            }
            catch (err) { }
            try {
                crate.onRemoteChange ? crate.onRemoteChange(oldData, newData) : '';
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