import("../pkg/index.js").catch(console.error);
import { EditorState } from "@codemirror/state"
import { EditorView, keymap } from "@codemirror/view"
import { defaultKeymap } from "@codemirror/commands"
import { basicSetup } from "codemirror"

let startState = EditorState.create({
    doc: "Hello World",
    extensions: [basicSetup, keymap.of(defaultKeymap)]
})

let view = new EditorView({
    state: startState,
    parent: document.body
})

