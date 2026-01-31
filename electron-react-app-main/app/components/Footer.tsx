import { useState } from "react"
import { useConveyor } from "../hooks/use-conveyor"
import { useAppContext } from "./AppContext"
import { Button } from "./ui/button"
import about from "@/resources/about.svg"
import enter from "@/resources/enter.svg"


export default function Footer() {
  const search = useConveyor("search")
  const [isIndexing, setIsIndexing] = useState(false)
  
  const {isIndexed, setIsIndexed} = useAppContext()
  
  const handleStartIndexing = async () => {
    const res = await search.openFileDialog()
    
    if (!res || res.length === 0) return // check to prevent indexing null 
    
    setIsIndexing(true)
    try {
      const indexRes = await search.index(res)
      if (indexRes) {
        setIsIndexed(true)
      }
    } catch (error) {
      console.error('Error indexing files:', error)
    } finally {
      setIsIndexing(false)
    }
  }
  
  return (
    <div className="flex flex-row justify-between items-center w-full h-full">
      <img src={about} alt="About" className="w-5 h-5 opacity-75" />
      
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
