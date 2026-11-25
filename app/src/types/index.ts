export interface WallpaperInfo {
  name: string
  description: string
  author: string
  path: string
  colors: string[]
}

export interface CommunityWallpaper {
  id: string
  name: string
  author: string
  resolution: string
  type: string
  tags: string[]
  rating: number
  downloads: number
  color: string
  available: boolean
}
