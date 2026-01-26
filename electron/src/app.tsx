import React from 'react';
import { createRoot } from 'react-dom/client';

const App = () => (
  <div>
    <h2>Hello from React!</h2>
    {/* Add routes or window-specific content here */}
  </div>
);

const root = createRoot(document.body);
root.render(<App />);
