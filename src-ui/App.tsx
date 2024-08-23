import { ChangeEventHandler, useState } from "react";
import Debugger from "./components/Debugger";
import FilePicker from "./components/FilePicker";
import cx from "classnames";

function App() {
  const [displayDebugger, setDisplayDebugger] = useState(false);
  const [parameters, setParameters] = useState<string[]>([]);
  const [file, setFile] = useState("");
  const fileName = file.substring(file.lastIndexOf("/") + 1);

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
        <div className="p-4 flex justify-center items-center h-full">
          <div className="border border-lime-600 p-6 flex flex-col gap-6 w-[30rem]">
            <FilePicker setFile={setFile} fileName={fileName} />
            <div className="flex flex-col gap-4">
              <label htmlFor="parameters" className="cursor-pointer">
                Parameters:
              </label>
              <textarea
                className="p-2 w-full bg-slate-800 text-white border border-slate-700 rounded resize-none h-44 focus:outline-none"
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
                { "hover:bg-lime-600 hover:text-slate-950": file }
              )}
              onClick={() => setDisplayDebugger(true)}
              disabled={!file}
            >
              Run Debugger
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
