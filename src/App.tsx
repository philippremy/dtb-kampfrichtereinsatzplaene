import { useEffect, useId, useState } from "react";
import "./App.css";
// @ts-ignore
// This error is a straight-up lie
import dtbLogo from "./assets/dtb-logo.svg";
import { FluentProvider, webLightTheme, webDarkTheme, Title2, Image, Button, useToastController, Toast, ToastTitle, ToastBody, Toaster, Dialog, DialogSurface, DialogBody, DialogTitle, DialogContent, DialogActions, DialogTrigger, Spinner } from "@fluentui/react-components";
import { FolderOpenFilled, FormNewFilled } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api";
import { open } from '@tauri-apps/api/dialog';
import { getCurrent } from "@tauri-apps/api/window";

function App() {

  // Theme Hook
  const useThemeDetector = () => {
    const [theme, setTheme] = useState(webLightTheme);  
    const mqListener = ((e: any) => {
        if(e.matches) {
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

  // Function to create a new wettkampf window
  async function createWettkampf() {
    invoke("create_wettkampf").then((result) => {
      if(result === "NoError") {
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
      if(backendResponse) {
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
        { timeout: -1, toastId: downloadToastId }
    );

    // Call the frontend to download chrome
    invoke("download_chrome", {}).then((response) => {
      if(response != "NoError") {
        updateToast({
          toastId: downloadToastId,
          timeout: 3000,
          content: <Toast><ToastTitle>Chrome konnte nicht heruntergeladen werden: {response as string}</ToastTitle></Toast>,
          intent: "error"
        });
        setButtonsDisabled(false);
      } else {
        updateToast({
          toastId: downloadToastId,
          timeout: 3000,
          content: <Toast><ToastTitle>Chrome erfolgreich heruntergeladen!</ToastTitle></Toast>,
          intent: "success"
        });
        setButtonsDisabled(false);
      }
    });

  }

  // Toaster Stuff
  const { dispatchToast, updateToast } = useToastController();
  const showBackendError = (error: string) => dispatchToast(
      <Toast>
        <ToastTitle>Ein Backend-Fehler ist aufgetreten.</ToastTitle>
        <ToastBody>Rust gab folgenden Fehler zurück: {error}</ToastBody>
      </Toast>,
      { intent: "error" }
  );

  // Function to open a Wettkampf
  function openWettkampf() {
    open({ title: "Wettkampfdatei öffnen...", multiple: false, filters: [{name: "Wettkampfdatei", extensions: ["wkdata"]}] }).then((file) => {
      if(file === null) {
        return;
      } else if(Array.isArray(file)) {
        invoke("import_wk_file_and_open_editor", {filepath: file[0]}).then((response) => {
          if(response === "NoError") {
            return;
          } else {
            showBackendError(response as string);
          }
        });
      } else {
        invoke("import_wk_file_and_open_editor", {filepath: file}).then((response) => {
          if(response === "NoError") {
            const thisWindow = getCurrent();
            thisWindow.close().then(() => {});
            return;
          } else {
            showBackendError(response as string);
          }
        });
      }
    });
  }

  return (
    <FluentProvider theme={theme}>
      <div id="mainContainer">
        <div id="startupHeader">
          <Image src={dtbLogo} id="startupDtbLogo"></Image>
          <Title2>Kampfrichtereinsatzplantool</Title2>
        </div>
        <div id="startupButtonContainer">
          <Button appearance="primary" icon={<FormNewFilled />} onClick={() => createWettkampf()} disabled={buttonsDisabled}>Wettkampf erstellen</Button>
          <Button appearance="secondary" icon={<FolderOpenFilled />} onClick={() => openWettkampf()} disabled={buttonsDisabled}>Wettkampf öffnen</Button>
        </div>
      </div>
      <Toaster></Toaster>
      <Dialog modalType={"alert"} open={dialogOpen} >
        <DialogSurface>
          <DialogBody>
            <DialogTitle>
              Keine Chrome-Installation gefunden
            </DialogTitle>
            <DialogContent>
              Es wurde keine lokale Chromium-Installation gefunden.
              Um die PDF-Funktion der Software zu nutzen, muss einmalig eine Chrome-Installation heruntergeladen werden.<br/><br/>
              Die geschätzte Dauer beträgt einmalig 2-3 Minuten. Soll Chrome heruntergeladen werden?<br/>
            </DialogContent>
            <DialogActions fluid={true}>
              <DialogTrigger disableButtonEnhancement>
                <Button appearance={"secondary"} onClick={() => setDialogOpen(false)}>Nicht herunterladen</Button>
              </DialogTrigger>
                <Button appearance={"primary"} onClick={() => downloadChrome()}>Herunterladen</Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </FluentProvider>
  );
}

export default App;
