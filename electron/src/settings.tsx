import React from 'react'
import { createRoot } from 'react-dom/client'

const SettingsApp = () => {
  return(
    <div>
      <h2>Hello from Settings!</h2>
      {/* Add routes or window-specific content here */}
    </div>  
  )
}

const root = createRoot(document.body);
root.render(<SettingsApp />);