### Gastronomy User Stories

1. Design Gastronomy Landing page
   - Issue ID: 13602
   - Type: Story
   - Description: Design landing page based on existing product templates, incorporating new gastronomy-specific features and ensuring high responsiveness across devices. Aim to highlight unique aspects of our gastronomy offerings.

2. Gastronomy landing page UI on sundae.fi
   - Issue ID: 13594
   - Type: Story
   - Description: Implement the landing page using the CMS. Ensure the UI is aligned with the current brand guidelines and provides a seamless user experience on mobile and desktop.

3. Show source files in gastronomy
   - Issue ID: 13474
   - Type: Story
   - Description: We have source maps; now show the text from these files directly in the gastronomy tool. Enable syntax highlighting and integrate with existing debug tools.

4. Fix gastronomy transaction support
   - Issue ID: 13473
   - Type: Story
   - Description: Lazy load traces so that the app stops hanging on large transactions. Improve error handling and add user feedback when transactions fail.

5. Clean up gastronomy
   - Issue ID: 13460
   - Type: Story
   - Description: Pi has run roughshod around gastronomy’s backend which now requires a clean-up. Prioritize removing unused libraries and updating outdated APIs.

6. Gastronomy source maps
   - Issue ID: 13458
   - Type: Story
   - Description: To collect funding for gastronomy, we need to provide detailed source maps to investors. These should include all dependencies and demonstrate compliance with licensing.

7. Gastronomy UI hand-off
   - Issue ID: 13442
   - Type: Story
   - Description: Generate styles and layout for the gastronomy application UI. Ensure all designs are handed off to the development team with detailed instructions.

8. Gastronomy app setup
   - Issue ID: 13356
   - Type: Story
   - Description: Set up and configure the basic Tauri app. Include web UI for easier navigation and ensure integration with existing backend services.

9. Create initial tauri API for uplc
   - Issue ID: 13410
   - Type: Story
   - Description: Develop one API call to open a file (“new trace”), which takes identifier and returns number of frames, maybe the first frame. Include high-level info about the trace.

10. Add support for uplc or flat encoded UPLC files
    - Issue ID: 13057
    - Type: Story
    - Description: Create a parser to handle both raw and encoded uplc files. Ensure compatibility with existing data structures.

11. Add a `run` command to uplc debugger project
    - Issue ID: 13056
    - Type: Story
    - Description: Read a raw uplc file, and “run” it to completion, using the UPLC crate, saving the machine state to disk on each step. Include links to example test data for uplc programs.

12. Submit PR to uplc crate to add step function
    - Issue ID: 13055
    - Type: Story
    - Description: There is an existing UPLC crate, which implements the core evaluation loop. Submit a PR to factor this into two methods for run and step functions.
