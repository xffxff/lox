import { execute } from 'lox_web';
import './style.css';
import { basicSetup, EditorView } from "codemirror"
import { EditorState } from "@codemirror/state"

// Create HTML structure
document.body.insertAdjacentHTML('beforeend', `
    <div id="playground">
        <button id="run-button">Run</button>
        <div id="editor-container">
            <div id="input-editor"></div>
            <pre id="output-display"></pre>
        </div>
    </div>
`);

// Initialize CodeMirror input editor
const inputState = EditorState.create({
    doc: `
fun hello() {
    print "hello lox!";
}

hello();
    `,
    extensions: [basicSetup]
});

const inputView = new EditorView({
    state: inputState,
    parent: document.getElementById('input-editor')
});

// Add event listener for run button
document.getElementById('run-button').addEventListener('click', () => {
    const code = inputView.state.doc.toString();
    try {
        const output = execute(code);
        document.getElementById('output-display').textContent = String(output);
    } catch (e) {
        document.getElementById('output-display').textContent = e.message;
    }
});
