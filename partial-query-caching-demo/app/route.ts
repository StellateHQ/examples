import { createYoga, Plugin } from 'graphql-yoga'
import { schema } from './schema'

const logPlugin: Plugin = {
  onParams({ params }) {
    console.log(params)
  },
}

const { handleRequest } = createYoga({
  schema,
  plugins: [logPlugin],
  graphqlEndpoint: '/',
  fetchAPI: { Response },
})

function setCors(res: Response) {
  res.headers.set('access-control-allow-credentials', 'true')
  res.headers.set('access-control-allow-headers', '*')
  res.headers.set('access-control-allow-methods', 'GET, POST, OPTIONS')
  res.headers.set('access-control-allow-origin', '*')
  res.headers.set('access-control-expose-headers', '*')
  res.headers.set('access-control-max-age', '3600')
}

async function withCors(
  req: Request,
  ctx: { params: Record<string, string | string[]> } | any,
) {
  const res = await handleRequest(req, ctx)
  setCors(res)
  return res
}

export { withCors as GET, withCors as POST }

export async function OPTIONS() {
  const res = new Response(null, { status: 204 })
  setCors(res)
  return res
}
