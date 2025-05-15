import { NextApiRequest, NextApiResponse } from 'next'
import { envelop, useSchema } from '@envelop/core'
import { renderPlaygroundPage } from 'graphql-playground-html'
import { Redis } from '@upstash/redis'
import { adminSchema } from '@/admin/schema'
import { validate as graphqlValidate } from 'graphql'
import { verify } from 'jsonwebtoken'

const redis = new Redis({
  url: process.env.UPSTASH_REDIS_REST_URL!,
  token: process.env.UPSTASH_REDIS_REST_TOKEN!,
})
const getEnveloped = envelop({ plugins: [useSchema(adminSchema)] })

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  if (req.method === 'GET') {
    res.setHeader('Content-Type', 'text/html')
    res.send(renderPlaygroundPage({ endpoint: '/api/admin' }))
    return
  }

  const auth = req.headers.authorization
  if (!auth)
    return res
      .status(401)
      .json({ errors: ['Authorization header is required'] })
  const m = /bearer (.+)/i.exec(auth)
  const token = m?.[1]
  if (!token)
    return res.status(401).json({ errors: ["Expected 'bearer <token>'"] })

  try {
    await verify(token, process.env.TOKEN_SECRET!)
  } catch (e: any) {
    return res.status(403).json({ errors: [`Token error: ${e.message}`] })
  }

  const { parse, execute } = getEnveloped({ req, res })
  const { query, variables, operationName } = req.body
  const documentAST = parse(query)
  const errs = graphqlValidate(adminSchema, documentAST)
  if (errs.length) return res.status(400).json({ errors: errs })

  const result = await execute({
    schema: adminSchema,
    document: documentAST,
    variableValues: variables,
    operationName,
    contextValue: { env: { STATE: redis } },
  })
  res.json(result)
}
