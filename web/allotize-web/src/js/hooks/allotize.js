import { useState, useEffect } from "react";
import { Data } from "allotize-js";

const allotizeDataStore = {};

export function useAllotize(data) {
    if (allotizeDataStore[data.route] == null) {
        allotizeDataStore[data.route] = Data({
            route: data.route,
            ...data.config,
            data: {
                state: data.data
            },
        });
    }
    const [allotizeData, setAllotizeData] = useState(allotizeDataStore[data.route]);

    allotizeData.onRemoteChange = function (oldData, newData) {
        setAllotizeData({ ...allotizeData });
    };

    const setStateProxy = function (state) {
        allotizeData.data.state = state;
        setAllotizeData({ ...allotizeData });
    }

    return [allotizeData.data.state, setStateProxy];
}
