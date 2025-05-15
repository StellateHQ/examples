import Link from 'next/link'
import { InferGetServerSidePropsType } from 'next'
import { Redis } from '@upstash/redis'

export default function Home({
  endpoints,
}: InferGetServerSidePropsType<typeof getServerSideProps>) {
  return (
    <div className='container mx-auto p-8'>
      <h1 className='mb-6 text-3xl font-bold'>GraphQL Demo (Next.js)</h1>

      <h2 className='mb-4 text-xl font-semibold'>Available Endpoints</h2>
      <ul className='mb-6 list-inside list-disc'>
        {endpoints.map((slug: string) => (
          <li key={slug}>
            <Link href={`/api/${slug}`} className='text-blue-600 underline'>
              {`/api/${slug}`}
            </Link>
          </li>
        ))}
      </ul>

      <h2 className='mb-4 text-xl font-semibold'>How to Use</h2>
      <ol className='list-inside list-decimal space-y-4 text-gray-800'>
        <li>
          <strong>Generate Token:</strong> <code>GET /api/token</code> to
          receive your JWT.
        </li>
        <li>
          <strong>Create Endpoint:</strong>{' '}
          <Link href='/api/admin' className='text-blue-600 underline'>
            /api/admin
          </Link>{' '}
          –
          <div className='mt-2'>
            <p>In GraphQL Playground:</p>
            <ul className='ml-6 list-disc'>
              <li>
                Click <em>HTTP HEADERS</em> and add:
              </li>
            </ul>
            <pre className='my-2 rounded bg-gray-100 p-2'>
              {`{ "Authorization": "bearer <your-jwt-here>" }`}
            </pre>
            <ul className='ml-6 list-disc'>
              <li>Then run:</li>
            </ul>
            <pre className='rounded bg-gray-100 p-2'>
              {`mutation {
  createEndpoint(slug: "my-first-slug")
}`}
            </pre>
          </div>
        </li>
        <li>
          <strong>Query Your Slug:</strong> Navigate to{' '}
          <code>/api/&lt;your-slug&gt;</code> or use <code>curl</code> as in the
          README.
        </li>
      </ol>
    </div>
  )
}

export async function getServerSideProps() {
  const redis = new Redis({
    url: process.env.UPSTASH_REDIS_REST_URL!,
    token: process.env.UPSTASH_REDIS_REST_TOKEN!,
  })
  const endpoints = await redis.keys('*')
  return { props: { endpoints } }
}
