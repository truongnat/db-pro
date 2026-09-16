# Native UI Ticket Scope

All UI upgrade issues created from the 2026-09-16 audit should follow these rules:

- start from latest main;
- one focused issue/branch at a time;
- reuse canonical native components and theme tokens;
- no new ad-hoc styling system;
- no business/provider logic in renderer code;
- actual native runtime evidence is mandatory;
- verify dark/light and 1280x800 / 1440x900 / 1920x1080 where applicable;
- verify empty/loading/error/disabled/long-content states;
- verify keyboard focus and mouse hover/pressed states;
- merge accepted issue to main before beginning the next dependent UI issue.
