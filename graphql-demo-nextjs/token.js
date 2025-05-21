const { sign } = require('jsonwebtoken')

if (!process.env.TOKEN_SECRET) {
  throw Error("No 'TOKEN_SECRET' environment variable found.")
}

const token = sign({}, process.env.TOKEN_SECRET, {
  audience: process.env.TOKEN_AUDIENCE || 'graphql-demo.nextjs.app',
  expiresIn: '100 years',
})

console.log(token)
