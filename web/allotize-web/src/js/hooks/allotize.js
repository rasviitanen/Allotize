import { useState, useEffect } from "react";
import { Crate } from "allotize-js";

const kretsloopCrates = {};

export function useAllotize(crate) {
    if (kretsloopCrates[crate.route] == null) {
        kretsloopCrates[crate.route] = Crate({
            route: crate.route,
            ...crate.config,
            data: {
                state: crate.data
            },
        });
    }
    const [kretsloopCrate, setKretsloopCrate] = useState(kretsloopCrates[crate.route]);

    kretsloopCrate.onRemoteChange = function (oldData, newData) {
        setKretsloopCrate({...kretsloopCrate});
    };

    const setStateProxy = function(state) {
        kretsloopCrate.data.state = state;
        setKretsloopCrate({...kretsloopCrate});
    }

    return [kretsloopCrate.data.state, setStateProxy];
}
