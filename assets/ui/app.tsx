import { render } from "supersolid";

function App() {
    return <div>
        <Title text="The Philosopher's Stone"/>
    </div>
}

function Title({text}: {text: string}){
    return <h1 class="text-lg text-white">
        {text}
    </h1>
}

render(() => <App />, document.getElementById("root"));

