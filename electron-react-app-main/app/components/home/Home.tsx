import { useEffect, useState } from 'react'
import {Searchbar} from '../ui/searchbar'
import { Badge } from '../ui/badge'
import { useConveyor } from '@/app/hooks/use-conveyor'
import './styles.css'

export default function Home() {
  const [query, setQuery] = useState("")
  const search = useConveyor("search")
  const [results, setResults] = useState<string[]>([])
  const [isIndexed, setIsIndexed] = useState<boolean>(false)
  const [index, setIndexed] = useState<boolean>(false)
  
  const handleSearch = async () => {
    const checkRes = await search.check(query)
    setIsIndexed(checkRes)
    const indexRes = await search.index(query)
    setIndexed(indexRes)
    const res = await search.search(query)
    setResults(res.results)
  }

  return (
    <div className="welcome-content flex flex-col gap-5">
      <div className="flex items-center h-[20%]">
        <Searchbar
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search…"
          onKeyDown={(e) => {
            if (e.key == 'Enter') {
              e.preventDefault();
              handleSearch();
            }
          }}
        />
      </div>
      <div className="flex justify-center items-center gap-4 opacity-50 hover:opacity-80 transition-opacity">
        <DarkModeToggle />
      </div>
    </div>
  )
}

const DarkModeToggle = () => {
  const [isDarkMode, setIsDarkMode] = useState(false)

  useEffect(() => {
    setIsDarkMode(document.documentElement.classList.contains('dark'))
  }, [])

  const toggleDarkMode = () => {
    document.documentElement.classList.toggle('dark')
    setIsDarkMode(!isDarkMode)
  }

  return (
    <div className="flex justify-center items-center gap-2 text-sm cursor-pointer">
      <Badge variant="secondary" onClick={toggleDarkMode}>
        {isDarkMode ? 'Dark Mode' : 'Light Mode'}
      </Badge>
    </div>
  )
}
