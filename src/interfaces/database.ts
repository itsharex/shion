export interface Plan {
  id: number
  name: string
  totalTime: number
}

export type CreatePlan = Pick<Plan, 'name'>

export interface Note {
  id: number
  startTime: number
  endTime: number
  description: string
  planId: number
}