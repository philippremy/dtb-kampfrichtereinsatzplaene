import { useEffect, useState } from "react";
import { FluentProvider, webLightTheme, webDarkTheme, Image, Subtitle2, Field, Input, Button, Toaster, useToastController, Toast, ToastTitle, ToastBody} from "@fluentui/react-components";
import "./CreateWettkampf.css";
import dtbLogo from "./assets/dtb-logo.svg";
import { CalendarFilled, PersonFilled, PinFilled, TimePickerFilled, TrophyFilled } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api";
import { getCurrent } from "@tauri-apps/api/window";

function CreateWettkampf() {

  // Kampfrichter Interface
  interface Kampfrichter {
    role: string,
    name: string,
  }

  // Kampfgericht Interface
  interface Kampfgericht {
    table_name: string,
    table_kind: string,
    table_is_finale: boolean,
    judges: Array<Kampfrichter>,
  }

  // Frontend Storage Interface
  interface FrontendStorage {
    wk_name: string,
    wk_date: string,
    wk_place: string,
    wk_responsible_person: string,
    wk_judgesmeeting_time: string,
    wk_replacement_judges: Array<string> | undefined,
    wk_judgingtables: Array<Kampfgericht> | undefined,
  }

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

  // States for Form Validation
  const [nameState, setNameState] = useState<"none" | "error" | "success" | "warning" | undefined>("none");
  const [placeState, setPlaceState] = useState<"none" | "error" | "success" | "warning" | undefined>("none");
  const [dateState, setDateState] = useState<"none" | "error" | "success" | "warning" | undefined>("none");
  const [timeState, setTimeState] = useState<"none" | "error" | "success" | "warning" | undefined>("none");
  const [personState, setPersonState] = useState<"none" | "error" | "success" | "warning" | undefined>("none");
  const [nameValidationMessage, setNameValidationMessage] = useState("");
  const [dateValidationMessage, setDateValidationMessage] = useState("");
  const [placeValidationMessage, setPlaceValidationMessage] = useState("");
  const [timeValidationMessage, setTimeValidationMessage] = useState("");
  const [personValidationMessage, setPersonValidationMessage] = useState("");

  // HashMap to track the entered data
  const wkData: FrontendStorage = {
    wk_name: "",
    wk_date: "",
    wk_place: "",
    wk_responsible_person: "",
    wk_judgesmeeting_time: "",
    wk_replacement_judges: undefined,
    wk_judgingtables: undefined
  }

  // Function for form validation
  function validateFormInput(data: React.FormEvent<HTMLInputElement>) {
    if(data.currentTarget.value == "") {
      switch (data.currentTarget.id) {
        case "name":
          setNameState("error");
          setNameValidationMessage("Es muss ein Wettkampfname vergeben werden.");
          break;
        case "place":
          setPlaceState("error");
          setPlaceValidationMessage("Es muss ein Wettkampfort angegeben werden.");
          break;
        case "date":
          setDateState("error");
          setDateValidationMessage("Es muss ein Wettkampfdatum angegeben werden.");
          break;
        case "time":
          setTimeState("error");
          setTimeValidationMessage("Es muss eine Uhrzeit für die Kampfrichterbesprechung angageben werden.");
          break;
        case "person":
          setPersonState("error");
          setPersonValidationMessage("Die/Der Kampfrichterbeauftragte für den Wettkampf muss benannt werden.");
          break;
      }
      setSubmitButtonEnabled(false);
    } else {
      switch (data.currentTarget.id) {
        case "name":
          setNameState("none");
          setNameValidationMessage("");
          wkData.wk_name = data.currentTarget.value;
          break;
        case "place":
          setPlaceState("none");
          setPlaceValidationMessage("");
          wkData.wk_place = data.currentTarget.value;
          break;
        case "date":
          setDateState("none");
          setDateValidationMessage("");
          wkData.wk_date = data.currentTarget.value;
          break;
        case "time":
          setTimeState("none");
          setTimeValidationMessage("");
          wkData.wk_judgesmeeting_time = data.currentTarget.value;
          break;
        case "person":
          setPersonState("none");
          setPersonValidationMessage("");
          wkData.wk_responsible_person = data.currentTarget.value;
          break;
      }
      for(const [_key, value] of Object.entries(wkData)) {
        if(value === "") {
          setSubmitButtonEnabled(false);
        } else {
          setSubmitButtonEnabled(true);
        }
      }
    }
  }

  // Button State
  const [submitButtonEnabled, setSubmitButtonEnabled] = useState(false);

  // Function for creating a new Wettkampf
  function createNewWettkampf() {
    invoke("sync_wk_data_and_open_editor", {data: wkData}).then((result) => {
      if(result === "NoError") {
        const thisWindow = getCurrent();
        thisWindow.close();
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

  return (
    <FluentProvider theme={theme}>
      <div id="mainContents">
        <div id="createWettkampfHeader">
          <Image src={dtbLogo} id="createWettkampfDtbLogo"></Image>
          <Subtitle2>Kampfrichtereinsatzplantool</Subtitle2>
        </div>
        <div id="formContainer">
          <Field label={"Wettkampfname"} validationState={nameState} validationMessage={nameValidationMessage} required={true} className="wkField">
            <Input id="name" onInput={(data) => validateFormInput(data)} contentBefore={<TrophyFilled></TrophyFilled>} />
          </Field>
          <Field label={"Wettkampfort"} validationState={placeState} validationMessage={placeValidationMessage} required={true} className="wkField">
            <Input id="place" onInput={(data) => validateFormInput(data)} contentBefore={<PinFilled></PinFilled>} />
          </Field>
          <Field label={"Wettkampfdatum"} validationState={dateState} validationMessage={dateValidationMessage} required={true} className="wkField">
            <Input id="date" type="date" placeholder="" onInput={(data) => validateFormInput(data)} contentBefore={<CalendarFilled></CalendarFilled>} />
          </Field>
          <Field label={"Kampfrichterbesprechung (Uhrzeit)"} validationState={timeState} validationMessage={timeValidationMessage} required={true} className="wkField">
            <Input id="time" type="time" placeholder="" onInput={(data) => validateFormInput(data)} contentBefore={<TimePickerFilled></TimePickerFilled>} />
          </Field>
          <Field label={"Kampfrichterbeauftragte*r"} validationState={personState} validationMessage={personValidationMessage} required={true} className="wkField">
            <Input id="person" onInput={(data) => validateFormInput(data)} contentBefore={<PersonFilled></PersonFilled>} />
          </Field>
        </div>
        <div id="confirmButtonDiv">
          <Button appearance="primary" disabled={!submitButtonEnabled} onClick={() => createNewWettkampf()}>Wettkampf erstellen</Button>
        </div>
      </div>
      <Toaster></Toaster>
    </FluentProvider>
  );
}

export default CreateWettkampf;
