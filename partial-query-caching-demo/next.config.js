/** @type {import('next').NextConfig} */
const nextConfig = {
  // Fix turbopack workspace root warning
  experimental: {
    turbo: {
      root: __dirname,
    },
  },
}

module.exports = nextConfig
