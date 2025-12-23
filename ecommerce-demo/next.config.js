/** @type {import('next').NextConfig} */
module.exports = {
  reactStrictMode: false,
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'picsum.photos',
        pathname: '/seed/**'
      },
      {
        protocol: 'https',
        hostname: 'fastly.picsum.photos',
        pathname: '/**'
      }
    ],
    deviceSizes: [640, 750, 828, 1080, 1200, 1920, 2048, 3840],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384],
    loader: 'default',
    minimumCacheTTL: 60,
    dangerouslyAllowSVG: false,
    contentDispositionType: 'inline'
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
