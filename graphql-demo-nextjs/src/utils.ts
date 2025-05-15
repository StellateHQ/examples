import { State } from './types'

export const createState = (): State => ({
  todos: [
    {
      id: '1',
      title: 'Deploy to GraphCDN',
      createdAt: new Date().toISOString(),
      user: '1',
    },
  ],
  users: [{ id: '1', name: 'GraphCDN User' }],
})

export const allowedOrigins = process.env.ALLOWED_ORIGINS?.split(',') || ['*']
