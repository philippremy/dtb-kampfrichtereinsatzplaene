use std::fs::File;
use std::io::Read;
use mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use crate::{APP_VERSION, GIT_BRANCH, GIT_COMMIT, LLVM_VER, STDERR_FILE, STDOUT_FILE, TARGET_TRIPLE};
use crate::types::ApplicationError;

#[derive(Clone)]
pub enum MessageKind {
    Bug(String),
    Feedback(String),
    Support(String),
    Panic((String, String)),
}

impl MessageKind {

    fn generate_subject(&self) -> String {
        return match self {
            MessageKind::Bug(bug_desc) => format!("[BUG]: {}", bug_desc).to_string(),
            MessageKind::Feedback(feedback_descr) => format!("[FEEDBACK]: {}", feedback_descr).to_string(),
            MessageKind::Support(support_descr) => format!("[SUPPORT]: {}", support_descr).to_string(),
            MessageKind::Panic(panic_descr) => format!("[PANIC]: {}", panic_descr.0).to_string(),
        }
    }

    fn generate_mail_heading(&self) -> String {
        return match self {
            MessageKind::Bug(bug_desc) => format!("<h3>[BUG]: {}</h3><h4>DTB Kampfrichtereinsatzpläne v{APP_VERSION}</h4><p>[COMMIT]: {GIT_BRANCH}-{GIT_COMMIT}<br />[TARGET_TRIPLE]: {TARGET_TRIPLE}<br />[LLVM]: LLVM {LLVM_VER}<br /></p>", bug_desc).to_string(),
            MessageKind::Feedback(feedback_desc) => format!("<h3>[FEEDBACK]: {}</h3><h4>DTB Kampfrichtereinsatzpläne v{APP_VERSION}</h4><p>[COMMIT]: {GIT_BRANCH}-{GIT_COMMIT}<br />[TARGET_TRIPLE]: {TARGET_TRIPLE}<br />[LLVM]: LLVM {LLVM_VER}<br /></p>", feedback_desc).to_string(),
            MessageKind::Support(support_desc) => format!("<h3>[SUPPORT]: {}</h3><h4>DTB Kampfrichtereinsatzpläne v{APP_VERSION}</h4><p>[COMMIT]: {GIT_BRANCH}-{GIT_COMMIT}<br />[TARGET_TRIPLE]: {TARGET_TRIPLE}<br />[LLVM]: LLVM {LLVM_VER}<br /></p>", support_desc).to_string(),
            MessageKind::Panic(panic_desc) => format!("<h3>[Panic]: {}</h3><h4>DTB Kampfrichtereinsatzpläne v{APP_VERSION}</h4><p>[COMMIT]: {GIT_BRANCH}-{GIT_COMMIT}<br />[TARGET_TRIPLE]: {TARGET_TRIPLE}<br />[LLVM]: LLVM {LLVM_VER}<br /></p>", panic_desc.0).to_string(),
        }
    }

}

// Global func to send a mail with contents
pub async fn send_mail(kind: MessageKind, body: String) -> ApplicationError {

    // Create the Client
    let mut client = match SmtpClientBuilder::new("smtp-mail.outlook.com", 587)
        .implicit_tls(false)
        .credentials(("dtb-kampfrichtereinsatzplaene@outlook.com", "fidhUd-herqu4-myqzof"))
        .connect()
        .await {
        Ok(client) => client,
        Err(err) => { println!("Failed to connect to the SMTP Server: {:?}", err); return ApplicationError::SMTPConnectionError; }
    };

    let sender_name = format!("DTB Kampfrichtereinsatzpläne v{APP_VERSION}");
    let mut message = MessageBuilder::new()
        .from((sender_name.as_str(), "dtb-kampfrichtereinsatzplaene@outlook.com"))
        .to(("Philipp Remy", "philipp.remy@dtb.de"))
        .subject(kind.generate_subject())
        .html_body(format!("{}{body}", kind.generate_mail_heading()));

    // Append logs, if we have any.
    // SAFETY: The static muts are only written to once (at the very beginning) and then act
    // like a regular static, which is not unsafe.
    // We don't expect race conditions because of the big time difference they would be accessed
    // from different actors.
    unsafe {
        if STDOUT_FILE.is_some() && STDERR_FILE.is_some() {

            // Read the files into a buffer
            let stdout_file = File::open(STDOUT_FILE.clone().unwrap());
            let stderr_file = File::open(STDERR_FILE.clone().unwrap());

            // If opening succeeded
            if stdout_file.is_ok() && stderr_file.is_ok() {

                let mut stdout_buffer = vec![];
                let mut stderr_buffer = vec![];
                let stdout_result = stdout_file.unwrap().read_to_end(&mut stdout_buffer);
                let stderr_result = stderr_file.unwrap().read_to_end(&mut stderr_buffer);

                // If reading was successful
                if stdout_result.is_ok() && stderr_result.is_ok() {

                    // Add data as an appendix
                    message = message.attachment("text/plain", "STDOUT.txt", stdout_buffer);
                    message = message.attachment("text/plain", "STDERR.txt", stderr_buffer);
                }
            }
        }
    }

    match client.send(message).await {
        Ok(()) => {},
        Err(err) => { println!("Failed to send Message: {:?}", err); return ApplicationError::MessageSendError; }
    }

    return ApplicationError::NoError;
}