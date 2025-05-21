import { NextApiRequest, NextApiResponse } from 'next'
import { envelop, useSchema } from '@envelop/core'
import { Redis } from '@upstash/redis'
import { validate as graphqlValidate } from 'graphql'
import { userSchema } from '@/mock/schema'
import { ALLOWED_ORIGINS } from '@/utils'
import { State } from '@/types'

const redis = new Redis({
  url: process.env.UPSTASH_REDIS_REST_URL!,
  token: process.env.UPSTASH_REDIS_REST_TOKEN!,
})
const getEnveloped = envelop({ plugins: [useSchema(userSchema)] })

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  const slug = req.query.slug as string
  if (!slug) return res.status(404).send('Not Found')
  if (req.method === 'GET') {
    const cacheRaw = await redis.get(slug)
    if (!cacheRaw) {
      return res.status(404).json({ errors: ['Slug not found'] })
    }

    const state: State =
      typeof cacheRaw === 'string' ? JSON.parse(cacheRaw) : (cacheRaw as State)

    const { parse, execute } = getEnveloped({ req, res })
    const documentAST = parse(`query { todos { id title createdAt user } }`)
    const errs = graphqlValidate(userSchema, documentAST)
    if (errs.length) return res.status(400).json({ errors: errs })

    const result = await execute({
      schema: userSchema,
      document: documentAST,
      contextValue: {
        state,
        setState: (s: State) => redis.set(slug, JSON.stringify(s)),
      },
    })

    res.setHeader('Content-Type', 'application/json')
    return res.status(200).json(result)
  }

  if (req.method === 'OPTIONS') {
    res.setHeader('Access-Control-Allow-Origin', ALLOWED_ORIGINS)
    res.setHeader('Access-Control-Allow-Methods', 'OPTIONS, POST')
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type')
    return res.status(204).end()
  }

  const cache = await redis.get(slug)
  if (!cache) return res.status(404).json({ errors: ['Slug not found'] })

  const state = JSON.parse(cache as string)
  const { parse, execute } = getEnveloped({ req, res })
  const { query, variables, operationName } = req.body
  const documentAST = parse(query)
  const errs = graphqlValidate(userSchema, documentAST)
  if (errs.length) return res.status(400).json({ errors: errs })

  const result = await execute({
    schema: userSchema,
    document: documentAST,
    variableValues: variables,
    operationName,
    contextValue: {
      state,
      setState: (s: any) => redis.set(slug, JSON.stringify(s)),
    },
  })

  const origin = req.headers.origin || ''
  const allow =
    ALLOWED_ORIGINS.includes('*') || ALLOWED_ORIGINS.includes(origin)
      ? origin
      : ALLOWED_ORIGINS[0]
  res.setHeader('Access-Control-Allow-Origin', allow)
  res.setHeader('Access-Control-Allow-Methods', 'OPTIONS, POST')
  res.setHeader('Content-Type', 'application/json')
  res.json(result)
}
