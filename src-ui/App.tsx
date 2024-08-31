import { ChangeEventHandler, useEffect, useMemo, useState } from "react";
import Debugger from "./components/Debugger";
import FilePicker from "./components/FilePicker";
import cx from "classnames";
import gastronomyLogo from "./assets/images/gastronomy-logo.svg";
import sundaeLogo from "./assets/images/sundae-logo.svg";
import { Store } from "tauri-plugin-store-api";

function App() {
  const [displayDebugger, setDisplayDebugger] = useState(false);
  const [parameters, setParameters] = useState<string[]>([]);
  const [file, setFile] = useState("");
  const [key, setKey] = useState(undefined as string | undefined);

  const store = useMemo(() => {
    return new Store("settings.json");
  }, []);

  useEffect(() => {
    (async () => {
      const data = (await store.get<string>("blockfrost.key")) ?? undefined;
      setKey(data);
    })();
  }, [store]);

  const fileName = useMemo(() => {
    const parts = file.split(/[/\\]/);
    return parts[parts.length - 1];
  }, [file]);

  const handleChange: ChangeEventHandler<HTMLTextAreaElement> = (event) => {
    setParameters(event.target.value.split("\n"));
  };

  const onQuit = () => {
    setDisplayDebugger(false);
    setFile("");
    setParameters([]);
  };

  return (
    <div className="bg-slate-950 h-svh font-['Source_Code_Pro'] text-lime-600">
      {displayDebugger ? (
        <Debugger
          file={file}
          fileName={fileName}
          onQuit={onQuit}
          parameters={parameters}
        />
      ) : (
        <div className="p-4 flex justify-center items-center h-full flex-col gap-11">
          <h1 className="text-6xl uppercase font-['Pixelify_Sans'] flex gap-2 items-center">
            <img src={gastronomyLogo} alt="Gastronomy" className="h-16" />
            Gastronomy
          </h1>
          <div className="border border-lime-600 p-6 flex flex-col gap-6 w-[30rem]">
            <input
              type="text"
              placeholder="Blockfrost API Key"
              value={key}
              onChange={(event) => setKey(event.target.value)}
              onBlur={async (event) => {
                try {
                  await store.set("blockfrost.key", event.target.value);
                  await store.save();
                  console.log("Saved");
                } catch (e) {
                  console.error(e);
                }
              }}
            ></input>
            <FilePicker setFile={setFile} fileName={fileName} />
            <div className="flex flex-col gap-4">
              <label htmlFor="parameters" className="cursor-pointer">
                Parameters:
              </label>
              <textarea
                className="p-2 w-full bg-slate-900 text-white border border-slate-800 rounded resize-none h-44 focus:outline-none"
                id="parameters"
                name="parameters"
                onChange={handleChange}
                placeholder="Enter parameters, each on a new line"
                value={parameters?.join("\n")}
              />
            </div>
            <button
              className={cx(
                "p-2 text-lime-600 border border-lime-600 transition-colors duration-300 ease-in-out",
                "disabled:opacity-50 disabled:cursor-not-allowed",
                { "hover:bg-lime-600 hover:text-slate-950": file },
              )}
              onClick={() => setDisplayDebugger(true)}
              disabled={!file}
            >
              Run Debugger
            </button>
          </div>
          <div className="flex items-center gap-2">
            <div>Created by </div>
            <div className="text-xl font-['Pixelify_Sans'] flex gap-2 items-center text-white">
              <img src={sundaeLogo} alt="Gastronomy" className="h-8" />
              Sundae Labs
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
