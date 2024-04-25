import KampfgerichtElement from "./Kampfgericht";
import React from "react";
import { FrontendStorage } from "./Editor";
import { Body2 } from "@fluentui/react-components";
import { QuestionCircle32Filled } from "@fluentui/react-icons";

class KampfgerichteRenderer extends React.Component<{storage: FrontendStorage, setStorage: React.Dispatch<React.SetStateAction<FrontendStorage>>}> {

    constructor(props: {storage: FrontendStorage, setStorage: React.Dispatch<React.SetStateAction<FrontendStorage>>}) {
        super(props);
    }

    createMap() {
        return Array.from(this.props.storage.wk_judgingtables!, (val) => val[1]);
    }

    render() {
        
        if(this.props.storage.wk_judgingtables === undefined || this.props.storage.wk_judgingtables === null) {
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