import { useEffect, useId, useState } from "react";
import "./App.css";
// @ts-ignore
// This error is a straight-up lie
import dtbLogo from "./assets/dtb-logo.svg";
import dtbLogoLight from "./assets/dtb-logo-light.svg";
import { FluentProvider, webLightTheme, webDarkTheme, Title2, Image, Button, useToastController, Toast, ToastTitle, ToastBody, Toaster, Dialog, DialogSurface, DialogBody, DialogTitle, DialogContent, DialogActions, DialogTrigger, Spinner, Field, ProgressBar } from "@fluentui/react-components";
import { FolderOpenFilled, FormNewFilled } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api";
import { open } from "@tauri-apps/api/dialog";
import { getCurrent } from "@tauri-apps/api/window";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { marked } from "marked";
import { relaunch } from "@tauri-apps/api/process";

function App() {
  function parseMarkdown(body: string) {
    return { __html: marked.parse(body, { async: false }) };
  }

  // Theme thing :)
  useEffect(() => {
    const darkModePreference = window.matchMedia("(prefers-color-scheme: dark)");
    darkModePreference.matches ? setIsLight(false) : setIsLight(true);
    darkModePreference.addEventListener("change", e => e.matches ? setIsLight(false) : setIsLight(true));
  }, []);
  const [isLight, setIsLight] = useState(true);

  // States for Updates
  const [downloadedChunks, setDownloadedChunks] = useState<number>(0);
  const [progressLabel, setProgressLabel] = useState("Auf Beginn des Downloads warten...");
  const [validationState, setValidationState] = useState<"none" | "success" | "error">("none");
  const [progressBarColor, setProgressBarColor] = useState<"brand" | "error" | "success">("brand");
  const [downloadStarted, setDownloadStarted] = useState(false);
  const [updaterButtonsDisabled, setUpdaterButtonsDisabled] = useState(false);
  const [updateButtonText, setUpdateButtonText] = useState("Installieren");

  // Hook for listening for updates
  useEffect(() => {
    // Lets create an array of Promises of UnlistenFns that we can iterate over later
    const unlistenArray = new Array<Promise<UnlistenFn>>();

    // LISTEN FOR UPDATE AVAILABLE
    unlistenArray.push(
      listen("updateIsAvailable", (event) => {
        setUpdateDialogTitle("Es ist ein Update verfügbar.");
        setUpdateDialogContent(
          <>
            <div dangerouslySetInnerHTML={
                // @ts-ignore
                parseMarkdown(event.payload.body)
              }
            hidden={downloadStarted}></div>
          </>,
        );
        setUpdateDialogOpen(true);
      }),
    );

    // LISTEN FOR UPDATE PENDING
    unlistenArray.push(
      // @ts-ignore
      listen("updateIsPending", (event) => {
        setUpdaterButtonsDisabled(true);
        setDownloadStarted(true);
        setDownloadedChunks(0);
        setUpdateDialogContent(<></>);
      }),
    );

    // LISTEN FOR DOWNLOAD PROGRESS
    unlistenArray.push(
      listen("updateHasProgress", (event) => {
        // @ts-ignore
        setDownloadedChunks((event.payload.chunk_len / event.payload.content_len));
        // @ts-ignore
        setProgressLabel("Fortschritt: " + ((event.payload.chunk_len / event.payload.content_len) * 100).toFixed(2) + " % heruntergeladen")
      }),
    );

    // LISTEN FOR DOWNLOAD FINISHED
    unlistenArray.push(
      // @ts-ignore
      listen("updateIsDownloaded", (event) => {
        setProgressBarColor("brand");
        setValidationState("none");
        setProgressLabel("Update erfolgreich heruntergeladen. Update wird installiert...");
        setDownloadedChunks(0);
      }),
    );

    // LISTEN FOR UPDATE FINISHED
    // @ts-ignore
    unlistenArray.push(listen("updateIsFinished", (event) => {
      setUpdateDialogOpen(true);
      setUpdateDialogTitle("Die App wurde erfolgreich aktualisiert");
      setProgressBarColor("success");
      setValidationState("success");
      setDownloadedChunks(1);
      setProgressLabel("Update erfolgreich! Die App muss neu gestartet werden, damit die Änderungen wirksam werden.");
      setUpdateButtonText("Neu starten");
      setUpdaterButtonsDisabled(false);
    }));

    // LISTEN FOR ALREADY UP-TO-DATE
    unlistenArray.push(
      // @ts-ignore
      listen("noUpdateAvailable", (event) => {
        {}
      }),
    );

    // LISTEN FOR UPDATE ERROR
    unlistenArray.push(
      listen("updateThrewError", (event) => {
        setDownloadedChunks(1);
        setProgressLabel("Ein Fehler ist aufgetreten: " + event.payload);
        setValidationState("error");
        setProgressBarColor("error");
        setUpdateButtonText("Schließen");
        setUpdaterButtonsDisabled(false);
      }),
    );

    // We mounted, so it is safe to receive any events now.
    invoke("update_mainwindow_loading_state", { visible: true }).then(() => {});

    return () => {
      unlistenArray.forEach((unlistenFn) => unlistenFn.then(f => f()));
    };
  }, []);

  // States for the update dialog
  const [updateDialogOpen, setUpdateDialogOpen] = useState(false);
  const [updateDialogTitle, setUpdateDialogTitle] = useState("");
  const [updateDialogContent, setUpdateDialogContent] = useState(<></>);

  // Function to start updating the App
  function updateApp(requested: boolean) {
    invoke("update_app", { requested: requested }).then(() => {});
  }

  // Function to create a new wettkampf window
  async function createWettkampf() {
    invoke("create_wettkampf").then((result) => {
      if (result === "NoError") {
        const thisWindow = getCurrent();
        thisWindow.close();
        return;
      }
      showBackendError(String(result));
    });
  }

  // Dialog states
  const [dialogOpen, setDialogOpen] = useState(false);

  // Check for a local chrome installation on startup
  useEffect(() => {
    async function call() {
      // We do this by invoking the backend
      let backendResponse: boolean = await invoke("check_for_chrome_binary", {});
      if (backendResponse) {
        return;
      } else {
        setDialogOpen(true);
      }
    }
    call();
  }, []);

  const [buttonsDisabled, setButtonsDisabled] = useState(false);
  const downloadToastId = useId();

  async function downloadChrome() {
    // Disable the dialog
    setDialogOpen(false);

    // Disable both buttons
    setButtonsDisabled(true);

    // Spawn a spinning toaster
    dispatchToast(
      <Toast>
        <ToastTitle media={<Spinner size="tiny" />}>
          Chrome wird heruntergeladen...
        </ToastTitle>
      </Toast>,
      { timeout: -1, toastId: downloadToastId },
    );

    // Call the frontend to download chrome
    invoke("download_chrome", {}).then((response) => {
      if (response != "NoError") {
        updateToast({
          toastId: downloadToastId,
          timeout: 3000,
          content: (
            <Toast>
              <ToastTitle>
                Chrome konnte nicht heruntergeladen werden: {response as string}
              </ToastTitle>
            </Toast>
          ),
          intent: "error",
        });
        setButtonsDisabled(false);
      } else {
        updateToast({
          toastId: downloadToastId,
          timeout: 3000,
          content: (
            <Toast>
              <ToastTitle>Chrome erfolgreich heruntergeladen!</ToastTitle>
            </Toast>
          ),
          intent: "success",
        });
        setButtonsDisabled(false);
      }
    });
  }

  // Toaster Stuff
  const { dispatchToast, updateToast } = useToastController();
  const showBackendError = (error: string) =>
    dispatchToast(
      <Toast>
        <ToastTitle>Ein Backend-Fehler ist aufgetreten.</ToastTitle>
        <ToastBody>Rust gab folgenden Fehler zurück: {error}</ToastBody>
      </Toast>,
      { intent: "error" },
    );

  // Function to open a Wettkampf
  function openWettkampf() {
    open({
      title: "Wettkampfdatei öffnen...",
      multiple: false,
      filters: [{ name: "Wettkampfdatei", extensions: ["wkdata"] }],
    }).then((file) => {
      if (file === null) {
        return;
      } else if (Array.isArray(file)) {
        invoke("import_wk_file_and_open_editor", { filepath: file[0] }).then(
          (response) => {
            if (response === "NoError") {
              return;
            } else {
              showBackendError(response as string);
            }
          },
        );
      } else {
        invoke("import_wk_file_and_open_editor", { filepath: file }).then(
          (response) => {
            if (response === "NoError") {
              const thisWindow = getCurrent();
              thisWindow.close().then(() => {});
              return;
            } else {
              showBackendError(response as string);
            }
          },
        );
      }
    });
  }

  async function handleUpdateButton(ev: React.MouseEvent<HTMLButtonElement>) {
    console.log(ev.currentTarget.ariaLabel);
    if(ev.currentTarget.ariaLabel === "Installieren") {
      updateApp(true)
    } else if(ev.currentTarget.ariaLabel === "Neu starten") {
      await relaunch();
    } else {
      setUpdateDialogOpen(false);
    }
  }

  return (
    <FluentProvider theme={isLight ? webLightTheme : webDarkTheme}>
      <div id="mainContainer">
        <div id="startupHeader">
          <Image
            src={isLight ? dtbLogo : dtbLogoLight}
            id="startupDtbLogo"
          ></Image>
          <Title2>Kampfrichtereinsatzplantool</Title2>
        </div>
        <div id="startupButtonContainer">
          <Button
            appearance="primary"
            icon={<FormNewFilled />}
            onClick={() => createWettkampf()}
            disabled={buttonsDisabled}
          >
            Wettkampf erstellen
          </Button>
          <Button
            appearance="secondary"
            icon={<FolderOpenFilled />}
            onClick={() => openWettkampf()}
            disabled={buttonsDisabled}
          >
            Wettkampf öffnen
          </Button>
        </div>
      </div>
      <Toaster></Toaster>
      <Dialog modalType={"alert"} open={dialogOpen}>
        <DialogSurface>
          <DialogBody>
            <DialogTitle>Keine Chrome-Installation gefunden</DialogTitle>
            <DialogContent>
              Es wurde keine lokale Chromium-Installation gefunden. Um die
              PDF-Funktion der Software zu nutzen, muss einmalig eine
              Chrome-Installation heruntergeladen werden.
              <br />
              <br />
              Die geschätzte Dauer beträgt einmalig 2-3 Minuten. Soll Chrome
              heruntergeladen werden?
              <br />
            </DialogContent>
            <DialogActions fluid={true}>
              <DialogTrigger disableButtonEnhancement>
                <Button
                  appearance={"secondary"}
                  onClick={() => setDialogOpen(false)}
                >
                  Nicht herunterladen
                </Button>
              </DialogTrigger>
              <Button appearance={"primary"} onClick={() => downloadChrome()}>
                Herunterladen
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
      <Dialog modalType={"alert"} open={updateDialogOpen}>
        <DialogSurface>
          <DialogBody>
            <DialogTitle>{updateDialogTitle}</DialogTitle>
            <DialogContent>
              {updateDialogContent}
              <div hidden={!downloadStarted}>
                <Field validationMessage={progressLabel} validationState={validationState}>
                  <ProgressBar
                      shape={"rounded"}
                      value={downloadedChunks === 0.00 ? undefined : downloadedChunks}
                      thickness={"large"}
                      color={progressBarColor}
                  />
                </Field>
              </div>
            </DialogContent>
            <DialogActions fluid={true}>
              <DialogTrigger disableButtonEnhancement>
                <Button
                  appearance={"secondary"}
                  onClick={() => {
                    updateApp(false);
                    setUpdateDialogOpen(false);
                  }}
                  disabled={updaterButtonsDisabled}
                >
                  Nicht installieren
                </Button>
              </DialogTrigger>
              <Button appearance={"primary"} onClick={(ev) => handleUpdateButton(ev)} disabled={updaterButtonsDisabled} aria-label={updateButtonText} >
                {updateButtonText}
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </FluentProvider>
  );
}

export default App;
