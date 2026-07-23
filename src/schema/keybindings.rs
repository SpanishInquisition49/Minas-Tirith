// src/schema/keybindings.rs
pub struct KeybindingEntry {
    pub section: &'static str,
    pub key: &'static str,
    pub action: &'static str,
}

pub const KEYBINDINGS: &[KeybindingEntry] = &[
    // Normal mode — Items focus
    KeybindingEntry {
        section: "Items",
        key: "j / Down",
        action: "Select next item",
    },
    KeybindingEntry {
        section: "Items",
        key: "k / Up",
        action: "Select previous item",
    },
    KeybindingEntry {
        section: "Items",
        key: "[",
        action: "Previous item type tab",
    },
    KeybindingEntry {
        section: "Items",
        key: "]",
        action: "Next item type tab",
    },
    KeybindingEntry {
        section: "Items",
        key: "a",
        action: "Open file explorer to add a new item",
    },
    KeybindingEntry {
        section: "Items",
        key: "e",
        action: "Edit selected item metadata",
    },
    KeybindingEntry {
        section: "Items",
        key: "b",
        action: "Copy selected item as BibTeX",
    },
    KeybindingEntry {
        section: "Items",
        key: "c",
        action: "Assign selected item to collections",
    },
    KeybindingEntry {
        section: "Items",
        key: "L",
        action: "Browse subscribed shared libraries",
    },
    KeybindingEntry {
        section: "Items",
        key: "Enter",
        action: "Open selected file",
    },
    KeybindingEntry {
        section: "Items",
        key: "Tab",
        action: "Switch focus to collections",
    },
    KeybindingEntry {
        section: "Items",
        key: "q",
        action: "Quit",
    },
    // Normal mode — Collections focus
    KeybindingEntry {
        section: "Collections",
        key: "j / Down",
        action: "Select next collection",
    },
    KeybindingEntry {
        section: "Collections",
        key: "k / Up",
        action: "Select previous collection",
    },
    KeybindingEntry {
        section: "Collections",
        key: "n",
        action: "Create a new collection",
    },
    KeybindingEntry {
        section: "Collections",
        key: "d",
        action: "Delete selected collection",
    },
    KeybindingEntry {
        section: "Collections",
        key: "c",
        action: "Assign items to selected collection",
    },
    KeybindingEntry {
        section: "Collections",
        key: "b",
        action: "Copy BibTeX for collection's items",
    },
    KeybindingEntry {
        section: "Collections",
        key: "p",
        action: "Publish selected collection as a shared library",
    },
    KeybindingEntry {
        section: "Collections",
        key: "L",
        action: "Manage your published shared libraries",
    },
    KeybindingEntry {
        section: "Collections",
        key: "Enter",
        action: "Use collection as active list filter",
    },
    KeybindingEntry {
        section: "Collections",
        key: "Tab",
        action: "Switch focus to items",
    },
    KeybindingEntry {
        section: "Collections",
        key: "q",
        action: "Quit",
    },
    // Global
    KeybindingEntry {
        section: "Global",
        key: "?",
        action: "Show this help screen",
    },
    // File explorer popup (Insert mode)
    KeybindingEntry {
        section: "File Explorer",
        key: "a",
        action: "Fetch metadata candidates for current file",
    },
    KeybindingEntry {
        section: "File Explorer",
        key: "Esc / Backspace / q",
        action: "Return to normal mode",
    },
    // Metadata selection popup
    KeybindingEntry {
        section: "Metadata Selection",
        key: "j / k",
        action: "Navigate candidates",
    },
    KeybindingEntry {
        section: "Metadata Selection",
        key: "Enter",
        action: "Open metadata edit form",
    },
    KeybindingEntry {
        section: "Metadata Selection",
        key: "Esc / q",
        action: "Cancel",
    },
    // Metadata edit popup
    KeybindingEntry {
        section: "Metadata Edit",
        key: "j / k",
        action: "Navigate fields",
    },
    KeybindingEntry {
        section: "Metadata Edit",
        key: "Enter",
        action: "Toggle field editing",
    },
    KeybindingEntry {
        section: "Metadata Edit",
        key: "t",
        action: "Cycle item type",
    },
    KeybindingEntry {
        section: "Metadata Edit",
        key: "Ctrl+S",
        action: "Save",
    },
    KeybindingEntry {
        section: "Metadata Edit",
        key: "Esc / q",
        action: "Cancel",
    },
    // Collection create popup
    KeybindingEntry {
        section: "Collection Create",
        key: "Enter",
        action: "Create collection",
    },
    KeybindingEntry {
        section: "Collection Create",
        key: "Esc",
        action: "Cancel",
    },
    // Collection assign popup
    KeybindingEntry {
        section: "Collection Assign",
        key: "j / k",
        action: "Navigate",
    },
    KeybindingEntry {
        section: "Collection Assign",
        key: "Space / Enter",
        action: "Toggle assignment",
    },
    KeybindingEntry {
        section: "Collection Assign",
        key: "Ctrl+S",
        action: "Save assignments",
    },
    KeybindingEntry {
        section: "Collection Assign",
        key: "Esc / q",
        action: "Cancel",
    },
    // Library publish popup
    KeybindingEntry {
        section: "Library Publish",
        key: "Tab",
        action: "Switch field",
    },
    KeybindingEntry {
        section: "Library Publish",
        key: "Ctrl+S",
        action: "Publish library and generate ticket",
    },
    KeybindingEntry {
        section: "Library Publish",
        key: "Esc",
        action: "Cancel",
    },
    // Library subscribe popup
    KeybindingEntry {
        section: "Library Subscribe",
        key: "Tab",
        action: "Switch field",
    },
    KeybindingEntry {
        section: "Library Subscribe",
        key: "Ctrl+S",
        action: "Subscribe using ticket",
    },
    KeybindingEntry {
        section: "Library Subscribe",
        key: "Esc",
        action: "Cancel",
    },
    // Library browse popup
    KeybindingEntry {
        section: "Library Browse",
        key: "Tab",
        action: "Switch focus (subscriptions / papers)",
    },
    KeybindingEntry {
        section: "Library Browse",
        key: "j / k",
        action: "Navigate",
    },
    KeybindingEntry {
        section: "Library Browse",
        key: "a",
        action: "Add a new subscription",
    },
    KeybindingEntry {
        section: "Library Browse",
        key: "d",
        action: "Remove selected subscription",
    },
    KeybindingEntry {
        section: "Library Browse",
        key: "Enter",
        action: "Import selected paper",
    },
    KeybindingEntry {
        section: "Library Browse",
        key: "Esc / q",
        action: "Close",
    },
    // Library manage popup
    KeybindingEntry {
        section: "Library Manage",
        key: "j / k",
        action: "Navigate published libraries",
    },
    KeybindingEntry {
        section: "Library Manage",
        key: "t",
        action: "Generate or refresh invite ticket",
    },
    KeybindingEntry {
        section: "Library Manage",
        key: "c",
        action: "Copy ticket to clipboard",
    },
    KeybindingEntry {
        section: "Library Manage",
        key: "d",
        action: "Delete publication",
    },
    KeybindingEntry {
        section: "Library Manage",
        key: "Esc / q",
        action: "Close",
    },
    // Help popup
    KeybindingEntry {
        section: "Help",
        key: "j / k",
        action: "Scroll",
    },
    KeybindingEntry {
        section: "Help",
        key: "Esc / q",
        action: "Close",
    },
];
