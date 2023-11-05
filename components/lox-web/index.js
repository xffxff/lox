import { greet } from 'lox_web';
import './style.css';

function component() {
    const element = document.createElement('div');

    element.innerHTML = greet("hello wasm!");
    element.classList.add('hello');

    return element;
}

document.body.appendChild(component());