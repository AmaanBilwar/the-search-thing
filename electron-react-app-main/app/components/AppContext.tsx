import { createContext, useContext, useState, ReactNode } from 'react'

interface AppContextType {
  isIndexed: boolean
  hasSearchResults: boolean
  setIsIndexed: (value: boolean) => void
  setHasSearchResults: (value: boolean) => void
  footerAction: 'index' | 'open'
}

const AppContext = createContext<AppContextType | undefined>(undefined)

export const AppProvider = ({ children }: { children: ReactNode }) => {
  const [isIndexed, setIsIndexed] = useState(false)
  const [hasSearchResults, setHasSearchResults] = useState(false)

  // Compute footer action based on state
  const footerAction: 'index' | 'open' = isIndexed && hasSearchResults ? 'open' : 'index'

  return (
    <AppContext.Provider
      value={{
        isIndexed,
        hasSearchResults,
        setIsIndexed,
        setHasSearchResults,
        footerAction,
      }}
    >
      {children}
    </AppContext.Provider>
  )
}

export const useAppContext = () => {
  const context = useContext(AppContext)
  if (!context) {
    throw new Error('useAppContext must be used within AppProvider')
  }
  return context
}
