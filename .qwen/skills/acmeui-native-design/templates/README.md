# AcmeUI App Starter Views

These files are composable `WidgetNode` starters, not complete runtime binaries.

## Agent integration procedure

1. Read `catalog.yaml` and select the template.
2. Copy the selected `.rs` file into `apps/<app>/src/view.rs` or a feature module.
3. Rename state/message types to the product domain.
4. Add the required `acme-ui` feature families.
5. Wire messages into the application's update/event handling.
6. Replace placeholder labels with real state.
7. Implement permission/loading/error states before visual sign-off.
8. Run compile tests and render the App for Visual QA.

For a new app, use `apps/playground` as the runtime reference, but do not copy its demo content or manual button indexing blindly.
