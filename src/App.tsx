import {createSignal, Index, onMount} from "solid-js";
import {invoke} from "@tauri-apps/api/core";
import "./App.css";

interface DisplayInfo {
  id: string,
  name: string,
  brightness: number
}

function App() {
  const [displays, setDisplays] = createSignal<DisplayInfo[]>([]);

  onMount(async () => {
    setDisplays(await invoke("list_displays"));
  });

  async function handleBrightnessChange(id: string, value: number) {
    setDisplays((prev: DisplayInfo[]) => {
      return prev.map((d: DisplayInfo) => {
        return d.id === id ? {...d, brightness: value} : d;
      });
    });

    try {
      await invoke("set_brightness", {id, inputValue: value});
    } catch (err) {
      console.error("Failed to set brightness:", err);
    }
  }


  return (
    <main class="antialiased p-2">
      <h1 class="text-2xl mb-2 font-bold">Welcome to Lumon</h1>
      <form class="row">
        <Index each={displays()} fallback={"Loading..."}>
          {(d, i) => (
            <div class="border-0 dark:bg-gray-800 bg-gray-400 drop-shadow-md rounded-lg p-2">
              <h2 class="text-xl font-semibold">{i + 1 + ". " + d().name}</h2>
              <label>Brightness: {d().brightness}%</label>
              <br/>
              <input
                type="range"
                min={0}
                max={100}
                step={5}
                value={d().brightness}
                class="w-full"
                onInput={async (e) =>
                  await handleBrightnessChange(d().id, e.currentTarget.valueAsNumber)
                }
              />
            </div>
          )}
        </Index>
      </form>
    </main>
  );
}

export default App;