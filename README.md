# GraphCDN examples

The GraphQL ecosystem is rich with server implementations and services.
GraphCDN tries to integrate with anything that is spec-compliant. This
repository contains curated examples that show how to set up GraphCDN
with different services.

## Cookie based authentication

Using a GraphQL API that performs cookie-based authentication with GraphCDN requires an extra step: You need to set up a custom domain on GraphCDN.

The reason for that is just how cookies over HTTP work. When sending a request to log in, the response usually contains a `Set-Cookie` header that stores some kind of token inside a cookie. However, the browser will by default not accept this cookie for a "cross-origin request", i.e. a request that was sent to a different domain.

GraphCDN gives you a subdomain where you can access your service out of the box at <service-name>.graphcdn.app, but this is a different domain than your website. To solve this, GraphCDN allows you to set a custom domain in the settings for your service. 

We currently have example codebases for two common authentication libraries available:
  * [next-auth](./next-auth), for when you are using [NextAuth.js](https://next-auth.js.org/)
  * [passport](./passport-auth), in case you are using [Passport.js](http://www.passportjs.org/)
  
## Contributing

We actively welcome pull requests. Learn how to [contribute](./.github/CONTRIBUTING.md).
