import { Body1Strong, Button, Checkbox, Divider, Field, FluentProvider, Input, Textarea, webDarkTheme, webLightTheme } from "@fluentui/react-components";
import { getCurrent } from "@tauri-apps/api/window";
import React from "react";
import "./BugReporter.css";
import { invoke } from "@tauri-apps/api";

export default class BugReporter extends React.Component<{}, {isLight: boolean, title: string | null, name: string | null , mail: string | null, subject: string, message: string, sendLogs: boolean}> {

    constructor(props: {}) {
        super(props);
        this.state = {
          isLight: true,
          title: null,
          name: null,
          mail: null,
          subject: "[BEISPIELBETREFF]",
          message: "[BEISPIELNACHRICHT]",
          sendLogs: false,
        }
    }

    componentDidMount(): void {
        const darkModePreference = window.matchMedia("(prefers-color-scheme: dark)");
        darkModePreference.matches ? this.setState({isLight: false}) : this.setState({isLight: true})
        darkModePreference.addEventListener("change", e => e.matches ? this.setState({isLight: false}) : this.setState({isLight: true}));
    }

    async submit() {
        if(this.state.name === "") {
          this.setState({name: null});
        }
        if(this.state.mail === "") {
          this.setState({mail: null});
        };
        invoke("send_mail_from_frontend", { name: this.state.name, mail: this.state.mail, subject: this.state.subject, message: this.state.message, sendlogs: this.state.sendLogs, kind: this.state.title }).then((response) => {
          console.log(response);
        });
    }

    async setTitle() {
        this.setState({title: await getCurrent().title()});
    }

    render() {

        if(this.state.title === null) {
          this.setTitle();
        }

        return(
            <>
              <FluentProvider theme={this.state.isLight ? webLightTheme : webDarkTheme}>
                <div id="mainContents">
                  <Body1Strong>E-Mail an die Entwickler senden:</Body1Strong>
                  <Divider inset={true} id="divider" />
                  <Field label={"Name:"} >
                    <Input type="text" placeholder="Vorname Nachname" onInput={(ev) => this.setState({name: ev.currentTarget.value})} />  
                  </Field>
                  <Field label={"E-Mail:"}>
                    <Input type="email" placeholder="person@domain.de" onInput={(ev) => this.setState({mail: ev.currentTarget.value})} />  
                  </Field>
                  <Field label={"Betreff:"} required={true} >
                    <Input type="text" placeholder="BUG/FEEDBACK/SUPPRT für..." onInput={(ev) => this.setState({subject: ev.currentTarget.value})} />  
                  </Field>
                  <Field label={"Nachricht:"} id="nachrichtField" required={true} >
                    <Textarea placeholder="Mir ist Folgendes aufgefallen: [...]" id="nachricht" resize="none" onInput={(ev) => this.setState({message: ev.currentTarget.value})} />
                  </Field>
                  <Checkbox label={"Logs mitsenden?"} defaultChecked={this.state.sendLogs} onChange={(_ev, data) => this.setState({sendLogs: data.checked as boolean})} />
                  <div id="buttonDiv">
                    <Button appearance="primary" onClick={() => this.submit()} >Absenden</Button>
                    <Button appearance="secondary" onClick={() => getCurrent().close()} >Abbrechen</Button>
                  </div>
                </div>
              </FluentProvider>
            </>
        );
    }
}
