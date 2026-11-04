import { createSignal, onMount, render, Show } from "supersolid";

function App() {
  const [ingredient1, setIngredient1] = createSignal("");
  const [ingredient2, setIngredient2] = createSignal("");
  const [ingredient3, setIngredient3] = createSignal("");
  const [ingredient4, setIngredient4] = createSignal("");
  const [gotStone, setGotStone] = createSignal(false);
  const [output, setOutput] = createSignal(
    "Cauldron output would appear here."
  );
  onMount(() => {
    bevy.on("outputReceived", ({ output }) => {
      setOutput(`Cauldron output: ${output}`);
      console.log("output received in ui")
    });
    bevy.on("gotStone", (input) => {
      setGotStone(true);
      setOutput(`You have received Philosopher's Stone, congratulations!!!`);
      console.log("received the stone in ui");
    });
  });
  const sending = () => {
    bevy.send("ingredientsPlaced", {
      ingredient_1: ingredient1(),
      ingredient_2: ingredient2(),
      ingredient_3: ingredient3(),
      ingredient_4: ingredient4(),
    });
  };
  return (
    <RootLayout>
      <Title text="The Philosopher's Stone" />
      <div class="pt-4 pb-4">
        <Subtitle
          text={[
            "The goal of the game is to put ingredients into cauldron until you are able to make Philosopher's Stone.",
            "You need to enter four different ingredients into cauldron for one attempt and then press Enter.",
            "Most of the time you will get useless output, but from time to time you will be able to receive a mystic components of Philosopher's Stone.",
          ].join("\n")}
        />
      </div>
      <IngredientInput
        value={ingredient1}
        setValue={setIngredient1}
        submit={sending}
      />
      <IngredientInput
        value={ingredient2}
        setValue={setIngredient2}
        submit={sending}
      />
      <IngredientInput
        value={ingredient3}
        setValue={setIngredient3}
        submit={sending}
      />
      <IngredientInput
        value={ingredient4}
        setValue={setIngredient4}
        submit={sending}
      />
      <Output text={output} />

      <div>
        <Stone gotStone={gotStone} />
      </div>
    </RootLayout>
  );
}

function Stone({ gotStone }: { gotStone: () => boolean }) {
  return (
    <Show when={gotStone()}>
      <img class="w-[500px]" src="./stone.png" />
    </Show>
  );
}

function RootLayout({
  children,
}: {
  children: string | JSX.Element | JSX.Element[];
}) {
  return <div class="flex flex-col p-2">{children}</div>;
}

function Title({ text }: { text: string }) {
  return <h1 class="text-lg text-white">{text}</h1>;
}

function Subtitle({ text }: { text: string }) {
  return <h1 class="text-sm text-white">{text}</h1>;
}

function IngredientInput({
  value,
  setValue,
  submit,
}: {
  value: () => string;
  setValue: (val: string) => void;
  submit: () => void;
}) {
  return (
    <input
      type="text"
      value={value()}
      placeholder="input ingredient"
      class="text-black bg-white border-2 border-blue-300 w-128"
      onInput={(e) => {
        setValue(e.target.value);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          console.log(e.target.value);
          submit();
        }
      }}
    ></input>
  );
}

function Output({ text }: { text: () => string }) {
  return <div>{text()}</div>;
}

render(() => <App />, document.getElementById("root"));
