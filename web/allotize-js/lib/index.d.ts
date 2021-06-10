import { App } from "allotize-core";
interface AllotizeCrate {
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
export declare function Crate(crate: AllotizeCrate): AllotizeCrate;
export declare function debounce(func: Function, ms?: number): (this: any, ...args: any[]) => void;
export declare function throttle(func: Function, limit?: number): (this: any, ...args: any[]) => any;
export declare function connect(crate: AllotizeCrate): Promise<void>;
export {};
//# sourceMappingURL=index.d.ts.map