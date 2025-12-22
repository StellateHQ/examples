/** @type {import('next').NextConfig} */
module.exports = {
  reactStrictMode: false,
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
