import { Button, Caption2, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface, DialogTitle, DialogTrigger, Field, FluentProvider, Input, Link, Menu, MenuButton, MenuButtonProps, MenuItem, MenuList, MenuPopover, MenuTrigger, Spinner, SplitButton, Subtitle2, Text, Toast, ToastBody, Toaster, ToastFooter, ToastIntent, ToastTitle, ToastTrigger, useToastController, webDarkTheme, webLightTheme } from "@fluentui/react-components";
import { AddFilled, CheckmarkFilled, DocumentFilled, ErrorCircleFilled, PenFilled, SaveFilled } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import "./Editor.css";
import { v4 as uuidv4 } from 'uuid';
import KampfgerichteRenderer from "./KampfgerichteRenderer";
import { ask, save } from "@tauri-apps/api/dialog";

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

    // Check if the user really wants to send anything to the backend!
    async function getUserApproval() {
        if(doublesExist) {
            return await ask("Soll der Wettkampf trotz der bestehenden Überschneidungen gespeichert werden?", {title: "Überschneidungen gefunden"});
        } else {
            return true;
        }
    }

    // This is the default button!
    const [lastSavePath, setLastSavePath] = useState<string | undefined>(undefined);
    async function saveWettkampf() {
        if(!await getUserApproval()) {
            return;
        }
        if(lastSavePath === undefined) {
            saveUnder();
        } else {
            syncToBackendAndSaveWettkampf(lastSavePath);
        }
    };

    // Speichern-unter... Funktion hihi
    function saveUnder() {
        save({filters: [{name: "Wettkampfdatei", extensions: ["wkdata"]}], title: "Wettkampf speichern unter..."}).then((filePath) => {
            if(filePath === null) {
                return;
            } else {
                syncToBackendAndSaveWettkampf(filePath);
            }
        });
    }

    function syncToBackendAndSaveWettkampf(path: string) {
        displayToast("saveToast", "Bitten warten", "Wettkampf wird gespeichert...", <Spinner size="tiny" />, -1);
        invoke("sync_to_backend_and_save", {frontendstorage: frontendStorage, filepath: path}).then((response) => {
            if(response !== "NoError") {
                updateToastWithID("saveToast", "error", "Fehler", "Ein Fehler ist aufgetreten: " +  response, <ErrorCircleFilled />, 3000);
            } else {
                updateToastWithID("saveToast", "success", "Speichern erfolgreich", "Der Wettkampf wurde gespeichert.", <CheckmarkFilled />, 3000);
                setLastSavePath(path);
            }
        });
    }

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

    // Variable for setting if we have any doubles at all
    var doublesExist: boolean = false;

    // Effect to check for potential doubles!
    useEffect(() => {

        // Check if this hook changed the bloody thing
        if(frontendStorage.changedByDoubleHook) {
            return;
        }

        doublesExist = false;

        var doublesNormal: Map<string, number> = new Map();
        var doublesFinale: Map<string, number> = new Map();
        
        // Collect the amount of names
        // But only if we are not undefined!
        if(frontendStorage.wk_judgingtables == undefined || frontendStorage.wk_judgingtables === null) {
            return;
        }

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
                        doublesExist = true;
                    }
                })
            } else {
                table.judges.forEach((judge) => {
                    let count = doublesNormal.get(judge.name)!;
                    if(count < 2) {
                        judge.doubleFound = false;
                    } else {
                        judge.doubleFound = true;
                        doublesExist = true;
                    }
                })
            }
        });

        setFrontendStorage(Object.assign({}, temp_storage));

    }, [frontendStorage]);

    // Toaster functions
    const { dispatchToast, updateToast, } = useToastController();
    // Most important function to display a toaster with a given title
    function displayToast(id: string, title: string, message: string, icon: React.JSX.Element, timeout?: number) {
        dispatchToast(
            <Toast>
                <ToastTitle title={title} media={icon} />
                <ToastBody>
                    <div className="toasterBody">
                        <Text>{message}</Text>
                    </div>
                </ToastBody>
                <ToastFooter>
                    <ToastTrigger>
                        <Link>Ausblenden</Link>
                    </ToastTrigger>
                </ToastFooter>
            </Toast>,
            {
                toastId: id,
                timeout: timeout,
            }
        );
    }

    function updateToastWithID(id: string, intent: ToastIntent, title: string, message: string, icon: JSX.Element, timeout: number) {
        updateToast({
            toastId: id,
            intent: intent,
            content: 
                <Toast>
                    <ToastTitle title={title} media={icon} />
                    <ToastBody>
                        <div className="toasterBody">
                            <Text>{message}</Text>
                        </div>
                    </ToastBody>
                    <ToastFooter>
                        <ToastTrigger>
                            <Link>Ausblenden</Link>
                        </ToastTrigger>
                    </ToastFooter>
                </Toast>,
            timeout: timeout,
        });
    }

    // Function to create plans as docx/pdf
    async function createPlans(type: string) {
        if(! await getUserApproval()) {
            return;
        }
        if(type === "pdf") {
            save({filters: [{name: "Adobe Acrobat Portable Document File", extensions: ["pdf"]}], title: "Einsatzplan speichern als PDF"}).then((filePath) => {
                if(filePath === null) {
                    return;
                } else {
                    syncWithBackendAndCreate(filePath, "pdf");
                }
            });
        } else {
            save({filters: [{name: "Open XML Wordprocessing Document", extensions: ["docx"]}], title: "Einsatzplan speichern als DOCX"}).then((filePath) => {
                if(filePath === null) {
                    return;
                } else {
                    syncWithBackendAndCreate(filePath, "docx");
                }
            });
        }
    }

    // Function to sync with backend and create the plans
    function syncWithBackendAndCreate(path: string, type: string) {
        displayToast("createToast", "Bitten warten", "Einsatzplan wird erstellt...", <Spinner size="tiny" />, -1);
        if(type === "docx") {
            invoke("sync_to_backend_and_create_docx", {frontendstorage: frontendStorage, filepath: path}).then((response) => {
                if(response !== "NoError") {
                    updateToastWithID("createToast", "error", "Fehler", "Ein Fehler ist aufgetreten: " +  response, <ErrorCircleFilled />, 3000);
                } else {
                    updateToastWithID("createToast", "success", "Speichern erfolgreich", "Der Einsatzplan wurde erfolgreich gespeichert.", <CheckmarkFilled />, 3000);
                }
            });
        } else if(type === "pdf") {
            invoke("sync_to_backend_and_create_pdf", {frontendstorage: frontendStorage, filepath: path}).then((response) => {
                if(response !== "NoError") {
                    updateToastWithID("createToast", "error", "Fehler", "Ein Fehler ist aufgetreten: " +  response, <ErrorCircleFilled />, 3000);
                } else {
                    updateToastWithID("createToast", "success", "Speichern erfolgreich", "Der Einsatzplan wurde erfolgreich gespeichert.", <CheckmarkFilled />, 3000);
                }
            });
        }
    }

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
                                <SplitButton appearance="secondary" icon={<SaveFilled></SaveFilled>} menuButton={triggerProps} primaryActionButton={<Text onClick={() => saveWettkampf()}>Wettkampf speichern</Text>}></SplitButton>
                            )}
                        </MenuTrigger>
                        <MenuPopover>
                            <MenuList>
                                <MenuItem onClick={() => saveUnder()}>Speichern unter...</MenuItem>
                            </MenuList>
                        </MenuPopover>
                    </Menu>
                    <Menu>
                        <MenuTrigger disableButtonEnhancement>
                            <MenuButton appearance="primary" icon={<DocumentFilled></DocumentFilled>}>Pläne erstellen</MenuButton>
                        </MenuTrigger>
                        <MenuPopover>
                            <MenuList>
                                <MenuItem onClick={() => createPlans("docx")}>Als Word-Datei</MenuItem>
                                <MenuItem onClick={() => createPlans("pdf")}>Als PDF</MenuItem>
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
                        { /* <MenuItem onClick={() => {setKindToCreate("Leer"); setOpen(true)}}>Leer</MenuItem> */ }
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
            <Toaster />
        </FluentProvider>
    );
}

export default Editor;