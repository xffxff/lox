import { compiler } from 'lox_web';
import './style.css';
import { basicSetup, EditorView } from "codemirror"
import { EditorState } from "@codemirror/state"
import Convert from "ansi-to-html"

// Create HTML structure
document.body.insertAdjacentHTML('beforeend', `
    <div id="playground">
        <button id="run-button">Run</button>
        <button id="syntax-button">Syntax</button>
        <button id="bytecode-button">Bytecode</button>
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

const lox_compiler = compiler();


// Modify the handleButtonClick function to handle the 'bytecode' action
function handleButtonClick(action) {
    const code = inputView.state.doc.toString();
    const convert = new Convert({
        newline: true
    });
    try {
        lox_compiler.set_source_text(code)
        let output;
        if (action === 'execute') {
            output = lox_compiler.execute();
        } else if (action === 'parse') {
            output = lox_compiler.parse();
        } else if (action === 'bytecode') {
            output = lox_compiler.bytecode(); // Assuming bytecode is a method that returns the bytecode
        }
        console.log(output);
        const outputHtml = convert.toHtml(output);
        document.getElementById('output-display').innerHTML = outputHtml;
    } catch (e) {
        document.getElementById('output-display').textContent = e.message;
    }
}

document.getElementById('run-button').addEventListener('click', () => handleButtonClick('execute'));
document.getElementById('syntax-button').addEventListener('click', () => handleButtonClick('parse'));
document.getElementById('bytecode-button').addEventListener('click', () => handleButtonClick('bytecode'));