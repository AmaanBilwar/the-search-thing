// src/renderer/app.tsx (or similar)
import React from 'react';
import { createRoot } from 'react-dom/client';

const App: React.FC = () => {
  const [title, setTitle] = React.useState('');

  const handleClick = () => {
    window.electronAPI.setTitle(title);
  };

  return (
    <div>
      <h2>Hello from React!</h2>
      <div>
        Title:{' '}
        <input
          value={title}
          onChange={e => setTitle(e.target.value)}
        />
      </div>
      <button onClick={handleClick}>Set</button>
    </div>
  );
};


const container = document.getElementById('root') ?? document.body;
const root = createRoot(container);
root.render(<App />);
