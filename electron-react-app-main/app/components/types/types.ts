export interface FileObject {
  file_id: string
  content: string
  path: string
}

export interface VideoObject {
  file_id: string
  content: string
  path: string
}


export interface SearchResults {
  success: boolean
  files: FileObject[]
  videos: VideoObject[]
}