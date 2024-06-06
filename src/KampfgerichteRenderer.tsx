import KampfgerichtElement from "./Kampfgericht";
import React from "react";
import { FrontendStorage, Kampfgericht } from "./Editor";
import { Body2 } from "@fluentui/react-components";
import { QuestionCircle32Filled } from "@fluentui/react-icons";

class KampfgerichteRenderer extends React.Component<{storage: FrontendStorage, setStorage: React.Dispatch<React.SetStateAction<FrontendStorage>>}> {

    constructor(props: {storage: FrontendStorage, setStorage: React.Dispatch<React.SetStateAction<FrontendStorage>>}) {
        super(props);
    }

    createMap() {
        let arr = Array.from(this.props.storage.wk_judgingtables!, (val) => val[1]);
        return this.sortArray(arr);
    }

    // The sorting function (callback)
    sortArrayCallback(elem1: Kampfgericht, elem2: Kampfgericht): number {
        if((elem1.table_is_finale && elem2.table_is_finale) || (!elem1.table_is_finale && !elem2.table_is_finale)) {
            let name1 = elem1.table_name.toUpperCase();
            let name2 = elem2.table_name.toUpperCase();
            if (name1 < name2) {
                return -1;
            }
            if (name2 > name1) {
                return 1;
            }
        } else if(elem1.table_is_finale && !elem2.table_is_finale) {
            return 1;
        } else if(!elem1.table_is_finale && elem2.table_is_finale) {
            return -1;
        }
        return 0;
    }

    // Sort the array before passing it to the map() function
    sortArray(arr: Kampfgericht[]) {
        return arr.sort((elem1, elem2) => this.sortArrayCallback(elem1, elem2));
    }

    render() {
        
        if(this.props.storage.wk_judgingtables === undefined || this.props.storage.wk_judgingtables === null || this.props.storage.wk_judgingtables.size === 0) {
            return(
                <div className="emptyJudgingView">
                    <Body2>Noch keine Kampfgerichte angelegt</Body2>
                    <QuestionCircle32Filled></QuestionCircle32Filled>
                </div>
            )
        } else {
            return(
                <>
                {
                    this.createMap().map(table => (
                        <KampfgerichtElement key={table.uniqueID} storage={this.props.storage} setStorage={this.props.setStorage} uniqueID={table.uniqueID} />
                    ))
                }
                </>
            )
        }

    }
}

export default KampfgerichteRenderer;
