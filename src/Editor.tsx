import { Button, Caption2, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface, DialogTitle, DialogTrigger, Field, FluentProvider, Input, Menu, MenuButton, MenuButtonProps, MenuItem, MenuList, MenuPopover, MenuTrigger, SplitButton, Subtitle2, webDarkTheme, webLightTheme } from "@fluentui/react-components";
import { AddFilled, DocumentFilled, PenFilled, SaveFilled } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import "./Editor.css";
import { v4 as uuidv4 } from 'uuid';
import KampfgerichteRenderer from "./KampfgerichteRenderer";

// Kampfrichter Interface
export type Kampfrichter = {
    role: string,
    name: string,
    doubleFound: boolean,
}

// Kampfgericht Interface
export type Kampfgericht = {
    uniqueID: string,
    table_name: string,
    table_kind: string,
    table_is_finale: boolean,
    judges: Map<string, Kampfrichter>,
}

// Frontend Storage Interface
export type FrontendStorage = {
    wk_name: string,
    wk_date: string,
    wk_place: string,
    wk_responsible_person: string,
    wk_judgesmeeting_time: string,
    wk_replacement_judges: Array<string> | undefined,
    wk_judgingtables: Map<string, Kampfgericht> | undefined,
    changedByDoubleHook: boolean,
}

function Editor() {

    // Theme Hook
    const useThemeDetector = () => {
        const [theme, setTheme] = useState(webLightTheme);
        const mqListener = ((e: any) => {
            if (e.matches) {
                setTheme(webDarkTheme);
            } else {
                setTheme(webLightTheme);
            }
        });

        useEffect(() => {
            const darkThemeMq = window.matchMedia("(prefers-color-scheme: dark)");
            darkThemeMq.addListener(mqListener);
            return () => darkThemeMq.removeListener(mqListener);
        }, []);
        return theme;
    }

    // Theme thing :)
    const theme = useThemeDetector();

    // Fetch initial data on mount
    useEffect(() => {
        invoke("get_wk_data_to_frontend").then((response: unknown) => {
            setFrontendStorage(response as FrontendStorage);
        })
    }, []);

    // Button stuff
    const onClickSaveButton = () => { };
    const primaryActionButtonProps = {
        onClickSaveButton,
    };

    // Global State for all the JudgingTableData (i.e., FrontendStorage)
    const [frontendStorage, setFrontendStorage] = useState<FrontendStorage>(() => {
        let storage: FrontendStorage = {
            wk_name: "",
            wk_date: "",
            wk_place: "",
            wk_responsible_person: "",
            wk_judgesmeeting_time: "",
            wk_replacement_judges: undefined,
            wk_judgingtables: undefined,
            changedByDoubleHook: false,
        };
        return storage;
    });

    // State for the Dialog
    const [open, setOpen] = useState(false);
    const [kindToCreate, setKindToCreate] = useState("");
    const [nameToCreate, setNameToCreate] = useState("");

    // Function for creating a table
    function createTable(kind: string, name: string) {

        const uniqueID = uuidv4();

        setFrontendStorage(() => {

            var judgingtables = new Map<string, Kampfgericht>();

            // Handle Arrays
            if(frontendStorage.wk_judgingtables === undefined || frontendStorage.wk_judgingtables === null) {
                var judgingtables = new Map<string, Kampfgericht>();
                judgingtables.set(uniqueID, {
                    uniqueID: uniqueID,
                    table_name: name,
                    table_kind: kind,
                    table_is_finale: false,
                    judges: new Map<string, Kampfrichter>(),
                });
            } else {
                judgingtables = frontendStorage.wk_judgingtables;
                judgingtables.set(uniqueID, {
                    uniqueID: uniqueID,
                    table_name: name,
                    table_kind: kind,
                    table_is_finale: false,
                    judges: new Map<string, Kampfrichter>(),
                });
            }

            var storage: FrontendStorage = {
                wk_name: frontendStorage.wk_name,
                wk_date: frontendStorage.wk_date,
                wk_place: frontendStorage.wk_place,
                wk_responsible_person: frontendStorage.wk_responsible_person,
                wk_judgesmeeting_time: frontendStorage.wk_judgesmeeting_time,
                wk_replacement_judges: frontendStorage.wk_replacement_judges,
                wk_judgingtables: judgingtables,
                changedByDoubleHook: false
            };
            return storage;
        });

    }

    // Effect to check for potential doubles!
    useEffect(() => {

        // Check if this hook changed the bloody thing
        if(frontendStorage.changedByDoubleHook) {
            return;
        }

        var doublesNormal: Map<string, number> = new Map();
        var doublesFinale: Map<string, number> = new Map();
        
        // Collect the amount of names
        frontendStorage.wk_judgingtables?.forEach((table) => {
            if(table.table_is_finale) {
                table.judges.forEach((judge) => {
                    if(doublesFinale.has(judge.name)) {
                        doublesFinale.set(judge.name, (doublesFinale.get(judge.name)! + 1));
                    } else {
                        doublesFinale.set(judge.name, 1);
                    }
                });
            } else {
                table.judges.forEach((judge) => {
                    if(doublesNormal.has(judge.name)) {
                        doublesNormal.set(judge.name, (doublesNormal.get(judge.name)! + 1));
                    } else {
                        doublesNormal.set(judge.name, 1);
                    }
                });
            }
        });

        // Iterate through all tables
        var temp_storage = frontendStorage;
        temp_storage.wk_judgingtables?.forEach((table) => {
            if(table.table_is_finale) {
                table.judges.forEach((judge) => {
                    let count = doublesFinale.get(judge.name)!;
                    if(count < 2) {
                        judge.doubleFound = false;
                    } else {
                        judge.doubleFound = true;
                    }
                })
            } else {
                table.judges.forEach((judge) => {
                    let count = doublesNormal.get(judge.name)!;
                    if(count < 2) {
                        judge.doubleFound = false;
                    } else {
                        judge.doubleFound = true;
                    }
                })
            }
        });

        setFrontendStorage(Object.assign({}, temp_storage));

    }, [frontendStorage]);

    return (
        <FluentProvider theme={theme}>
            <div id="editorHeader">
                <div id="wkInfoContainerWithButton">
                    <div id="wkInfoContainer">
                        <Subtitle2 id="wettkampfname">{frontendStorage.wk_name}</Subtitle2>
                        <Caption2 id="wettkampfInfo">am {frontendStorage.wk_date} in {frontendStorage.wk_place}</Caption2>
                    </div>
                    <Button appearance="subtle" icon={<PenFilled></PenFilled>} id="changeWkInfoButton" />
                </div>
                <div id="saveButtonContainer">
                    <Menu positioning={"below-end"}>
                        <MenuTrigger disableButtonEnhancement>
                            {(triggerProps: MenuButtonProps) => (
                                // @ts-ignore
                                // Works fine for now, I have no idea what the problem is.
                                <SplitButton size="small" appearance="secondary" icon={<SaveFilled></SaveFilled>} menuButton={triggerProps} primaryActionButton={primaryActionButtonProps}>Wettkampf speichern</SplitButton>
                            )}
                        </MenuTrigger>
                        <MenuPopover>
                            <MenuList>
                                <MenuItem>Speichern unter...</MenuItem>
                            </MenuList>
                        </MenuPopover>
                    </Menu>
                    <Menu>
                        <MenuTrigger disableButtonEnhancement>
                            <MenuButton size="small" appearance="primary" icon={<DocumentFilled></DocumentFilled>}>Pläne erstellen</MenuButton>
                        </MenuTrigger>
                        <MenuPopover>
                            <MenuList>
                                <MenuItem>Als Word-Datei</MenuItem>
                                <MenuItem>Als PDF</MenuItem>
                            </MenuList>
                        </MenuPopover>
                    </Menu>
                </div>
            </div>
            <div id="mainContents">
                <KampfgerichteRenderer storage={frontendStorage} setStorage={setFrontendStorage} />
                <div className="filler" />
            </div>
            <Menu>
                <MenuTrigger disableButtonEnhancement>
                    <MenuButton appearance="primary" icon={<AddFilled></AddFilled>} id="addButton"></MenuButton>
                </MenuTrigger>
                <MenuPopover>
                    <MenuList>
                        <MenuItem onClick={() => {setKindToCreate("Geradeturnen ohne Musik"); setOpen(true)}}>Geradeturnen ohne Musik</MenuItem>
                        <MenuItem onClick={() => {setKindToCreate("Geradeturnen auf Musik"); setOpen(true)}}>Geradeturnen auf Musik</MenuItem>
                        <MenuItem onClick={() => {setKindToCreate("Spiraleturnen"); setOpen(true)}}>Spiraleturnen</MenuItem>
                        <MenuItem onClick={() => {setKindToCreate("Sprung"); setOpen(true)}}>Sprung</MenuItem>
                        <MenuItem onClick={() => {setKindToCreate("Leer"); setOpen(true)}}>Leer</MenuItem>
                    </MenuList>
                </MenuPopover>
            </Menu>
            <Dialog open={open} onOpenChange={(_ev, data) => setOpen(data.open)}>
                <DialogSurface>
                    <DialogBody>
                        <DialogTitle>Namen für das Kampfgericht vergeben</DialogTitle>
                        <DialogContent>
                            <Field label={"Name des Kampfgerichts"} required={true}>
                                <Input onInput={(data) => setNameToCreate(data.currentTarget.value)} />
                            </Field>
                        </DialogContent>
                        <DialogActions>
                            <DialogTrigger disableButtonEnhancement>
                                <Button appearance="secondary">Schließen</Button>
                            </DialogTrigger>
                            <Button appearance="primary" onClick={() => {createTable(kindToCreate, nameToCreate); setOpen(false)}}>Erstellen</Button>
                        </DialogActions>
                    </DialogBody>
                </DialogSurface>
            </Dialog>
        </FluentProvider>
    );
}

export default Editor;