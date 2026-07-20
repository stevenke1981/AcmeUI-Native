//! CommandPalette component — a dialog with a text input and a list of commands.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a command palette.
pub struct CommandPaletteBuilder<M> {
    pub id: WidgetKey,
    pub open: bool,
    pub commands: Vec<String>,
    pub on_select: Option<M>,
}

/// Create a command palette builder.
pub fn command_palette<M: Clone + 'static>(id: impl Into<WidgetKey>) -> CommandPaletteBuilder<M> {
    CommandPaletteBuilder {
        id: id.into(),
        open: false,
        commands: vec![],
        on_select: None,
    }
}

impl<M: Clone> CommandPaletteBuilder<M> {
    /// Open or close the palette.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Add a command string.
    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.commands.push(cmd.into());
        self
    }

    /// Set the message fired when a command is selected.
    pub fn on_select(mut self, msg: M) -> Self {
        self.on_select = Some(msg);
        self
    }

    /// Build the widget node tree.
    ///
    /// Renders a `Dialog` containing a `TextInput` for search/filter followed
    /// by a `Column` of `Label` command entries.
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str();

        let search_key = format!("{id_prefix}_input");
        let search_input: WidgetNode<M> = crate::text_input(search_key.as_str())
            .placeholder("Search commands…")
            .build();

        let mut list_col = crate::column::<M>().gap(2.0);
        for (i, cmd) in self.commands.iter().enumerate() {
            let cmd_key = format!("{id_prefix}_cmd_{i}");
            if let Some(ref msg) = self.on_select {
                list_col = list_col
                    .child(crate::button(cmd_key.as_str(), cmd.clone()).on_click(msg.clone()));
            } else {
                list_col = list_col.child(crate::label::<M>(cmd.clone()));
            }
        }
        let cmd_list = list_col.build();

        let content = crate::column::<M>()
            .child(search_input)
            .child(cmd_list)
            .gap(8.0)
            .build();

        crate::dialog::<M>(self.id.clone(), content)
            .title("Command Palette")
            .open(self.open)
            .width(480.0)
            .height(320.0)
            .build()
    }
}

impl<M: Clone + 'static> From<CommandPaletteBuilder<M>> for WidgetNode<M> {
    fn from(b: CommandPaletteBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Select,
    }

    #[test]
    fn command_palette_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = command_palette::<Msg>("palette")
            .open(true)
            .command("Save")
            .command("Open")
            .command("Close")
            .on_select(Msg::Select)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Dialog leaf
        assert_eq!(layout.style.kind, acme_layout::LayoutKind::Leaf);
        assert_eq!(layout.style.width, crate::Length::px(480.0));
    }

    #[test]
    fn command_palette_empty_commands() {
        let node: WidgetNode<Msg> = command_palette::<Msg>("p").open(true).build();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, acme_layout::LayoutKind::Leaf);
    }

    #[test]
    fn command_palette_key_is_stored() {
        let p = command_palette::<Msg>("my-palette");
        assert_eq!(p.id.as_str(), "my-palette");
    }
}
