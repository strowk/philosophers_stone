import { createSignal, For, Keyed, onMount, render, Show } from "supersolid";

function App() {
  const [ingredient1, setIngredient1] = createSignal("");
  const [ingredient2, setIngredient2] = createSignal("");
  const [ingredient3, setIngredient3] = createSignal("");
  const [ingredient4, setIngredient4] = createSignal("");
  const [output, setOutput] = createSignal(
    "Cauldron output would appear here. Type in 4 ingredients and press Enter."
  );
  const [gotStone, setGotStone] = createSignal(false);
  // loading state

  onMount(() => {
    bevy.on("outputReceived", ({ output }) => {
      setOutput(`Cauldron output: ${output}`);
      console.log("output received in ui");
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
            "Once you have collected all of them - the Philosopher's Stone will be yours.",
          ].join("\n")}
        />
      </div>
      <div class="flex flex-col">
        <Show when={!gotStone()}>
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
        </Show>
      </div>
      <Output text={output} />
      <Loading />
      <div>
        <Stone gotStone={gotStone} />
      </div>
      <EssencesInventory />
      <Trivia />
    </RootLayout>
  );
}

function EssencesInventory() {
  const [essences, setEssences] = createSignal<Array<string>>([
    // "fire",
    // "fire",
    // "earth",
    // "air",
    // "water",
  ]);
  onMount(() => {
    bevy.on("essences", ({ essences }) => {
      setEssences(essences);
    });
  });

  return (
    <div>
      <Show when={essences().length > 0}>
        <div class="flex flex-col gap-2 pt-4">
          <div>Essences available:</div>
          <div>
            <For each={essences()}>
              {(essence) => <div>{<Essence essence={essence} />}</div>}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
}

function Essence({ essence }: { essence: string }) {
  return (
    <div class="p-1">
      <EssenceBadge essence={essence} />
    </div>
  );
}

function EssenceBadge({ essence }: { essence: string }) {
  switch (essence) {
    case "fire":
      return <FireEssence />;
    case "earth":
      return <EarthEssence />;
    case "air":
      return <AirEssence />;
    case "water":
      return <WaterEssence />;
    case "aether":
      return <AetherEssence />;
  }
  return <div>{essence}</div>;
}

function EssenceText({ children }: { children: string }) {
  return <div class="text-gray-900 p-1">{children}</div>;
}

function WaterEssence() {
  return (
    <div class="bg-blue-400 rounded-md">
      <EssenceText>water</EssenceText>
    </div>
  );
}

function AirEssence() {
  return (
    <div class="bg-blue-200 rounded-md">
      <EssenceText>air</EssenceText>
    </div>
  );
}

function EarthEssence() {
  return (
    <div class="bg-green-300 rounded-md">
      <EssenceText>earth</EssenceText>
    </div>
  );
}

function FireEssence() {
  return (
    <div class="bg-amber-300 rounded-md">
      <EssenceText>fire</EssenceText>
    </div>
  );
}

function AetherEssence() {
  return (
    <div class="bg-purple-300 rounded-md">
      <EssenceText>aether</EssenceText>
    </div>
  );
}

const trivia = [
  "Alchemy is an ancient branch of natural philosophy, a philosophical and protoscientific tradition that was once practised in China, India, the Muslim world, and Europe.",
  "Greek-speaking alchemists often named their craft 'the Art' or 'Knowledge', and it was often characterised as mystic, sacred, or divine.",
  "Alchemical writers used classical figures from Greek (e.g., Hades), Roman (e.g., Lucius), and Egyptian mythology to illuminate their works and allegorize alchemical transmutation.",
  "The earliest known written mention of the philosopher's stone is in the Cheirokmeta by Zosimos of Panopolis (c. 300 AD).",
  "In alchemy and philosophy, prima materia, materia prima, or first matter, is the ubiquitous starting material required for the alchemical magnum opus and the creation of the philosopher's stone.",
  "The Magnum Opus or Great Work is a term for the process of working with the prima materia to create the philosopher's stone.",
  "According to Plato, the four elements are derived from a common source or prima materia (first matter), associated with chaos. Prima materia is also the name alchemists assign to the starting ingredient for the creation of the philosopher's stone.",
  "The Squared Circle is an Alchemical Symbol illustrating the interplay of the four elements of matter symbolizing the philosopher's stone - the result of the 'Great Work'",
];

function Trivia() {
  const [currentIndex, setCurrentIndex] = createSignal(0);
  const [current, setCurrent] = createSignal(trivia[0]);
  const [next, setNext] = createSignal("");
  onMount(() => {
    setInterval(() => {
      let newCurrentIndex = currentIndex() + 1;
      if (newCurrentIndex >= trivia.length) {
        newCurrentIndex = 0;
      }
      let newNextIndex = newCurrentIndex + 1;
      if (newNextIndex >= trivia.length) {
        newNextIndex = 0;
      }
      setCurrentIndex(newCurrentIndex);
      setCurrent(trivia[newCurrentIndex]);
      setNext(trivia[newNextIndex]);
      setTimeout(() => {
        setCurrent(trivia[newNextIndex]);
        setNext("");
      }, 1000);
    }, 20000);
  });
  return (
    <div class="pt-2 flex flex-col w-full">
      <div class="text-md mb-2 w-full">Trivia:</div>
      <div class="relative text-xs">
        <Show when={next() !== ""}>
          <div class="absolute top-0 w-full moving-from-0">{current()}</div>
          <div class="absolute top-0 w-full moving-from-100">{next()}</div>
        </Show>
        <Show when={next() === ""}>
          <div class="absolute top-0 w-full">{current()}</div>
        </Show>
      </div>
    </div>
  );
}

function Loading() {
  const [showLoading, setShowLoading] = createSignal(false);
  const [loadingName, setLoadingName] = createSignal("");
  const [loadingPercent, setLoadingPercent] = createSignal(0);

  onMount(() => {
    bevy.on("loading", ({ show, name, percent }) => {
      setShowLoading(show);
      setLoadingName(name);
      setLoadingPercent(percent);
    });
  });
  return (
    <div>
      {
        <Show when={showLoading()}>
          <div class="rounded-md pulsing-color-anim w-full bg-green-900 p-2 pt-4">
            <div class="flex flex-col w-full">
              <div
                class="h-4 z-1 pulsing-color-bg-anim"
                style={`width: ${loadingPercent()}%`}
              ></div>
              <div class="z-2 p-2 text-sm">
                Cauldron preparing: {loadingName()}
              </div>
            </div>
          </div>
        </Show>
      }
    </div>
  );
}

function Stone({ gotStone }: { gotStone: () => boolean }) {
  return (
    <Show when={gotStone()}>
      {/* <Show when={true}> */}
      <img class="w-[150px] pulsing-size-anim" src="./stone.png" />
    </Show>
  );
}

function RootLayout({
  children,
}: {
  children: string | JSX.Element | JSX.Element[];
}) {
  return (
    <div class="flex flex-row w-full justify-center">
      <div class="flex flex-col p-2 pt-8 max-w-[800px]">{children}</div>
    </div>
  );
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
      class="text-black bg-green-200 focus:bg-green-100 border-2 focus:border-4 border-green-900 focus:border-green-300 w-128 m-1 p-1 rounded-md"
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
  return <div class="m-4">{text()}</div>;
}

render(() => <App />, document.getElementById("root"));
