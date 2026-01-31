import { useState } from "react"
import { useConveyor } from "../hooks/use-conveyor"
import { useAppContext } from "./AppContext"
import { Button } from "./ui/button"
import about from "@/resources/about.svg"
import enter from "@/resources/enter.svg"


export default function Footer() {
  const search = useConveyor("search")
  const [isIndexing, setIsIndexing] = useState(false)
  const [indexResult, setIndexResult] = useState("")
  const [errorMessage, setErrorMessage] = useState("")
  
  const {isIndexed, setIsIndexed} = useAppContext()
  
  const handleStartIndexing = async () => {
    const res = await search.openFileDialog()
    
    if (!res || res.length === 0) return // check to prevent indexing null 
    
    setIsIndexing(true)
    setErrorMessage("")
    try {
      const indexRes = await search.index(res)
      console.error('Index response:', indexRes)
      if (indexRes) {
        setIsIndexed(true)
        setIndexResult(indexRes)
      } else {
        setErrorMessage("No response from indexing")
        setIndexResult("Indexing returned no data")
      }
    } catch (error) {
      console.error('Error indexing files:', error)
      setErrorMessage(`Indexing failed: ${error}`)
      setIndexResult("")
    } finally {
      setIsIndexing(false)
    }
  }
  
  return (
    <div className="flex flex-row justify-between items-center w-full h-full">
      <img src={about} alt="About" className="w-5 h-5 opacity-75" />
      
      <div className="text-sm">
        {isIndexing ? (
          <span className="opacity-75">Indexing...</span>
        ) : errorMessage ? (
          <span className="text-red-500">{errorMessage}</span>
        ) : (
          <span>{indexResult}</span>
        )}
      </div>
      
      {!isIndexed ? (
        <Button variant="transparent" onClick={handleStartIndexing}>
          Index <img src={enter} alt="About" className="w-5 h-6 opacity-75" />
        </Button>
      ) : (
        <Button variant="transparent" onClick={handleStartIndexing}>
          Open <img src={enter} alt="About" className="w-5 h-6 opacity-75" />
        </Button>
      )}
    </div>
  )
}
