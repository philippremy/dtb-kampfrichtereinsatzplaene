import React from "react";
import { FrontendStorage } from "./Editor.tsx";
import { Body1Stronger, Button, Caption1, Card, CardFooter, CardHeader, Field, Input } from "@fluentui/react-components";
import { AddFilled, DeleteFilled, DeleteRegular } from "@fluentui/react-icons";
import "./ReplacementJudges.css";

export default class ReplacementJudges extends React.Component<{storage: FrontendStorage, setStorage: React.Dispatch<React.SetStateAction<FrontendStorage>>, hidden: boolean, setHidden: React.Dispatch<React.SetStateAction<boolean>>}> {

    constructor(props: {storage: FrontendStorage, setStorage: React.Dispatch<React.SetStateAction<FrontendStorage>>, hidden: boolean, setHidden: React.Dispatch<React.SetStateAction<boolean>>}) {
        super(props);
    }

    addReplacementJudge() {
        let temp_storage = this.props.storage;
        temp_storage.wk_replacement_judges?.push("");
        temp_storage.changedByDoubleHook = false;
        this.props.setStorage(Object.assign({}, temp_storage));
    }

    removeAllReplacementJudges() {
        let temp_storage = this.props.storage;
        temp_storage.wk_replacement_judges = [];
        temp_storage.changedByDoubleHook = false;
        this.props.setHidden(false);
        this.props.setStorage(Object.assign({}, temp_storage));
    }

    reactToInputChange(element: React.FormEvent<HTMLInputElement>) {
        let oldValue = element.currentTarget.ariaLabel;
        if(oldValue !== null) {
            let temp_storage = this.props.storage;
            for(let i=0; i < (temp_storage.wk_replacement_judges ? temp_storage.wk_replacement_judges.length : 0); i++) {
                if(temp_storage.wk_replacement_judges !== undefined) {
                    if(temp_storage.wk_replacement_judges[i] === oldValue) {
                        temp_storage.wk_replacement_judges[i] = element.currentTarget.value;
                        temp_storage.changedByDoubleHook = false;
                        element.currentTarget.ariaLabel = element.currentTarget.value;
                    }
                }
            }
            this.props.setStorage(Object.assign({}, temp_storage));
        }
    }

    removeCurrent(element: React.MouseEvent<HTMLButtonElement, MouseEvent>) {
        let value = element.currentTarget.ariaLabel;
        if(value !== null) {
            let temp_storage = this.props.storage;
            for(let i=0; i < (temp_storage.wk_replacement_judges ? temp_storage.wk_replacement_judges.length : 0); i++) {
                if(temp_storage.wk_replacement_judges !== undefined) {
                    if(temp_storage.wk_replacement_judges[i] === value) {
                        temp_storage.wk_replacement_judges.splice(i, 1);
                        if(temp_storage.wk_replacement_judges.length === 0) {
                            this.props.setHidden(false);
                        }
                        temp_storage.changedByDoubleHook = false;
                    }
                }
            }
            this.props.setStorage(Object.assign({}, temp_storage));
        }
    }

    render() {

        if(this.props.hidden) {
            return(
                <></>
            )
        } else {

            if(this.props.storage.wk_replacement_judges === undefined || this.props.storage.wk_replacement_judges.length === 0) {
                let temp_storage = this.props.storage;
                temp_storage.wk_replacement_judges?.push("");
                temp_storage.changedByDoubleHook = false;
                this.props.setStorage(Object.assign({}, temp_storage));
            }

            // TODO für Irgendwann. Hover einbauen, braucht wahrscheinlich nen State ^^
            const buttonAppearance: React.CSSProperties = {
                backgroundColor: "#ffffff",
                color: "#d13438",
            }

            // Keep track of how many replacement judges we have!
            let noOfReplacementJudges = 0;

            return(
                <div id={"headContainer"}>
                    <Card className="cardType">
                        <CardHeader header={
                            <div className="cardHeaderDiv">
                                <div className="tableInfoContainer">
                                    <Body1Stronger>{"Ersatzkampfrichter"}</Body1Stronger>
                                    <Caption1>{""}</Caption1>
                                </div>
                                <Button style={buttonAppearance} appearance="transparent" icon={<DeleteFilled />} onClick={() => {this.removeAllReplacementJudges()}}></Button>
                            </div>
                        } />
                        <div className={"fieldContainerReplacementJudges"}>
                            {
                                this.props.storage.wk_replacement_judges!.map((name) =>
                                    {
                                        noOfReplacementJudges++;
                                        return(
                                            <Field>
                                                <Input aria-label={name} contentBefore={noOfReplacementJudges.toString() + "."} defaultValue={name} className="inputTypeReplacement" onInput={(element) => {this.reactToInputChange(element)}} contentAfter={
                                                    <Button aria-label={name} icon={<DeleteRegular />} appearance={"transparent"} size={"small"} onClick={(element) => {this.removeCurrent(element)}} autoCapitalize={"off"} autoCorrect={"off"} />
                                                }></Input>
                                            </Field>
                                        )
                                    }
                                )
                            }
                        </div>
                    <CardFooter>
                        <Button appearance="primary" icon={<AddFilled />} size={"small"} onClick={() => {this.addReplacementJudge()}} />
                    </CardFooter>
                    </Card>
                </div>
            )
        }
    }
}