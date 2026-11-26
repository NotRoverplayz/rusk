# RUSK Web

## Status

### Implemented
- Native RUSK web scripting compiled from the existing Lexer, Parser and AST.
- `let`, `set`, functions, conditions, repeat loops, arrays, numbers, strings and booleans.
- Automatic registration of RUSK functions for matching RSML events.
- `bind` synchronization for text, inputs, textareas, selects and checkboxes.
- Explicit `<window script="main.rusk">` and sibling `main.rusk` discovery.
- Browser built-ins: `length`, `to_number`, `random`, `random_int`, `say`, `print`.
- Desktop-only built-ins report clear browser errors instead of executing.
- Existing `rusk build-web <file.rsml>` HTML build command.
- RSML elements for layout, text, controls, sections, and media.
- HTML and CSS escaping with CSS value validation.
- Flex, grid, spacing, sizing, and basic styling attributes.
- Mobile and tablet grid breakpoints through `mobile-columns` and `tablet-columns`.
- Browser event hooks and a `window.RuskWeb` state bridge.
- RSML comments.
- CSS keyframes for fade, slide, scale, bounce, and rotate animations.

### Experimental
- Browser event handlers can call RUSK functions or an application-defined `window.RuskApp` JavaScript function.
- Data binding through `bind` and `RuskWeb.setState`.

### Planned
- Components, routing, assets, development server, and multi-page builds.
