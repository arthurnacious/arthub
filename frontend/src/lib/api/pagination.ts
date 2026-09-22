export interface PaginatedResult<T> {
  data: T[]
  page: number
  pageSize: number
  total: number
}

export interface PaginationParams {
  page: number
  pageSize: number
  search: string
}
