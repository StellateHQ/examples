import { introspectionFromSchema } from 'graphql'
import { schema } from './app/schema'

console.log(JSON.stringify(introspectionFromSchema(schema)))
