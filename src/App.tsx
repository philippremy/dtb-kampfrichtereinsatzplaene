import { useEffect, useState } from "react";
import "./App.css";
// @ts-ignore
// This error is a straight-up lie
import dtbLogo from "./assets/dtb-logo.svg";
import { FluentProvider, webLightTheme, webDarkTheme, Title2, Image, Button, useToastController, Toast, ToastTitle, ToastBody, Toaster } from "@fluentui/react-components";
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

  // Toaster Stuff
  const { dispatchToast } = useToastController();
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
          <Button appearance="primary" icon={<FormNewFilled />} onClick={() => createWettkampf()}>Wettkampf erstellen</Button>
          <Button appearance="secondary" icon={<FolderOpenFilled />} onClick={() => openWettkampf()}>Wettkampf öffnen</Button>
        </div>
      </div>
      <Toaster></Toaster>
    </FluentProvider>
  );
}

export default App;