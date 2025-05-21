// pages/api/token.ts
import type { NextApiRequest, NextApiResponse } from 'next'
import { sign } from 'jsonwebtoken'

export default function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'GET') {
    res.setHeader('Allow', 'GET')
    return res.status(405).end('Method Not Allowed')
  }

  const secret = process.env.TOKEN_SECRET
  if (!secret) {
    return res.status(500).json({ error: 'TOKEN_SECRET not configured' })
  }

  // generate a long-lived token
  const token = sign({}, secret, {
    audience: process.env.TOKEN_AUDIENCE || 'graphql-demo.nextjs.app',
    expiresIn: '100 years',
  })

  return res.status(200).json({ token })
}
