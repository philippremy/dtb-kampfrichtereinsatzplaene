import React, { useEffect, useId, useState } from "react";
import { Button, Field, FluentProvider, Image, Input, Subtitle2, Toast, ToastBody, Toaster, ToastTitle, useToastController, webDarkTheme, webLightTheme } from "@fluentui/react-components";
import "./CreateWettkampf.css";
// @ts-ignore
import dtbLogo from "./assets/dtb-logo.svg";
import dtbLogoLight from "./assets/dtb-logo-light.svg";
import { CalendarFilled, PersonFilled, PinFilled, TimePickerFilled, TrophyFilled } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api";
import { getCurrent } from "@tauri-apps/api/window";
import { FrontendStorage } from "./Editor.tsx";

function CreateWettkampf() {

  // Theme thing :)
  useEffect(() => {
    const darkModePreference = window.matchMedia("(prefers-color-scheme: dark)");
    darkModePreference.matches ? setIsLight(false) : setIsLight(true);
    darkModePreference.addEventListener("change", e => e.matches ? setIsLight(false) : setIsLight(true));
  }, []);
  const [isLight, setIsLight] = useState(true);

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

  // Function for date generation
  function formatDate(inputDate: string) {
    // Parse the input date string as a Date object
    let date = new Date(inputDate);

    // Extract day, month, and year components
    let day = date.getDate();
    let month = date.getMonth() + 1; // Months are zero-based, so add 1
    let year = date.getFullYear();

    let dayStr;
    let monthStr;
    let yearStr = year.toString();

    // Pad day and month with leading zeros if necessary
    if (day < 10) {
      dayStr = '0' + day;
    } else {
      dayStr = day.toString();
    }
    if (month < 10) {
      monthStr = '0' + month;
    } else {
      monthStr = month.toString();
    }

    // Construct the formatted date string
    return dayStr + '.' + monthStr + '.' + yearStr;
  }

  // Function for form validation
  function validateFormInput(data: React.FormEvent<HTMLInputElement>) {
    switch (data.currentTarget.id) {
      case nameInput:
        if(data.currentTarget.value === "" || data.currentTarget.value === null || data.currentTarget.value === undefined) {
          setNameState("error");
          setNameValidationMessage("Es muss ein Wettkampfname vergeben werden.");
        } else {
          setNameState("none");
          setNameValidationMessage("");
        }
        break;
      case dateInput:
        if(data.currentTarget.value === "" || data.currentTarget.value === null || data.currentTarget.value === undefined) {
          setDateState("error");
          setDateValidationMessage("Es muss ein Wettkampfdatum gewählt werden.");
        } else {
          setDateState("none");
          setDateValidationMessage("");
        }
        break;
      case timeInput:
        if(data.currentTarget.value === "" || data.currentTarget.value === null || data.currentTarget.value === undefined) {
          setTimeState("error");
          setTimeValidationMessage("Es muss eine Zeit für die Kampfrichterbesprechung gewählt werden.");
        } else {
          setTimeState("none");
          setTimeValidationMessage("");
        }
        break;
      case placeInput:
        if(data.currentTarget.value === "" || data.currentTarget.value === null || data.currentTarget.value === undefined) {
          setPlaceState("error");
          setPlaceValidationMessage("Es muss ein Wettkampfort angegeben werden.");
        } else {
          setPlaceState("none");
          setPlaceValidationMessage("");
        }
        break;
      case responsiblePersonInput:
        if(data.currentTarget.value === "" || data.currentTarget.value === null || data.currentTarget.value === undefined) {
          setPersonState("error");
          setPersonValidationMessage("Es muss ein Wettkampfort angegeben werden.");
        } else {
          setPersonState("none");
          setPersonValidationMessage("");
        }
        break;
    }
  }

  // IDs
  const nameInput = useId();
  const dateInput = useId();
  const placeInput = useId();
  const timeInput = useId();
  const responsiblePersonInput = useId();

  // Function for creating a new Wettkampf
  function createNewWettkampf() {

    let wkData: FrontendStorage = {
      changedByDoubleHook: false,
      wk_date: formatDate(document.getElementById(dateInput)!.getAttribute("value")!),
      wk_judgesmeeting_time: document.getElementById(timeInput)!.getAttribute("value")!,
      wk_judgingtables: undefined,
      wk_name: document.getElementById(nameInput)!.getAttribute("value")!,
      wk_place: document.getElementById(placeInput)!.getAttribute("value")!,
      wk_replacement_judges: undefined,
      wk_responsible_person: document.getElementById(responsiblePersonInput)!.getAttribute("value")!

    };
    invoke("sync_wk_data_and_open_editor", {data: wkData}).then((result) => {
      if(result === "NoError") {
        const thisWindow = getCurrent();
        thisWindow.close().then(() => {});
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
    <FluentProvider theme={isLight ? webLightTheme : webDarkTheme}>
      <div id="mainContents">
        <div id="createWettkampfHeader">
          <Image src={isLight ? dtbLogo : dtbLogoLight} id="createWettkampfDtbLogo"></Image>
          <Subtitle2>Kampfrichtereinsatzplantool</Subtitle2>
        </div>
        <div id="formContainer">
          <Field label={"Wettkampfname"} validationState={nameState} validationMessage={nameValidationMessage} required={true} className="wkField">
            <Input id={nameInput} onInput={(data) => validateFormInput(data)} contentBefore={<TrophyFilled></TrophyFilled>} autoCapitalize={"off"} autoCorrect={"off"} />
          </Field>
          <Field label={"Wettkampfort"} validationState={placeState} validationMessage={placeValidationMessage} required={true} className="wkField">
            <Input id={placeInput} onInput={(data) => validateFormInput(data)} contentBefore={<PinFilled></PinFilled>} autoCapitalize={"off"} autoCorrect={"off"} />
          </Field>
          <Field label={"Wettkampfdatum"} validationState={dateState} validationMessage={dateValidationMessage} required={true} className="wkField">
            <Input id={dateInput} type="date" placeholder="" onInput={(data) => validateFormInput(data)} contentBefore={<CalendarFilled></CalendarFilled>} />
          </Field>
          <Field label={"Kampfrichterbesprechung (Uhrzeit)"} validationState={timeState} validationMessage={timeValidationMessage} required={true} className="wkField">
            <Input id={timeInput} type="time" placeholder="" onInput={(data) => validateFormInput(data)} contentBefore={<TimePickerFilled></TimePickerFilled>} />
          </Field>
          <Field label={"Kampfrichterbeauftragte*r"} validationState={personState} validationMessage={personValidationMessage} required={true} className="wkField">
            <Input id={responsiblePersonInput} onInput={(data) => validateFormInput(data)} contentBefore={<PersonFilled></PersonFilled>} autoCapitalize={"off"} autoCorrect={"off"} />
          </Field>
        </div>
        <div id="confirmButtonDiv">
          <Button appearance="primary" onClick={() => createNewWettkampf()}>Wettkampf erstellen</Button>
        </div>
      </div>
      <Toaster></Toaster>
    </FluentProvider>
  );
}

export default CreateWettkampf;
