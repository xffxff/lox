import { execute } from 'lox_web';
import './style.css';
import { basicSetup, EditorView } from "codemirror"
import { EditorState } from "@codemirror/state"
import Convert from "ansi-to-html"

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
    const convert = new Convert({
        newline: true
    });
    try {
        const output = execute(code);
        const outputHtml = convert.toHtml(output);
        document.getElementById('output-display').innerHTML = outputHtml;
    } catch (e) {
        document.getElementById('output-display').textContent = e.message;
    }
});
