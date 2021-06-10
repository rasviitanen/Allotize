import { useState, useEffect } from "react";
import { Allotize } from "allotize";

const kretsloopCrates = {};

export function useAllotizeState(crate) {
    if (kretsloopCrates[crate.route] == null) {
        kretsloopCrates[crate.route] = Allotize.Crate({
            route: crate.route,
            ...crate.config,
            data: {
                state: crate.state
            },
        });
    }
    const [kretsloopCrate, setKretsloopCrate] = useState(kretsloopCrates[crate.route]);

    kretsloopCrate.onRemoteChange = function (oldData, newData) {
        console.log(newData);
        setKretsloopCrate({...kretsloopCrate});
    };

    const setStateProxy = function(state) {
        kretsloopCrate.data.state = state;
        setKretsloopCrate({...kretsloopCrate});
    }

    return [kretsloopCrate.data.state, setStateProxy];
}