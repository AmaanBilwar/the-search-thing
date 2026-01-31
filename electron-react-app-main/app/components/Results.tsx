import * as React from "react"
import { useAppContext } from "./AppContext"

interface ResultsProps {
  results: string[]
  query: string
}

export default function Results({ results, query }: ResultsProps) {
  const { isIndexed } = useAppContext()
  
  // // No results yet
  // if (results.length === 0 && !query) {
  //   return <div className="text-zinc-500">Start typing to search...</div>
  // }
  
  // Searched but found nothing
  if (results.length === 0 && query) {
    return(
      <div className="flex items-center justify-center w-full h-full">
        <div className="text-zinc-500">No results found for "{query}"</div>
      </div>
    )
  }
  
  // Show results if we have them
  return (
    <div className="flex items-center justify-center w-full h-full">
      {!isIndexed ? (
        <div className="flex items-center justify-center w-full h-full">
          <div className="text-zinc-500">No files have been indexed yet...</div>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center w-full h-full">
          {results.map((result, idx) => (
            <div key={idx} className="result-item">
              {result}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
