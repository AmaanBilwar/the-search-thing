
export default function Footer() {
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
    <div>
      
    </div>
  )
}