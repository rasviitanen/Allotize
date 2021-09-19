import { App } from "allotize-core";
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
export declare class WatchChannel implements Channel {
    route: string;
    onMsg?: (arg0: any) => void;
    onLocalMsg?: (arg0: any) => void;
    onRemoteMsg?: (arg0: any) => void;
    onMsgCallbacks?: ((newData: any) => void)[];
    data?: any;
    throttleInterval?: number;
    constructor(route: string);
    send(msg: any): void;
    read(msg: any): any;
    connect(): Promise<void>;
}
export declare class BoundedChannel implements Channel {
    route: string;
    bound: number;
    onMsg?: (arg0: any) => void;
    onLocalMsg?: (arg0: any) => void;
    onRemoteMsg?: (arg0: any) => void;
    onMsgCallbacks?: ((newData: any) => void)[];
    data?: any;
    received: any[];
    throttleInterval?: number;
    constructor(route: string, bound: number);
    send(msg: any): void;
    read(): any;
    readAll(): any[];
    connect(): Promise<void>;
}
export declare const upperCase: (str: string) => string;
export declare const username: string;
export declare const allotize: App;
export declare function metadata(): Promise<any>;
export declare function getAll(): Promise<({
    key: string;
    clock: any;
    value: any;
} | {
    key: string;
    value: never;
    clock?: undefined;
})[]>;
export declare function getRange(start: string, end?: string): Promise<({
    key: string;
    clock: any;
    value: any;
} | {
    key: string;
    value: unknown;
    clock?: undefined;
})[]>;
export declare function beginsWith(prefix: string): Promise<({
    key: string;
    clock: any;
    value: any;
} | {
    key: string;
    value: never;
    clock?: undefined;
})[]>;
export declare function remove(key: string): Promise<any>;
export declare function subscribe(key: string, callback: (arg0: any) => void): (string | ((e: any) => void))[];
export declare function unsubscribe(key: string, callback: (arg0: any) => void): void;
export declare function Data(data: AllotizeData): AllotizeData;
export declare function debounce(func: Function, ms?: number): (this: any, ...args: any[]) => void;
export declare function throttle(func: Function, limit?: number): (this: any, ...args: any[]) => any;
export declare function connect(crate: AllotizeData): Promise<void>;
export {};
//# sourceMappingURL=index.d.ts.map