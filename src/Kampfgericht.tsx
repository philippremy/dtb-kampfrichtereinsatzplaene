import React from "react";
import { FrontendStorage, Kampfgericht } from "./Editor";
import "./Kampfgericht.css"
import { Body1Stronger, Button, Caption1, Card, CardFooter, CardHeader, Checkbox, CheckboxOnChangeData, Combobox, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface, DialogTitle, DialogTrigger, Field, Input, Option } from "@fluentui/react-components";
import { CheckmarkFilled, PenFilled, WarningFilled } from "@fluentui/react-icons";

interface StateType {
    ok: boolean | undefined
    sk1: boolean | undefined
    sk2: boolean | undefined
    ak1: boolean | undefined
    ak2: boolean | undefined
    ak3: boolean | undefined
    ak4: boolean | undefined
    aik1: boolean | undefined
    aik2: boolean | undefined
    aik3: boolean | undefined
    aik4: boolean | undefined
    tableName: string | undefined,
    tableDiscipline: string | undefined,
    dialogOpen: boolean,
}

class KampfgerichtElement extends React.Component<{storage: FrontendStorage, setStorage: React.Dispatch<React.SetStateAction<FrontendStorage>>, uniqueID: string}, StateType> {

    constructor(props: {storage: FrontendStorage, setStorage: React.Dispatch<React.SetStateAction<FrontendStorage>>, uniqueID: string}) {
        super(props);
        this.state = {
            ok: undefined,
            sk1: undefined,
            sk2: undefined,
            ak1: undefined,
            ak2: undefined,
            ak3: undefined,
            ak4: undefined,
            aik1: undefined,
            aik2: undefined,
            aik3: undefined,
            aik4: undefined,
            tableName: props.storage.wk_judgingtables?.get(this.props.uniqueID)?.table_name,
            tableDiscipline: props.storage.wk_judgingtables?.get(this.props.uniqueID)?.table_kind,
            dialogOpen: false,
        }
        // We do check here for our data, so why not perform checking if there is a double in our data here?
        for(const table of this.props.storage.wk_judgingtables!) {
            if(table[0] === this.props.uniqueID) {
                this.dataSelf = table[1];
                break;
            }
        }
    }

    updateValues(data: React.FormEvent<HTMLInputElement>) {

        // First, update the own stuff in the global Storage container
        for(const table of this.props.storage.wk_judgingtables!) {
            if(table[0] === this.props.uniqueID) {
                // We will remove a key, if the field is empty. So we can save RAM.
                if(data.currentTarget.value === "" || data.currentTarget.value === null) {
                    table[1].judges.delete(data.currentTarget.id);
                    let temp_storage = this.props.storage;
                    temp_storage.wk_judgingtables?.set(table[0], table[1]);
                    // After that set every empty thing back to undefined
                    switch(data.currentTarget.id) {
                        case "ok":
                            if(this.state.ok !== undefined){this.setState({ok: undefined});}
                            break;
                        case "sk1":
                            if(this.state.sk1 !== undefined){this.setState({sk1: undefined});}
                            break;
                        case "sk2":
                            if(this.state.sk2 !== undefined){this.setState({sk2: undefined});}
                            break;
                        case "ak1":
                            if(this.state.ak1 !== undefined){this.setState({ak1: undefined});}
                            break;
                        case "ak2":
                            if(this.state.ak2 !== undefined){this.setState({ak2: undefined});}
                            break;
                        case "ak3":
                            if(this.state.ak3 !== undefined){this.setState({ak3: undefined});}
                            break;
                        case "ak4":
                            if(this.state.ak4 !== undefined){this.setState({ak4: undefined});}
                            break;
                        case "aik1":
                            if(this.state.aik1 !== undefined){this.setState({aik1: undefined});}
                            break;
                        case "aik2":
                            if(this.state.aik2 !== undefined){this.setState({aik2: undefined});}
                            break;
                        case "aik3":
                            if(this.state.aik3 !== undefined){this.setState({aik3: undefined});}
                            break;
                        case "aik4":
                            if(this.state.aik4 !== undefined){this.setState({aik4: undefined});}
                            break;
                    }
                    // Tell the frontendStorage that the Tables changed it
                    temp_storage.changedByDoubleHook = false;
                    // We have to trick React into thinking that this is another object!
                    this.props.setStorage(Object.assign({}, temp_storage));
                    break;
                } else {
                    table[1].judges.set(data.currentTarget.id, {role: data.currentTarget.id, name: data.currentTarget.value, doubleFound: false});
                    let temp_storage = this.props.storage;
                    temp_storage.wk_judgingtables?.set(table[0], table[1]);
                    // Tell the frontendStorage that the Tables changed it
                    temp_storage.changedByDoubleHook = false;
                    // We have to trick React into thinking that this is another object!
                    this.props.setStorage(Object.assign({}, temp_storage));
                    break;
                }
            }
        }

    }

    setIcon(type: string) {
        switch(type) {
            case "ok":
                if(this.state.ok === undefined) {
                    return undefined;
                } else {
                    return this.state.ok ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "sk1":
                if(this.state.sk1 === undefined) {
                    return undefined;
                } else {
                    return this.state.sk1 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "sk2":
                if(this.state.sk2 === undefined) {
                    return undefined;
                } else {
                    return this.state.sk2 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "ak1":
                if(this.state.ak1 === undefined) {
                    return undefined;
                } else {
                    return this.state.ak1 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "ak2":
                if(this.state.ak2 === undefined) {
                    return undefined;
                } else {
                    return this.state.ak2 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "ak3":
                if(this.state.ak3 === undefined) {
                    return undefined;
                } else {
                    return this.state.ak3 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "ak4":
                if(this.state.ak4 === undefined) {
                    return undefined;
                } else {
                    return this.state.ak4 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "aik1":
                if(this.state.aik1 === undefined) {
                    return undefined;
                } else {
                    return this.state.aik1 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "aik2":
                if(this.state.aik2 === undefined) {
                    return undefined;
                } else {
                    return this.state.aik2 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "aik3":
                if(this.state.aik3 === undefined) {
                    return undefined;
                } else {
                    return this.state.aik3 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
            case "aik4":
                if(this.state.aik4 === undefined) {
                    return undefined;
                } else {
                    return this.state.aik4 ? <WarningFilled color="#fde300" /> : <CheckmarkFilled color="#00cc6a"/>;
                }
        }
    }

    matchTypeAndGetElements() {

        switch (this.dataSelf.table_kind) {
            case "Geradeturnen auf Musik":
                return(
                    <div className="fieldContainer">
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ok") ? this.dataSelf.judges.get("ok")!.name : ""} contentBefore={"OK"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ok")} className="inputType" id="ok" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("sk1") ? this.dataSelf.judges.get("sk1")!.name : ""} contentBefore={"SK1"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("sk1")} className="inputType" id="sk1" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("sk2") ? this.dataSelf.judges.get("sk2")!.name : ""} contentBefore={"SK2"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("sk2")} className="inputType" id="sk2" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ak1") ? this.dataSelf.judges.get("ak1")!.name : ""} contentBefore={"AK1"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ak1")} className="inputType" id="ak1" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ak2") ? this.dataSelf.judges.get("ak2")!.name : ""} contentBefore={"AK2"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ak2")} className="inputType" id="ak2" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ak3") ? this.dataSelf.judges.get("ak3")!.name : ""} contentBefore={"AK3"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ak3")} className="inputType" id="ak3" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ak4") ? this.dataSelf.judges.get("ak4")!.name : ""} contentBefore={"AK4"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ak4")} className="inputType" id="ak4" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("aik1") ? this.dataSelf.judges.get("aik1")!.name : ""} contentBefore={"AIK1"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("aik1")} className="inputType" id="aik1" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("aik2") ? this.dataSelf.judges.get("aik2")!.name : ""} contentBefore={"AIK2"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("aik2")} className="inputType" id="aik2" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("aik3") ? this.dataSelf.judges.get("aik3")!.name : ""} contentBefore={"AIK3"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("aik3")} className="inputType" id="aik3" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("aik4") ? this.dataSelf.judges.get("aik4")!.name : ""} contentBefore={"AIK4"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("aik4")} className="inputType" id="aik4" />
                        </Field>
                    </div>
                );
            default:
                return(
                    <div className="fieldContainer">
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ok") ? this.dataSelf.judges.get("ok")!.name : ""} contentBefore={"OK"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ok")} className="inputType" id="ok" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("sk1") ? this.dataSelf.judges.get("sk1")!.name : ""} contentBefore={"SK1"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("sk1")} className="inputType" id="sk1" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("sk2") ? this.dataSelf.judges.get("sk2")!.name : ""} contentBefore={"SK2"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("sk2")} className="inputType" id="sk2" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ak1") ? this.dataSelf.judges.get("ak1")!.name : ""} contentBefore={"AK1"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ak1")} className="inputType" id="ak1" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ak2") ? this.dataSelf.judges.get("ak2")!.name : ""} contentBefore={"AK2"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ak2")} className="inputType" id="ak2" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ak3") ? this.dataSelf.judges.get("ak3")!.name : ""} contentBefore={"AK3"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ak3")} className="inputType" id="ak3" />
                        </Field>
                        <Field>
                            <Input defaultValue={this.dataSelf.judges.get("ak4") ? this.dataSelf.judges.get("ak4")!.name : ""} contentBefore={"AK4"} onInput={(data) => this.updateValues(data)} contentAfter={this.setIcon("ak4")} className="inputType" id="ak4" />
                        </Field>
                    </div>
                );
        }
    }

    updateToFinale(data: CheckboxOnChangeData) {

        if(data.checked) {
            for(const table of this.props.storage.wk_judgingtables!) {
                if(table[0] === this.props.uniqueID) {
                    let temp_table = table;
                    temp_table[1].table_is_finale = true;
                    let temp_storage = this.props.storage;
                    temp_storage.wk_judgingtables?.set(temp_table[0], temp_table[1]);
                    // Tell the frontendStorage that the Tables changed it
                    temp_storage.changedByDoubleHook = false;
                    this.props.setStorage(Object.assign({}, temp_storage));
                    break;
                }
            }
        } else {
            for(const table of this.props.storage.wk_judgingtables!) {
                if(table[0] === this.props.uniqueID) {
                    let temp_table = table;
                    temp_table[1].table_is_finale = false;
                    let temp_storage = this.props.storage;
                    temp_storage.wk_judgingtables?.set(temp_table[0], temp_table[1]);
                    // Tell the frontendStorage that the Tables changed it
                    temp_storage.changedByDoubleHook = false;
                    this.props.setStorage(Object.assign({}, temp_storage));
                    break;
                }
            }
        }

    }

    changeValues() {
        let temp_storage = this.props.storage;
        let table = temp_storage.wk_judgingtables!.get(this.props.uniqueID)!;
        table.table_name = this.state.tableName!;
        table.table_kind = this.state.tableDiscipline!;
        temp_storage.wk_judgingtables!.set(this.props.uniqueID, table);
        temp_storage.changedByDoubleHook = false;
        this.props.setStorage(Object.assign({}, temp_storage));
    }

    removeTable() {
        let temp_storage = this.props.storage;
        temp_storage.wk_judgingtables?.delete(this.props.uniqueID);
        temp_storage.changedByDoubleHook = false;
        this.props.setStorage(Object.assign({}, temp_storage));
    }

    render() {

        // TODO für Irgendwann. Hover einbauen, braucht wahrscheinlich nen State ^^
        const buttonAppearance: React.CSSProperties = {
            backgroundColor: "#d13438",
            color: "#ffffff",
        }

        for(const table of this.props.storage.wk_judgingtables!) {
            if(table[0] === this.props.uniqueID) {
                table[1].judges.forEach((judge, key) => {
                    switch(key) {
                        case "ok":
                            if(this.state.ok !== judge.doubleFound){this.setState({ok: judge.doubleFound});}
                            break;
                        case "sk1":
                            if(this.state.sk1 !== judge.doubleFound){this.setState({sk1: judge.doubleFound});}
                            break;
                        case "sk2":
                            if(this.state.sk2 !== judge.doubleFound){this.setState({sk2: judge.doubleFound});}
                            break;
                        case "ak1":
                            if(this.state.ak1 !== judge.doubleFound){this.setState({ak1: judge.doubleFound});}
                            break;
                        case "ak2":
                            if(this.state.ak2 !== judge.doubleFound){this.setState({ak2: judge.doubleFound});}
                            break;
                        case "ak3":
                            if(this.state.ak3 !== judge.doubleFound){this.setState({ak3: judge.doubleFound});}
                            break;
                        case "ak4":
                            if(this.state.ak4 !== judge.doubleFound){this.setState({ak4: judge.doubleFound});}
                            break;
                        case "aik1":
                            if(this.state.aik1 !== judge.doubleFound){this.setState({aik1: judge.doubleFound});}
                            break;
                        case "aik2":
                            if(this.state.aik2 !== judge.doubleFound){this.setState({aik2: judge.doubleFound});}
                            break;
                        case "aik3":
                            if(this.state.aik3 !== judge.doubleFound){this.setState({aik3: judge.doubleFound});}
                            break;
                        case "aik4":
                            if(this.state.aik4 !== judge.doubleFound){this.setState({aik4: judge.doubleFound});}
                            break;
                    }
                });
                break;
            }
        }

        return(
            <div id="headContainer">
                <Card className="cardType">
                    <CardHeader header={
                        <div className="cardHeaderDiv">
                            <div className="tableInfoContainer">
                                <Body1Stronger>{this.dataSelf.table_name}</Body1Stronger>
                                <Caption1>{this.dataSelf.table_kind}</Caption1>
                            </div>
                            <Button appearance="subtle" icon={<PenFilled></PenFilled>} onClick={() => this.setState({dialogOpen: true})}></Button>
                        </div>
                    } />
                    {this.matchTypeAndGetElements()}
                    <CardFooter>
                        <Checkbox label={"Finale?"} onChange={(_ev, data) => this.updateToFinale(data)} defaultChecked={this.dataSelf.table_is_finale} />
                    </CardFooter>
                </Card>
                <Dialog open={this.state.dialogOpen} onOpenChange={(_ev, data) => this.setState({dialogOpen: data.open})}>
                <DialogSurface>
                    <DialogBody>
                        <DialogTitle>Einstellungen des Kampfgerichts</DialogTitle>
                        <DialogContent>
                            <div className="tableDialogContainer">
                                <Field label={"Name des Kampfgerichts"} required={true}>
                                    <Input defaultValue={this.dataSelf.table_name} placeholder={this.dataSelf.table_name} onInput={(data) => this.setState({tableName: data.currentTarget.value})} />
                                </Field>
                                <Field label={"Disziplin"} required={true}>
                                    <Combobox placeholder={this.dataSelf.table_kind} defaultValue={this.dataSelf.table_kind} onOptionSelect={(_ev, data) => this.setState({tableDiscipline: data.optionText})}>
                                        <Option>Geradeturnen auf Musik</Option>
                                        <Option>Geradeturnen ohne Musik</Option>
                                        <Option>Spiraleturnen</Option>
                                        <Option>Sprung</Option>
                                    </Combobox>
                                </Field>
                            </div>
                        </DialogContent>
                        <DialogActions>
                            <Button style={buttonAppearance} onClick={() => {this.removeTable(); this.setState({dialogOpen: false})}}>Löschen</Button>
                            <DialogTrigger disableButtonEnhancement>
                                <Button appearance="secondary">Schließen</Button>
                            </DialogTrigger>
                            <Button appearance="primary" onClick={() => {this.changeValues(); this.setState({dialogOpen: false})}}>Erstellen</Button>
                        </DialogActions>
                    </DialogBody>
                </DialogSurface>
            </Dialog>
            </div>
        )
    }

    dataSelf: Kampfgericht = {
        uniqueID: "ERR",
        table_name: "ERR",
        table_kind: "ERR",
        table_is_finale: false,
        judges: new Map(),
    };

}

export default KampfgerichtElement;