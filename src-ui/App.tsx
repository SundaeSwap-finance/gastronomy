import { useEffect, useState } from 'react';
import reactLogo from './assets/react.svg';
import viteLogo from '/vite.svg';
import { invoke } from '@tauri-apps/api';
import './App.css';

async function greet(name: string): Promise<string> {
  const result: string = await invoke('greet', { name });
  return result;
}

function App() {
  const [count, setCount] = useState(0);
  const [greeting, setGreeting] = useState('Loading...');
  useEffect(() => {
    greet('World')
      .then(res => setGreeting(res))
      .catch(err => setGreeting(err.toString()));
  });

  return (
    <>
      <div>
        <a href="https://vitejs.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>{greeting}</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <p>
          Edit <code>src-ui/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  );
}

export default App;
