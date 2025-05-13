import { graphql } from 'gql.tada';

export const CartItemFragment = graphql(`
  fragment CartItemFragment on CartItem {
    quantity
    product {
      id
      slug
      name
      price
      coverImage {
        src
        alt
      }
    }
  }
`);

export const CartFragment = graphql(
  `
    fragment CartFragment on Cart {
      id
      totalPrice
      items {
        quantity
        product {
          slug
        }
        ...CartItemFragment
      }
    }
  `,
  [CartItemFragment]
);
