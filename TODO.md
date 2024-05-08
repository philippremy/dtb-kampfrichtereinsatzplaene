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
- [ ] Implement feedback windows
- [x] Fix the way how we find the path of the build library (use the glob crate instead) [Windows only!]
- [x] Move the Unix and *nix systems to store their data in the Application Support folders as well
- [x] Add copying the shared object on Linux systems into the bundle
- [x] Implement functionality to open created files in explorer/finder
- [x] Implement a panic hook that sends fatal errors via mail to the developer

### To-Do's for 1.1.0
- [ ] Build an updater (in app)
- [ ] Implement missing menus

### Possible, but not planned as of now
- [ ] Implement a sanitizing algorithm which sanitizes all templates (i.e., makes sure that all Text is in one Run)