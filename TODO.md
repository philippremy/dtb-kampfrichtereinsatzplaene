### To-Do's for v1.0.0

- [x] Implement appending the Final Tables to the end of the document
- [x] Log all of StdOut to a file as a logging system
- [x] Sort the Kampfgerichte in the frontend, so they get rendered correctly!
- [x] Implement setting replacement judges in the frontend
- [x] Implement writing replacement judges to the backend
- [x] Implement writing a PDF (from the generated docx)
- [x] Implement disabling the PDF button when there is no chromium path at the frontend
- [x] Implement a more robust failure system with better feedback messages
- [x] Implement a license window
- [x] Fix window titles
- [x] Implement feedback windows
- [x] Fix the way how we find the path of the build library (use the glob crate instead) [Windows only!]
- [x] Move the Unix and *nix systems to store their data in the Application Support folders as well
- [x] Add copying the shared object on Linux systems into the bundle
- [x] Implement functionality to open created files in explorer/finder
- [x] Implement a panic hook that sends fatal errors via mail to the developer
- [x] Update the error logging in the backend library
- [x] Implement an automatic build number incrementer
- [x] Fix theme and DTB logo color bugs
- [x] Sort tables in Editor following a pattern of nonfinal --> A-Z --> final --> A-Z
- [x] Build an updater (in app)

### To-Do's for 1.0.1
- [x] Fix Chromium Downloader --> When we are checking, we should *not* allow any downloads! We *ask*!
- [x] Disable AutoCorrection in Input fields
- [x] Create custom environments for pre-release and debug builds in GitHub Actions

### To-Do's for 1.1.0
- [x] Add Cyr Wheel Judging Tables

### To-Do's for 1.2.0
- [x] Add ability to copy existing tables safely
- [x] Prevent standard page reloading in the Editor
- [x] Add line for clothing rules in template
- [x] Fix compile time error when not providing mail credentials via environment secrets
- [x] Fix runtime `dyld` Swift zlib error on macOS x86_64 when using the wrong linker in `ilc` (use `ld_classic`)

### To-Do's for 1.X.0
- [ ] Implement missing menus
- [ ] Refactor frontend syncing, so we don't have x million loc for the same thing. Makes compiling slow and we could just #[inline(always)] this
- [ ] Generate Menus using a function --> Reduce loc
- [ ] Create algorithm for inserting Judges based on an exported list from GymNet
- [ ] Implement Copy, Cut and Paste functionality in WebView contexts
- [ ] Implement custom About menu using a WebView window --> Enables this functionality on Windows

### Possible, but not planned as of now
- [ ] Implement a sanitizing algorithm which sanitizes all templates (i.e., makes sure that all Text is in one Run)
