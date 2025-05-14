/** @type {import('next').NextConfig} */
module.exports = {
  reactStrictMode: false,
  eslint: {
    // Disabling on production builds because we're running checks on PRs via GitHub Actions.
    ignoreDuringBuilds: true
  },
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'picsum.photos',
        pathname: '/seed/**'
      }
    ]
  },
  async rewrites() {
    return [
      {
        source: '/uncached/graphql',
        destination: 'https://demo-shop-gql-api.recc.workers.dev/graphql'
      },
      {
        source: '/cached/graphql',
        destination: 'https://defer-demo.stellate.sh'
      }
    ];
  }
};
