import { useState } from "react";
import Debugger from "./components/Debugger";
import FilePicker from "./components/FilePicker";

function App() {
  const [displayDebugger] = useState(true);

  if (displayDebugger) {
    return <Debugger />;
  }

  return <FilePicker />;
}

export default App;
