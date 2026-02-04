import * as React from 'react'
import { useAppContext } from './AppContext'
import noFiles from '@/resources/no-files-found.svg'
import { ResultProps } from './types/types'

const Results: React.FC<ResultProps> = ({ searchResults, query, hasSearched }) => {
  const { isIndexed } = useAppContext()

  // Extract results safely from the searchResults object
  const files = searchResults?.files || []
  const videos = searchResults?.videos || []
  const allResults = [...files, ...videos]

  // Searched but found nothing
  if (hasSearched && allResults.length === 0 && query) {
    return (
      <div className="flex flex-col items-center gap-4 w-full h-full pt-30">
        <img src={noFiles} alt="No files" className="w-15 h-15 opacity-75" />
        <div className="text-zinc-500">No results have been found for "{query}"</div>
      </div>
    )
  }

  return (
    <div className="flex items-center w-full h-full">
      {!isIndexed ? (
        <div className="flex flex-col items-center gap-4 w-full h-full pt-30">
          <img src={noFiles} alt="No files" className="w-15 h-15 opacity-75" />
          <div className="text-zinc-500">No files have been indexed :(</div>
        </div>
      ) : (
        <div className="flex flex-col items-center w-full h-full overflow-y-auto">
          {allResults.map((result) => (
            <div key={result.file_id} className="result-item p-2 border-b border-zinc-700 w-full">
              <p className="text-white">{result.path}</p>
              <small className="text-zinc-400">{result.content}</small>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default Results